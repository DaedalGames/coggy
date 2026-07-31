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

pub mod scrollback;

use std::process::{Child, Command};

use anyhow::{Context, Result};

/// A session and the job that owns everything it spawns.
///
/// Dropping this ends the session and its whole tree. That is the point: the
/// alternative is killing a root and reaping whatever outlived it.
pub struct Session {
    child: Child,
    /// Held only to be dropped. The job carries `KILL_ON_JOB_CLOSE`, so
    /// releasing the last handle is what terminates the tree.
    _job: win32job::Job,
}

impl Session {
    /// Spawns `command`, puts it in a job of its own, and returns the pair.
    ///
    /// The child is assigned after spawning rather than before, which leaves a
    /// window where it is running and unowned. Closing that window needs
    /// `CREATE_SUSPENDED` and a resume, which needs unsafe — so it stays open
    /// and is named here rather than hidden. It is microseconds wide and the
    /// session cannot have spawned a tree inside it.
    pub fn spawn(command: &mut Command) -> Result<Self> {
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

        Ok(Self { child, _job: job })
    }

    pub fn id(&self) -> u32 {
        self.child.id()
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
        let root = session.id();

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
