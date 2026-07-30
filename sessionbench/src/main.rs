// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use sessionbench::Rows;
use sessionbench::axes::{self, AxisStatus};
use sessionbench::format::human_bytes;
use sessionbench::host::{HeldExclusion, HostFacts};
use sessionbench::machine::{BackgroundLoad, Machine};
use sessionbench::observe::{self, ObserveConfig, RunReport};
use sessionbench::provenance::Provenance;
use sessionbench::ramp::{self, Drift, RampConfig, RampReport};
use sessionbench::sampler::Sampler;
use sessionbench::session::SessionMode;
use sessionbench::tree::Membership;

/// Measures redline: the maximum concurrent sessions this machine sustains,
/// and the condition that limits it.
#[derive(Parser)]
#[command(name = "sessionbench", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Report the machine and which measurement axes are available.
    Doctor {
        /// Exit non-zero when any axis is unavailable.
        ///
        /// Off by default so the report always prints. Turn it on when
        /// something downstream is about to trust the result.
        #[arg(long)]
        strict: bool,
    },

    /// Run one session to completion and record what holding it costs.
    ///
    /// The step that comes before the ramp. Flag names follow `psrecord`,
    /// which settled this shape long before we needed it.
    Observe {
        /// Name for this run, used in the output directory and the report.
        #[arg(long, default_value = "session")]
        label: String,

        /// Directory runs are written under.
        #[arg(long, default_value = "bench-out")]
        out: PathBuf,

        /// Seconds between samples.
        #[arg(long, default_value_t = 1.0, value_name = "SECONDS")]
        interval: f64,

        /// Stop after this long, whether or not the session has finished.
        #[arg(long, value_name = "SECONDS")]
        duration: Option<f64>,

        /// Give the session a pseudoconsole instead of pipes.
        ///
        /// Running the same workload both ways is the direct evidence for or
        /// against defaulting to pipes: the difference is a conhost per
        /// session, resident for as long as the session lives.
        #[arg(long)]
        pty: bool,

        /// The command to run, after `--`.
        #[arg(last = true, required = true, value_name = "COMMAND")]
        command: Vec<String>,
    },

    /// Measure what a Defender exclusion is worth, by running one workload
    /// twice.
    ///
    /// Adds a path exclusion for the directory the second run writes into and
    /// removes it afterwards. That is a change to this machine's real-time
    /// protection while the run lasts, held for the shortest window the
    /// measurement allows and removed even if the run fails.
    ExclusionDelta {
        #[arg(long, default_value = "exclusion")]
        label: String,

        #[arg(long, default_value = "bench-out")]
        out: PathBuf,

        #[arg(long, default_value_t = 1.0, value_name = "SECONDS")]
        interval: f64,

        /// How long each half runs.
        ///
        /// Both halves are cut at the same startup window, so this has to
        /// outlast it by enough to leave a steady state.
        #[arg(long, default_value_t = 90.0, value_name = "SECONDS")]
        duration: f64,

        /// How many watched/excluded pairs to run, alternating.
        ///
        /// One pair cannot separate the exclusion from whatever else the
        /// machine was doing: background activity here drifted by three times
        /// the signal in twenty seconds. Pairs run adjacent in time so slow
        /// drift lands on both halves, and the spread across pairs is what
        /// says whether the mean means anything.
        #[arg(long, default_value_t = 3)]
        repeats: u32,

        #[arg(long)]
        pty: bool,

        #[arg(last = true, required = true, value_name = "COMMAND")]
        command: Vec<String>,
    },

    /// Climb the session ladder until a redline condition breaks.
    ///
    /// Each rung holds its session count for the whole window, replacing any
    /// that finish, and is judged against all four conditions before the next
    /// is attempted.
    Ramp {
        /// Name for this ramp, used in the output directory and the report.
        #[arg(long, default_value = "ramp")]
        label: String,

        /// Directory ramps are written under.
        #[arg(long, default_value = "bench-out")]
        out: PathBuf,

        /// Seconds between samples.
        #[arg(long, default_value_t = 1.0, value_name = "SECONDS")]
        interval: f64,

        /// How long each rung is held before it is judged.
        ///
        /// The first third is spin-up and is not measured, since sessions
        /// spawned together fault in their pages together.
        #[arg(long, default_value_t = 90.0, value_name = "SECONDS")]
        hold: f64,

        /// Highest rung to attempt.
        ///
        /// The full ladder reaches 200 sessions and will take the machine with
        /// it for the duration. Lower this when something else needs the box.
        #[arg(long, default_value_t = 200)]
        max_sessions: u32,

        /// How tight to narrow the bracket before reporting a redline.
        ///
        /// The ladder is coarse and only locates the interval the ceiling
        /// falls in; each halving of that interval costs one more hold.
        #[arg(long, default_value_t = sessionbench::redline::DEFAULT_RESOLUTION)]
        resolution: u32,

        /// Give every session a pseudoconsole instead of pipes.
        #[arg(long)]
        pty: bool,

        /// Skip holding the lowest saturated rung once more at the end.
        ///
        /// That repeat is the ramp's only control on itself: everything it
        /// reports assumes the machine at the last rung is the machine that
        /// was at the first, and nothing else here checks it. It costs one
        /// hold.
        #[arg(long)]
        skip_drift_check: bool,

        /// Hide the sessions' writes from real-time scanning for this ramp.
        ///
        /// The exclusion axis at the scale it belongs at: run the same ladder
        /// with and without, and compare the two redlines. **This changes the
        /// machine's real-time protection for the length of the ramp**, over a
        /// directory the benchmark created, and removes it afterwards.
        #[arg(long)]
        exclude_scratch: bool,

        /// The command each session runs, after `--`.
        #[arg(last = true, required = true, value_name = "COMMAND")]
        command: Vec<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Doctor { strict } => doctor(strict),
        Command::Observe {
            label,
            out,
            interval,
            duration,
            pty,
            command,
        } => {
            let mode = if pty {
                SessionMode::Pty
            } else {
                SessionMode::Pipe
            };
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or_default();

            let out_dir = out.join(format!("{stamp}-{label}-{}", mode.label()));
            let config = ObserveConfig {
                scratch: out_dir.join("scratch"),
                out_dir,
                label,
                interval: Duration::from_secs_f64(interval),
                mode,
                max_duration: duration.map(Duration::from_secs_f64),
                command,
            };

            println!(
                "observing {} · {} · sampling every {interval}s",
                config.command.join(" "),
                config.mode.label(),
            );
            let report = observe::run(&config)?;
            print_run(&report, &config.out_dir);
            Ok(())
        }
        Command::ExclusionDelta {
            label,
            out,
            interval,
            duration,
            repeats,
            pty,
            command,
        } => exclusion_delta(label, out, interval, duration, repeats, pty, command),
        Command::Ramp {
            label,
            out,
            interval,
            hold,
            max_sessions,
            resolution,
            pty,
            skip_drift_check,
            exclude_scratch,
            command,
        } => {
            let mode = if pty {
                SessionMode::Pty
            } else {
                SessionMode::Pipe
            };
            let config = RampConfig {
                out_dir: out.join(format!("{}-{label}-{}", stamp(), mode.label())),
                label,
                interval: Duration::from_secs_f64(interval),
                hold: Duration::from_secs_f64(hold),
                max_sessions,
                resolution,
                exclude_scratch,
                skip_drift_check,
                mode,
                command,
            };

            println!(
                "ramping {} · {} · holding each rung {hold}s, sampling every {interval}s",
                config.command.join(" "),
                config.mode.label(),
            );
            let report = ramp::run(&config)?;
            print_ramp(&report, &config.out_dir);
            Ok(())
        }
    }
}

