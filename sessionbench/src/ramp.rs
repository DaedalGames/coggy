// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! The monotonic ramp, which is what produces a redline.
//!
//! Each rung holds exactly N sessions alive for a fixed window, replacing any
//! that finish, and then asks whether all four conditions still held. The
//! redline is the last rung where they did, paired with the one that broke on
//! the next.
//!
//! Replacement is not a nicety. A step that let finished sessions stay finished
//! would be measuring a decaying count while reporting the count it asked for,
//! and the machine gets easier as that happens — so the number would drift
//! upward exactly when the machine was struggling most.
//!
//! Climb to bracket, then refine inside the bracket. The climb is what makes
//! the break an observation rather than an assumption; the refinement is what
//! turns "somewhere above ten and at or below twenty-five" into a number.
//!
//! An earlier version of this file climbed and stopped, on the argument that
//! bisecting would assume the conditions behave monotonically in the session
//! count. That argument is wrong: it applies to bisecting the whole range, not
//! to halving an interval whose two ends have both been observed. Standard
//! capacity practice is exactly this — ramp until compliant runs bracket a
//! candidate region, then narrow it — and the cost of not doing it showed up
//! in the first real run, which reported a redline of ten when the ceiling was
//! somewhere in the eleven-to-twenty-five range.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sysinfo::Pid;

use crate::format::human_bytes;
use crate::host::HostFacts;
use crate::machine::Machine;
use crate::provenance::Provenance;
use crate::redline::{
    self, LimitingCondition, RAMP_STEPS, REPLACEMENT_BUDGET_SECS, RSS_BUDGET_FRACTION, Redline,
    WORK_RATE_BUDGET_FACTOR,
};
use crate::sampler::{Sample, Sampler};
use crate::session::{self, Output, Session, SessionMode};
use crate::tree::{ArmedTree, Membership, SessionTree};

/// Share of each hold spent letting the rung settle before measuring.
///
/// Sessions spawned together fault in their pages together, so the first
/// moments of a rung are a spike that belongs to nothing being asked about.
/// Floored at [`STARTUP_WINDOW`], which is the same cut `observe` makes for
/// the same reason: at the default ninety-second hold a third *is* thirty
/// seconds and the two agree, but a short hold left the spin-up at five and
/// let page faults into the measurement. A single session at quarter duty read
/// 0.4 cores that way against a true 0.24.
const SPINUP_FRACTION: f64 = 1.0 / 3.0;

/// Fewest measured samples a rung needs before its verdict means anything.
///
/// A saturated machine can starve the sampler badly enough that a fifteen
/// second hold yields one reading forty seconds late. Everything derived from
/// that reads as a catastrophic collapse — zero work, zero cores — and the
/// first ramp run reported exactly that as a broken condition. A rung that
/// could not be measured is not a rung that failed.
const MIN_SAMPLES_PER_RUNG: usize = 5;

/// How long a rung waits for its sessions to leave before terminating them.
///
/// Generous because waiting is cheap and killing a process that is already
/// terminating is not: at fifty pseudoconsole sessions, killing cost 66
/// seconds where waiting costs whatever the hosts actually need.
const EXIT_GRACE: Duration = Duration::from_secs(20);

/// Everything a ramp needs before it starts.
pub struct RampConfig {
    pub label: String,
    pub out_dir: PathBuf,
    pub interval: Duration,
    pub hold: Duration,
    pub max_sessions: u32,
    /// How tight the bracket must be before the redline is reported.
    pub resolution: u32,
    /// Hold a Defender path exclusion over the sessions' scratch root for the
    /// whole ramp.
    ///
    /// This is the exclusion axis at the scale it belongs at. Measured against
    /// one session it was invisible — Defender competes with nobody on a
    /// machine with fifteen idle cores — and its CPU rate, the only visible
    /// term there, sits under machine-wide noise. Two ramps compared by their
    /// redlines have neither problem: at a hundred sessions the write volume
    /// is a hundredfold, and a redline is an integer from window deltas.
    pub exclude_scratch: bool,
    /// Skip repeating a rung at the end to check the machine did not drift.
    ///
    /// The check costs one hold and is what says whether the ladder measured
    /// one machine or several. Worth skipping only when the ramp is being run
    /// for its shape rather than its number.
    pub skip_drift_check: bool,
    pub mode: SessionMode,
    pub command: Vec<String>,
}

