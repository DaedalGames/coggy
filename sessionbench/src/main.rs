// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use sessionbench::Rows;
use sessionbench::axes::{self, AxisStatus};
use sessionbench::format::human_bytes;
use sessionbench::host::HostFacts;
use sessionbench::machine::Machine;
use sessionbench::observe::{self, ObserveConfig, RunReport};
use sessionbench::provenance::Provenance;
use sessionbench::ramp::{self, RampConfig, RampReport};
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

        /// Give every session a pseudoconsole instead of pipes.
        #[arg(long)]
        pty: bool,

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

            let config = ObserveConfig {
                out_dir: out.join(format!("{stamp}-{label}-{}", mode.label())),
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
        Command::Ramp {
            label,
            out,
            interval,
            hold,
            max_sessions,
            pty,
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
        (Some(redline), _) => println!(
            "redline: {} sessions ({:?}) · {target} · {} · {} · {defender}",
            redline.sessions,
            redline.limited_by,
            report.mode.label(),
            report.machine.label(),
        ),
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
