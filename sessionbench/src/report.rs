// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! The human-readable half of every run's output.
//!
//! Each run writes two artifacts: the JSON, which is the record, and this,
//! which is what gets read. Generating it is the point — the first two
//! measurements were written up by copying numbers out of a terminal into a
//! table by hand, and a figure retyped is a figure that can be retyped wrong.
//!
//! This is deliberately not the same rendering the console prints. The console
//! is a summary watched while a run is going; this is complete, and carries
//! every field the JSON does in the order the report format calls for — the
//! headline first, the curves under it, the machine last.

use std::fmt::Write as _;

use crate::format::human_bytes;
use crate::machine::Machine;
use crate::observe::RunReport;
use crate::provenance::Provenance;
use crate::ramp::{Drift, RampReport};

/// The markdown report for a ramp.
pub fn ramp_markdown(report: &RampReport) -> String {
    let mut out = String::new();
    let target = target_name(&report.command);

    match (
        &report.redline,
        report.steps.last().and_then(|s| s.inconclusive.as_ref()),
    ) {
        (Some(redline), _) => {
            let _ = writeln!(
                out,
                "# redline: {} sessions ({:?}) · {target} · {} · {} · {}\n",
                redline.sessions,
                redline.limited_by,
                report.mode.label(),
                report.machine.label(),
                defender_state(report.host.defender.realtime_protection),
            );
            if let Some(fit) = &redline.fitted {
                let _ = writeln!(
                    out,
                    "**Solved on the slope, not searched for.** A line through {} saturated rungs gains {:.4} slowdown per session and meets the 2x budget at {:.1}. The ladder's own search stopped at {}, and the gap between them is how flat the curve is where the budget cuts it — searching lets one rung decide, fitting lets all of them.\n",
                    fit.rungs, fit.slowdown_per_session, fit.crossing, fit.ladder_sessions
                );
            }
            if !redline.limited_by.is_edge() {
                let _ = writeln!(
                    out,
                    "**{:?} is a budget drawn across a slope, not an edge.** This count sits where the budget was drawn and moves when it moves.\n",
                    redline.limited_by
                );
            }
        }
        (None, Some(reason)) => {
            let _ = writeln!(
                out,
                "# no redline — the ramp ran out of measurement, not machine\n\n{reason}\n"
            );
        }
        (None, None) => {
            let _ = writeln!(
                out,
                "# no redline: every rung up to {} sessions held · {target} · {} · {} · {}\n\nThe ladder ended before the machine did, so this is a floor rather than a ceiling.\n",
                report.steps.last().map(|s| s.sessions).unwrap_or(0),
                report.mode.label(),
                report.machine.label(),
                defender_state(report.host.defender.realtime_protection),
            );
        }
    }

    // Outside the match, matching the console: a ramp that reached no redline
    // still measured rungs, and whether the machine held still while it did is
    // the same question either way.
    drift_paragraph(&mut out, report);

    let _ = writeln!(out, "## Rungs\n");
    let _ = writeln!(
        out,
        "| Sessions | RSS | Per-session rate | Against solo | Cores | Processes | conhost | Replaced | Dropped | Worst tick | Verdict |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|---|");
    for step in &report.steps {
        let against_solo =
            if step.units_per_session_per_sec > 0.0 && report.solo_units_per_sec > 0.0 {
                format!(
                    "{:.2}×",
                    report.solo_units_per_sec / step.units_per_session_per_sec
                )
            } else {
                "—".into()
            };
        let verdict = match (&step.inconclusive, step.broken.first()) {
            (Some(_), _) => "inconclusive".to_string(),
            (None, Some(condition)) => format!("broke on {condition:?}"),
            (None, None) => "held".to_string(),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {:.2} units/s | {against_solo} | {:.1} | {} | {} | {} | {} | {} ms | {verdict} |",
            step.sessions,
            human_bytes(step.total_rss_bytes),
            step.units_per_session_per_sec,
            step.session_cores + step.defender_cores,
            step.processes,
            step.pseudoconsoles,
            step.replacements,
            step.dropped_units,
            step.worst_tick.total_ms(),
        );
    }

    let _ = writeln!(
        out,
        "\nRungs are listed in the order they were run: the ladder climbs to bracket the ceiling, then halves the bracket. Solo rate was {:.2} units/s, and every rate above is read against it.\n",
        report.solo_units_per_sec
    );

    let _ = writeln!(out, "## How it was measured\n");
    let _ = writeln!(out, "| | |");
    let _ = writeln!(out, "|---|---|");
    let _ = writeln!(out, "| Command | `{}` |", report.command.join(" "));
    let _ = writeln!(
        out,
        "| Hold per rung | {} s, first third unmeasured |",
        report.hold_ms / 1000
    );
    let _ = writeln!(out, "| Sample interval | {} ms |", report.interval_ms);
    let _ = writeln!(
        out,
        "| Bracket narrowed to | {} session(s) |",
        report.resolution
    );
    let _ = writeln!(
        out,
        "| Membership | {} |",
        match report.membership_fallback_reason.as_deref() {
            None => "job object".to_string(),
            Some(reason) => format!("parent walk — {reason}"),
        }
    );
    let _ = writeln!(
        out,
        "| Sampler priority | {} |",
        match report.sampler_unprioritised_reason.as_deref() {
            None => "raised above the sessions".to_string(),
            Some(reason) => format!("**ordinary — every figure is suspect**: {reason}"),
        }
    );

    out.push_str(&machine_and_provenance(&report.machine, &report.provenance));
    out
}

