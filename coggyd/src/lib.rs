// coggyd — the session supervisor.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Owning a session's lifetime, which is the thing a supervisor is for.
//!
//! [G0](../../docs/measurements/2026-07-31-150258-g0-frozen.md) measured that
//! killing the process you spawned does not end a session: fifty sessions
//! wrapped in a shell left [exactly fifty stragglers and a teardown 361×
//! slower](../../docs/measurements/2026-07-31-141334-the-shell-costs-teardown.md),
//! because the shell dies and its child does not. A pseudoconsole shows the
//! same asymmetry from the other side — [it belongs to whoever created it
//! rather than to the session it
//! serves](../../docs/measurements/2026-07-30-101141-conhost-and-defender.md).
//!
//! **One job object per session is the answer, and the per-session part is
//! load-bearing.** `sessionbench` creates a single job and joins it with
//! `assign_current_process`, which answers attribution and cannot answer
//! termination: ending that job would end the benchmark. A supervisor needs a
//! job it can end without ending itself.
//!
//! Prior art: `win32job` (MIT OR Apache-2.0) for the safe wrapper, taken whole
//! rather than reached for through FFI, because the workspace forbids unsafe
//! code and the crate covers exactly this. The kill-on-close limit is the
//! mechanism — dropping the handle takes the tree — so nothing here calls
//! `TerminateJobObject`.

#![cfg(windows)]

pub mod pool;
pub mod scrollback;

use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use anyhow::{Context, Result};

use crate::scrollback::{MAX_LINE_BYTES, Scrollback};

/// Expands every `${session}` in an argv to `id`.
///
/// **Prior art: `envsubst` (MIT OR Apache-2.0), taken rather than written.**
/// The mechanism searched for was argument templating in a supervisor, which
/// is what systemd's template units have done for years with `%i`, and what
/// that experience says bites is escaping the marker and deciding what a name
/// nobody defined means.
///
/// **`subst` is the better-known crate and was tried first.** It escapes with
/// a backslash, which is the Windows path separator, so the very first test —
/// three sessions writing under `C:\Users\...` — failed on `\U` as an invalid
/// escape sequence. A placeholder whose whole purpose is building paths cannot
/// ask for every separator to be doubled. `envsubst` recognises only
/// `${braces}` and has no escape character, so a Windows path passes through
/// untouched.
///
/// **The refusal is the load-bearing half**, and it is composed here rather
/// than assumed. `substitute` does not document what it does with a name that
/// is not in the map; removing the check below and running the test answers
/// it — the placeholder is left as written and `Ok` is returned, so the spawn
/// succeeds and every session gets the same value. That is the
/// shared-directory failure this exists to prevent, arriving silently. So
/// `is_templated` is asked afterwards and anything left over fails the spawn.
fn expand(argv: &[String], id: u64) -> Result<Vec<String>> {
    let vars: std::collections::HashMap<String, String> =
        [(SESSION_PLACEHOLDER.to_string(), id.to_string())].into();
    argv.iter()
        .map(|arg| {
            let filled = envsubst::substitute(arg, &vars)
                .map_err(|e| anyhow::anyhow!("expanding {arg:?} for session {id}: {e}"))?;
            if envsubst::is_templated(&filled) {
                anyhow::bail!(
                    "{arg:?} names a placeholder that is not {SESSION_PLACEHOLDER:?}, \
                     and leaving it would give every session the same value"
                );
            }
            Ok(filled)
        })
        .collect()
}

/// Lines a session keeps by default.
///
/// **It bounds the per-line overhead, not the memory.** That reading is the
/// correction: this doc used to say a hundred sessions holding this many lines
/// *of a few hundred bytes* came to tens of megabytes, and nothing made lines
/// a few hundred bytes. Paired with [`scrollback::MAX_LINE_BYTES`] it reaches
/// 13.1 GB across a hundred sessions, against [a gate written for
/// four](../../ROADMAP.md#m1--headless-daemon). [`DEFAULT_SCROLLBACK_BYTES`]
/// is what bounds the content; this bounds the fixed cost of holding it, which
/// [an hour-long hold put at roughly 90 bytes a
/// line](../../docs/measurements/2026-08-01-103225-an-hour-of-a-hundred-sessions.md)
/// on top of whatever the line itself carries.
///
/// What neither may do is grow with the session's lifetime, which is how a
/// supervisor of long-lived sessions runs out of memory without anything
/// looking wrong.
pub const DEFAULT_SCROLLBACK_LINES: usize = 2_000;