/// What one tick of the instrument cost.
///
/// A scaling benchmark has to know its own overhead, because the one failure it
/// cannot detect from the outside is the observer becoming the bottleneck. The
/// first ramp against a saturating workload spent seventy-five seconds on a
/// fifteen second hold, and without this there was no way to say which part of
/// the tick had eaten it.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TickCost {
    /// Walking every process on the machine.
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

    fn keep_worse(&mut self, other: TickCost) {
        if other.total_ms() > self.total_ms() {
            *self = other;
        }
    }
}

/// What ending a rung cost, stage by stage.
///
/// Teardown is instrumented for the same reason ticks are: it is the
/// instrument's own time, it is invisible from the outside, and it does not
/// scale the way the measured part does. A hundred pseudoconsole sessions took
/// eleven minutes to shut down while seventy-five took seconds, and there was
/// no way to say which stage from anything the report carried.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TeardownCost {
    /// Terminating each session's root process.
    pub kill_ms: u64,
    /// Waiting for the pseudoconsole hosts to leave once their handles closed.
    pub settle_ms: u64,
    /// Re-reading whatever did not leave.
    pub reap_refresh_ms: u64,
    /// Terminating it.
    pub reap_kill_ms: u64,
    /// How many were still alive when the grace period ran out.
    pub stragglers: usize,
    /// Removing the directory the sessions wrote into.
    pub scratch_ms: u64,
    /// Releasing the pty handles, which is where a pseudoconsole is closed.
    pub release_ms: u64,
}

impl TeardownCost {
    pub fn total_ms(&self) -> u64 {
        self.kill_ms
            + self.release_ms
            + self.settle_ms
            + self.reap_refresh_ms
            + self.reap_kill_ms
            + self.scratch_ms
    }
}

/// What the repeated rung said about the machine underneath the ladder.
#[derive(Debug, Clone, PartialEq)]
pub enum Drift {
    /// The repeat produced no usable reading.
    ///
    /// Distinct from a machine that slowed to nothing, which is what a bare
    /// zero would otherwise be reported as.
    Unmeasurable(String),
    /// Both readings of the same session count, early and late.
    Measured {
        sessions: u32,
        early_units_per_sec: f64,
        late_units_per_sec: f64,
        /// Positive means the machine was slower the second time.
        slower_percent: f64,
    },
}

/// One rung of the ladder.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Step {
    pub sessions: u32,
    pub samples: usize,
    pub total_rss_bytes: u64,
    pub rss_budget_bytes: u64,
    /// The figure the work-rate condition compares against its solo value.
    pub units_per_session_per_sec: f64,
    pub session_cores: f64,
    pub defender_cores: f64,
    pub processes: usize,
    pub pseudoconsoles: usize,
    /// Sessions that finished and were started again during the hold.
    pub replacements: u32,
    /// The slowest replacement, which is what the condition is set against.
    pub worst_replacement_secs: Option<f64>,
    /// Units a session reported starting but never wrote a line for.
    pub dropped_units: u64,
    /// Wall-clock the rung actually took, which exceeds the hold when the
    /// machine could not keep the sampler running.
    pub elapsed_secs: f64,
    /// The most expensive tick of the rung, broken down.
    pub worst_tick: TickCost,
    /// What ending the rung cost, broken down.
    pub teardown: TeardownCost,
    /// Why this rung supports no verdict, when it supports none.
    ///
    /// Takes precedence over `broken`: a rung the instrument could not measure
    /// has not failed, and reporting it as a break would put a redline on the
    /// board that describes the observer rather than the machine.
    pub inconclusive: Option<String>,
    /// Conditions that broke here. Empty means the rung held.
    pub broken: Vec<LimitingCondition>,
}

/// The ramp's artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RampReport {
    pub label: String,
    pub command: Vec<String>,
    pub mode: SessionMode,
    pub machine: Machine,
    pub provenance: Provenance,
    pub host: HostFacts,
    pub membership: Membership,
    pub membership_fallback_reason: Option<String>,
    /// Set when the sampling thread could not be raised above the sessions.
    ///
    /// Every figure below is suspect while this is present, because a starved
    /// sampler reports its own scheduling as the machine's behaviour.
    pub sampler_unprioritised_reason: Option<String>,
    pub started_unix: u64,
    pub interval_ms: u64,
    pub hold_ms: u64,
    /// How tight the bracket was narrowed before the redline was reported.
    pub resolution: u32,
    /// Whether the sessions' writes were hidden from real-time scanning.
    pub scratch_excluded: bool,
    /// Per-session rate at one session, which every later rung is read against.
    pub solo_units_per_sec: f64,
    pub steps: Vec<Step>,
    /// `None` when the ladder ran out before any condition broke, which means
    /// the redline is somewhere above what was tried rather than unknown.
    pub redline: Option<Redline>,
    /// The lowest saturated rung, held once more after the ladder finished.
    ///
    /// A control rather than a rung. Compared against its own first reading in
    /// `steps` it says whether the machine ran the same at the end as at the
    /// start — which every other figure here assumes and none of them checks.
    pub drift_check: Option<Step>,
}

