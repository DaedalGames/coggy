// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Watches a single session from start to finish.
//!
//! This runs before the ramp harness exists, and deliberately so. The ramp
//! applies the redline conditions; one session run to completion is what says
//! whether those conditions are pointed at the right thing. Multiplying a real
//! session by a hundred on paper costs an afternoon, and if the shape
//! contradicts the conditions they get fixed before anything is built to
//! enforce them.
//!
//! It also produces the solo figures the ramp compares against, which is why it
//! outlives the pre-check it was written for.
//!
//! Prior art: `psrecord` (BSD) settled the shape of this tool years ago —
//! attach or launch, `--interval`, `--duration`, children included — so the
//! flags match it rather than inventing a dialect. `procpath` is where the idea
//! of recording a queryable history instead of a summary comes from, which here
//! is `samples.jsonl`.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sysinfo::Pid;

use crate::format::human_bytes;
use crate::host::HostFacts;
use crate::machine::Machine;
use crate::provenance::Provenance;
use crate::redline::RSS_BUDGET_FRACTION;
use crate::sampler::{self, Sample};
use crate::session::{self, Output, SessionMode, Spawned};
use crate::tree::{ArmedTree, Membership};

/// How much of a run counts as startup.
///
/// Generation writes files for the whole session, so the question is not what
/// Defender costs to get going but whether it keeps charging afterwards. The
/// split is what makes those two separable, and every steady figure for CPU is
/// read off the window past it.
/// **Thirty was a guess and has since been measured to carry five times the
/// margin it needs.** A session's first six seconds run 46% fast on
/// single-core boost, [found by chasing down a drift control's false alarm at
/// −18.8%](../../docs/measurements/2026-07-31-162959-the-first-six-seconds.md).
/// Six is what has to be excluded; thirty is what is.
pub const STARTUP_WINDOW: Duration = Duration::from_secs(30);

/// Session count the report projects to.
///
/// The number the whole plan is sized around, so a projection to any other
/// count would have to be re-read against it.
const PROJECTED_SESSIONS: u32 = 100;

/// How often the run prints a line while it is going.
const HEARTBEAT: Duration = Duration::from_secs(30);

/// Everything a run needs to know before it starts.
pub struct ObserveConfig {
    pub label: String,
    pub out_dir: PathBuf,
    /// Where the session may write.
    ///
    /// Separate from `out_dir` so two runs can share one, which is what the
    /// exclusion delta needs: the same path has to be the excluded one in both
    /// halves, or the comparison is between two different directories.
    pub scratch: PathBuf,
    pub interval: Duration,
    pub mode: SessionMode,
    pub max_duration: Option<Duration>,
    pub command: Vec<String>,
}

/// What the run adds up to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    /// Least physical memory the machine had free at any point in the run.
    ///
    /// The condition reads a working set, which is what a process holds in RAM
    /// *now* — and that figure falls when Windows starts paging, so it goes
    /// quiet during exactly the failure it exists to catch. Commit charge has
    /// the opposite fault: a runtime that reserves a heap and never touches it
    /// reads as pressure that never arrives. Neither is a fact about the
    /// machine, and this is: it was collected on every sample from the start
    /// and reported nowhere.
    pub min_available_memory_bytes: u64,
    pub peak_rss_bytes: u64,
    /// Median RSS over the first measured quarter.
    ///
    /// The single-session control, and the counterpart to [the ramp repeating
    /// a rung](../../docs/measurements/2026-07-30-164912-redline-reproducibility.md):
    /// an RSS figure is only a ceiling if the session held the same amount
    /// throughout, and comparing the two ends of one run is what says whether
    /// it did. A memory-limited redline has no slope to fit, so this and
    /// repeats are the whole of its rigour.
    pub early_rss_bytes: u64,
    /// Median RSS over the final quarter of the run.
    ///
    /// The figure that matters for residency. A peak is a moment; this is what
    /// the machine is asked to hold, with startup allocation behind it.
    pub steady_rss_bytes: u64,
    pub peak_processes: usize,
    pub peak_pseudoconsoles: usize,
    pub output_bytes: u64,
    pub output_bytes_per_sec: f64,
    /// Units the workload reported finishing, one per line of stdout.
    pub work_units: u64,
    /// The rate the work-rate condition is measured against.
    ///
    /// Comparable only against the same workload, since a unit means whatever
    /// that workload says it means.
    pub work_units_per_sec: f64,
    /// Work rate over the first measured quarter and over the last.
    ///
    /// The counterpart to the RSS control above, and it guards the figure G0
    /// leans on hardest: `d` is read off this run's cores, and a machine that
    /// moved while it was being observed puts a number in that field with
    /// nothing beside it to say so. A ramp repeats a rung to catch this; a
    /// single session has no rung to repeat, so it compares its own ends.
    #[serde(default)]
    pub early_work_units_per_sec: f64,
    #[serde(default)]
    pub late_work_units_per_sec: f64,
    /// Cores the session occupied in steady state, where 1.0 is one full core.
    ///
    /// Recorded beside Defender's because `WorkRate` names a symptom: a redline
    /// limited by work rate has to say where the cores went, or the pair
    /// collapses into a number with a label on it. `None` when the run never
    /// outlasted [`STARTUP_WINDOW`].
    pub session_cores: Option<f64>,
    /// Cores Defender occupied over the same window.
    pub defender_cores: Option<f64>,
    pub defender: Option<DefenderCost>,
}

