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

/// The agent driving the benchmark, whose cost lands in the residual.
///
/// **`rest_cores_median` is everything outside the session's job, which has
/// always included the operator.** Measured 2026-08-12: during a hold's first
/// five seconds the harness cost 0.064 cores and the agent cost 1.58, and three
/// baselines taken in a nine-minute window with the machine's ten-core tenant
/// entirely absent read 2.29, 2.71 and 2.17 against a floor near 0.85.
///
/// **It is worse than a constant offset because the cost is event-driven**: it
/// spikes when a watcher fires, which is exactly when a hold begins. So the
/// contamination correlates with the measurement rather than averaging out.
///
/// This is NOT subtracted from `machine_cpu_percent`. Redefining that column
/// would make it mean something different from every artifact already on disk,
/// and would hide the quantity rather than measure it — so it is recorded
/// beside it and any analysis subtracts knowingly.
const OBSERVER_PROCESS: &str = "claude";

/// The neighbour whose presence sets how many cores this machine offers.
///
/// WHY A NAME AND NOT A RESIDUAL. `rest_cores_median` is `machine - job`, so it
/// is anonymous by construction: it can say ten cores are held elsewhere and
/// never by what. On 2026-08-12 that mattered more than any other column here.
/// A hundred sessions get a machine delivering **5.1 cores with this process
/// absent and 14.5 with it present** — 2.8x — while a 3.7x change in the
/// sessions' own duty moves that total by 5%. Three holds minutes apart read
/// 14.53, 5.15 and 14.60 as it came and went. The neighbour keeps cores
/// unparked and a hundred self-limiting sessions cannot, so an artifact that
/// does not record whether it was there cannot say which machine it measured.
///
/// Matched on a prefix for the same reason as the observer above.
const TENANT_PROCESS: &str = "chrome-headless-shell";

/// What one tick of the instrument cost.
///
/// A scaling benchmark has to know its own overhead, because the one failure it
/// cannot detect from the outside is the observer becoming the bottleneck. The
/// first ramp against a saturating workload spent seventy-five seconds on a
/// fifteen second hold, and without this there was no way to say which part of
/// the tick had eaten it.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TickCost {
    /// Walking the tracked processes, or the whole table when membership needs it.
    pub refresh_ms: u64,
    /// Checking for finished sessions and restarting them.
    pub replace_ms: u64,
    /// Reading the job's members and their memory.
    pub sample_ms: u64,
    /// Serialising the sample and flushing it to disk.
    pub write_ms: u64,
}

impl TickCost {
    pub fn total_ms(&self) -> u64 {
        self.refresh_ms + self.replace_ms + self.sample_ms + self.write_ms
    }

    /// Keeps whichever tick cost more in total.
    pub(crate) fn keep_worse(&mut self, other: TickCost) {
        if other.total_ms() > self.total_ms() {
            *self = other;
        }
    }
}

