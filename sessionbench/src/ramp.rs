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
//! **That paragraph was a description and is now enforced.** It holds for pipe
//! and pty because this file restarts what exits, and a target that holds the
//! sessions on the ramp's behalf cannot be assumed to. So a rung may report the
//! fewest it saw alive, and one that fell short of what it asked for is
//! inconclusive rather than slow.
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
use std::sync::{Arc, Mutex};
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
/// let page faults into the measurement. [A single session at quarter duty read
/// 0.4 cores that way against a true
/// 0.24](../../docs/measurements/2026-07-30-120002-first-redlines.md), and
/// 0.240 once the window cleared thirty seconds — which is the measurement this
/// constant exists because of, and the floor below is what it bought.
const SPINUP_FRACTION: f64 = 1.0 / 3.0;

/// Fewest measured samples a rung needs before its verdict means anything.
///
/// A saturated machine can starve the sampler badly enough that a fifteen
/// second hold yields one reading forty seconds late. Everything derived from
/// that reads as a catastrophic collapse — zero work, zero cores — and the
/// first ramp run reported exactly that as a broken condition. A rung that
/// could not be measured is not a rung that failed.
///
/// **This comment is the only surviving record of that run**, which is worth
/// noticing rather than fixing: no measurement record holds it, the artifacts
/// are long pruned, and the observation exists here because someone wrote it
/// beside the constant instead of into `docs/measurements/`. A figure that
/// decided something and was never written up is not recoverable later.
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
    /// Skip both end-of-run controls.
    ///
    /// One hold each: the lowest saturated rung again, which says whether the
    /// ladder measured one machine or several, and the solo rung again, which
    /// says whether the baseline every rate is read against reproduces. Worth
    /// skipping only when the ramp is being run for its shape rather than its
    /// number.
    pub skip_drift_check: bool,
    pub mode: SessionMode,
    /// Hold each rung's sessions under this daemon instead of spawning them
    /// here.
    ///
    /// **Not a [`SessionMode`] variant, and the type's own doc is why.** That
    /// enum says how a session's *output* is wired, and a daemon-held session
    /// is still on pipes — a report reading `mode: daemon` would tell a reader
    /// only that it was not a pseudoconsole. The two are axes that multiply:
    /// nothing stops a daemon rung holding pty sessions later, and folding
    /// them into one enum makes that unsayable.
    pub daemon: Option<PathBuf>,
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
    /// Least physical memory the machine had free during the rung.
    ///
    /// The RSS condition compares a working set against a budget, and a
    /// working set falls once Windows starts paging — so the condition goes
    /// quiet during the failure it exists to catch. This is the machine's own
    /// account of the same moment, and a rung that held while this approached
    /// zero held on paper.
    pub min_available_memory_bytes: u64,
    /// The figure the work-rate condition compares against its solo value.
    pub units_per_session_per_sec: f64,
    pub session_cores: f64,
    /// What this rung held of the machine, median first.
    ///
    /// **`session_cores` above is a mean, and a mean hides an interruption.**
    /// A rung that lost the machine for part of its window reads as a rung that
    /// was slower, which is what a ladder calls saturation — so a redline can
    /// be set by something that was not the sessions. Three twenty-minute holds
    /// showed the size of it: one lost 1.173 cores of 15.39 where two others
    /// lost 0.061 and 0.052.
    pub occupancy: Option<crate::sampler::Occupancy>,
    pub defender_cores: f64,
    pub processes: usize,
    pub pseudoconsoles: usize,
    /// Sessions that finished and were started again during the hold.
    pub replacements: u32,
    /// The slowest replacement, which is what the condition is set against.
    pub worst_replacement_secs: Option<f64>,
    /// Units a session reported starting but never wrote a line for.
    ///
    /// **`None` means the rung could not look, and that is not zero.** Gaps
    /// are found by watching ordinals in a session's own output, which needs
    /// this process to hold the reading end. A target where something else
    /// holds it — a daemon draining into its own scrollback — leaves nothing
    /// to count, and reporting `0` there would put a condition's tolerance of
    /// zero against a number nobody measured. The condition is then skipped
    /// and [the report says so](../report.rs) rather than passing it.
    pub dropped_units: Option<u64>,
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
    /// The solo rung, held once more after everything else.
    ///
    /// Every rate in this report is read against one measured window at one
    /// session, taken once and never repeated — and that same figure is the
    /// fingerprint [`compare`](crate::compare) uses to decide whether two ramps
    /// may be set against each other. It is load-bearing twice and was measured
    /// neither time. This is what a rung reproduces to under the run's own
    /// conditions, which is the floor on any difference two ramps can claim.
    #[serde(default)]
    pub solo_check: Option<Step>,
}

