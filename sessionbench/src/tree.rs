// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! What one session actually costs, counted across every process it drags
//! along.
//!
//! Membership comes from a Windows job object, which the kernel maintains for
//! us: a process joins its parent's job automatically, so the session, whatever
//! it spawns, and the pseudoconsole host all land in one set without anything
//! here walking a process table. The job is the wheel not to carve — hand-rolled
//! membership has to guess at pid reuse, and loses a whole subtree whenever an
//! intermediate shell exits and orphans its children.
//!
//! Job assignment can still fail. Cargo, which uses job objects for the same
//! reason, documents that nesting is unsupported on older installs and that its
//! own test suite silently skipped tests on builders already inside someone
//! else's job. So a parent-walking fallback stays, and every report records
//! which of the two produced its numbers.
//!
//! Prior art: `win32job` (MIT OR Apache-2.0) for the safe wrapper,
//! `rust-lang/cargo`'s `util/job.rs` for the containment pattern, and
//! `psrecord` (BSD) for the shape of "sample a process and its children on an
//! interval".

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System};

/// Process names Windows uses for the pseudoconsole host.
///
/// `OpenConsole.exe` is what Windows Terminal ships and prefers;
/// `conhost.exe` is the in-box fallback. Either one is the resident cost that
/// PLAN's first decision proposes to stop paying by defaulting to pipes.
const PSEUDOCONSOLE_NAMES: [&str; 2] = ["conhost.exe", "openconsole.exe"];

/// Where a sample's process list came from.
///
/// Recorded per run, because the two are not equally trustworthy and a reader
/// comparing two reports needs to know which they are holding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Membership {
    /// The kernel's own list, via `QueryInformationJobObject`.
    JobObject,
    /// Derived by following parent pids, with start times used as identity.
    ParentWalk,
}

/// Why a process is counted against the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Attribution {
    /// The spawned command itself.
    Root,
    /// Started by the session, directly or otherwise.
    Descendant,
    /// A pseudoconsole host started on the session's behalf.
    ///
    /// `CreatePseudoConsole` parents the host to whoever called it, so under a
    /// PTY this is a sibling of the session rather than a child. It is still
    /// the session's cost, and counting it anywhere else would leave the
    /// conhost axis measuring nothing.
    Pseudoconsole,
}

/// One process at one instant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSample {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub rss_bytes: u64,
    /// Percent of a single core, as sysinfo reports it.
    pub cpu_percent: f32,
    pub attribution: Attribution,
}

/// A job object joined before any session exists.
///
/// Separate from [`SessionTree`] so the ordering cannot be got wrong: a process
/// only inherits job membership from its parent at creation, so joining after
/// spawning would leave the session outside the job it is supposed to be in.
pub struct ArmedTree {
    observer: Pid,
    job: Option<win32job::Job>,
    /// Processes already in the job before the session started.
    ///
    /// The provenance and Defender queries shell out, and although both have
    /// exited by the time sampling begins, excluding them by identity beats
    /// relying on that.
    baseline: HashSet<usize>,
    /// Why the job is unavailable, when it is.
    pub fallback_reason: Option<String>,
}

impl ArmedTree {
    /// Creates a job and joins it, falling back to parent-walking if either
    /// step fails.
    pub fn arm(observer: Pid) -> Self {
        match Self::join_job() {
            Ok((job, baseline)) => Self {
                observer,
                job: Some(job),
                baseline,
                fallback_reason: None,
            },
            Err(reason) => Self {
                observer,
                job: None,
                baseline: HashSet::new(),
                fallback_reason: Some(reason),
            },
        }
    }

    fn join_job() -> Result<(win32job::Job, HashSet<usize>), String> {
        let job = win32job::Job::create().map_err(|e| format!("creating a job object: {e}"))?;
        job.assign_current_process()
            .map_err(|e| format!("joining the job object: {e}"))?;
        let baseline = job
            .query_process_id_list()
            .map_err(|e| format!("reading the job's process list: {e}"))?
            .into_iter()
            .collect();
        Ok((job, baseline))
    }

    pub fn membership(&self) -> Membership {
        match self.job {
            Some(_) => Membership::JobObject,
            None => Membership::ParentWalk,
        }
    }

    /// Binds the armed job to the session that has now been spawned.
    pub fn attach(self, root: Pid) -> SessionTree {
        SessionTree {
            root,
            observer: self.observer,
            job: self.job,
            baseline: self.baseline,
            walked: HashMap::new(),
        }
    }
}

