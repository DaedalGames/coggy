// coggyd — the session supervisor.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Many sessions, and the accounting that says how many are really running.
//!
//! [Gate M1](../../ROADMAP.md#m1--headless-daemon) asks for a hundred sessions
//! held for an hour. Holding them is one session's problem repeated; **saying
//! how many are held is this file's**, and it is where the three things G0
//! measured can go wrong together:
//!
//! - A session is alive while its job is, not while its root is. Reaping on
//!   the root would free slots that [fifty stragglers were still
//!   using](../../docs/measurements/2026-07-31-141334-the-shell-costs-teardown.md).
//! - A slot is keyed by a session identity, never a pid, because Windows hands
//!   those out again.
//! - Reclaiming takes the tree, or the session count falls while the process
//!   count does not.
//!
//! Admission against a ceiling is [M3](../../ROADMAP.md#m3--resource-governor)
//! and deliberately absent here. This counts; it does not judge.

use std::collections::BTreeMap;
use std::process::Command;

use anyhow::Result;

use crate::{Session, Status};

/// What the pool's sessions have done with their output, summed.
///
/// **Three numbers because the gate means one of them.** A line the daemon
/// never read is a gate failure; a line the scrollback aged out is policy.
/// Reporting one total would let the second hide inside the first, which is
/// why [`Scrollback`](crate::scrollback::Scrollback) keeps them apart — and
/// that separation was invisible outside this process until the pool summed
/// it.
///
/// The daemon cannot compute the *difference* between what a session emitted
/// and what arrived — that subtraction needs the [workload contract's
/// ordinals](../../workloads/README.md#the-contract), which the benchmark
/// holds. **What it can do is name the only way a line goes missing on this
/// side**, which is `failed_reads`: a pipe blocks rather than dropping, so
/// nothing is lost between a session's `write` and this struct unless the
/// reader stops asking. That used to be invisible, because a failed read
/// returned from the drain thread on the same branch as a clean end-of-file.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Output {
    pub read: u64,
    /// Bytes behind those lines, which is the output path's own axis.
    pub read_bytes: u64,
    pub evicted: u64,
    pub truncated: u64,
    /// Streams whose drain gave up on an error rather than reaching EOF.
    ///
    /// **Zero is the gate's third condition holding, and it is a real zero**
    /// rather than an absent measurement — the counter exists for the whole
    /// life of every session, so nothing here is the same as nothing happened.
    pub failed_reads: u64,
}

/// The sessions this daemon is holding.
#[derive(Default)]
pub struct Pool {
    sessions: BTreeMap<u64, Session>,
}

