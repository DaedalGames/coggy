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
//! Climbing rather than bisecting, for the same reason vtebench and every soak
//! test do: the breaking point is observed on the way past it, and a bisection
//! would have to assume the conditions are monotonic in N when contention is
//! the thing being measured.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
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
    LimitingCondition, RAMP_STEPS, REPLACEMENT_BUDGET_SECS, RSS_BUDGET_FRACTION, Redline,
    WORK_RATE_BUDGET_FACTOR,
};
use crate::sampler::{Sample, Sampler};
use crate::session::{self, Output, Session, SessionMode};
use crate::tree::{ArmedTree, Membership, SessionTree};

/// Share of each hold spent letting the step settle before measuring.
///
/// Sessions spawned together fault in their pages together, so the first
/// moments of a rung are a spike that belongs to nothing being asked about.
const SPINUP_FRACTION: f64 = 1.0 / 3.0;

/// Fewest measured samples a rung needs before its verdict means anything.
///
/// A saturated machine can starve the sampler badly enough that a fifteen
/// second hold yields one reading forty seconds late. Everything derived from
/// that reads as a catastrophic collapse — zero work, zero cores — and the
/// first ramp run reported exactly that as a broken condition. A rung that
/// could not be measured is not a rung that failed.
const MIN_SAMPLES_PER_RUNG: usize = 5;

/// Everything a ramp needs before it starts.
pub struct RampConfig {
    pub label: String,
    pub out_dir: PathBuf,
    pub interval: Duration,
    pub hold: Duration,
    pub max_sessions: u32,
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
    fn total_ms(&self) -> u64 {
        self.refresh_ms + self.replace_ms + self.sample_ms + self.write_ms
    }

    fn keep_worse(&mut self, other: TickCost) {
        if other.total_ms() > self.total_ms() {
            *self = other;
        }
    }
}