/// Runs one workload twice, the second time with its directory excluded from
/// real-time scanning, and reports the difference.
///
/// The two halves write into sibling directories under one root, both fresh:
/// scanning charges far less the second time a file is written, so reusing the
/// path would credit the exclusion with the cache's work.
fn exclusion_delta(
    label: String,
    out: PathBuf,
    interval: f64,
    duration: f64,
    repeats: u32,
    pty: bool,
    command: Vec<String>,
) -> anyhow::Result<()> {
    let mode = if pty {
        SessionMode::Pty
    } else {
        SessionMode::Pipe
    };
    let root = out.join(format!("{}-{label}-{}", stamp(), mode.label()));
    let scratch_root = root.join("scratch");
    let tick = Duration::from_secs_f64(interval);
    let mut sampler = Sampler::new();
    let mut pairs = Vec::new();

    for pair in 1..=repeats.max(1) {
        // Fresh directories every time. Scanning charges far less the second
        // time a file is written, so a reused path would credit whichever half
        // went second with the cache's work.
        let half = |name: &str| ObserveConfig {
            label: format!("{label}-{name}-{pair}"),
            out_dir: root.join(format!("{pair:02}-{name}")),
            scratch: scratch_root.join(format!("{pair:02}-{name}")),
            interval: tick,
            mode,
            max_duration: Some(Duration::from_secs_f64(duration)),
            command: command.clone(),
        };

        println!("\n=== pair {pair} of {repeats} ===");
        let idle_watched = sampler.watch_defender(IDLE_BASELINE, tick);
        println!("idle: {}", describe_idle(idle_watched));
        let watched = observe::run(&half("watched"))?;

        let held = HeldExclusion::add(&scratch_root).map_err(|e| anyhow::anyhow!(e))?;
        let idle_excluded = sampler.watch_defender(IDLE_BASELINE, tick);
        println!(
            "idle: {}  (exclusion in place)",
            describe_idle(idle_excluded)
        );
        let excluded = observe::run(&half("excluded"));

        // Removed before the half's result is even examined. The drop guard
        // would catch it, but a machine should not sit unprotected for the
        // length of a report.
        let mut held = held;
        let removal = held.remove();
        let excluded = excluded?;
        removal.map_err(|e| anyhow::anyhow!(e))?;

        let pair = Pair {
            watched,
            excluded,
            idle_watched,
            idle_excluded,
        };
        println!("  {}", pair.describe());
        pairs.push(pair);
    }

    print_exclusion_delta(&pairs, &root);
    Ok(())
}