/// Bytes of session output a scrollback keeps by default.
///
/// **Taken from Ghostty, which is the one prior art that bounds the quantity
/// the gate is written in.** tmux, WezTerm, Alacritty and GNOME Terminal all
/// cap by line count, and that is sound for them: they store a grid, so a
/// line's width is bounded by the terminal's columns and the line count bounds
/// bytes for free. `coggyd` is on pipes and has no columns, so it inherited
/// the convention without the property that made it safe. The failure is not
/// hypothetical — tmux carries [a request for a `history-bytes`
/// option](https://github.com/tmux/tmux/issues/4859) opened after redraw
/// traffic put about 48 GB into scrollback buffers across a few panes.
///
/// What we did not take is Ghostty's user-facing complaint, that a byte limit
/// makes the retained line count vary with terminal size. Nothing here scrolls
/// a buffer by eye; a benchmark reads it.
///
/// A hundred sessions at this budget hold 25 MiB of content, plus at most
/// `DEFAULT_SCROLLBACK_LINES` lines of overhead apiece — about 43 MB together,
/// or 1% of the gate, where the line count alone allowed three times the whole
/// of it.
///
/// **Seen binding, on the second attempt.** A session emitting four thousand
/// 104-byte lines evicted two thousand of them and never came near this — the
/// line count bound first and the retained content sat at 209 KB, four fifths
/// of the budget. It takes a thousand 404-byte lines to make bytes bind: 352
/// are evicted with the line count nowhere near, and the 648 kept multiply to
/// 261,727 against a ceiling of 262,144. The first run looked like a
/// demonstration and was one of the other ceiling.
pub const DEFAULT_SCROLLBACK_BYTES: usize = 256 * 1024;

/// The name a session's argv uses to mean *my own id*: `${session}`.
///
/// Braces are required and there is no escape character, so a Windows path
/// needs no special handling — see [`expand`] for why that decided the crate.
pub const SESSION_PLACEHOLDER: &str = "session";

/// A session and the job that owns everything it spawns.
///
/// Dropping this ends the session and its whole tree. That is the point: the
/// alternative is killing a root and reaping whatever outlived it.
pub struct Session {
    id: u64,
    child: Child,
    /// Taken in `drop` before the drains are joined. The job carries
    /// `KILL_ON_JOB_CLOSE`, so releasing the last handle is what terminates
    /// the tree — and an `Option` is what lets that happen in a chosen order
    /// rather than whatever order the fields were declared in.
    job: Option<win32job::Job>,
    /// One per stream. They end when their pipe reaches end-of-file, which
    /// happens when the tree dies, which is why the job goes first.
    drains: Vec<JoinHandle<()>>,
    scrollback: Arc<Mutex<Scrollback>>,
}

impl Session {
    /// Spawns `command` in a job of its own, with the default ceilings.
    pub fn spawn(command: &mut Command) -> Result<Self> {
        Self::spawn_with_scrollback(command, DEFAULT_SCROLLBACK_LINES, DEFAULT_SCROLLBACK_BYTES)
    }

    /// Spawns an argv whose [`SESSION_PLACEHOLDER`] expands to this session's
    /// own id, so N sessions started from one command line can differ.
    ///
    /// **Neither side learns about the other, which is the point.** A caller
    /// wanting a hundred sessions to hold a hundred directories cannot ask the
    /// program to work it out — [a workload that takes a COGGY-specific path
    /// stops being evidence](../../workloads/README.md#the-contract) — and the
    /// daemon may not know what that caller calls its scratch. A placeholder
    /// the daemon expands generically leaves the caller writing an ordinary
    /// path and the program receiving one.
    ///
    /// Expansion happens here rather than in the caller because this is the
    /// only place the id exists before the process does.
    pub fn spawn_template(argv: &[String], capacity: usize, byte_capacity: usize) -> Result<Self> {
        let id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        let expanded = expand(argv, id)?;
        let (program, args) = expanded
            .split_first()
            .context("no command to start a session from")?;
        let mut command = Command::new(program);
        command.args(args);
        Self::spawn_as(id, &mut command, capacity, byte_capacity)
    }