impl Pool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a session and keeps it.
    pub fn spawn(&mut self, command: &mut Command) -> Result<u64> {
        self.keep(Session::spawn(command)?)
    }

    /// Starts a session from an argv whose `${session}` becomes its own id.
    ///
    /// What a caller starting N of them from one command line needs, since
    /// [the workload contract wants each its own
    /// directory](../../workloads/README.md#the-contract) and neither side is
    /// allowed to know the other's naming.
    pub fn spawn_template(&mut self, argv: &[String]) -> Result<u64> {
        self.keep(Session::spawn_template(
            argv,
            crate::DEFAULT_SCROLLBACK_LINES,
            crate::DEFAULT_SCROLLBACK_BYTES,
        )?)
    }

    fn keep(&mut self, session: Session) -> Result<u64> {
        let id = session.id();
        self.sessions.insert(id, session);
        Ok(id)
    }

    /// How many sessions are held, whether or not they are still running.
    ///
    /// Distinct from [`Pool::running`] on purpose: a session that has finished
    /// still occupies memory and a slot until it is reaped, and a supervisor
    /// that conflated the two would admit against a number it had not freed.
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// How many still have something alive in their job.
    pub fn running(&mut self) -> usize {
        self.sessions
            .values_mut()
            .fold(0, |n, s| n + usize::from(s.status() == Status::Running))
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Session> {
        self.sessions.get_mut(&id)
    }

    /// Sums every held session's output counters.
    ///
    /// Held rather than running, deliberately: a session that has exited still
    /// owns the lines it produced, and dropping them at exit would make the
    /// total fall while the run was still going.
    pub fn output(&self) -> Output {
        self.sessions
            .values()
            .fold(Output::default(), |mut total, session| {
                let back = session.scrollback();
                total.read += back.read();
                total.read_bytes += back.read_bytes();
                total.evicted += back.evicted();
                total.truncated += back.truncated();
                total.failed_reads += back.failed_reads();
                total
            })
    }

    /// Drops every session whose job has emptied, and reports how many went.
    ///
    /// **`Unknown` is kept rather than reaped.** A job that could not be read
    /// says nothing about whether its session is running, and freeing a slot
    /// on that reading is how a supervisor hands out a seat someone is in.
    pub fn reap(&mut self) -> usize {
        let mut done = Vec::new();
        for (id, session) in &mut self.sessions {
            if matches!(session.status(), Status::Exited { .. }) {
                done.push(*id);
            }
        }
        for id in &done {
            self.sessions.remove(id);
        }
        done.len()
    }

    /// Ends every session and empties the pool.
    pub fn clear(&mut self) {
        self.sessions.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    // `super` is this module, so the crate-root helper needs naming.
    use crate::wait_until;

    /// How many `waitfor` processes the machine is holding.
    ///
    /// Asked of the operating system rather than of the pool, because the
    /// whole question is whether the pool's count and the machine's agree.
    fn waitfors() -> usize {
        let out = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq WAITFOR.EXE", "/NH"])
            .output()
            .expect("tasklist");
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| l.to_ascii_lowercase().contains("waitfor.exe"))
            .count()
    }

    fn quick() -> Command {
        let mut c = Command::new("cmd");
        c.args(["/c", "exit 0"]);
        c
    }

    /// A session that stays up, using something no other test counts.
    ///
    /// Deliberately not `ping`: the tests in `lib.rs` count every
    /// `ping.exe` on the machine and serialise on a lock this module
    /// cannot see. Sharing the program would make both suites depend on
    /// running order, and a test that passes when it happens to go first
    /// is not a test.
    /// Every waiter gets its own signal name.
    ///
    /// `waitfor` takes a name, and two waiters on the same one collide:
    /// the second exits at once and reads here as a session that never
    /// ran. Isolating the program from the other module was not enough,
    /// because the shared thing moved from the process table to that
    /// program's own namespace.
    static NEXT_SIGNAL: AtomicUsize = AtomicUsize::new(0);

    fn slow() -> Command {
        let n = NEXT_SIGNAL.fetch_add(1, Ordering::Relaxed);
        let mut c = Command::new("waitfor");
        // Alphanumeric: `waitfor` rejects a hyphen in a signal name and
        // exits at once with status 0, which reads here as a session that
        // never ran rather than as a bad argument.
        c.args(["/t".to_string(), "30".to_string(), format!("coggydpool{n}")]);
        c
    }

    #[test]
    fn a_finished_session_is_held_until_it_is_reaped() {
        let mut pool = Pool::new();
        pool.spawn(&mut quick()).expect("spawn");
        assert!(
            wait_until(|| pool.running() == 0),
            "the session never finished"
        );

        // Held and not running are different numbers, and a supervisor that
        // admitted against the first would be counting a seat it had freed.
        assert_eq!(pool.len(), 1, "still occupying a slot");
        assert_eq!(pool.running(), 0, "but nothing of it is alive");

        assert_eq!(pool.reap(), 1);
        assert!(pool.is_empty());
    }

    #[test]
    fn a_running_session_survives_a_reap() {
        let mut pool = Pool::new();
        pool.spawn(&mut slow()).expect("spawn");
        std::thread::sleep(std::time::Duration::from_millis(900));

        assert_eq!(pool.running(), 1);
        assert_eq!(pool.reap(), 0, "a reap may not take a live session");
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn many_sessions_are_counted_and_reclaimed_without_leaking_a_tree() {
        // The gate's shape at a size that fits a test: hold a batch, watch
        // the count, and check that clearing takes every tree rather than
        // every root. Twenty rather than a hundred because this is a
        // correctness check, not the ramp — the ramp is sessionbench's.
        const BATCH: usize = 20;
        let mut pool = Pool::new();
        for _ in 0..BATCH {
            pool.spawn(&mut slow()).expect("spawn");
        }
        std::thread::sleep(std::time::Duration::from_millis(1500));

        assert_eq!(pool.len(), BATCH, "every session kept its slot");
        assert_eq!(pool.running(), BATCH, "and every one is alive");
        assert_eq!(pool.reap(), 0, "a reap may not take a live batch");

        let waiting_before = waitfors();
        assert!(
            waiting_before >= BATCH,
            "the batch should be visible to the operating system, saw {waiting_before}"
        );

        pool.clear();
        assert!(
            wait_until(|| waitfors() == waiting_before - BATCH),
            "clearing never took the processes"
        );
        assert!(pool.is_empty());
        assert_eq!(
            waitfors(),
            waiting_before - BATCH,
            "clearing has to take the processes, not just the bookkeeping"
        );
    }

    #[test]
    fn the_pool_sums_what_its_sessions_read_and_keeps_it_after_they_exit() {
        // The number a benchmark subtracts from what its workload emitted, so
        // a pool that summed only the running half would report the remainder
        // as dropped output — the gate's failure — when it was bookkeeping.
        let mut pool = Pool::new();
        for _ in 0..2 {
            let mut c = Command::new("cmd");
            c.args(["/c", "echo 1&echo 2&echo 3"]);
            pool.spawn(&mut c).expect("spawn");
        }

        let mut seen = Output::default();
        for _ in 0..40 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            seen = pool.output();
            if seen.read >= 6 && pool.running() == 0 {
                break;
            }
        }

        assert_eq!(seen.read, 6, "two sessions of three lines each");
        assert_eq!(seen.evicted, 0, "well inside the default capacity");
        assert_eq!(seen.truncated, 0, "no line came near the cap");

        assert_eq!(pool.running(), 0, "both finished");
        assert_eq!(
            pool.output().read,
            6,
            "a session that exited still owns the lines it produced"
        );
    }

    #[test]
    fn one_command_line_gives_each_session_its_own_path() {
        // The whole reason the placeholder exists: N sessions from one argv,
        // each writing somewhere of its own. Without it a ramp reproduces the
        // shared-directory defect, where ten sessions deleted each other's
        // files and the run read as one session going fast.
        let dir = std::env::temp_dir().join(format!(
            "coggyd-template-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("a clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("scratch");
        let pattern = dir.join("s${session}.txt");

        let mut pool = Pool::new();
        let argv: Vec<String> = vec![
            "cmd".into(),
            "/c".into(),
            format!("echo hello> {}", pattern.display()),
        ];
        for _ in 0..3 {
            pool.spawn_template(&argv).expect("spawn");
        }
        // The same load sensitivity as the reaping test: three redirections
        // finish in well under a second idle and not at all reliably on a
        // busy box, and asserting on the directory before they land reads as
        // the placeholder having failed to expand.
        assert!(
            wait_until(|| {
                std::fs::read_dir(&dir).map_or(0, |d| d.flatten().count()) == 3
                    && pool.running() == 0
            }),
            "the three sessions never wrote"
        );

        let written: Vec<_> = std::fs::read_dir(&dir)
            .expect("readable")
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            written.len(),
            3,
            "three sessions, three files, saw {written:?}"
        );
        assert!(
            !written.iter().any(|n| n.contains('$')),
            "the placeholder was expanded, not passed through: {written:?}"
        );

        pool.clear();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_placeholder_nobody_defined_refuses_the_spawn() {
        // The load-bearing half. Left as written, an unknown name would hand
        // every session the same path and say nothing — the failure this
        // exists to prevent, arriving silently.
        let mut pool = Pool::new();
        let argv: Vec<String> = vec!["cmd".into(), "/c".into(), "echo ${sesion}".into()];
        let err = pool
            .spawn_template(&argv)
            .expect_err("a typo is not a path");
        assert!(
            format!("{err:#}").contains("sesion"),
            "the error names what it could not expand: {err:#}"
        );
        assert!(pool.is_empty(), "and nothing was started");
    }

    #[test]
    fn every_session_gets_its_own_slot_even_when_spawned_from_one_command() {
        let mut pool = Pool::new();
        let a = pool.spawn(&mut quick()).expect("spawn");
        let b = pool.spawn(&mut quick()).expect("spawn");
        assert_ne!(a, b);
        assert_eq!(pool.len(), 2, "two identities, two slots");
    }
}