/// One watched run and the excluded run that followed it, with the idle
/// baseline each was measured against.
///
/// Paired and adjacent in time on purpose: background drift that a single
/// comparison cannot separate from the exclusion lands on both halves of a
/// pair, and what survives across pairs is what the exclusion did.
struct Pair {
    watched: RunReport,
    excluded: RunReport,
    idle_watched: Option<f64>,
    idle_excluded: Option<f64>,
}

impl Pair {
    /// Defender's steady rate for a run, in seconds per minute.
    fn rate(report: &RunReport) -> Option<f64> {
        report
            .summary
            .defender
            .as_ref()
            .and_then(|cost| cost.steady_cpu_seconds_per_min)
    }

    /// What the workload itself cost, with the room's share taken off.
    fn attributable(during: Option<f64>, idle: Option<f64>) -> Option<f64> {
        Some(during? - idle? * 60.0)
    }

    /// Seconds per minute the exclusion saved. Negative means it cost.
    fn saving(&self) -> Option<f64> {
        let watched = Self::attributable(Self::rate(&self.watched), self.idle_watched)?;
        let excluded = Self::attributable(Self::rate(&self.excluded), self.idle_excluded)?;
        Some(watched - excluded)
    }

    /// How far the two idle baselines moved. Noise the pair could not cancel.
    fn drift(&self) -> Option<f64> {
        Some((self.idle_excluded? - self.idle_watched?).abs() * 60.0)
    }

    fn describe(&self) -> String {
        format!(
            "saved {} · baselines moved {}",
            self.saving()
                .map_or("—".into(), |v| format!("{v:+.2} s/min")),
            self.drift().map_or("—".into(), |v| format!("{v:.2} s/min")),
        )
    }
}

/// How long to watch Defender with nothing running, before each half.
const IDLE_BASELINE: Duration = Duration::from_secs(20);

fn describe_idle(cores: Option<f64>) -> String {
    match cores {
        Some(cores) => format!(
            "{:.2} s per minute of Defender with no session up",
            cores * 60.0
        ),
        None => "Defender not running".into(),
    }
}