impl RampReport {
    /// Compares the repeated rung against its own first reading.
    ///
    /// Lives here rather than in either reporter because both of them need it
    /// and a figure computed twice is a figure that ends up disagreeing with
    /// itself.
    /// How far the solo rung moved when it was held a second time, as a
    /// percentage of the first reading.
    ///
    /// The floor under any cross-ramp claim: two ramps cannot be told apart
    /// more finely than one ramp reproduces its own baseline.
    pub fn solo_spread_percent(&self) -> Option<f64> {
        let again = self.solo_check.as_ref()?;
        if again.inconclusive.is_some() || self.solo_units_per_sec <= 0.0 {
            return None;
        }
        Some(
            (again.units_per_session_per_sec - self.solo_units_per_sec) / self.solo_units_per_sec
                * 100.0,
        )
    }

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

    // The fingerprint's own error bar, and the last thing measured so it also
    // catches a machine that moved after the drift control cleared.
    let solo_check = if config.skip_drift_check {
        None
    } else {
        println!("\nsolo check: holding 1 session again");
        search.measure_as(1, "solo-repeat")?;
        search.steps.pop()
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
        solo_check,
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
            // Only the ladder's own first rung sets the baseline. The solo
            // control is also a one-session rung, and letting it through here
            // made it overwrite the figure it exists to check: the first ramp
            // to carry it read 77.24 units/s at rung one, repeated at 79.34,
            // and reported +0.0% because by then it was comparing 79.34 with
            // itself. A control that cannot fail is not a control, and it took
            // the report's whole against-solo column with it.
            if sessions == 1 && self.solo_units_per_sec <= 0.0 {
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

    let mut daemon_left_early = None;
    while started.elapsed() < config.hold {
        // **A daemon rung has to ask whether its daemon is still there.** Its
        // reports stop and the watch keeps the last one, so every figure goes
        // on reading as it did at the moment things went wrong. `hold` learned
        // this first and the fix lived only there; the two loops are separate
        // code and the second had the same hole.
        //
        // `peak_processes == 0` catches a daemon that never started, not one
        // that left after a few samples.
        if let Some(why) = pool.daemon_left(started.elapsed()) {
            daemon_left_early = Some(why);
            break;
        }

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
    let peak_processes = measured.iter().map(|s| s.processes).max().unwrap_or(0);
    let inconclusive = why_unmeasurable(&Reading {
        samples: measured.len(),
        peak_processes,
        sessions,
        replacements,
        hold: config.hold,
        spinup,
        elapsed_secs,
        fewest_alive: pool.fewest_alive(),
        left_early: daemon_left_early,
    });

    let total_rss_bytes = median(measured.iter().map(|s| s.rss_bytes)).unwrap_or(0);
    let mut broken = Vec::new();
    if inconclusive.is_none() {
        if total_rss_bytes > budget {
            broken.push(LimitingCondition::Rss);
        }
        // Some(0) is a measurement and None is an absence, and only the first
        // may satisfy a condition whose tolerance is zero.
        if dropped_units.is_some_and(|n| n > 0) {
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
        min_available_memory_bytes: measured
            .iter()
            .map(|s| s.available_memory_bytes)
            .min()
            .unwrap_or(0),
        units_per_session_per_sec,
        session_cores: mean(measured.iter().map(|s| f64::from(s.cpu_percent) / 100.0)),
        occupancy: crate::sampler::Occupancy::of(&measured),
        defender_cores: mean(
            measured
                .iter()
                .map(|s| f64::from(s.defender_cpu_percent.unwrap_or(0.0)) / 100.0),
        ),
        processes: peak_processes,
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

/// What a rung came back with, before anything is concluded from it.
struct Reading {
    samples: usize,
    peak_processes: usize,
    sessions: u32,
    replacements: u32,
    hold: Duration,
    spinup: Duration,
    elapsed_secs: f64,
    /// The fewest sessions seen alive at any sample, when the target can say.
    ///
    /// `None` for targets that cannot lose one without the ramp noticing:
    /// pipe and pty hold a slot each and restart what exits, so the count
    /// asked for is the count held by construction. A daemon holds them on
    /// the ramp's behalf and does not restart anything, so it has to be
    /// asked.
    fewest_alive: Option<u32>,
    /// Set when whatever held the rung's sessions stopped before the rung did.
    left_early: Option<String>,
}

/// Why a rung describes the observer rather than the machine, when it does.
///
/// Separate from `hold_rung` so it can be exercised without spawning anything.
/// All three cases below were found by a run going wrong rather than by a
/// test, which is the argument for making the decision reachable from one.
fn why_unmeasurable(reading: &Reading) -> Option<String> {
    // First, because it explains every other symptom the rung will show. A
    // sampler with nothing left to sample reads as a collapse, and the
    // collapse is not what happened.
    if let Some(why) = &reading.left_early {
        return Some(why.clone());
    }

    // Two ways to come back with too little, named apart. A hold that never
    // outlasts its own spin-up has nothing to measure by construction; a hold
    // that did outlast it and still came back with almost nothing was starved.
    // Calling both "the sampler could not keep up" sends the next reader to
    // fix the wrong thing.
    if reading.samples < MIN_SAMPLES_PER_RUNG {
        return Some(if reading.hold <= reading.spinup {
            format!(
                "a {:.1}s hold leaves nothing after the {:.1}s spin-up, which is where sessions are still faulting in their pages",
                reading.hold.as_secs_f64(),
                reading.spinup.as_secs_f64(),
            )
        } else {
            format!(
                "{} sample(s) over {:.1}s of a {:.1}s hold — the sampler could not keep up, so nothing here describes the machine",
                reading.samples,
                reading.elapsed_secs,
                reading.hold.as_secs_f64(),
            )
        });
    }

    // The third, and the only one that reads as a result rather than a
    // failure. A command that cannot start still produces a work rate, because
    // the pool keeps respawning it: a wrapper mistyped by the shell once gave
    // four rungs of "held" at zero bytes resident and three thousand
    // replacements, and the ramp reported a floor of fifty sessions from it.
    // Nothing was ever running to scale.
    if reading.peak_processes == 0 {
        return Some(format!(
            "{} session(s) asked for and not one process resident across {} sample(s), with {} replacement(s) — the command exits faster than it can be seen, so this rung measured the spawn loop and not a workload",
            reading.sessions, reading.samples, reading.replacements,
        ));
    }

    // The fourth, and the only one written before a run went wrong rather than
    // after. Per-session work rate divides by the count the rung asked for,
    // which pipe and pty may do because they restart what exits. A target that
    // holds the sessions itself and does not restart them breaks that: the
    // numerator falls as sessions die, the denominator does not, the rate
    // reads low, low reads as saturation, and the ladder returns a redline
    // from a target that was working perfectly.
    //
    // Dividing by the live count instead would produce a number, and it would
    // be a number about a different session count than the one on the rung.
    reading
        .fewest_alive
        .filter(|alive| *alive < reading.sessions)
        .map(|alive| {
            format!(
                "{} session(s) asked for and only {alive} alive at some sample — the rung stopped being a rung at {} sessions, and dividing by either count would describe something nobody asked for",
                reading.sessions, reading.sessions,
            )
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
    /// Present when one process holds the rung's sessions instead of this one.
    ///
    /// **The seam, and it is deliberately one field rather than a second
    /// `Pool`.** Everything below the three methods that consult it — the
    /// sampler, the tree, the redline arithmetic, the report — stays shared,
    /// because a second pool is a second bench and [the M0 baseline stops
    /// being comparable](../../ROADMAP.md#m1--headless-daemon) the moment
    /// that happens.
    ///
    /// What it changes is small and exact: units come from the daemon's own
    /// report rather than from drains this process owns, dropped output
    /// becomes unmeasurable rather than zero, and the live session count has
    /// to be asked for because nothing here restarts what exits.
    daemon: Option<Arc<Mutex<crate::daemon::Watch>>>,
    /// The daemon itself, when there is one.
    ///
    /// **Two fields rather than one, and the split is deliberate.** The three
    /// methods above need only what the daemon *said*, which is a watch and
    /// can be exercised without spawning anything; teardown needs the process.
    /// Folding them into one would make every test of the query half spawn a
    /// daemon, and the tests are what caught this seam's own silent zero.
    ///
    /// [`Pool::new`] is the only place that sets them, and it sets both.
    daemon_process: Option<crate::daemon::Held>,
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
        // A daemon rung is one process holding N, so it spawns nothing here.
        // The tree is already armed by the caller, and job membership is
        // inherited at creation — the daemon and everything under it land in
        // the measurement without being told to.
        if let Some(path) = &config.daemon {
            let held = crate::daemon::Held::start(
                path,
                sessions,
                &config.command,
                step_dir.join("daemon.log"),
            )
            .with_context(|| format!("starting {}", path.display()))?;
            tree.add_root(Pid::from_u32(held.child.id()));
            return Ok(Self {
                slots: Vec::new(),
                daemon: Some(Arc::clone(&held.watch)),
                daemon_process: Some(held),
                retired_units: 0,
                retired_dropped: 0,
                replacements: 0,
                worst_replacement_secs: None,
            });
        }

        let mut slots = Vec::with_capacity(sessions as usize);
        for index in 0..sessions {
            slots.push(start(config, step_dir, scratch, index, 0, tree)?);
        }
        Ok(Self {
            slots,
            // A pool of slots holds its own sessions and has nothing to watch.
            daemon: None,
            daemon_process: None,
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
        if self.daemon.is_some() {
            // **Not a no-op for convenience.** The one slot here is the daemon,
            // and restarting it would restart every session it holds — that is
            // a new rung rather than a replacement, and timing it would report
            // the rung's own startup as a replacement latency. A rung that
            // loses sessions is refused by the live count instead.
            return Ok(());
        }
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
        if let Some(watch) = &self.daemon {
            // Zero only until the daemon has spoken, which the rung's own
            // minimum-samples check already refuses.
            return watch
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .units()
                .unwrap_or(0);
        }
        self.retired_units + self.slots.iter().map(|s| s.output.units()).sum::<u64>()
    }

    /// Why the rung ended early, when its daemon stopped before it did.
    ///
    /// `None` for a pool of slots: a session that exits there is replaced, and
    /// a rung with nothing left is caught by the peak process count.
    fn daemon_left(&mut self, at: Duration) -> Option<String> {
        let held = self.daemon_process.as_mut()?;
        let status = held.exited().ok().flatten()?;
        Some(format!(
            "the daemon exited {status} after {:.1}s of the hold, so every count after that is the last one it managed to report",
            at.as_secs_f64(),
        ))
    }

    /// The fewest sessions seen alive, when the target can be asked.
    ///
    /// `None` for a pool of slots: it restarts what exits, so the count asked
    /// for is the count held by construction and there is nothing to ask.
    fn fewest_alive(&self) -> Option<u32> {
        let watch = self.daemon.as_ref()?;
        let seen = watch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .fewest_running()?;
        Some(u32::try_from(seen).unwrap_or(u32::MAX))
    }

    /// Units the sessions announced but never delivered.
    ///
    /// Read from the counters the drains keep rather than by scanning the logs
    /// afterwards, which is what lets the logs be capped: a workload whose
    /// payload is its output would otherwise make the disk the ceiling of the
    /// axis that exists to measure the output path.
    fn dropped_units(&self) -> Option<u64> {
        if self.daemon.is_some() {
            // Not zero. Gaps are ordinals in a session's own stream, and under
            // a daemon that stream ends in its scrollback — nobody here holds
            // what the workload emitted, so there is nothing to subtract from.
            return None;
        }
        Some(
            self.retired_dropped
                + self
                    .slots
                    .iter()
                    .map(|s| s.output.dropped_units())
                    .sum::<u64>(),
        )
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
        // One kill takes every tree here: each session sits in a job carrying
        // KILL_ON_JOB_CLOSE, and the last handle to those jobs closes when the
        // daemon dies however it dies. Measured leaving zero survivors on both
        // the graceful and the forced path, at a hundred sessions.
        if let Some(held) = &mut self.daemon_process {
            let _ = held.child.kill();
            return;
        }
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
    let name = format!("s{index:03}-g{generation:02}");
    let base = step_dir.join(&name);
    // One directory per session rather than one per rung. The workload contract
    // forbids shared paths, and handing every session the same one broke that
    // from the instrument's side: the three synthetic workloads name their
    // files uniquely inside it and never noticed, while the first workload to
    // build a fixed-name subtree had ten sessions deleting each other's.
    // Per generation as well as per index, so a replaced session starts clean
    // rather than on top of whatever its predecessor was killed in the middle
    // of.
    let scratch = scratch.join(&name);
    fs::create_dir_all(&scratch)?;
    let spawned = session::spawn(&config.command, config.mode, &base, &scratch)?;
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

    /// A rung that could not watch for drops does not thereby pass the
    /// condition that forbids them.
    ///
    /// The tolerance is zero, so the check is `> 0` and an unmeasured rung
    /// reporting `0` would clear it forever — a condition satisfied by never
    /// having been asked. `Option` is what keeps *measured none* and *not
    /// measured* apart, and this is the assertion that keeps it that way.
    #[test]
    fn an_unmeasured_drop_count_is_not_a_measured_zero() {
        let measured_none: Option<u64> = Some(0);
        let measured_some: Option<u64> = Some(3);
        let unmeasured: Option<u64> = None;

        assert!(!measured_none.is_some_and(|n| n > 0), "none is not a break");
        assert!(measured_some.is_some_and(|n| n > 0), "some is a break");
        assert!(
            !unmeasured.is_some_and(|n| n > 0),
            "and an absence is not a break either — but it is also not a pass, \
             which is what the report renders as a dash rather than a zero"
        );

        // The rendering is the half a reader sees, so it is asserted here too.
        let shown = |d: Option<u64>| d.map_or_else(|| "—".to_string(), |n| n.to_string());
        assert_eq!(shown(measured_none), "0");
        assert_eq!(shown(unmeasured), "—");
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
            solo_check: None,
        }
    }

    #[test]
    fn a_solo_rung_that_repeats_itself_reports_no_spread() {
        let mut report = report_with(vec![rung(25, 40.0)], None);
        report.solo_units_per_sec = 60.0;
        report.solo_check = Some(rung(1, 60.0));
        assert_eq!(report.solo_spread_percent(), Some(0.0));
    }

    #[test]
    fn a_solo_rung_that_moved_reports_which_way() {
        let mut report = report_with(vec![rung(25, 40.0)], None);
        report.solo_units_per_sec = 60.0;
        report.solo_check = Some(rung(1, 57.0));
        let spread = report
            .solo_spread_percent()
            .expect("a repeat that measured");
        assert!((spread + 5.0).abs() < 1e-9, "got {spread}");
    }

    #[test]
    fn a_solo_repeat_that_could_not_be_measured_is_not_a_spread_of_zero() {
        let mut report = report_with(vec![rung(25, 40.0)], None);
        let mut unusable = rung(1, 0.0);
        unusable.inconclusive = Some("too few samples".to_string());
        report.solo_check = Some(unusable);
        assert_eq!(report.solo_spread_percent(), None);
    }

    /// A rung that sampled fine and had sessions resident throughout.
    fn reading() -> Reading {
        Reading {
            samples: 40,
            peak_processes: 25,
            sessions: 25,
            replacements: 0,
            hold: Duration::from_secs(60),
            spinup: Duration::from_secs(20),
            elapsed_secs: 60.0,
            fewest_alive: None,
            left_early: None,
        }
    }

    /// An empty pool, with or without a daemon watching for it.
    fn pool_with(daemon: Option<Arc<Mutex<crate::daemon::Watch>>>) -> Pool {
        Pool {
            slots: Vec::new(),
            daemon,
            daemon_process: None,
            retired_units: 0,
            retired_dropped: 0,
            replacements: 0,
            worst_replacement_secs: None,
        }
    }

    #[test]
    fn a_daemon_pool_answers_the_three_questions_differently() {
        // The whole seam. Everything below these three -- the sampler, the
        // tree, the redline arithmetic, the report -- stays shared, because a
        // second pool is a second bench.
        let slots = pool_with(None);
        assert_eq!(
            slots.dropped_units(),
            Some(0),
            "a drain counted, and found none"
        );
        assert_eq!(slots.fewest_alive(), None, "slots restart what exits");

        let watch = Arc::new(Mutex::new(crate::daemon::Watch::default()));
        watch.lock().expect("fresh").observe(
            "held 8 · running 6 · read 40 · bytes 900 · evicted 0 · truncated 0 · failed_reads 0",
        );
        let daemon = pool_with(Some(Arc::clone(&watch)));

        assert_eq!(daemon.total_units(), 40, "units come from the report");
        assert_eq!(
            daemon.dropped_units(),
            None,
            "not zero — the ordinals never reach this process"
        );
        assert_eq!(
            daemon.fewest_alive(),
            Some(6),
            "and the live count has to be asked, since nothing restarts a session"
        );
    }

    #[test]
    fn a_daemon_that_left_is_reported_ahead_of_the_symptoms_it_caused() {
        // A rung whose daemon went stops sampling, so it also has too few
        // samples and no processes resident. Those are true and neither is
        // what happened -- reporting either sends the next reader to fix the
        // sampler.
        let died = Reading {
            samples: 1,
            peak_processes: 0,
            left_early: Some("the daemon exited exit code: 1 after 5.0s of the hold".into()),
            ..reading()
        };
        let why = why_unmeasurable(&died).expect("a rung whose daemon left is inconclusive");
        assert!(
            why.contains("daemon exited"),
            "the cause, not a symptom: {why}"
        );
        assert!(!why.contains("sample(s) over"), "{why}");
    }

    #[test]
    fn a_rung_that_lost_sessions_is_not_a_rung_at_the_count_it_asked_for() {
        // Written before a run went wrong rather than after, which none of the
        // other three were. Per-session rate divides by the count asked for,
        // and a target that does not restart what exits makes that denominator
        // a lie in the direction of saturation — a redline from a daemon that
        // was working perfectly.
        let lost = Reading {
            fewest_alive: Some(18),
            ..reading()
        };
        let why = why_unmeasurable(&lost).expect("a short rung is inconclusive");
        assert!(why.contains("18"), "it names how many were left: {why}");
        assert!(why.contains("25"), "and what was asked for: {why}");

        // Holding everything asked for says nothing, and neither does a target
        // that cannot lose one without the ramp noticing.
        assert!(
            why_unmeasurable(&Reading {
                fewest_alive: Some(25),
                ..reading()
            })
            .is_none()
        );
        assert!(why_unmeasurable(&reading()).is_none());
    }

    #[test]
    fn a_rung_that_ran_is_measurable() {
        assert_eq!(why_unmeasurable(&reading()), None);
    }

    #[test]
    fn a_rung_whose_command_never_started_is_not_a_rung_that_held() {
        // The shape a mistyped wrapper produced: the pool respawning a command
        // that exits at once, sampled plenty, holding nothing.
        let never_started = Reading {
            peak_processes: 0,
            replacements: 3000,
            sessions: 50,
            ..reading()
        };
        let reason = why_unmeasurable(&never_started).expect("zero processes cannot be a hold");
        assert!(reason.contains("3000 replacement"), "got {reason}");
        assert!(reason.contains("spawn loop"), "got {reason}");
    }

    #[test]
    fn starvation_is_reported_ahead_of_an_empty_job() {
        // Both wrong at once: too few samples is the one to fix first, since
        // an unsampled rung cannot say whether anything was resident.
        let starved = Reading {
            samples: 0,
            peak_processes: 0,
            ..reading()
        };
        let reason = why_unmeasurable(&starved).expect("no samples cannot be a hold");
        assert!(reason.contains("sampler could not keep up"), "got {reason}");
    }

    #[test]
    fn a_hold_inside_its_own_spinup_says_so_rather_than_blaming_the_sampler() {
        let too_short = Reading {
            samples: 0,
            hold: Duration::from_secs(10),
            spinup: Duration::from_secs(20),
            ..reading()
        };
        let reason = why_unmeasurable(&too_short).expect("nothing measured cannot be a hold");
        assert!(reason.contains("spin-up"), "got {reason}");
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
    fn the_markdown_carries_the_drift_line_and_says_what_it_means() {
        // CLAUDE.md tells a reader to check this before quoting anything else
        // in a run, and for one commit the line existed only on the console —
        // so the rule pointed at something a reader holding `ramp.md` did not
        // have.
        let slowed = crate::report::ramp_markdown(&report_with(
            vec![rung(25, 40.02)],
            Some(rung(25, 38.11)),
        ));
        assert!(slowed.contains("40.02"), "{slowed}");
        assert!(slowed.contains("38.11"), "{slowed}");
        assert!(slowed.contains("reads low"), "{slowed}");

        let steady = crate::report::ramp_markdown(&report_with(
            vec![rung(25, 40.02)],
            Some(rung(25, 40.00)),
        ));
        assert!(steady.contains("held still"), "{steady}");
        assert!(!steady.contains("reads low"), "{steady}");
    }

    #[test]
    fn a_ramp_that_reached_no_redline_still_reports_its_drift() {
        // The console prints the control outside the redline match, and the
        // markdown has to agree: a ladder that ran out still measured rungs,
        // and whether the machine held still is the same question.
        let markdown = crate::report::ramp_markdown(&report_with(
            vec![rung(25, 40.02)],
            Some(rung(25, 38.11)),
        ));
        assert!(markdown.contains("no redline"), "{markdown}");
        assert!(markdown.contains("Drift check"), "{markdown}");
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