/// How much of the machine the sessions held, and how steadily.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Occupancy {
    pub median_cores: f64,
    pub mean_cores: f64,
    /// Cores held by everything outside the job, at the median.
    ///
    /// **Unlike `lost_cores`, this one is worth having on a single session**,
    /// and the difference is which side of the subtraction the session is on.
    /// `lost_cores` measures the job against itself, so at one session its
    /// floor and its signal sit under two-fold apart and it is left out.
    /// The rest of the machine does not care how big the job is: three
    /// back-to-back solo holds gave **2.99, 11.46 and 11.60 cores** here while
    /// the job held 0.24 to 0.26 in all three, and the two fast-and-quiet
    /// versus slow-and-crowded groups are what said [a 30% spread was a tenant
    /// rather than the slow machine
    /// state](../../docs/measurements/2026-08-03-070018-a-solo-triple-spread-thirty-percent-and-the-column-said-why.md).
    pub rest_cores_median: f64,
    /// Median cores held by the agent driving the run, over the same window.
    ///
    /// **`rest_cores_median` includes it, and this says how much.** Measured
    /// 2026-08-12 at 0.18 cores idle and 1.58–1.99 while working — and the cost
    /// is event-driven, spiking when a watcher fires, which is when a hold
    /// begins. It is reported rather than subtracted so every figure already on
    /// disk keeps its meaning, and so a reader can see the contamination rather
    /// than receive a corrected number with no provenance.
    pub observer_cores_median: f64,
    /// Cores held by the named neighbour, over the same window as
    /// `rest_cores_median`.
    ///
    /// **This is the column that says which machine a hold measured.** On
    /// 2026-08-12 a hundred sessions met a box delivering 5.1 cores with this
    /// process absent and 14.5 with it present, and three holds minutes apart
    /// read 14.53, 5.15 and 14.60 as it came and went. Reported rather than
    /// subtracted, for the same reason as the observer above: every figure
    /// already on disk keeps its meaning, and a reader sees the neighbour rather
    /// than receiving a corrected number with no provenance.
    pub tenant_cores_median: f64,
    /// Mean cores below the median, summed over samples under it.
    ///
    /// **The one number that would have caught it**, and it has a floor rather
    /// than a zero. Every sample under the median counts, so ordinary tick
    /// noise contributes: applied to the three twenty-minute holds it gives
    /// **1.173 cores against 0.061 and 0.052**, and to a clean eight-session
    /// hold 0.014. As a share of each run's median that is 7.6% against
    /// 0.2–0.4%, so **read it against the median, not against zero.**
    ///
    /// A version counting only samples more than 2% under the median gives
    /// 1.096 against 0.014 and 0.004 — a wider separation for the cost of a
    /// constant that would have to be right on every machine. The
    /// thresholdless one separates by twenty times, which is enough.
    ///
    /// **It is not worth reporting for a single session, and that is measured.**
    /// `observe` holds one session on a machine with fifteen spare cores, so a
    /// disturbance elsewhere barely touches it — but its own floor is higher,
    /// because one session's CPU sits near a whole core and tick noise is a
    /// larger share of it. Seven one-session runs lose **0.7–4.1% of their
    /// median** where an undisturbed hundred-session hold loses 0.2–0.4% and a
    /// disturbed one 7.6%. At one session the floor and the signal are under
    /// two-fold apart, so the number could not tell them apart and is left out
    /// rather than shipped as a column nobody can read.
    ///
    /// The seven are `workdrift-check`, `workdrift-long`, `workdrift-fixed`,
    /// `rss-20`, `rss-33` and `pathA-observe` under `bench-out/`. Named
    /// because a figure whose artifact is not named is a figure the next
    /// prune deletes the evidence for — which has already happened once, to
    /// the seven runs behind a redline's ±12.5%.
    pub lost_cores: f64,
}