    /// Spawns `command`, puts it in a job of its own, and returns the pair.
    ///
    /// Both ceilings are taken, because either alone leaves the other
    /// unbounded.
    ///
    /// The child is assigned after spawning rather than before, which leaves a
    /// window where it is running and unowned. Closing that window needs
    /// `CREATE_SUSPENDED` and a resume, which needs unsafe — so it stays open
    /// and is named here rather than hidden. It is microseconds wide and the
    /// session cannot have spawned a tree inside it.
    pub fn spawn_with_scrollback(
        command: &mut Command,
        capacity: usize,
        byte_capacity: usize,
    ) -> Result<Self> {
        let id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        Self::spawn_as(id, command, capacity, byte_capacity)
    }

    /// The spawn itself, once an identity has been settled on.
    ///
    /// Separate so that [`Session::spawn_template`] can expand a placeholder
    /// against the id *before* the process exists, which is the only order
    /// that works.
    fn spawn_as(
        id: u64,
        command: &mut Command,
        capacity: usize,
        byte_capacity: usize,
    ) -> Result<Self> {
        // Piped rather than inherited, because a session nobody drains fills
        // its pipe and blocks — which reads as a slow session rather than a
        // stuck one, and is the failure condition 3 exists to catch.
        command.env(SESSION_ID_VAR, id.to_string());
        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut info = win32job::ExtendedLimitInfo::new();
        info.limit_kill_on_job_close();
        let job = win32job::Job::create_with_limit_info(&info)
            .map_err(|e| anyhow::anyhow!("creating a job object: {e}"))?;

        let mut child = command.spawn().context("spawning the session")?;

        // Membership is inherited downward, so everything this child starts
        // lands in the same job without being told to.
        //
        // **The failure path has to kill, and this is the one place it must.**
        // `std::process::Child` does not kill on drop, so returning the error
        // straight through would leave the session running with no owner —
        // the exact straggler this type exists to prevent, produced by the
        // type itself. Between the spawn above and here it belongs to nobody,
        // which is why the kill is explicit rather than left to a destructor.
        if let Err(e) = job.assign_process(handle_of(&child)) {
            let killed = child.kill().is_ok();
            let _ = child.wait();
            anyhow::bail!(
                "assigning the session to its job: {e} — the session was {} rather than left unowned",
                if killed { "killed" } else { "already gone" }
            );
        }

        let scrollback = Arc::new(Mutex::new(Scrollback::new(capacity, byte_capacity)));
        let mut drains = Vec::new();
        if let Some(out) = child.stdout.take() {
            drains.push(drain(Reader::Out(out), Arc::clone(&scrollback)));
        }
        if let Some(err) = child.stderr.take() {
            drains.push(drain(Reader::Err(err), Arc::clone(&scrollback)));
        }

        Ok(Self {
            id,
            child,
            job: Some(job),
            drains,
            scrollback,
        })
    }

    /// This session's identity, which outlives the pid and is never reused.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// The root process id, for asking the operating system about it.
    ///
    /// Distinct from [`Session::id`] on purpose: this one is the kernel's
    /// and may be handed to something else once the session is gone.
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Whether anything in this session's job is still running.
    ///
    /// **Asked of the job rather than the root**, because G0 measured that
    /// those are different questions: fifty sessions whose roots had been
    /// killed left fifty children running. A supervisor that reads the root
    /// would call those slots free and hand them out again.
    pub fn status(&mut self) -> Status {
        let code = match self.child.try_wait() {
            Ok(Some(exit)) => exit.code(),
            Ok(None) => return Status::Running,
            // The root is unreadable, but the job may still hold its tree.
            Err(_) => None,
        };
        match self.job.as_ref().map(win32job::Job::query_process_id_list) {
            Some(Ok(pids)) if pids.is_empty() => Status::Exited { code },
            Some(Ok(_)) => Status::Running,
            _ => Status::Unknown,
        }
    }