fn print_exclusion_delta(pairs: &[Pair], out_dir: &std::path::Path) {
    let savings: Vec<f64> = pairs.iter().filter_map(Pair::saving).collect();
    let drifts: Vec<f64> = pairs.iter().filter_map(Pair::drift).collect();

    let mut rows: Rows = pairs
        .iter()
        .enumerate()
        .map(|(index, pair)| {
            (
                "pair",
                format!(
                    "{}  {}  ·  {:.2} against {:.2} units/s",
                    index + 1,
                    pair.describe(),
                    pair.watched.summary.work_units_per_sec,
                    pair.excluded.summary.work_units_per_sec,
                ),
            )
        })
        .collect();

    if savings.is_empty() {
        rows.push(("verdict", "no pair produced a steady state".into()));
        block("defender exclusion delta", rows);
        println!("\nwritten to {}", out_dir.display());
        return;
    }

    let mean = savings.iter().sum::<f64>() / savings.len() as f64;
    let low = savings.iter().copied().fold(f64::INFINITY, f64::min);
    let high = savings.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let worst_drift = drifts.iter().copied().fold(0.0, f64::max);

    rows.push(("mean saving", format!("{mean:+.2} s per minute")));
    rows.push((
        "across pairs",
        format!("{low:+.2} to {high:+.2} s per minute"),
    ));
    rows.push((
        "worst baseline drift",
        format!("{worst_drift:.2} s per minute"),
    ));

    // Two ways the answer is not an answer, and both are about the spread
    // rather than the mean. A range that contains zero has not established a
    // direction; a range wider than its own centre has not established a size.
    // Averaging a noisy pair with another noisy pair produces a confident
    // number and no more information, which is the failure worth naming.
    let straddles_zero = low <= 0.0 && high >= 0.0;
    let spread_exceeds_signal = (high - low) > mean.abs();
    rows.push((
        "verdict",
        if straddles_zero || spread_exceeds_signal {
            "inconclusive — the spread across pairs is larger than what separates them".into()
        } else {
            format!("the exclusion saves {mean:.2} s of Defender CPU per minute")
        },
    ));

    block("defender exclusion delta", rows);

    if straddles_zero || spread_exceeds_signal {
        println!(
            "\nA quieter machine or more pairs is the only fix. Background drift reached\n{worst_drift:.2} s/min here, and no amount of averaging separates a signal from noise\nthat moves further than the signal does."
        );
    }
    println!("\nwritten to {}", out_dir.display());
}

/// Seconds since the epoch, for naming an output directory.
fn stamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

/// Reports hardware, provenance, and axis availability.
///
/// Run before trusting any result. The Defender axis needs elevation, and
/// without it a run silently measures five axes out of six yet still prints a
/// redline — which is not a smaller result but a wrong one. Unavailable axes
/// are named here rather than discovered afterward.
fn doctor(strict: bool) -> anyhow::Result<()> {
    let machine = Machine::detect();
    let provenance = Provenance::current();
    let facts = HostFacts::query();
    let statuses = axes::availability(&facts);

    block("machine", machine.rows());
    // Sampled before anything else runs, so the figure describes the machine a
    // ramp would start on rather than one this command already disturbed.
    let background = BackgroundLoad::measure(Duration::from_secs(5));
    block("background", background.rows());
    block("provenance", provenance.rows());

    println!("\naxes");
    for (index, status) in statuses.iter().enumerate() {
        print_axis(index + 1, status);
    }

    if !facts.errors.is_empty() {
        println!("\nquery errors");
        for error in &facts.errors {
            println!("  {error}");
        }
    }

    println!();
    if !background.is_quiet() {
        println!(
            "this machine is not quiet: {:.2} of {} cores are spoken for before a session starts, \
             and the sessions will be taking those back from something",
            background.mean_cores, background.logical_cores,
        );
    }
    let unavailable = statuses.iter().filter(|s| !s.available).count();
    match unavailable {
        0 if provenance.is_reproducible() => println!("all six axes available"),
        0 => println!("all six axes available, but this build is not reproducible"),
        n => {
            println!("{n} of 6 axes unavailable — a redline taken now would be wrong, not smaller")
        }
    }

    if strict && unavailable > 0 {
        anyhow::bail!("{unavailable} of 6 axes unavailable");
    }
    Ok(())
}

