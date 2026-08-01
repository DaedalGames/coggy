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
        let session = Session::spawn(command)?;
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
        std::thread::sleep(std::time::Duration::from_millis(900));

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
        std::thread::sleep(std::time::Duration::from_millis(1500));
        assert!(pool.is_empty());
        assert_eq!(
            waitfors(),
            waiting_before - BATCH,
            "clearing has to take the processes, not just the bookkeeping"
        );
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