/// Defender's cost, split the way the milestone asks for it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenderCost {
    pub startup_cpu_seconds: f64,
    /// The rate that decides whether Defender is a fixed toll or a running one.
    ///
    /// Derived from the same steady window as everything else, rather than
    /// integrated separately — one quantity computed twice is one quantity that
    /// can disagree with itself. `None` when the run never outlasted
    /// [`STARTUP_WINDOW`], which is distinct from zero: a short run has no
    /// steady state, while a zero would claim Defender stops charging.
    pub steady_cpu_seconds_per_min: Option<f64>,
}

/// What a hundred of this session would cost if nothing interfered.
///
/// Linear and unvalidated by construction. Contention, cache pressure, and I/O
/// queueing all make the real curve worse, so this is a floor rather than an
/// estimate — useful because a floor that already breaks a condition settles
/// the question without running the ramp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projection {
    pub sessions: u32,
    pub rss_bytes: u64,
    pub rss_budget_bytes: u64,
    pub rss_condition_holds: bool,
    pub processes: usize,
    pub pseudoconsoles: usize,
    pub output_bytes_per_sec: f64,
    /// Cores a hundred of these would ask for, including Defender's share.
    ///
    /// `None` when the run had no steady state to project from.
    pub cores_needed: Option<f64>,
    pub cores_available: usize,
    /// Whether the machine is asked for more cores than it has.
    ///
    /// Not a fifth condition. It is the mechanism by which the work-rate
    /// condition trips, recorded so the redline can name a cause instead of
    /// only a symptom.
    pub cpu_oversubscribed: Option<bool>,
}

/// The committed artifact for one run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub label: String,
    pub command: Vec<String>,
    pub mode: SessionMode,
    pub machine: Machine,
    pub provenance: Provenance,
    pub host: HostFacts,
    /// Whether the kernel or a parent walk decided which processes counted.
    pub membership: Membership,
    /// Why the job object was unavailable, when it was.
    pub membership_fallback_reason: Option<String>,
    pub started_unix: u64,
    pub duration_ms: u64,
    pub interval_ms: u64,
    pub sample_count: usize,
    /// `None` when the session was killed rather than exiting on its own.
    pub exit_code: Option<i32>,
    /// Whether `--duration` ended the run rather than the session finishing.
    pub stopped_at_limit: bool,
    pub summary: Summary,
    pub projection: Projection,
}