impl Occupancy {
    /// What a run's samples say it held of the machine: the median, and how
    /// much of it the run lost to intervals below that median.
    ///
    /// **A mean alone made three hours of wrong conclusions.** Three holds were
    /// compared on mean occupancy and read as a footprint effect on `η` worth
    /// 11.4%; their medians agreed to 0.7%, and the difference was one run
    /// losing the machine in 18% of its samples to two multi-minute episodes.
    /// The dips were real — output fell with them — but nothing in the report
    /// said they had happened, so the mean carried them into a conclusion about
    /// the workload.
    ///
    /// Lost is measured against this run's own median rather than a fixed
    /// threshold, because a cut taken from one run only ever finds that run:
    /// the 14-core line that made the episodes visible came from the run that
    /// had them, and against their own medians the other two lose 0.014 and
    /// 0.004 cores where that one loses 1.096.
    ///
    /// `None` when nothing was sampled.
    pub fn of(samples: &[Sample]) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }

        let series: Vec<f64> = samples
            .iter()
            .map(|s| f64::from(s.cpu_percent) / 100.0)
            .collect();
        // **Spin-up is not a disturbance**, and counting it as one makes every
        // short hold look interrupted: a twenty-second hold at two-second
        // ticks spends its first samples starting sessions, which dragged a
        // mean of 7.97 to 6.48 and reported 1.50 cores lost. Dropping
        // everything before the run first reaches its own median needs no
        // constant and no window length.
        let mut sorted = series.clone();
        sorted.sort_by(f64::total_cmp);
        let rough = sorted[sorted.len() / 2];
        let from = series.iter().position(|c| *c >= rough).unwrap_or(0);
        let mut cores: Vec<f64> = series[from..].to_vec();
        if cores.is_empty() {
            return None;
        }
        // The same window, so the two describe the same stretch of the run.
        let mut rest: Vec<f64> = samples[from..]
            .iter()
            .map(|s| f64::from(s.machine_cpu_percent - s.cpu_percent) / 100.0)
            .collect();
        rest.sort_by(f64::total_cmp);
        let rest_median = rest[rest.len() / 2];
        // Over the SAME window as `rest`, so the two can be subtracted without
        // mixing moments — the error this repository has made more than once.
        let mut observer: Vec<f64> = samples[from..]
            .iter()
            .map(|s| f64::from(s.observer_cpu_percent) / 100.0)
            .collect();
        observer.sort_by(f64::total_cmp);
        let observer_median = observer[observer.len() / 2];
        // Same window again. A neighbour that arrives mid-hold is exactly what
        // this exists to expose, so it must not be averaged over a different
        // stretch than the residual it explains.
        let mut tenant: Vec<f64> = samples[from..]
            .iter()
            .map(|s| f64::from(s.tenant_cpu_percent) / 100.0)
            .collect();
        tenant.sort_by(f64::total_cmp);
        let tenant_median = tenant[tenant.len() / 2];
        cores.sort_by(f64::total_cmp);
        let median = cores[cores.len() / 2];
        let lost = cores.iter().map(|c| (median - c).max(0.0)).sum::<f64>() / cores.len() as f64;
        Some(Self {
            median_cores: median,
            mean_cores: cores.iter().sum::<f64>() / cores.len() as f64,
            lost_cores: lost,
            rest_cores_median: rest_median,
            observer_cores_median: observer_median,
            tenant_cores_median: tenant_median,
        })
    }
}

/// One instant of one session's cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub t_ms: u64,
    pub rss_bytes: u64,
    pub processes: usize,
    pub pseudoconsoles: usize,
    pub cpu_percent: f32,
    /// The whole machine's CPU as a percentage of one core, on the same scale
    /// as [`Sample::cpu_percent`] so the two subtract — which needs the raw
    /// `global_cpu_usage` multiplied onto the logical core count, since that
    /// call averages the CPUs into 0–100 and a process's `cpu_usage` is its
    /// share of a single one.
    ///
    /// **What is left is everything this instrument does not attribute**, and
    /// until it was recorded that quantity was inferred from the core count.
    /// One twenty-minute hold lost the machine in 18% of its samples, down to
    /// 0.58 cores against a median of 15.37, and its mean carried the loss into
    /// an `η` that was read as a fact about the session's footprint. The dips
    /// were real — output fell with them — but nothing said what took the
    /// cores, because nothing was counting outside the job.
    ///
    /// **The first readings were taken by converting, and the stored column was
    /// not.** At eight sessions the difference from the job came to 2.56–4.21
    /// cores and agreed with what `doctor` reported minutes earlier; at a
    /// hundred the machine came to 16.00 cores, saturated. Both are right and
    /// neither could have come out of these two fields as they were then, which
    /// is [how the scale defect survived being looked
    /// at](../../docs/measurements/2026-08-03-061911-the-machine-column-was-narrower-than-the-job.md):
    /// the analysis carried the multiplication that the sampler did not, so the
    /// numbers checked out while the artifact did not.
    ///
    /// Read it as `(machine_cpu_percent - cpu_percent) / 100.0` for cores.
    /// Saturation costs resolution rather than the discrimination that matters:
    /// when a hold dips, a machine still pinned at every core says something
    /// else took them, and a machine dipping with the job says the job stopped
    /// using them.
    pub machine_cpu_percent: f32,
    /// `None` when Defender is not running, which is itself worth recording.
    pub defender_cpu_percent: Option<f32>,
    pub defender_rss_bytes: Option<u64>,
    /// CPU held by the agent driving the run, in whole-machine percent.
    ///
    /// Sampled in the same tick as [`Sample::machine_cpu_percent`], because a
    /// subtraction across two moments is the error this repository has made
    /// more than once.
    pub observer_cpu_percent: f32,
    /// How many processes that figure came from.
    ///
    /// **Travels with the figure so a zero can be told from a miss.** The
    /// observer is matched by name, which is fragile: a renamed or relaunched
    /// agent goes uncounted and `observer_cpu_percent` silently reads 0.0 —
    /// the passing value. A count of 0 says "found nothing" where the percent
    /// alone would say "cost nothing".
    pub observer_processes: usize,
    /// CPU held by [`TENANT_PROCESS`], sampled in the same tick as
    /// [`Sample::machine_cpu_percent`] so the two can be read against each
    /// other. Included in `machine_cpu_percent` and NOT subtracted from it: this
    /// names part of the residual rather than redefining it.
    pub tenant_cpu_percent: f32,
    /// How many matched, so a zero can be told from an absence. A rename or a
    /// suffix makes the tenant go uncounted and `tenant_cpu_percent` silently
    /// reads 0.0, which is indistinguishable from a quiet machine — the exact
    /// confusion this field exists to prevent.
    pub tenant_processes: usize,
    pub available_memory_bytes: u64,
    pub output_bytes: u64,
    /// Lines the session has written, which is its own count of work done.
    pub work_units: u64,
    /// Every member, so the artifact can be re-read for which process names
    /// hold what share without taking the run again.
    pub members: Vec<ProcessSample>,
}