/// The markdown report for a single observed session.
pub fn run_markdown(report: &RunReport) -> String {
    let mut out = String::new();
    let summary = &report.summary;
    let ending = match (report.stopped_at_limit, report.exit_code) {
        (true, _) => "stopped at the time limit".to_string(),
        (false, Some(0)) => "exit 0".to_string(),
        (false, Some(code)) => format!("exit {code}"),
        (false, None) => "terminated without an exit code".to_string(),
    };

    let _ = writeln!(
        out,
        "# {} · {} · {:.1}s · {ending}\n",
        target_name(&report.command),
        report.mode.label(),
        report.duration_ms as f64 / 1000.0,
    );

    let _ = writeln!(out, "## What holding it cost\n");
    let _ = writeln!(out, "| | |");
    let _ = writeln!(out, "|---|---|");
    let _ = writeln!(
        out,
        "| Steady RSS | {} |",
        human_bytes(summary.steady_rss_bytes)
    );
    let _ = writeln!(
        out,
        "| Peak RSS | {} |",
        human_bytes(summary.peak_rss_bytes)
    );
    let _ = writeln!(out, "| Peak processes | {} |", summary.peak_processes);
    let _ = writeln!(out, "| Peak conhost | {} |", summary.peak_pseudoconsoles);
    let _ = writeln!(
        out,
        "| Work rate | {:.2} units/s over {} units |",
        summary.work_units_per_sec, summary.work_units
    );
    let _ = writeln!(
        out,
        "| Cores | {} |",
        match (summary.session_cores, summary.defender_cores) {
            (Some(session), Some(defender)) =>
                format!("{session:.2} session + {defender:.2} Defender"),
            _ => "no steady state to measure over".into(),
        }
    );
    let _ = writeln!(
        out,
        "| Output | {} at {}/s |",
        human_bytes(summary.output_bytes),
        human_bytes(summary.output_bytes_per_sec as u64)
    );
    let _ = writeln!(
        out,
        "| Defender | {} |",
        match &summary.defender {
            Some(cost) => match cost.steady_cpu_seconds_per_min {
                Some(rate) => format!(
                    "{:.2} s over startup, then {rate:.2} s per minute",
                    cost.startup_cpu_seconds
                ),
                None => format!(
                    "{:.2} s over startup; too short to have a steady state",
                    cost.startup_cpu_seconds
                ),
            },
            None => "not running".into(),
        }
    );
    let _ = writeln!(out, "| Samples | {} |", report.sample_count);

    let projection = &report.projection;
    let _ = writeln!(
        out,
        "\n## Projected to {} sessions\n\nLinear, and therefore a floor rather than a forecast. Contention, cache pressure, and I/O queueing all make the real curve worse — which is what makes a floor that already breaks a condition worth having.\n",
        projection.sessions
    );
    let _ = writeln!(out, "| | |");
    let _ = writeln!(out, "|---|---|");
    let _ = writeln!(
        out,
        "| RSS | {} against a {} budget — {} |",
        human_bytes(projection.rss_bytes),
        human_bytes(projection.rss_budget_bytes),
        if projection.rss_condition_holds {
            "holds"
        } else {
            "**breaks the RSS condition**"
        }
    );
    let _ = writeln!(
        out,
        "| Cores | {} |",
        match (projection.cores_needed, projection.cpu_oversubscribed) {
            (Some(needed), Some(over)) => format!(
                "{needed:.1} needed against {} available — {}",
                projection.cores_available,
                if over {
                    "**oversubscribed, which is how the work-rate condition trips**"
                } else {
                    "fits"
                }
            ),
            _ => "not projectable — the run had no steady state".into(),
        }
    );
    let _ = writeln!(out, "| Processes | {} |", projection.processes);
    let _ = writeln!(out, "| conhost | {} |", projection.pseudoconsoles);

    out.push_str(&machine_and_provenance(&report.machine, &report.provenance));
    out
}