/// Runs the session and writes both artifacts into `config.out_dir`.
pub fn run(config: &ObserveConfig) -> Result<RunReport> {
    let machine = Machine::detect();
    let provenance = Provenance::current();
    let host = HostFacts::query();

    fs::create_dir_all(&config.out_dir)
        .with_context(|| format!("creating {}", config.out_dir.display()))?;
    let mut samples_file = BufWriter::new(File::create(config.out_dir.join("samples.jsonl"))?);

    // Created here, and removed here only when this run owns it. A session
    // ended by `--duration` is killed, and a killed process runs no cleanup.
    let scratch = config.scratch.clone();
    let owns_scratch = scratch.starts_with(&config.out_dir);
    fs::create_dir_all(&scratch)?;

    let started_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();

    // Joining the job has to happen before the session exists: a process
    // inherits job membership from its parent only at creation.
    let armed = ArmedTree::arm(Pid::from_u32(std::process::id()));
    let membership = armed.membership();
    let membership_fallback_reason = armed.fallback_reason.clone();
    if let Some(reason) = &membership_fallback_reason {
        eprintln!("warning: falling back to parent-walk membership — {reason}");
    }

    let started = Instant::now();
    let Spawned {
        mut session,
        output,
        drains,
    } = session::spawn(
        &config.command,
        config.mode,
        &config.out_dir.join("session"),
        &scratch,
    )?;
    let mut tree = armed.attach(Pid::from_u32(
        session
            .pid()
            .context("the session exited before it could be observed")?,
    ));

    let mut sampler = sampler::Sampler::new();
    let mut samples: Vec<Sample> = Vec::new();
    let mut last_heartbeat = Instant::now();

    let (exit_code, stopped_at_limit) = loop {
        std::thread::sleep(config.interval);

        if let Some(code) = session.try_wait()? {
            break (code, false);
        }

        let sample = sampler.take(&mut tree, &output, started.elapsed());
        writeln!(samples_file, "{}", serde_json::to_string(&sample)?)?;
        samples_file.flush()?;

        if last_heartbeat.elapsed() >= HEARTBEAT {
            println!(
                "  {:>6}s  rss {:>10}  procs {:>3}  units {:>5}",
                sample.t_ms / 1000,
                human_bytes(sample.rss_bytes),
                sample.processes,
                sample.work_units,
            );
            last_heartbeat = Instant::now();
        }
        samples.push(sample);

        if config
            .max_duration
            .is_some_and(|limit| started.elapsed() >= limit)
        {
            session.kill()?;
            let code = session.try_wait()?.flatten();
            // Killing the root leaves its children running. The job object
            // knows every one of them, so the last sample doubles as the kill
            // list — used rather than KILL_ON_JOB_CLOSE, which fires when this
            // process's handle closes and would take the run down with it
            // before the report was written.
            sampler.reap(samples.last());
            break (code, true);
        }
    };

    // Dropping the session closes the pipes, which is what lets the drain
    // threads see EOF and stop counting.
    drop(session);
    Spawned::finish(drains)?;
    if owns_scratch {
        let _ = fs::remove_dir_all(&scratch);
    }

    let duration = started.elapsed();
    let summary = summarize(&samples, &output, duration);
    let projection = project(&summary, &machine);

    let report = RunReport {
        label: config.label.clone(),
        command: config.command.clone(),
        mode: config.mode,
        machine,
        provenance,
        host,
        membership,
        membership_fallback_reason,
        started_unix,
        duration_ms: duration.as_millis() as u64,
        interval_ms: config.interval.as_millis() as u64,
        sample_count: samples.len(),
        exit_code,
        stopped_at_limit,
        summary,
        projection,
    };

    fs::write(
        config.out_dir.join("run.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    fs::write(
        config.out_dir.join("run.md"),
        crate::report::run_markdown(&report),
    )?;
    Ok(report)
}

/// CPU over the window after startup, in cores.
///
/// One window for every steady figure. Averaging Defender over a shorter one
/// swung a hundred-session projection by a factor of three and a half between
/// otherwise identical runs, because its CPU arrives in bursts and a short mean
/// lands wherever the bursts fell.
#[derive(Debug, Clone, Copy)]
pub struct SteadyCpu {
    pub session: f64,
    pub defender: f64,
}

pub fn steady_cpu(samples: &[Sample]) -> Option<SteadyCpu> {
    let after: Vec<&Sample> = samples
        .iter()
        .filter(|s| Duration::from_millis(s.t_ms) >= STARTUP_WINDOW)
        .collect();
    if after.is_empty() {
        return None;
    }
    Some(SteadyCpu {
        session: mean(after.iter().map(|s| f64::from(s.cpu_percent) / 100.0)),
        defender: mean(
            after
                .iter()
                .map(|s| f64::from(s.defender_cpu_percent.unwrap_or(0.0)) / 100.0),
        ),
    })
}

fn summarize(samples: &[Sample], output: &Output, duration: Duration) -> Summary {
    let seconds = duration.as_secs_f64().max(f64::EPSILON);
    let output_bytes = output.total();
    let work_units = output.units();

    // Memory settles within seconds, so its steady figure reads off the final
    // quarter. CPU needs the wider post-startup window instead, for the reason
    // on SteadyCpu.
    let quarter = |window: &[Sample]| -> u64 {
        let mut rss: Vec<u64> = window.iter().map(|s| s.rss_bytes).collect();
        rss.sort_unstable();
        rss.get(rss.len() / 2).copied().unwrap_or(0)
    };
    // Units arrive as a running total, so a window's rate is its own delta over
    // its own span. A window of one sample spans nothing and rates as zero,
    // which `work_drift_percent` reads as unmeasurable rather than as a stop.
    let window_rate = |window: &[Sample]| -> f64 {
        match (window.first(), window.last()) {
            (Some(first), Some(last)) if last.t_ms > first.t_ms => {
                last.work_units.saturating_sub(first.work_units) as f64
                    / ((last.t_ms - first.t_ms) as f64 / 1000.0)
            }
            _ => 0.0,
        }
    };
    // A quarter, or one sample, or nothing at all — a run shorter than a single
    // sample has no ends to compare and must not be sliced as though it did.
    let cut = (samples.len() / 4).max(1).min(samples.len());

    // Work rate takes the post-startup window CPU uses, not the whole-run
    // quarters RSS uses, and getting that wrong made this control's first real
    // reading a false alarm. A single core boosts hard for its first seconds:
    // an unpinned session read 50.31 units/s over 1–6s and then 34.0 to 35.0
    // flat for the remaining forty, at 98% CPU throughout. Quartering the whole
    // run straddles that and reports −18.8% of drift where the machine never
    // moved. RSS has no such transient, which is why it can quarter everything.
    let steady: Vec<Sample> = samples
        .iter()
        .filter(|s| Duration::from_millis(s.t_ms) >= STARTUP_WINDOW)
        .cloned()
        .collect();
    let work_cut = (steady.len() / 4).max(1).min(steady.len());
    let cpu = steady_cpu(samples);

    Summary {
        min_available_memory_bytes: samples
            .iter()
            .map(|s| s.available_memory_bytes)
            .min()
            .unwrap_or(0),
        peak_rss_bytes: samples.iter().map(|s| s.rss_bytes).max().unwrap_or(0),
        early_rss_bytes: quarter(&samples[..cut]),
        steady_rss_bytes: quarter(&samples[samples.len() - cut..]),
        peak_processes: samples.iter().map(|s| s.processes).max().unwrap_or(0),
        peak_pseudoconsoles: samples.iter().map(|s| s.pseudoconsoles).max().unwrap_or(0),
        output_bytes,
        output_bytes_per_sec: output_bytes as f64 / seconds,
        work_units,
        work_units_per_sec: work_units as f64 / seconds,
        early_work_units_per_sec: window_rate(&steady[..work_cut]),
        late_work_units_per_sec: window_rate(&steady[steady.len().saturating_sub(work_cut)..]),
        session_cores: cpu.map(|c| c.session),
        defender_cores: cpu.map(|c| c.defender),
        defender: defender_cost(samples, cpu),
    }
}

/// Zero for an empty run, so a session shorter than one sample reports nothing
/// rather than dividing by it.
fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let (sum, count) = values.fold((0.0, 0usize), |(sum, count), v| (sum + v, count + 1));
    if count == 0 { 0.0 } else { sum / count as f64 }
}