    /// What the session has said, under its ceiling.
    pub fn scrollback(&self) -> std::sync::MutexGuard<'_, Scrollback> {
        self.scrollback.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// The variable a session finds itself by, following the `CMUX_WORKSPACE_ID`
/// and `CMUX_SURFACE_ID` precedent [the contracts bind us
/// to](../../docs/PLAN.md#fixed-contracts).
pub const SESSION_ID_VAR: &str = "COGGY_SESSION_ID";

/// Hands out session identities.
///
/// **Not the pid, and the reason is measured.** Windows reuses process ids,
/// and the instrument already carries a note about what that costs: a
/// remembered set of pids silently deletes a session Windows later handed one
/// of those numbers to. A supervisor whose identity is a pid would do worse —
/// hand a new process the dead session's slot and keep counting.
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

/// Whether a session is still running, and nothing else.
///
/// **The daemon knows only whether a session is alive.** Retry, repair and
/// verification verdicts belong to the harness and never enter here — [a
/// fixed contract](../../docs/PLAN.md#fixed-contracts) rather than a
/// simplification, so this type has no room to grow one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Something in the session's job is still running.
    Running,
    /// The job is empty. The code is the root's, when it was still there to
    /// be asked; a session whose root exited before its children finished
    /// reports `None` rather than inventing one.
    Exited { code: Option<i32> },
    /// The job could not be read, so nothing here is a fact about the
    /// session. Distinct from `Exited`, because reporting a session gone on
    /// the strength of a failed query is how a supervisor loses one.
    Unknown,
}

/// Which stream a drain is reading.
///
/// Both land in one scrollback, in the order they arrive, because a reader
/// asking what a session said wants what it said.
/// The bytes a line ends with, named because writing them as literals
/// here has been collapsed by tooling twice.
const NEWLINE: u8 = 10;
const CARRIAGE_RETURN: u8 = 13;

enum Reader {
    Out(ChildStdout),
    Err(ChildStderr),
}

/// Reads one stream to end-of-file, recording every line.
///
/// A lossy decode rather than a failure: a session that emits one invalid byte
/// has not stopped being worth reading, and dropping the whole line would look
/// exactly like the gap condition 3 reports.
///
/// **An error is not end-of-file, and reading it as one is how output gets
/// dropped here.** These two arms shared a branch: a failed read returned from
/// the thread exactly as a clean EOF does, so everything the session wrote
/// afterwards went nowhere and nothing recorded that it had. That is [gate M1's
/// third condition](../../ROADMAP.md#m1--headless-daemon) failing silently in
/// the one place able to notice — a pipe does not lose data, it blocks, so the
/// only way a line disappears between `write` and `read` is if this loop stops
/// asking. Counted now, so the condition can be answered instead of called
/// unmeasurable.
fn drain(reader: Reader, into: Arc<Mutex<Scrollback>>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buf: Box<dyn BufRead> = match reader {
            Reader::Out(out) => Box::new(BufReader::new(out)),
            Reader::Err(err) => Box::new(BufReader::new(err)),
        };
        let mut line = Vec::new();
        loop {
            line.clear();
            match read_line_capped(&mut buf, &mut line) {
                Ok(0) => return,
                Err(_) => {
                    into.lock().unwrap_or_else(|e| e.into_inner()).fail_read();
                    return;
                }
                Ok(_) => {
                    while matches!(line.last(), Some(&NEWLINE | &CARRIAGE_RETURN)) {
                        line.pop();
                    }
                    let text = String::from_utf8_lossy(&line).into_owned();
                    into.lock().unwrap_or_else(|e| e.into_inner()).push(text);
                }
            }
        }
    })
}