fn print_run(report: &RunReport, out_dir: &std::path::Path) {
    let summary = &report.summary;
    let ending = match (report.stopped_at_limit, report.exit_code) {
        (true, _) => "stopped at the time limit".to_string(),
        (false, Some(0)) => "exit 0".to_string(),
        (false, Some(code)) => format!("exit {code}"),
        (false, None) => "terminated without an exit code".to_string(),
    };

    block(
        &format!(
            "observed {} · {} · {:.1}s · {ending}",
            report.label,
            report.mode.label(),
            report.duration_ms as f64 / 1000.0
        ),
        vec![
            ("samples", format!("{}", report.sample_count)),
            ("peak rss", human_bytes(summary.peak_rss_bytes)),
            (
                "steady rss",
                format!(
                    "{} (median of the last quarter)",
                    human_bytes(summary.steady_rss_bytes)
                ),
            ),
            ("rss drift", rss_drift(summary)),
            ("peak processes", format!("{}", summary.peak_processes)),
            ("peak conhost", format!("{}", summary.peak_pseudoconsoles)),
            (
                "work rate",
                format!(
                    "{:.2} units/s ({} units)",
                    summary.work_units_per_sec, summary.work_units
                ),
            ),
            (
                "cores",
                match (summary.session_cores, summary.defender_cores) {
                    (Some(session), Some(defender)) => {
                        format!("{session:.2} session + {defender:.2} defender")
                    }
                    _ => "no steady state to measure over".into(),
                },
            ),
            (
                "output",
                format!(
                    "{} at {}/s",
                    human_bytes(summary.output_bytes),
                    human_bytes(summary.output_bytes_per_sec as u64)
                ),
            ),
            (
                "defender",
                match &summary.defender {
                    Some(cost) => match cost.steady_cpu_seconds_per_min {
                        Some(rate) => format!(
                            "{:.2}s over startup, then {rate:.2}s per minute",
                            cost.startup_cpu_seconds
                        ),
                        None => format!(
                            "{:.2}s over startup; the run was too short to have a steady state",
                            cost.startup_cpu_seconds
                        ),
                    },
                    None => "not running".into(),
                },
            ),
            (
                "membership",
                match report.membership {
                    Membership::JobObject => "job object".into(),
                    Membership::ParentWalk => {
                        "parent walk — the kernel's list was unavailable".into()
                    }
                },
            ),
        ],
    );

    let projection = &report.projection;
    block(
        &format!(
            "\nprojection to {} sessions — linear, and therefore a floor rather than an estimate",
            projection.sessions
        ),
        vec![
            (
                "rss",
                format!(
                    "{} against a {} budget — {}",
                    human_bytes(projection.rss_bytes),
                    human_bytes(projection.rss_budget_bytes),
                    if projection.rss_condition_holds {
                        "holds"
                    } else {
                        "BREAKS the RSS condition"
                    }
                ),
            ),
            (
                "cores",
                match (projection.cores_needed, projection.cpu_oversubscribed) {
                    (Some(needed), Some(over)) => format!(
                        "{needed:.1} needed against {} available — {}",
                        projection.cores_available,
                        if over {
                            "OVERSUBSCRIBED, which is how the work-rate condition trips"
                        } else {
                            "fits"
                        }
                    ),
                    _ => "not projectable — the run had no steady state".into(),
                },
            ),
            ("processes", format!("{}", projection.processes)),
            ("conhost", format!("{}", projection.pseudoconsoles)),
            (
                "output",
                format!("{}/s", human_bytes(projection.output_bytes_per_sec as u64)),
            ),
        ],
    );

    println!("\nwritten to {}", out_dir.display());
}

/// Says whether the ladder measured one machine or several.
///
/// The lowest saturated rung, held once at the start and once after everything
/// else. Averaging over rungs takes noise out of the redline but carries drift
/// straight through — a machine that slows as the ramp runs steepens the fitted
/// slope and reports a ceiling that is too low, with no sign of it anywhere in
/// the numbers.
/// How much the session's own footprint moved between the two ends of a run.
///
/// The single-session counterpart to the ramp's repeated rung. A memory-limited
/// redline is the budget divided by this figure, so it is a ceiling only if the
/// session held the same amount throughout — and a run whose ends disagree is
/// reporting an average of two different sessions.
fn rss_drift(summary: &observe::Summary) -> String {
    if summary.early_rss_bytes == 0 {
        return "no early samples to compare against".to_string();
    }
    let moved = (summary.steady_rss_bytes as f64 - summary.early_rss_bytes as f64)
        / summary.early_rss_bytes as f64
        * 100.0;
    let verdict = if moved.abs() < 5.0 {
        "held"
    } else if moved > 0.0 {
        "still growing — this is not a steady figure"
    } else {
        "still settling — this is not a steady figure"
    };
    format!(
        "{} early, {:+.1}% by the end — {verdict}",
        human_bytes(summary.early_rss_bytes),
        moved
    )
}