impl RampReport {
    /// Compares the repeated rung against its own first reading.
    ///
    /// Lives here rather than in either reporter because both of them need it
    /// and a figure computed twice is a figure that ends up disagreeing with
    /// itself.
    pub fn drift(&self) -> Option<Drift> {
        let again = self.drift_check.as_ref()?;
        if let Some(reason) = &again.inconclusive {
            return Some(Drift::Unmeasurable(reason.clone()));
        }
        let early = self
            .steps
            .iter()
            .find(|step| step.sessions == again.sessions)?
            .units_per_session_per_sec;
        let late = again.units_per_session_per_sec;
        (early > 0.0 && late > 0.0).then(|| Drift::Measured {
            sessions: again.sessions,
            early_units_per_sec: early,
            late_units_per_sec: late,
            slower_percent: (early - late) / early * 100.0,
        })
    }
}

/// Runs the ladder and writes the artifact into `config.out_dir`.
pub fn run(config: &RampConfig) -> Result<RampReport> {
    let machine = Machine::detect();
    let provenance = Provenance::current();
    let host = HostFacts::query();
    let budget = (machine.total_memory_bytes as f64 * RSS_BUDGET_FRACTION) as u64;

    fs::create_dir_all(&config.out_dir)
        .with_context(|| format!("creating {}", config.out_dir.display()))?;
    let started_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();

    // One job for the whole ramp, joined before any session exists. Every
    // session lands in it by inheritance, so the aggregate is the kernel's
    // count rather than a hundred separate walks summed up.
    let armed = ArmedTree::arm(Pid::from_u32(std::process::id()));
    let membership = armed.membership();
    let membership_fallback_reason = armed.fallback_reason.clone();
    if let Some(reason) = &membership_fallback_reason {
        eprintln!("warning: falling back to parent-walk membership — {reason}");
    }
    let mut tree = armed.attach_pool();
    let mut sampler = Sampler::new();

    // Held for the whole ramp when asked for, over the sessions' scratch root
    // and nothing else. Dropped at the end of this function, and again by its
    // own guard if anything below returns early.
    let scratch_root = scratch_root(&config.out_dir);
    fs::create_dir_all(&scratch_root)?;
    let exclusion = match config.exclude_scratch {
        false => None,
        true => {
            let held = crate::host::HeldExclusion::add(&scratch_root)
                .map_err(|error| anyhow::anyhow!(error))?;
            println!("excluded {} from real-time scanning", held.path());
            Some(held)
        }
    };

    let mut search = Search {
        config,
        tree: &mut tree,
        sampler: &mut sampler,
        budget,
        solo_units_per_sec: 0.0,
        steps: Vec::new(),
    };

    // Climb until a rung breaks. This is what makes the break an observation:
    // both ends of the interval it leaves behind have been measured.
    let mut held_at = 0;
    let mut bracket = None;
    for sessions in RAMP_STEPS.into_iter().filter(|n| *n <= config.max_sessions) {
        match search.measure(sessions)? {
            Outcome::Held => held_at = sessions,
            Outcome::Broke(condition) => {
                bracket = Some((held_at, sessions, condition));
                break;
            }
            // The ladder stopped because the instrument ran out, not because
            // the machine did, so there is nothing to bracket.
            Outcome::Unmeasurable => break,
        }
    }

    // Refine inside the bracket. Halving an interval whose ends have both been
    // observed assumes nothing about behaviour outside it, which is the whole
    // difference between this and bisecting the range from the start.
    let mut redline = None;
    if let Some((mut held, mut broke, mut condition)) = bracket {
        while broke - held > config.resolution.max(1) {
            let probe = held + (broke - held) / 2;
            match search.measure(probe)? {
                Outcome::Held => held = probe,
                Outcome::Broke(next) => {
                    broke = probe;
                    condition = next;
                }
                Outcome::Unmeasurable => break,
            }
        }
        // Only work rate. An edge is absent and then present, so there is
        // nothing between two rungs to interpolate. RSS and replacement lag
        // are slopes and could each be fitted, but against their own quantity
        // and their own budget — fitting slowdown against the work-rate budget
        // would answer a question nobody asked when one of those is what broke.
        let fitted = (condition == LimitingCondition::WorkRate)
            .then(|| {
                let solo = search.solo_units_per_sec;
                let rungs: Vec<(u32, f64)> = search
                    .steps
                    .iter()
                    .filter(|step| {
                        step.inconclusive.is_none() && step.units_per_session_per_sec > 0.0
                    })
                    .map(|step| (step.sessions, solo / step.units_per_session_per_sec))
                    .collect();
                redline::fit_crossing(&rungs, WORK_RATE_BUDGET_FACTOR, held)
            })
            .flatten();

        redline = Some(Redline {
            // The fit is drawn through every saturated rung, so it survives a
            // single one landing off; `held` is whichever rung the search
            // happened to stop on.
            sessions: fitted.map_or(held, |fit| fit.crossing.floor() as u32),
            limited_by: condition,
            fitted,
        });
    }

    // The control. Every figure above assumes the machine at the last rung is
    // the machine that was at the first, and nothing so far has checked it.
    // Averaging over rungs removes noise but carries drift straight through:
    // if later rungs run slower for being later, the fitted slope steepens and
    // the redline reads low, faithfully.
    //
    // Repeat the lowest saturated rung. Same session count, different position,
    // which is the one comparison that separates the two — three runs found
    // rung 32 reading 2.06 against rung 34's 2.01 in the same runs, and 32 had
    // been measured last every time.
    let drift_check = if config.skip_drift_check {
        None
    } else {
        let solo = search.solo_units_per_sec;
        let repeat = search
            .steps
            .iter()
            .filter(|step| {
                step.inconclusive.is_none()
                    && step.units_per_session_per_sec > 0.0
                    && redline::is_saturated(solo / step.units_per_session_per_sec)
            })
            .map(|step| step.sessions)
            .min();
        match repeat {
            Some(sessions) => {
                println!("\ndrift check: holding {sessions} session(s) again");
                search.measure_as(sessions, &format!("drift-{sessions:03}"))?;
                // Off the ladder and into its own field: it is a control rather
                // than a rung, and leaving it in `steps` would put the same
                // session count in the curve twice.
                search.steps.pop()
            }
            None => None,
        }
    };

    let solo_units_per_sec = search.solo_units_per_sec;
    let steps = search.steps;

    let report = RampReport {
        label: config.label.clone(),
        command: config.command.clone(),
        mode: config.mode,
        machine,
        provenance,
        host,
        membership,
        membership_fallback_reason,
        sampler_unprioritised_reason: sampler.unprioritised_reason().map(str::to_string),
        started_unix,
        interval_ms: config.interval.as_millis() as u64,
        hold_ms: config.hold.as_millis() as u64,
        resolution: config.resolution.max(1),
        scratch_excluded: config.exclude_scratch,
        solo_units_per_sec,
        steps,
        redline,
        drift_check,
    };

    // Two artifacts: the record, and the thing anyone actually reads. The
    // second is generated rather than transcribed, because a figure retyped is
    // a figure that can be retyped wrong.
    // Removed before the artifacts are written, so the machine is not left
    // unprotected while a report renders.
    if let Some(mut held) = exclusion {
        held.remove().map_err(|error| anyhow::anyhow!(error))?;
        println!("exclusion removed");
    }
    let _ = fs::remove_dir_all(&scratch_root);

    fs::write(
        config.out_dir.join("ramp.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    fs::write(
        config.out_dir.join("ramp.md"),
        crate::report::ramp_markdown(&report),
    )?;
    Ok(report)
}

/// What one rung said.
#[derive(Debug, Clone, Copy)]
enum Outcome {
    Held,
    Broke(LimitingCondition),
    Unmeasurable,
}

/// The climb and the refinement, sharing one job, one sampler, and one solo
/// figure to read every later rung against.
struct Search<'a> {
    config: &'a RampConfig,
    tree: &'a mut SessionTree,
    sampler: &'a mut Sampler,
    budget: u64,
    solo_units_per_sec: f64,
    steps: Vec<Step>,
}