/// Reads one line, keeping at most [`MAX_LINE_BYTES`] of it.
///
/// **The cap has to be here rather than on the way into the scrollback.**
/// `read_until` grows its buffer until it meets a newline, so a session
/// emitting a gigabyte without one has already taken the daemon down before
/// anything downstream gets a chance to trim it. Truncating what is kept is
/// not a memory ceiling; refusing to hold it in the first place is.
///
/// Bytes past the cap are consumed and discarded rather than left in the
/// pipe, because leaving them would stall the session — the failure this
/// whole drain exists to avoid.
fn read_line_capped(buf: &mut dyn BufRead, out: &mut Vec<u8>) -> std::io::Result<usize> {
    let mut total = 0usize;
    loop {
        let available = match buf.fill_buf() {
            Ok(&[]) => return Ok(total),
            Ok(bytes) => bytes,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        let (chunk, done) = match available.iter().position(|b| *b == NEWLINE) {
            Some(at) => (&available[..=at], true),
            None => (available, false),
        };
        let room = MAX_LINE_BYTES.saturating_sub(out.len());
        out.extend_from_slice(&chunk[..room.min(chunk.len())]);
        let taken = chunk.len();
        buf.consume(taken);
        total += taken;
        if done {
            return Ok(total);
        }
    }
}

impl Drop for Session {
    /// Ends the tree, then waits for the readers it just unblocked.
    ///
    /// The order is the whole of it. Dropping the job kills everything in it,
    /// which closes the write ends, which is the only thing that brings the
    /// drains to end-of-file — join them first and this blocks for as long as
    /// the session would have run. `Vec<JoinHandle>` detaches on its own, so
    /// leaving it to the default would let a reader outlive the session it
    /// reads, which is this crate's own failure wearing a thread.
    fn drop(&mut self) {
        drop(self.job.take());
        let _ = self.child.wait();
        for drain in self.drains.drain(..) {
            let _ = drain.join();
        }
    }
}

/// The child's process handle as the raw integer `win32job` wants.
///
/// `AsRawHandle` is the documented way across this boundary and needs no
/// unsafe of ours; the cast is the one the crate's own signature asks for.
fn handle_of(child: &Child) -> isize {
    use std::os::windows::io::AsRawHandle;
    child.as_raw_handle() as isize
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// Both tests count every `ping.exe` on the machine, so they cannot run
    /// beside each other — and `cargo test` runs in parallel by default. A
    /// test that only passes when it happens to go first is not a test.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

    /// Whether a pid is still alive, asked of the operating system rather than
    /// of our own bookkeeping — the whole question here is what survives us.
    fn alive(pid: u32) -> bool {
        let out = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .expect("tasklist");
        String::from_utf8_lossy(&out.stdout).contains(&pid.to_string())
    }

    /// How many `ping.exe` are alive right now.
    ///
    /// The child is counted rather than the root because **the root proves
    /// nothing**: a plain kill removes it too. What separates job termination
    /// from killing a process is whether the thing underneath it goes, and
    /// that is where fifty stragglers came from.
    fn pings() -> usize {
        let out = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq PING.EXE", "/NH"])
            .output()
            .expect("tasklist");
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| l.to_ascii_lowercase().contains("ping.exe"))
            .count()
    }

    #[test]
    fn dropping_a_session_takes_the_tree_and_not_only_the_root() {
        let _serial = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        // The shape that leaked fifty stragglers: a shell that outlives its
        // own start and a child doing the work. `ping` is the child because it
        // is always present, runs for seconds, and is not something we ship.
        let mut command = Command::new("cmd");
        command.args(["/c", "ping -n 30 127.0.0.1 >nul"]);

        let before = pings();
        let session = Session::spawn(&mut command).expect("spawn");
        // The kernel is being asked, so the kernel's number is what it wants.
        let root = session.pid();

        // Give the shell time to start its child, or the test proves nothing:
        // an empty tree is trivially reclaimed.
        std::thread::sleep(std::time::Duration::from_millis(900));
        assert!(alive(root), "the session should still be running");
        assert_eq!(
            pings(),
            before + 1,
            "the shell should have started a child for the job to have to reach"
        );

        drop(session);
        std::thread::sleep(std::time::Duration::from_millis(500));

        assert!(
            !alive(root),
            "dropping the job should have taken the shell with it"
        );
        assert_eq!(
            pings(),
            before,
            "and the child it spawned — this is the assertion the root cannot make"
        );
    }

    /// The error path leaves nothing running, which is the claim that matters
    /// because `Child` does not kill on drop.
    ///
    /// Forcing `assign_process` to fail from outside is not available, so this
    /// exercises the same sequence directly: spawn, decide the session cannot
    /// be owned, kill it. What it guards is that the decision kills rather
    /// than returning and letting a destructor not do it.
    #[test]
    fn a_session_that_cannot_be_owned_is_killed_rather_than_leaked() {
        let _serial = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        let before = pings();
        let mut child = Command::new("cmd")
            .args(["/c", "ping -n 30 127.0.0.1 >nul"])
            .spawn()
            .expect("spawn");
        std::thread::sleep(std::time::Duration::from_millis(900));
        assert_eq!(pings(), before + 1, "the child should be running");

        // The branch `spawn` takes when a job cannot take the session.
        let killed = child.kill().is_ok();
        let _ = child.wait();
        assert!(killed);
        std::thread::sleep(std::time::Duration::from_millis(500));

        // The root is gone; the grandchild is not, and that is exactly why the
        // job has to be created before the spawn rather than after a failure.
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "ping.exe"])
            .output();
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert_eq!(pings(), before, "nothing of this test may outlive it");
    }

    #[test]
    fn a_session_records_what_it_says_and_says_how_much_it_read() {
        let _serial = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        // Four lines out of a ceiling of two: the gate cares that all four
        // were read, the ceiling cares that only two are kept, and one
        // counter could not say both.
        let mut command = Command::new("cmd");
        command.args(["/c", "echo one& echo two& echo three& echo four"]);

        let session = Session::spawn_with_scrollback(&mut command, 2, DEFAULT_SCROLLBACK_BYTES)
            .expect("spawn");
        std::thread::sleep(std::time::Duration::from_millis(900));

        let back = session.scrollback();
        assert_eq!(back.read(), 4, "every line the session emitted was read");
        assert_eq!(back.retained(), 2, "and the ceiling held");
        assert_eq!(back.evicted(), 2, "the difference is policy, not a gap");
        assert_eq!(back.tail(2), vec!["three", "four"]);
    }

    #[test]
    fn a_session_reads_as_running_while_its_child_outlives_its_root() {
        let _serial = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        // The exact shape G0 measured: a shell that exits while the thing it
        // started keeps going. Asked of the root this session is finished;
        // asked of the job it is not, and the job is right.
        let before = pings();
        let mut command = Command::new("cmd");
        command.args(["/c", "start /b ping -n 30 127.0.0.1"]);
        let mut session = Session::spawn(&mut command).expect("spawn");

        std::thread::sleep(std::time::Duration::from_millis(1200));
        assert_eq!(pings(), before + 1, "the grandchild should be running");
        assert!(
            !alive(session.pid()),
            "and the root should already have exited, or this proves nothing"
        );
        assert_eq!(
            session.status(),
            Status::Running,
            "a slot is not free while anything in its job is alive"
        );

        drop(session);
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(pings(), before, "and dropping still takes the tree");
    }

    #[test]
    fn a_session_can_find_itself_by_the_variable_the_contract_names() {
        let _serial = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        let mut command = Command::new("cmd");
        command.args(["/c", "echo %COGGY_SESSION_ID%"]);
        let session = Session::spawn(&mut command).expect("spawn");
        let id = session.id();
        std::thread::sleep(std::time::Duration::from_millis(900));

        let back = session.scrollback();
        assert_eq!(
            back.tail(1),
            vec![id.to_string()],
            "the child has to be able to say where it lives"
        );
    }

    #[test]
    fn two_sessions_never_share_an_identity_even_when_one_has_gone() {
        let _serial = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        // The hazard this identity exists against: a pid is the operating
        // system's to reuse, and a supervisor keyed on one would hand a dead
        // session's slot to whatever inherited the number.
        let mut first = Command::new("cmd");
        first.args(["/c", "exit 0"]);
        let a = Session::spawn(&mut first).expect("spawn");
        let a_id = a.id();
        drop(a);

        let mut second = Command::new("cmd");
        second.args(["/c", "exit 0"]);
        let b = Session::spawn(&mut second).expect("spawn");
        assert_ne!(a_id, b.id(), "an identity is never handed out twice");
    }

    #[test]
    fn a_finished_session_reads_as_exited_with_its_code() {
        let _serial = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        let mut command = Command::new("cmd");
        command.args(["/c", "exit 3"]);
        let mut session = Session::spawn(&mut command).expect("spawn");
        std::thread::sleep(std::time::Duration::from_millis(900));
        assert_eq!(session.status(), Status::Exited { code: Some(3) });
    }

    #[test]
    fn a_line_without_an_end_cannot_grow_past_the_ceiling() {
        // The shape that makes a line-count ceiling stop being a memory
        // ceiling: bytes with no newline in them. Read through a reader that
        // never ends so the cap is what stops it, not the input.
        let flood = vec![b'x'; MAX_LINE_BYTES * 3];
        let mut buf: Box<dyn BufRead> = Box::new(BufReader::new(&flood[..]));
        let mut line = Vec::new();
        let consumed = read_line_capped(&mut buf, &mut line).expect("read");

        assert_eq!(
            consumed,
            MAX_LINE_BYTES * 3,
            "everything has to leave the pipe or the session stalls"
        );
        assert_eq!(
            line.len(),
            MAX_LINE_BYTES,
            "and only the ceiling may be held"
        );
    }

    #[test]
    fn a_line_that_ends_is_kept_whole_and_stripped() {
        let input = b"first line\r\nsecond";
        let mut buf: Box<dyn BufRead> = Box::new(BufReader::new(&input[..]));
        let mut line = Vec::new();
        read_line_capped(&mut buf, &mut line).expect("read");
        assert_eq!(line.last(), Some(&NEWLINE), "the terminator comes through");

        // What the drain does with it, so the assertion covers the pair.
        while matches!(line.last(), Some(&NEWLINE | &CARRIAGE_RETURN)) {
            line.pop();
        }
        assert_eq!(String::from_utf8_lossy(&line), "first line");
    }

    #[test]
    fn dropping_a_session_does_not_leave_its_readers_behind() {
        let _serial = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        // A session that never exits on its own: if drop joined the drains
        // before ending the tree, this would hang rather than fail.
        let before = pings();
        let mut command = Command::new("cmd");
        command.args(["/c", "ping -n 30 127.0.0.1"]);
        let session = Session::spawn(&mut command).expect("spawn");
        std::thread::sleep(std::time::Duration::from_millis(900));

        let at = std::time::Instant::now();
        drop(session);
        assert!(
            at.elapsed() < std::time::Duration::from_secs(5),
            "drop waited on a reader it had not yet unblocked"
        );
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(pings(), before, "and the tree went with it");
    }

    /// The negative control: the same tree, killed the way a supervisor
    /// without a job would kill it.
    ///
    /// Without this the test above cannot claim the job is what did the work,
    /// since a passing assertion proves nothing when the alternative also
    /// passes. Fifty stragglers say it does not.
    #[test]
    fn killing_the_root_alone_leaves_the_child_behind() {
        let _serial = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        let before = pings();
        let mut root = Command::new("cmd")
            .args(["/c", "ping -n 30 127.0.0.1 >nul"])
            .spawn()
            .expect("spawn");

        std::thread::sleep(std::time::Duration::from_millis(900));
        assert_eq!(pings(), before + 1, "the child should be running");

        root.kill().expect("kill");
        let _ = root.wait();
        std::thread::sleep(std::time::Duration::from_millis(500));

        assert_eq!(
            pings(),
            before + 1,
            "killing the root leaves the child, which is the whole reason for the job"
        );

        // Leave nothing behind for the next test to miscount.
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "ping.exe"])
            .output();
    }
}