/// The whole machine on a process's scale: `global_cpu_usage` averages the CPUs
/// and tops out at 100, while a process's `cpu_usage` is its share of a single
/// one and a job of a hundred sessions reaches 1600.
///
/// **Subtracting them raw is wrong by the width of the machine, and it read as
/// a plausible number rather than as a zero.** Four holds recorded before this
/// existed put the job above the whole machine in 31 of their 36 samples —
/// 796 against 61.5 at eight sessions — and nothing complained, because the
/// column had been added and not yet subtracted from anything.
fn machine_percent(global: f32, logical_cores: f32) -> f32 {
    global * logical_cores
}

pub struct Sampler {
    sys: System,
    /// The machine's logical processor count, held because
    /// [`Sample::machine_cpu_percent`] has to be multiplied onto it and
    /// `sysinfo` will not report it once the process list is being refreshed
    /// selectively.
    logical_cores: f32,
    /// Defender's pid, so it can be refreshed by name only once.
    defender: Option<Pid>,
    /// The agent's pids, found on a full walk and refreshed by pid after.
    observers: Vec<Pid>,
    tenants: Vec<Pid>,
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
            // Set below, after the priming refresh: `System::new()` has not
            // enumerated the CPUs yet, and a zero here would silently flatten
            // the whole machine column to nothing.
            logical_cores: 1.0,
            defender: None,
            observers: Vec::new(),
            tenants: Vec::new(),
            unprioritised_reason: raise_current_thread().err(),
        };
        if let Some(reason) = &sampler.unprioritised_reason {
            eprintln!("warning: the sampler runs at ordinary priority — {reason}");
        }
        sampler.refresh(None);
        // `sys.cpus().len()`, the same route [`crate::machine`] takes, so the
        // two never disagree about how wide this box is. Floored at one because
        // it multiplies rather than divides.
        sampler.logical_cores = (sampler.sys.cpus().len() as f32).max(1.0);
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
        // **The whole machine, read from the CPU counters rather than by
        // walking processes.** Everything below attributes only what is in the
        // job, so what the rest of the machine was doing has never been
        // recorded — and `16 - job` is an inference that once turned one run's
        // interruptions into a three-hour conclusion about session footprints.
        // Summing every process would answer it and is the one thing this
        // method exists to avoid: the full table cost eighty seconds at
        // twenty-five sessions. This is per core, not per process.
        //
        // **THAT BASELINE IS STATE-DEPENDENT, and the figures below were taken
        // on an unparked box.** Recomputed from every hold on disk on
        // 2026-08-12: the +23.7 to +23.9 ms readings all come from holds
        // delivering 13.8-15.3 job cores, while holds on the same binaries
        // delivering 3.5 run **+29 to +62 ms**. So a future change measured
        // against the numbers below would be compared across the parked and
        // unparked states — the confound that cost a whole night here. Take a
        // matched pair in the same state, and check `median_cores` agrees
        // before believing either side of it.
        //
        // **Its own cost is measured, at a matched pair.** Tick slip against
        // the interval asked for, all at a hundred sessions on a 5 s tick:
        // +23.7 and +23.8 ms before this line existed, +23.9 after —
        // **0.2 ms, or 0.004% of the tick**, with the median at exactly
        // +24.0 in all three. Rule 5 of keeping it honest is that the
        // observer becoming the bottleneck is the one failure that does not
        // announce itself, and a per-core read is the cheapest way to ask
        // the whole machine what it was doing.
        self.sys.refresh_cpu_usage();
        let kind = ProcessRefreshKind::nothing().with_cpu().with_memory();

        match (tracked, self.defender) {
            (Some(pids), Some(defender)) => {
                let mut list =
                    Vec::with_capacity(pids.len() + 1 + self.observers.len() + self.tenants.len());
                list.extend_from_slice(pids);
                list.push(defender);
                list.extend_from_slice(&self.observers);
                list.extend_from_slice(&self.tenants);
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
                // Matched on a PREFIX rather than an exact name, because the
                // agent's executable carries a suffix on some platforms and an
                // exact match would report zero cost rather than no match.
                self.observers = self
                    .sys
                    .processes()
                    .iter()
                    .filter(|(_, p)| {
                        p.name()
                            .to_string_lossy()
                            .to_ascii_lowercase()
                            .starts_with(OBSERVER_PROCESS)
                    })
                    .map(|(pid, _)| *pid)
                    .collect();
                self.tenants = self
                    .sys
                    .processes()
                    .iter()
                    .filter(|(_, p)| {
                        p.name()
                            .to_string_lossy()
                            .to_ascii_lowercase()
                            .starts_with(TENANT_PROCESS)
                    })
                    .map(|(pid, _)| *pid)
                    .collect();
            }
        }
    }

    /// Samples one session against the current refresh.
    pub fn sample(&self, tree: &mut SessionTree, output: &Output, elapsed: Duration) -> Sample {
        let members = tree.sample(&self.sys);
        let defender = self.defender.and_then(|pid| self.sys.process(pid));
        let observers: Vec<_> = self
            .observers
            .iter()
            .filter_map(|pid| self.sys.process(*pid))
            .collect();
        let tenants: Vec<_> = self
            .tenants
            .iter()
            .filter_map(|pid| self.sys.process(*pid))
            .collect();

        Sample {
            t_ms: elapsed.as_millis() as u64,
            rss_bytes: members.iter().map(|m| m.rss_bytes).sum(),
            processes: members.len(),
            pseudoconsoles: members
                .iter()
                .filter(|m| m.attribution == Attribution::Pseudoconsole)
                .count(),
            cpu_percent: members.iter().map(|m| m.cpu_percent).sum(),
            machine_cpu_percent: machine_percent(self.sys.global_cpu_usage(), self.logical_cores),
            defender_cpu_percent: defender.map(|p| p.cpu_usage()),
            defender_rss_bytes: defender.map(|p| p.memory()),
            observer_cpu_percent: observers.iter().map(|p| p.cpu_usage()).sum(),
            observer_processes: observers.len(),
            tenant_cpu_percent: tenants.iter().map(|p| p.cpu_usage()).sum(),
            tenant_processes: tenants.len(),
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

#[cfg(test)]
mod tests {
    use super::machine_percent;

    /// One multiplication is the whole of the conversion, so this is what
    /// stands between the column and the scale it shipped with.
    #[test]
    fn the_machine_column_is_read_in_cores_not_in_one_core() {
        assert!((machine_percent(100.0, 16.0) - 1600.0).abs() < f32::EPSILON);
        assert!((machine_percent(50.0, 16.0) - 800.0).abs() < f32::EPSILON);
    }

    /// The shipped shape, in its own numbers: nine sessions summing to 796 on a
    /// machine column reading 61.5 — a machine narrower than the work on it.
    /// Converted, the same reading clears the job with room to spare.
    #[test]
    fn a_job_can_no_longer_exceed_the_machine_it_runs_on() {
        let job = 796.0;
        let raw_global = 61.5;
        assert!(raw_global < job, "this is the reading that shipped");
        assert!(machine_percent(raw_global, 16.0) > job);
    }
}