impl Search<'_> {
    /// Runs one rung of the ladder.
    fn measure(&mut self, sessions: u32) -> Result<Outcome> {
        self.measure_as(sessions, &format!("step-{sessions:03}"))
    }

    /// Runs one rung and records it.
    ///
    /// `tag` names the directory this rung's logs land in. It exists because
    /// the drift check holds a count the ladder already held, and a directory
    /// named after the count alone would leave `step-025` holding the repeat's
    /// logs while the report described the original's.
    fn measure_as(&mut self, sessions: u32, tag: &str) -> Result<Outcome> {
        println!("\nholding {sessions} session(s) for {:?}", self.config.hold);
        let step_dir = self.config.out_dir.join(tag);
        fs::create_dir_all(&step_dir)?;

        let mut step = hold(
            self.config,
            &step_dir,
            sessions,
            self.tree,
            self.sampler,
            self.budget,
        )?;

        if step.inconclusive.is_none() {
            if sessions == 1 {
                self.solo_units_per_sec = step.units_per_session_per_sec;
            }
            // The work-rate condition only means anything against the solo
            // figure, so it is checked here rather than inside the hold.
            if self.solo_units_per_sec > 0.0
                && self.solo_units_per_sec / step.units_per_session_per_sec.max(f64::EPSILON)
                    > WORK_RATE_BUDGET_FACTOR
            {
                step.broken.push(LimitingCondition::WorkRate);
            }
        }

        print_step(&step, self.solo_units_per_sec);
        let outcome = match (step.inconclusive.is_some(), step.broken.first().copied()) {
            (true, _) => Outcome::Unmeasurable,
            (false, Some(condition)) => Outcome::Broke(condition),
            (false, None) => Outcome::Held,
        };
        self.steps.push(step);
        Ok(outcome)
    }
}

