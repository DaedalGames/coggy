// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading the machine on a fixed interval.
//!
//! Refreshing and sampling are separate calls on purpose. One refresh walks
//! every process on the machine, so a ramp holding a hundred sessions has to
//! pay for that once per tick rather than once per session — otherwise the
//! instrument's own cost grows with the number it is measuring, which is the
//! one shape a scaling benchmark cannot have.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::session::Output;
use crate::tree::{Attribution, ProcessSample, SessionTree};

/// Windows Defender's scanning service.
const DEFENDER_PROCESS: &str = "MsMpEng.exe";

/// One instant of one session's cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub t_ms: u64,
    pub rss_bytes: u64,
    pub processes: usize,
    pub pseudoconsoles: usize,
    pub cpu_percent: f32,
    /// `None` when Defender is not running, which is itself worth recording.
    pub defender_cpu_percent: Option<f32>,
    pub defender_rss_bytes: Option<u64>,
    pub available_memory_bytes: u64,
    pub output_bytes: u64,
    /// Lines the session has written, which is its own count of work done.
    pub work_units: u64,
    /// Every member, so the artifact can be re-read for which process names
    /// hold what share without taking the run again.
    pub members: Vec<ProcessSample>,
}

pub struct Sampler {
    sys: System,
    /// Defender's pid, so it can be refreshed by name only once.
    defender: Option<Pid>,
    /// Why the sampling thread could not be raised above the sessions, when it
    /// could not. Carried into reports, since a starved sampler produces
    /// numbers that describe the observer.
    unprioritised_reason: Option<String>,
}

impl Sampler {
    /// Builds a sampler on the calling thread, raised above what it will watch,
    /// with its CPU counters already primed.
    ///
    /// sysinfo reports usage as the delta between two refreshes, so a sampler
    /// used immediately would report every process at zero.
    pub fn new() -> Self {
        let mut sampler = Self {
            sys: System::new(),
            defender: None,
            unprioritised_reason: raise_current_thread().err(),
        };
        if let Some(reason) = &sampler.unprioritised_reason {
            eprintln!("warning: the sampler runs at ordinary priority — {reason}");
        }
        sampler.refresh(None);
        sampler
    }

    /// `None` when the sampling thread outranks the sessions, as it should.
    pub fn unprioritised_reason(&self) -> Option<&str> {
        self.unprioritised_reason.as_deref()
    }

    /// Re-reads the machine. Call once per tick, however many sessions are up.
    ///
    /// `tracked` is the exact set to refresh, which is what keeps a tick from
    /// costing more as the ramp climbs. Reading the whole process table
    /// instead took 98 ms at one session, 150 ms at ten, and **eighty seconds
    /// at twenty-five** — the instrument became the bottleneck and reported the
    /// collapse as the machine's. `None` falls back to the full table, which
    /// the parent-walk membership has no way to avoid.
    pub fn refresh(&mut self, tracked: Option<&[Pid]>) {
        self.sys.refresh_memory();
        let kind = ProcessRefreshKind::nothing().with_cpu().with_memory();

        match (tracked, self.defender) {
            (Some(pids), Some(defender)) => {
                let mut list = Vec::with_capacity(pids.len() + 1);
                list.extend_from_slice(pids);
                list.push(defender);
                self.sys
                    .refresh_processes_specifics(ProcessesToUpdate::Some(&list), true, kind);
                // Defender restarting changes its pid, and a targeted refresh
                // would never notice. Losing it drops back to a full walk on
                // the next tick rather than silently reporting no scanning.
                if self.sys.process(defender).is_none() {
                    self.defender = None;
                }
            }
            _ => {
                self.sys
                    .refresh_processes_specifics(ProcessesToUpdate::All, true, kind);
                self.defender = self
                    .sys
                    .processes()
                    .iter()
                    .find(|(_, p)| p.name().eq_ignore_ascii_case(DEFENDER_PROCESS))
                    .map(|(pid, _)| *pid);
            }
        }
    }

    /// Samples one session against the current refresh.
    pub fn sample(&self, tree: &mut SessionTree, output: &Output, elapsed: Duration) -> Sample {
        let members = tree.sample(&self.sys);
        let defender = self.defender.and_then(|pid| self.sys.process(pid));

        Sample {
            t_ms: elapsed.as_millis() as u64,
            rss_bytes: members.iter().map(|m| m.rss_bytes).sum(),
            processes: members.len(),
            pseudoconsoles: members
                .iter()
                .filter(|m| m.attribution == Attribution::Pseudoconsole)
                .count(),
            cpu_percent: members.iter().map(|m| m.cpu_percent).sum(),
            defender_cpu_percent: defender.map(|p| p.cpu_usage()),
            defender_rss_bytes: defender.map(|p| p.memory()),
            available_memory_bytes: self.sys.available_memory(),
            output_bytes: output.total(),
            work_units: output.units(),
            members,
        }
    }