/// The block every report ends with, because a result without it is not
/// reproducible and therefore not quotable.
/// The control, written directly under the headline.
///
/// It goes first among the caveats because it decides whether the rest of the
/// document is worth reading: every other figure here assumes the machine at
/// the last rung was the machine at the first, and this is the only line that
/// checks it. A drift past a few percent means the redline reads low, since
/// the fitted slope averages noise away but carries drift straight through.
fn drift_paragraph(out: &mut String, report: &RampReport) {
    match report.drift() {
        Some(Drift::Unmeasurable(reason)) => {
            let _ = writeln!(
                out,
                "**The drift check produced no reading** — {reason}. Nothing here can say whether the machine held still while the ladder ran.\n"
            );
        }
        Some(Drift::Measured {
            sessions,
            early_units_per_sec,
            late_units_per_sec,
            slower_percent,
        }) => {
            let verdict = if slower_percent.abs() < 2.0 {
                "The machine held still, so the rungs are comparable with each other."
            } else if slower_percent > 0.0 {
                "**The machine slowed under its own ladder, so this redline reads low.** Treat it as a draft and find what else was running."
            } else {
                "**The machine sped up as the ladder ran, so this redline reads high.** Something was competing early and stopped."
            };
            let _ = writeln!(
                out,
                "**Drift check:** {sessions} sessions ran {early_units_per_sec:.2} units/s early in the ramp and {late_units_per_sec:.2} after it finished, {slower_percent:+.1}%. {verdict}\n"
            );
        }
        None => {}
    }
}

fn machine_and_provenance(machine: &Machine, provenance: &Provenance) -> String {
    let mut out = String::from("\n## Machine and provenance\n\n| | |\n|---|---|\n");
    for (label, value) in machine.rows().into_iter().chain(provenance.rows()) {
        let _ = writeln!(out, "| {label} | {value} |");
    }
    if !provenance.is_reproducible() {
        out.push_str(
            "\n**Not reproducible from this label alone** — the commit is unknown or the tree had uncommitted changes.\n",
        );
    }
    out
}

/// The workload's name, without its path or extension.
fn target_name(command: &[String]) -> String {
    command
        .first()
        .map(|first| {
            std::path::Path::new(first)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| first.clone())
        })
        .unwrap_or_default()
}

fn defender_state(realtime: Option<bool>) -> &'static str {
    match realtime {
        Some(true) => "Defender on",
        Some(false) => "Defender off",
        None => "Defender unknown",
    }
}