fn print_drift(report: &RampReport) {
    match report.drift() {
        // A repeat the instrument could not measure is not a machine that
        // slowed to nothing, and reporting it as one would put a drift on the
        // board describing the observer.
        Some(Drift::Unmeasurable(reason)) => {
            println!("  drift check: could not be re-measured — {reason}")
        }
        Some(Drift::Measured {
            sessions,
            early_units_per_sec,
            late_units_per_sec,
            slower_percent,
        }) => println!(
            "  drift check: {sessions} sessions ran {early_units_per_sec:.2} units/s early and {late_units_per_sec:.2} at the end ({slower_percent:+.1}% slower)"
        ),
        None => {}
    }
}

fn print_ramp(report: &RampReport, out_dir: &std::path::Path) {
    let target = report
        .command
        .first()
        .map(|c| {
            std::path::Path::new(c)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| c.clone())
        })
        .unwrap_or_default();
    let defender = match report.host.defender.realtime_protection {
        Some(true) => "Defender on",
        Some(false) => "Defender off",
        None => "Defender unknown",
    };

    let unmeasurable = report.steps.last().and_then(|s| s.inconclusive.as_ref());
    println!();
    match (&report.redline, unmeasurable) {
        (Some(redline), _) => {
            println!(
                "redline: {} sessions ({:?}) · {target} · {} · {} · {defender}",
                redline.sessions,
                redline.limited_by,
                report.mode.label(),
                report.machine.label(),
            );
            match &redline.fitted {
                Some(fit) => {
                    println!(
                        "  fitted at {:.1} through {} saturated rungs, gaining {:.4} slowdown per session",
                        fit.crossing, fit.rungs, fit.slowdown_per_session
                    );
                    if fit.ladder_sessions != redline.sessions {
                        println!(
                            "  the ladder's own search stopped at {} — the gap is how flat the curve is where the budget cuts it",
                            fit.ladder_sessions
                        );
                    }
                }
                None if !redline.limited_by.is_edge() => println!(
                    "  {:?} is a budget across a slope, not an edge — this count moves if the budget does",
                    redline.limited_by
                ),
                None => {}
            }
        }
        // Not a redline, and not a smaller one either. The ladder stopped
        // because the instrument ran out rather than because the machine did.
        (None, Some(reason)) => println!(
            "no redline: the ramp stopped at {} sessions without a usable reading — {reason}",
            report.steps.last().map(|s| s.sessions).unwrap_or(0),
        ),
        // The ladder ran out with every condition still holding, which locates
        // the ceiling above what was tried rather than at it.
        (None, None) => println!(
            "no redline reached: every rung up to {} sessions held · {target} · {} · {} · {defender}",
            report.steps.last().map(|s| s.sessions).unwrap_or(0),
            report.mode.label(),
            report.machine.label(),
        ),
    }
    // Outside the match: the control says whether the ladder measured one
    // machine, which is worth knowing whether or not it found a redline.
    print_drift(report);

    println!("\nrungs");
    for step in &report.steps {
        let verdict = if step.broken.is_empty() {
            "held".to_string()
        } else {
            format!("broke on {:?}", step.broken)
        };
        println!(
            "  {:>3}  rss {:>10}  {:.2} units/s/session  {:.1} cores  {:>2} replaced  {} dropped  {verdict}",
            step.sessions,
            human_bytes(step.total_rss_bytes),
            step.units_per_session_per_sec,
            step.session_cores + step.defender_cores,
            step.replacements,
            step.dropped_units,
        );
    }

    println!("\nwritten to {}", out_dir.display());
}

/// Two columns wide enough for the longest label in any block.
const LABEL_WIDTH: usize = 20;

fn block(title: &str, rows: Rows) {
    println!("\n{title}");
    for (label, value) in rows {
        println!("  {label:<LABEL_WIDTH$} {value}");
    }
}

fn print_axis(number: usize, status: &AxisStatus) {
    let verdict = if status.available {
        "available"
    } else {
        "UNAVAILABLE"
    };
    let label = status.axis.label();
    match &status.note {
        Some(note) => println!("  {number}  {label:<LABEL_WIDTH$} {verdict:<12} {note}"),
        None => println!("  {number}  {label:<LABEL_WIDTH$} {verdict}"),
    }
}