/// One rung of the ladder.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Per-session rate at one session, which every later rung is read against.
    pub solo_units_per_sec: f64,
    pub steps: Vec<Step>,
    /// `None` when the ladder ran out before any condition broke, which means
    /// the redline is somewhere above what was tried rather than unknown.
    pub redline: Option<Redline>,
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

    let mut steps: Vec<Step> = Vec::new();
    let mut solo_units_per_sec = 0.0;
    let mut redline = None;

    for sessions in RAMP_STEPS.into_iter().filter(|n| *n <= config.max_sessions) {
        println!("\nholding {sessions} session(s) for {:?}", config.hold);
        let step_dir = config.out_dir.join(format!("step-{sessions:03}"));
        fs::create_dir_all(&step_dir)?;

        let mut step = hold(config, &step_dir, sessions, &mut tree, &mut sampler, budget)?;
        if step.inconclusive.is_none() {
            if sessions == 1 {
                solo_units_per_sec = step.units_per_session_per_sec;
            }
            // The work-rate condition only means anything against the solo
            // figure, so it is checked here rather than inside the hold.
            if solo_units_per_sec > 0.0
                && solo_units_per_sec / step.units_per_session_per_sec.max(f64::EPSILON)
                    > WORK_RATE_BUDGET_FACTOR
            {
                step.broken.push(LimitingCondition::WorkRate);
            }
        }

        print_step(&step, solo_units_per_sec);
        let unmeasurable = step.inconclusive.is_some();
        let broke = step.broken.first().copied();
        steps.push(step);

        // No redline: the ladder stopped because the instrument ran out, not
        // because the machine did.
        if unmeasurable {
            break;
        }

        if let Some(condition) = broke {
            // The redline is the rung before the break, which is zero when even
            // one session could not be sustained.
            let last_good = steps
                .iter()
                .rev()
                .skip(1)
                .find(|s| s.broken.is_empty())
                .map(|s| s.sessions)
                .unwrap_or(0);
            redline = Some(Redline {
                sessions: last_good,
                limited_by: condition,
            });
            break;
        }
    }

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
        solo_units_per_sec,
        steps,
        redline,
    };

    fs::write(
        config.out_dir.join("ramp.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    Ok(report)
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
    let mut pool = Pool::new(config, step_dir, sessions, tree)?;
    let mut samples_file = BufWriter::new(File::create(step_dir.join("samples.jsonl"))?);

    let started = Instant::now();
    let spinup = config.hold.mul_f64(SPINUP_FRACTION);
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
        pool.replace_finished(config, step_dir, tree)?;
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
    pool.shut_down(sampler, last.as_ref())?;
    let dropped_units = count_dropped_units(step_dir)?;

    let elapsed_secs = started.elapsed().as_secs_f64();
    let inconclusive = (measured.len() < MIN_SAMPLES_PER_RUNG).then(|| {
        format!(
            "{} sample(s) over {elapsed_secs:.1}s of a {:.1}s hold — the sampler could not keep up, so nothing here describes the machine",
            measured.len(),
            config.hold.as_secs_f64(),
        )
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
        tree: &mut SessionTree,
    ) -> Result<Self> {
        let mut slots = Vec::with_capacity(sessions as usize);
        for index in 0..sessions {
            slots.push(start(config, step_dir, index, 0, tree)?);
        }
        Ok(Self {
            slots,
            retired_units: 0,
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
        tree: &mut SessionTree,
    ) -> Result<()> {
        for slot in &mut self.slots {
            if slot.session.try_wait()?.is_none() {
                continue;
            }
            let replacing = Instant::now();
            self.retired_units += slot.output.units();
            slot.generation += 1;
            *slot = start(config, step_dir, slot.index, slot.generation, tree)?;

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

    /// A view of the pool's output for the sampler, which wants one counter.
    fn aggregate_output(&self) -> Output {
        Output::from_counts(
            self.slots.iter().map(|s| s.output.total()).sum(),
            self.total_units(),
        )
    }

    fn shut_down(&mut self, sampler: &mut Sampler, last: Option<&Sample>) -> Result<()> {
        for slot in &mut self.slots {
            let _ = slot.session.kill();
        }
        // Killing a root leaves its children behind, and the last sample is the
        // list of everything the job still held.
        sampler.reap(last);
        Ok(())
    }
}

fn start(
    config: &RampConfig,
    step_dir: &Path,
    index: u32,
    generation: u32,
    tree: &mut SessionTree,
) -> Result<Slot> {
    let base = step_dir.join(format!("s{index:03}-g{generation:02}"));
    let spawned = session::spawn(&config.command, config.mode, &base)?;
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

/// Units a session announced but never finished a line for.
///
/// Every workload line begins with the unit's ordinal, so a gap below the
/// highest ordinal seen is output that went missing between the session and
/// this process. A truncated final line is not a gap — that is a session that
/// was killed, which is how every rung ends.
fn count_dropped_units(step_dir: &Path) -> Result<u64> {
    let mut dropped = 0;
    for entry in fs::read_dir(step_dir)? {
        let path = entry?.path();
        let is_session_log = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("-stdout.log") || n.ends_with("-output.log"));
        if !is_session_log {
            continue;
        }

        let mut seen = BTreeSet::new();
        for line in BufReader::new(File::open(&path)?)
            .lines()
            .map_while(Result::ok)
        {
            if let Some(ordinal) = line
                .split_whitespace()
                .next()
                .and_then(|token| token.parse::<u64>().ok())
            {
                seen.insert(ordinal);
            }
        }
        if let Some(highest) = seen.last() {
            dropped += highest - seen.len() as u64;
        }
    }
    Ok(dropped)
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
    match (&step.inconclusive, step.broken.is_empty()) {
        (Some(reason), _) => println!("      INCONCLUSIVE: {reason}"),
        (None, true) => println!("      held"),
        (None, false) => println!("      BROKE: {:?}", step.broken),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway directory holding one session log.
    fn logged(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sessionbench-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("a writable temporary directory");
        fs::write(dir.join("s000-g00-stdout.log"), body).expect("a writable log");
        dir
    }

    #[test]
    fn a_gap_in_the_ordinals_counts_as_dropped_output() {
        let dir = logged("gap", "1 a\n2 b\n4 d\n");
        assert_eq!(count_dropped_units(&dir).expect("readable"), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_session_killed_partway_has_dropped_nothing() {
        // Every rung ends by killing its sessions, so a log that simply stops
        // is the normal case and must not read as a drop.
        let dir = logged("killed", "1 a\n2 b\n3 c\n");
        assert_eq!(count_dropped_units(&dir).expect("readable"), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_truncated_final_line_is_not_a_gap() {
        let dir = logged("truncated", "1 a\n2 b\n3 c");
        assert_eq!(count_dropped_units(&dir).expect("readable"), 0);
        let _ = fs::remove_dir_all(&dir);
    }
}