/// The one directory every rung's sessions write under.
///
/// One root rather than one per rung, so a single Defender exclusion can cover
/// the whole ramp — and cover only what the sessions wrote, leaving this
/// process's own logs and samples on the scanned side of the line where they
/// belong.
pub fn scratch_root(out_dir: &Path) -> PathBuf {
    out_dir.join("scratch")
}

/// Where one rung's sessions write.
///
/// Removed when the rung ends, which is the only chance anything has to remove
/// it: the rung ends by killing every session, and a killed process runs no
/// cleanup.
fn scratch_dir(out_dir: &Path, sessions: u32) -> PathBuf {
    scratch_root(out_dir).join(format!("step-{sessions:03}"))
}

/// Holds `sessions` alive for one hold window and measures what it cost.
fn hold(
    config: &RampConfig,
    step_dir: &Path,
    sessions: u32,
    tree: &mut SessionTree,
    sampler: &mut Sampler,
    budget: u64,
) -> Result<Step> {
    let scratch = scratch_dir(&config.out_dir, sessions);
    fs::create_dir_all(&scratch)?;
    let mut pool = Pool::new(config, step_dir, sessions, &scratch, tree)?;
    let mut samples_file = BufWriter::new(File::create(step_dir.join("samples.jsonl"))?);

    let started = Instant::now();
    let spinup = config
        .hold
        .mul_f64(SPINUP_FRACTION)
        .max(crate::observe::STARTUP_WINDOW);
    let mut measured: Vec<Sample> = Vec::new();
    let mut units_at_spinup = None;
    let mut last: Option<Sample> = None;
    let mut worst_tick = TickCost::default();

    while started.elapsed() < config.hold {
        // Paced from the top of each tick rather than sleeping a full interval
        // after the work: under load the work is the interval, and adding one
        // on top turns a fifteen second hold into forty.
        let tick = Instant::now();
        let mut cost = TickCost::default();

        let at = Instant::now();
        let tracked = tree.known_pids();
        sampler.refresh(tracked.as_deref());
        cost.refresh_ms = at.elapsed().as_millis() as u64;

        let at = Instant::now();
        pool.replace_finished(config, step_dir, &scratch, tree)?;
        cost.replace_ms = at.elapsed().as_millis() as u64;

        let at = Instant::now();
        let sample = sampler.sample(tree, &pool.aggregate_output(), started.elapsed());
        cost.sample_ms = at.elapsed().as_millis() as u64;

        let at = Instant::now();
        writeln!(samples_file, "{}", serde_json::to_string(&sample)?)?;
        samples_file.flush()?;
        cost.write_ms = at.elapsed().as_millis() as u64;
        worst_tick.keep_worse(cost);

        if started.elapsed() >= spinup {
            if units_at_spinup.is_none() {
                units_at_spinup = Some((pool.total_units(), started.elapsed()));
            }
            measured.push(sample.clone());
        }
        last = Some(sample);

        if let Some(rest) = config.interval.checked_sub(tick.elapsed()) {
            std::thread::sleep(rest);
        }
    }

    let (units_start, at) = units_at_spinup.unwrap_or((pool.total_units(), started.elapsed()));
    let measured_secs = (started.elapsed() - at).as_secs_f64().max(f64::EPSILON);
    let units = pool.total_units().saturating_sub(units_start);
    let units_per_session_per_sec = units as f64 / measured_secs / f64::from(sessions);

    let replacements = pool.replacements;
    let worst_replacement_secs = pool.worst_replacement_secs;

    // Read before anything is torn down, since the counters live in the slots.
    let dropped_units = pool.dropped_units();

    let mut teardown = TeardownCost::default();
    let at = Instant::now();
    pool.kill_all();
    teardown.kill_ms = at.elapsed().as_millis() as u64;

    let at = Instant::now();
    // Released before the reap, not after. Terminating a pseudoconsole host
    // while this process still holds its handle costs well over a second each,
    // serially: at a hundred sessions that was four minutes of teardown for a
    // thirty second rung. Closing the handles first asks the hosts to leave
    // instead of killing them, and the release itself measures in single-digit
    // milliseconds at every rung.
    pool.release();
    teardown.release_ms = at.elapsed().as_millis() as u64;

    if let Some(sample) = last.as_ref() {
        let (waited, stragglers) = sampler.wait_for_exit(sample, EXIT_GRACE);
        teardown.settle_ms = waited;
        teardown.stragglers = stragglers;
    }

    let (refresh_ms, kill_ms) = sampler.reap(last.as_ref());
    teardown.reap_refresh_ms = refresh_ms;
    teardown.reap_kill_ms = kill_ms;

    let at = Instant::now();
    // Errors ignored on purpose: a file still held by a session that has not
    // finished dying is not worth failing a measured rung over, and the next
    // run starts from a fresh output directory anyway.
    let _ = fs::remove_dir_all(&scratch);
    teardown.scratch_ms = at.elapsed().as_millis() as u64;

    let elapsed_secs = started.elapsed().as_secs_f64();
    // Two different failures, named apart. A hold that never outlasts its own
    // spin-up has nothing to measure by construction; a hold that did outlast
    // it and still came back with almost nothing was starved. Calling both
    // "the sampler could not keep up" sends the next reader to fix the wrong
    // thing.
    let inconclusive = (measured.len() < MIN_SAMPLES_PER_RUNG).then(|| {
        if config.hold <= spinup {
            format!(
                "a {:.1}s hold leaves nothing after the {:.1}s spin-up, which is where sessions are still faulting in their pages",
                config.hold.as_secs_f64(),
                spinup.as_secs_f64(),
            )
        } else {
            format!(
                "{} sample(s) over {elapsed_secs:.1}s of a {:.1}s hold — the sampler could not keep up, so nothing here describes the machine",
                measured.len(),
                config.hold.as_secs_f64(),
            )
        }
    });

    let total_rss_bytes = median(measured.iter().map(|s| s.rss_bytes)).unwrap_or(0);
    let mut broken = Vec::new();
    if inconclusive.is_none() {
        if total_rss_bytes > budget {
            broken.push(LimitingCondition::Rss);
        }
        if dropped_units > 0 {
            broken.push(LimitingCondition::OutputDrop);
        }
        if worst_replacement_secs.is_some_and(|s| s > REPLACEMENT_BUDGET_SECS as f64) {
            broken.push(LimitingCondition::ReplacementLag);
        }
    }

    Ok(Step {
        sessions,
        samples: measured.len(),
        total_rss_bytes,
        rss_budget_bytes: budget,
        units_per_session_per_sec,
        session_cores: mean(measured.iter().map(|s| f64::from(s.cpu_percent) / 100.0)),
        defender_cores: mean(
            measured
                .iter()
                .map(|s| f64::from(s.defender_cpu_percent.unwrap_or(0.0)) / 100.0),
        ),
        processes: measured.iter().map(|s| s.processes).max().unwrap_or(0),
        pseudoconsoles: measured.iter().map(|s| s.pseudoconsoles).max().unwrap_or(0),
        replacements,
        worst_replacement_secs,
        dropped_units,
        elapsed_secs,
        worst_tick,
        teardown,
        inconclusive,
        broken,
    })
}