/// Defender's cost: CPU-seconds spent before the session was up, and the rate
/// it kept charging afterwards.
///
/// Returns `None` when Defender never appeared, so a machine without it reads
/// as unmeasured rather than as costing nothing. The steady rate comes from
/// `cpu` rather than being integrated here, so the two can never disagree.
fn defender_cost(samples: &[Sample], cpu: Option<SteadyCpu>) -> Option<DefenderCost> {
    if !samples.iter().any(|s| s.defender_cpu_percent.is_some()) {
        return None;
    }

    // Integrated against the gap each sample actually covers rather than the
    // requested interval, since a loaded machine delivers samples late and the
    // difference lands straight in the total.
    let startup: f64 = samples
        .iter()
        .scan(0u64, |previous, sample| {
            let elapsed = sample.t_ms.saturating_sub(*previous) as f64 / 1000.0;
            *previous = sample.t_ms;
            Some((sample, elapsed))
        })
        .filter(|(sample, _)| Duration::from_millis(sample.t_ms) < STARTUP_WINDOW)
        .map(|(sample, elapsed)| {
            f64::from(sample.defender_cpu_percent.unwrap_or(0.0)) / 100.0 * elapsed
        })
        .sum();

    Some(DefenderCost {
        startup_cpu_seconds: startup,
        steady_cpu_seconds_per_min: cpu.map(|c| c.defender * 60.0),
    })
}