/// The set of processes belonging to one session, sampled over time.
pub struct SessionTree {
    root: Pid,
    observer: Pid,
    job: Option<win32job::Job>,
    baseline: HashSet<usize>,
    /// Start times of members found by walking, keyed by pid. Unused when the
    /// job object is available.
    walked: HashMap<Pid, u64>,
}

impl SessionTree {
    /// Samples every process currently belonging to the session.
    pub fn sample(&mut self, sys: &System) -> Vec<ProcessSample> {
        let members = if self.job.is_some() {
            self.job_members()
        } else {
            self.walked_members(sys)
        };

        let mut samples: Vec<ProcessSample> = members
            .into_iter()
            .filter_map(|pid| {
                let process = sys.process(pid)?;
                let name = process.name().to_string_lossy().into_owned();
                let attribution = if pid == self.root {
                    Attribution::Root
                } else if is_pseudoconsole(&name) {
                    Attribution::Pseudoconsole
                } else {
                    Attribution::Descendant
                };

                Some(ProcessSample {
                    pid: pid.as_u32(),
                    parent_pid: process.parent().map(|p| p.as_u32()),
                    name,
                    rss_bytes: process.memory(),
                    cpu_percent: process.cpu_usage(),
                    attribution,
                })
            })
            .collect();

        samples.sort_by_key(|s| s.pid);
        samples
    }

    /// Everything the kernel says is in the job, minus the observer and
    /// anything that was already there.
    ///
    /// `query_process_id_list` reads at most 1024 entries. One session cannot
    /// approach that, and the ramp gives each session its own job, so the cap
    /// is recorded rather than worked around.
    fn job_members(&self) -> Vec<Pid> {
        self.job
            .as_ref()
            .and_then(|job| job.query_process_id_list().ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|pid| !self.baseline.contains(pid))
            .map(|pid| Pid::from_u32(pid as u32))
            .filter(|pid| *pid != self.observer)
            .collect()
    }

    /// Follows parent pids from the session root, plus pseudoconsole hosts
    /// parented to the observer.
    ///
    /// Membership only grows while a process lives: an exited process is
    /// dropped, but an adopted one stays adopted, because Windows leaves
    /// orphans pointing at a dead parent and a strict walk would lose a whole
    /// subtree the moment an intermediate shell exits. Start times are the
    /// identity check, since Windows reuses pids and a reused one would
    /// silently add an unrelated process's memory to the total.
    fn walked_members(&mut self, sys: &System) -> Vec<Pid> {
        self.walked.retain(|pid, start_time| {
            sys.process(*pid)
                .is_some_and(|process| process.start_time() == *start_time)
        });

        if let Some(process) = sys.process(self.root) {
            self.walked.insert(self.root, process.start_time());
        }

        loop {
            let mut adopted = 0;
            for (pid, process) in sys.processes() {
                if self.walked.contains_key(pid) {
                    continue;
                }
                let parent = process.parent();
                let inherited = parent.is_some_and(|p| self.walked.contains_key(&p));
                let borrowed = parent == Some(self.observer)
                    && is_pseudoconsole(&process.name().to_string_lossy());
                if inherited || borrowed {
                    self.walked.insert(*pid, process.start_time());
                    adopted += 1;
                }
            }
            // Repeats because the process map is unordered, so a grandchild can
            // be visited before its parent joins.
            if adopted == 0 {
                break;
            }
        }

        self.walked.keys().copied().collect()
    }
}

pub fn is_pseudoconsole(name: &str) -> bool {
    PSEUDOCONSOLE_NAMES.contains(&name.to_ascii_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pseudoconsole_names_match_regardless_of_case() {
        assert!(is_pseudoconsole("conhost.exe"));
        assert!(is_pseudoconsole("OpenConsole.exe"));
        assert!(!is_pseudoconsole("pwsh.exe"));
    }

    #[test]
    fn arming_reports_which_membership_source_it_got() {
        let armed = ArmedTree::arm(Pid::from_u32(std::process::id()));
        // Either outcome is valid — a job may be refused when this already runs
        // inside one — but the reason has to travel with the fallback.
        match armed.membership() {
            Membership::JobObject => assert!(armed.fallback_reason.is_none()),
            Membership::ParentWalk => assert!(armed.fallback_reason.is_some()),
        }
    }
}