/// The N sessions a rung keeps alive.
struct Pool {
    slots: Vec<Slot>,
    /// Units from sessions that have already finished, so the running total
    /// never goes backwards when one is replaced.
    retired_units: u64,
    /// Drops from those same sessions, kept for the same reason.
    retired_dropped: u64,
    replacements: u32,
    worst_replacement_secs: Option<f64>,
}

struct Slot {
    index: u32,
    generation: u32,
    session: Session,
    output: Arc<Output>,
}

impl Pool {
    fn new(
        config: &RampConfig,
        step_dir: &Path,
        sessions: u32,
        scratch: &Path,
        tree: &mut SessionTree,
    ) -> Result<Self> {
        let mut slots = Vec::with_capacity(sessions as usize);
        for index in 0..sessions {
            slots.push(start(config, step_dir, scratch, index, 0, tree)?);
        }
        Ok(Self {
            slots,
            retired_units: 0,
            retired_dropped: 0,
            replacements: 0,
            worst_replacement_secs: None,
        })
    }

    /// Restarts any session that finished, timing how long the gap lasted.
    ///
    /// The gap is measured across one sampling interval rather than to the
    /// millisecond, since that is the resolution the whole run has.
    fn replace_finished(
        &mut self,
        config: &RampConfig,
        step_dir: &Path,
        scratch: &Path,
        tree: &mut SessionTree,
    ) -> Result<()> {
        for slot in &mut self.slots {
            if slot.session.try_wait()?.is_none() {
                continue;
            }
            let replacing = Instant::now();
            self.retired_units += slot.output.units();
            self.retired_dropped += slot.output.dropped_units();
            slot.generation += 1;
            *slot = start(config, step_dir, scratch, slot.index, slot.generation, tree)?;

            let took = replacing.elapsed().as_secs_f64() + config.interval.as_secs_f64();
            self.replacements += 1;
            self.worst_replacement_secs = Some(
                self.worst_replacement_secs
                    .map_or(took, |worst: f64| worst.max(took)),
            );
        }
        Ok(())
    }