    /// Refreshes and samples in one call, for a run watching a single session.
    pub fn take(&mut self, tree: &mut SessionTree, output: &Output, elapsed: Duration) -> Sample {
        let tracked = tree.known_pids();
        self.refresh(tracked.as_deref());
        self.sample(tree, output, elapsed)
    }

    /// Watches Defender alone for a while, in cores.
    ///
    /// Defender is one machine-wide process, so a run that attributes all of
    /// its CPU to the session being measured is also attributing everything
    /// else on the machine. Taking this immediately before a half is what makes
    /// that visible: a baseline near zero says the machine was quiet, and one
    /// that is not says the result underneath it is describing the room.
    ///
    /// `None` when Defender is not running.
    pub fn watch_defender(&mut self, duration: Duration, interval: Duration) -> Option<f64> {
        let started = std::time::Instant::now();
        let mut readings = Vec::new();

        while started.elapsed() < duration {
            std::thread::sleep(interval);
            self.refresh(Some(&[]));
            let pid = self.defender?;
            readings.push(f64::from(self.sys.process(pid)?.cpu_usage()) / 100.0);
        }
        (!readings.is_empty()).then(|| readings.iter().sum::<f64>() / readings.len() as f64)
    }

    /// Waits for a sample's processes to exit on their own, up to `grace`.
    ///
    /// Closing a pseudoconsole asks its host to leave, and at small counts they
    /// all have by the time anything looks. At larger ones they are still on
    /// their way out — and terminating a process that is already terminating
    /// costs the better part of a second each, serially, which turned a rung's
    /// teardown into a minute. Waiting is what killing was standing in for.
    ///
    /// Returns how long it waited and how many were still alive at the end.
    pub fn wait_for_exit(&mut self, sample: &Sample, grace: Duration) -> (u64, usize) {
        let pids: Vec<Pid> = sample
            .members
            .iter()
            .map(|m| Pid::from_u32(m.pid))
            .collect();
        let started = std::time::Instant::now();

        loop {
            self.refresh(Some(&pids));
            let alive = pids
                .iter()
                .filter(|p| self.sys.process(**p).is_some())
                .count();
            if alive == 0 || started.elapsed() >= grace {
                return (started.elapsed().as_millis() as u64, alive);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Terminates whatever a session left behind after its root was killed.
    ///
    /// Killing the root does not take its children with it, and the last
    /// sample is the list of what to finish off. Returns how long the refresh
    /// and the kills took separately, because they are very different costs
    /// and a rung's teardown is dominated by exactly one of them.
    pub fn reap(&mut self, last: Option<&Sample>) -> (u64, u64) {
        let Some(sample) = last else {
            return (0, 0);
        };
        let pids: Vec<Pid> = sample
            .members
            .iter()
            .map(|m| Pid::from_u32(m.pid))
            .collect();

        let at = std::time::Instant::now();
        self.refresh(Some(&pids));
        let refresh_ms = at.elapsed().as_millis() as u64;

        let at = std::time::Instant::now();
        for member in &sample.members {
            if let Some(process) = self.sys.process(Pid::from_u32(member.pid)) {
                process.kill();
            }
        }
        (refresh_ms, at.elapsed().as_millis() as u64)
    }

    /// Memory the operating system currently reports as free for new work.
    pub fn available_memory(&self) -> u64 {
        self.sys.available_memory()
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new()
    }
}

/// Puts the sampling thread ahead of the sessions it measures.
///
/// Without this, twenty-five sessions that never yield leave the sampler
/// unscheduled for fifteen seconds at a time, and everything it then reports
/// describes the observer rather than the machine. The thread costs about
/// fifty milliseconds a second and sleeps for the rest, so outranking the
/// sessions takes nothing measurable from them — and what it does take is
/// recorded, since every tick's cost is in the artifact.
///
/// One step below time-critical on purpose. The sampler has no business being
/// the highest-priority thread on someone's machine.
fn raise_current_thread() -> Result<(), String> {
    thread_priority::set_current_thread_priority(desired_priority())
        .map_err(|error| format!("{error:?}"))
}

#[cfg(windows)]
fn desired_priority() -> thread_priority::ThreadPriority {
    thread_priority::ThreadPriority::Os(thread_priority::WinAPIThreadPriority::Highest.into())
}

#[cfg(not(windows))]
fn desired_priority() -> thread_priority::ThreadPriority {
    thread_priority::ThreadPriority::Max
}
