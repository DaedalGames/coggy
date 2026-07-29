// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use sessionbench::Rows;
use sessionbench::axes::{self, AxisStatus};
use sessionbench::host::HostFacts;
use sessionbench::machine::Machine;
use sessionbench::observe::{self, ObserveConfig, RunReport, SessionMode, human_bytes};
use sessionbench::provenance::Provenance;
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
    }
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
                    Some(cost) => format!(
                        "{:.2}s over startup, then {:.2}s per minute",
                        cost.startup_cpu_seconds, cost.steady_cpu_seconds_per_min
                    ),
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