    fn total_units(&self) -> u64 {
        self.retired_units + self.slots.iter().map(|s| s.output.units()).sum::<u64>()
    }

    /// Units the sessions announced but never delivered.
    ///
    /// Read from the counters the drains keep rather than by scanning the logs
    /// afterwards, which is what lets the logs be capped: a workload whose
    /// payload is its output would otherwise make the disk the ceiling of the
    /// axis that exists to measure the output path.
    fn dropped_units(&self) -> u64 {
        self.retired_dropped
            + self
                .slots
                .iter()
                .map(|s| s.output.dropped_units())
                .sum::<u64>()
    }

    /// A view of the pool's output for the sampler, which wants one counter.
    fn aggregate_output(&self) -> Output {
        Output::from_counts(
            self.slots.iter().map(|s| s.output.total()).sum(),
            self.total_units(),
        )
    }

    /// Terminates every session's root process.
    ///
    /// Their children survive this, which is what `Sampler::reap` is for.
    fn kill_all(&mut self) {
        for slot in &mut self.slots {
            let _ = slot.session.kill();
        }
    }

    /// Releases the pty handles.
    ///
    /// Separate from killing, and ordered after the reap, because a
    /// pseudoconsole cannot close while its host is alive: the drain thread
    /// holds a reader on it, and the reader cannot reach end-of-file until the
    /// host exits. Dropping the handles first is how a rung ends up waiting on
    /// two things that are each waiting on the other.
    fn release(&mut self) {
        self.slots.clear();
    }
}

fn start(
    config: &RampConfig,
    step_dir: &Path,
    scratch: &Path,
    index: u32,
    generation: u32,
    tree: &mut SessionTree,
) -> Result<Slot> {
    let base = step_dir.join(format!("s{index:03}-g{generation:02}"));
    let spawned = session::spawn(&config.command, config.mode, &base, scratch)?;
    if let Some(pid) = spawned.session.pid() {
        tree.add_root(Pid::from_u32(pid));
    }
    // The drains are detached rather than joined: a replaced session's counter
    // has already been read, and joining here would stall the rung.
    Ok(Slot {
        index,
        generation,
        session: spawned.session,
        output: spawned.output,
    })
}

fn median(values: impl Iterator<Item = u64>) -> Option<u64> {
    let mut sorted: Vec<u64> = values.collect();
    sorted.sort_unstable();
    sorted.get(sorted.len() / 2).copied()
}

fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let (sum, count) = values.fold((0.0, 0usize), |(sum, count), v| (sum + v, count + 1));
    if count == 0 { 0.0 } else { sum / count as f64 }
}

fn print_step(step: &Step, solo: f64) {
    let ratio = if step.units_per_session_per_sec > 0.0 {
        solo / step.units_per_session_per_sec
    } else {
        f64::INFINITY
    };
    println!(
        "  {:>3} sessions  rss {:>10}  {:.2} units/s/session ({ratio:.2}x solo)  {:.1} cores  {} procs",
        step.sessions,
        human_bytes(step.total_rss_bytes),
        step.units_per_session_per_sec,
        step.session_cores + step.defender_cores,
        step.processes,
    );
    let tick = &step.worst_tick;
    println!(
        "      worst tick {} ms = refresh {} + replace {} + sample {} + write {}",
        tick.total_ms(),
        tick.refresh_ms,
        tick.replace_ms,
        tick.sample_ms,
        tick.write_ms
    );
    let down = &step.teardown;
    println!(
        "      teardown {} ms = kill {} + release {} + settle {} + reap {}+{} + scratch {} ({} straggler(s))",
        down.total_ms(),
        down.kill_ms,
        down.release_ms,
        down.settle_ms,
        down.reap_refresh_ms,
        down.reap_kill_ms,
        down.scratch_ms,
        down.stragglers
    );
    match (&step.inconclusive, step.broken.is_empty()) {
        (Some(reason), _) => println!("      INCONCLUSIVE: {reason}"),
        (None, true) => println!("      held"),
        (None, false) => println!("      BROKE: {:?}", step.broken),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rung(sessions: u32, units_per_session_per_sec: f64) -> Step {
        Step {
            sessions,
            units_per_session_per_sec,
            ..Step::default()
        }
    }

    /// A report carrying only what the drift comparison reads.
    fn report_with(steps: Vec<Step>, drift_check: Option<Step>) -> RampReport {
        RampReport {
            label: String::new(),
            command: Vec::new(),
            mode: SessionMode::Pipe,
            // Defaults rather than the real thing: neither is read by the
            // drift comparison, and `Provenance::current` shells out to git.
            machine: Machine::default(),
            provenance: Provenance::default(),
            host: HostFacts::default(),
            membership: Membership::ParentWalk,
            membership_fallback_reason: None,
            sampler_unprioritised_reason: None,
            started_unix: 0,
            interval_ms: 0,
            hold_ms: 0,
            resolution: 1,
            scratch_excluded: false,
            solo_units_per_sec: 60.0,
            steps,
            redline: None,
            drift_check,
        }
    }

    #[test]
    fn a_machine_that_slowed_reports_how_much() {
        // The figures from the run that first used the control.
        let report = report_with(vec![rung(25, 40.02)], Some(rung(25, 38.11)));
        let Some(Drift::Measured { slower_percent, .. }) = report.drift() else {
            panic!("expected a measured drift");
        };
        assert!((slower_percent - 4.77).abs() < 0.01, "got {slower_percent}");
    }

    #[test]
    fn a_machine_that_held_still_reports_near_zero() {
        let report = report_with(vec![rung(25, 40.00)], Some(rung(25, 40.00)));
        let Some(Drift::Measured { slower_percent, .. }) = report.drift() else {
            panic!("expected a measured drift");
        };
        assert_eq!(slower_percent, 0.0);
    }

    #[test]
    fn a_repeat_that_could_not_be_measured_is_not_a_machine_that_stopped() {
        // Zero units is what an unmeasurable rung carries, and reporting it as
        // a 100% slowdown would describe the observer rather than the machine.
        let mut unusable = rung(25, 0.0);
        unusable.inconclusive = Some("too few samples".to_string());
        let report = report_with(vec![rung(25, 40.02)], Some(unusable));
        assert_eq!(
            report.drift(),
            Some(Drift::Unmeasurable("too few samples".to_string()))
        );
    }

    #[test]
    fn no_repeat_and_no_matching_rung_both_give_nothing() {
        assert_eq!(report_with(vec![rung(25, 40.0)], None).drift(), None);
        // The ladder never held the count the control repeated, so there is
        // nothing to compare it against.
        assert_eq!(
            report_with(vec![rung(31, 33.0)], Some(rung(25, 38.0))).drift(),
            None
        );
    }
}