fn project(summary: &Summary, machine: &Machine) -> Projection {
    let sessions = u64::from(PROJECTED_SESSIONS);
    let rss = summary.steady_rss_bytes.saturating_mul(sessions);
    let budget = (machine.total_memory_bytes as f64 * RSS_BUDGET_FRACTION) as u64;
    let cores_available = machine
        .physical_cores
        .unwrap_or(machine.logical_cores)
        .max(1);
    // Defender scales with the sessions rather than sitting flat, since what it
    // charges for is the files they write.
    let cores_needed = summary
        .session_cores
        .zip(summary.defender_cores)
        .map(|(session, defender)| (session + defender) * sessions as f64);

    Projection {
        sessions: PROJECTED_SESSIONS,
        rss_bytes: rss,
        rss_budget_bytes: budget,
        rss_condition_holds: rss <= budget,
        processes: summary.peak_processes * PROJECTED_SESSIONS as usize,
        pseudoconsoles: summary.peak_pseudoconsoles * PROJECTED_SESSIONS as usize,
        output_bytes_per_sec: summary.output_bytes_per_sec * sessions as f64,
        cores_needed,
        cores_available,
        cpu_oversubscribed: cores_needed.map(|needed| needed > cores_available as f64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(t_ms: u64, cpu_percent: f32, defender_cpu_percent: Option<f32>) -> Sample {
        Sample {
            t_ms,
            rss_bytes: 1024,
            processes: 1,
            pseudoconsoles: 0,
            cpu_percent,
            machine_cpu_percent: cpu_percent,
            defender_cpu_percent,
            defender_rss_bytes: None,
            available_memory_bytes: 0,
            output_bytes: 0,
            work_units: 0,
            members: Vec::new(),
        }
    }

    /// Eight samples a second apart, carrying a running unit total that grows
    /// at `early` units/s for the first half and `late` for the second.
    ///
    /// Placed past [`STARTUP_WINDOW`], because that is where the work-rate
    /// windows look: a single core boosts for its first seconds, and quartering
    /// across that reports drift the machine never had.
    fn ramping_units(early: u64, late: u64) -> Vec<Sample> {
        let base = STARTUP_WINDOW.as_millis() as u64;
        let mut total = 0;
        (0..8)
            .map(|i| {
                let mut s = sample(base + i * 1000, 0.0, None);
                s.work_units = total;
                total += if i < 4 { early } else { late };
                s
            })
            .collect()
    }

    #[test]
    fn work_drift_ignores_the_startup_window_the_way_the_cores_figure_does() {
        // Fast for the first six seconds and flat after, which is the shape a
        // real run has: 50.31 units/s over 1–6s, then 34 for the next forty.
        let mut total = 0;
        let samples: Vec<Sample> = (0..40)
            .map(|i| {
                let mut s = sample(i * 1000, 0.0, None);
                s.work_units = total;
                total += if i < 6 { 50 } else { 34 };
                s
            })
            .collect();
        let summary = summarize(&samples, &Output::new(), Duration::from_secs(40));
        let moved = (summary.late_work_units_per_sec - summary.early_work_units_per_sec)
            / summary.early_work_units_per_sec
            * 100.0;
        assert!(
            moved.abs() < 1.0,
            "the boost is before the window, so nothing should have drifted — got {moved:+.1}% \
             from {:.2} early and {:.2} late",
            summary.early_work_units_per_sec,
            summary.late_work_units_per_sec
        );
    }

    #[test]
    fn a_steady_session_reports_no_work_drift() {
        let summary = summarize(
            &ramping_units(10, 10),
            &Output::new(),
            Duration::from_secs(8),
        );
        assert_eq!(summary.early_work_units_per_sec, 10.0);
        assert_eq!(summary.late_work_units_per_sec, 10.0);
    }

    #[test]
    fn a_session_that_slowed_shows_it_in_the_ends_rather_than_the_mean() {
        let summary = summarize(
            &ramping_units(20, 5),
            &Output::new(),
            Duration::from_secs(8),
        );
        assert!(
            summary.early_work_units_per_sec > summary.late_work_units_per_sec,
            "early {} late {}",
            summary.early_work_units_per_sec,
            summary.late_work_units_per_sec
        );
    }

    #[test]
    fn a_window_too_short_to_span_time_rates_as_zero_rather_than_dividing_by_it() {
        let summary = summarize(
            &[sample(0, 0.0, None)],
            &Output::new(),
            Duration::from_secs(1),
        );
        assert_eq!(summary.early_work_units_per_sec, 0.0);
        assert_eq!(summary.late_work_units_per_sec, 0.0);
    }

    #[test]
    fn a_session_with_no_samples_summarizes_to_zero_rather_than_panicking() {
        let summary = summarize(&[], &Output::new(), Duration::from_secs(1));
        assert_eq!(summary.peak_rss_bytes, 0);
        assert_eq!(summary.steady_rss_bytes, 0);
        assert!(summary.session_cores.is_none());
        assert!(summary.defender.is_none());
    }

    #[test]
    fn a_run_too_short_for_a_steady_state_says_so_instead_of_reporting_zero() {
        let samples: Vec<Sample> = (0..5).map(|i| sample(i * 1000, 0.0, Some(10.0))).collect();
        let cost =
            defender_cost(&samples, steady_cpu(&samples)).expect("Defender reported a reading");
        assert!(cost.startup_cpu_seconds > 0.0);
        // Five seconds never reaches the startup window, so there is no steady
        // state to report. Zero here would claim Defender stopped charging.
        assert!(cost.steady_cpu_seconds_per_min.is_none());
    }

    #[test]
    fn the_steady_window_survives_a_run_that_reaches_it() {
        let samples: Vec<Sample> = (0..40)
            .map(|i| sample(i * 1000, 25.0, Some(50.0)))
            .collect();

        let cpu = steady_cpu(&samples).expect("the run outlasted the startup window");
        assert_eq!(cpu.session, 0.25);
        assert_eq!(cpu.defender, 0.5);
        // The rate the report prints is this same figure in other units, never
        // a second integration that could drift from it.
        let cost = defender_cost(&samples, Some(cpu)).expect("Defender reported a reading");
        assert_eq!(cost.steady_cpu_seconds_per_min, Some(30.0));
    }

    #[test]
    fn a_lone_sample_is_its_own_steady_state() {
        // The final quarter of a one-sample run is that sample, and a slice
        // index of zero has to survive rather than reach past the end.
        let summary = summarize(
            &[sample(0, 50.0, None)],
            &Output::new(),
            Duration::from_secs(2),
        );
        assert_eq!(summary.steady_rss_bytes, 1024);
        // Memory settles in seconds, so one sample is a usable steady figure.
        // CPU does not, and one sample taken before the startup window is over
        // is not a steady state however the slice is indexed.
        assert!(summary.session_cores.is_none());
    }
}
