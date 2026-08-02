// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which of the six axes this machine can actually measure.
//!
//! `README.md` defines what each axis means; this module only decides whether
//! it is reachable from here. The distinction matters because an unavailable
//! axis does not shrink a result, it invalidates it: a redline is a conjunction
//! of four conditions, so a run that quietly skipped an axis still prints a
//! number and that number is wrong.

use serde::{Deserialize, Serialize};

use crate::host::HostFacts;

/// The six curves a run plots against concurrent session count.
///
/// Ordered as `README.md` numbers them: four that hold for any long-lived
/// session workload, then the two that motivate the benchmark existing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    WorkRate,
    TotalRss,
    ProcessCount,
    OutputBytes,
    ConhostCount,
    DefenderExclusionDelta,
}

impl Axis {
    pub const ALL: [Axis; 6] = [
        Axis::WorkRate,
        Axis::TotalRss,
        Axis::ProcessCount,
        Axis::OutputBytes,
        Axis::ConhostCount,
        Axis::DefenderExclusionDelta,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Axis::WorkRate => "work rate",
            Axis::TotalRss => "total rss",
            Axis::ProcessCount => "process count",
            Axis::OutputBytes => "output bytes",
            Axis::ConhostCount => "conhost count",
            Axis::DefenderExclusionDelta => "defender exclusions",
        }
    }
}

/// One axis and whether this machine can produce it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisStatus {
    pub axis: Axis,
    pub available: bool,
    /// Why it is unavailable, or a caveat that survives availability.
    pub note: Option<String>,
}

/// Checks every axis against the host.
///
/// The first four need only the sampler, so they are available wherever the
/// binary runs. The last two are the reason this function exists.
pub fn availability(facts: &HostFacts) -> Vec<AxisStatus> {
    Axis::ALL
        .into_iter()
        .map(|axis| match axis {
            Axis::ConhostCount => conhost_axis(),
            Axis::DefenderExclusionDelta => defender_axis(facts),
            _ => AxisStatus {
                axis,
                available: true,
                note: None,
            },
        })
        .collect()
}

/// Opens and immediately drops a pseudoconsole.
///
/// Probed rather than assumed: the axis exists to compare sessions that have a
/// conhost against sessions that do not, so a machine that cannot open a
/// ConPTY can only ever measure one side of it.
fn conhost_axis() -> AxisStatus {
    let probe = portable_pty::native_pty_system().openpty(portable_pty::PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    });

    match probe {
        Ok(pair) => {
            drop(pair);
            AxisStatus {
                axis: Axis::ConhostCount,
                available: true,
                note: None,
            }
        }
        Err(err) => AxisStatus {
            axis: Axis::ConhostCount,
            available: false,
            note: Some(format!("cannot open a pseudoconsole: {err}")),
        },
    }
}

/// The axis that needs permission rather than hardware.
///
/// Measuring the delta means adding an exclusion, running the workload, and
/// removing it again. Reading the current configuration is not enough, so a
/// report taken unelevated is missing this axis however complete it looks.
fn defender_axis(facts: &HostFacts) -> AxisStatus {
    let status = |available, note: &str| AxisStatus {
        axis: Axis::DefenderExclusionDelta,
        available,
        note: Some(note.to_string()),
    };

    match facts.defender.present {
        None => return status(false, "could not read Defender state"),
        Some(false) => return status(false, "Defender is not present on this machine"),
        Some(true) => {}
    }

    if facts.elevated != Some(true) {
        return status(false, "needs elevation to add and remove exclusions");
    }

    if facts.defender.realtime_protection == Some(false) {
        return status(
            true,
            "real-time protection is off, so the delta will measure nothing",
        );
    }

    let existing = facts.defender.exclusion_paths.len() + facts.defender.exclusion_processes.len();
    if existing > 0 {
        return AxisStatus {
            axis: Axis::DefenderExclusionDelta,
            available: true,
            note: Some(format!(
                "{existing} exclusion(s) already configured — check none covers the workload"
            )),
        };
    }

    AxisStatus {
        axis: Axis::DefenderExclusionDelta,
        available: true,
        note: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::DefenderFacts;

    fn facts(present: Option<bool>, elevated: Option<bool>) -> HostFacts {
        HostFacts {
            elevated,
            defender: DefenderFacts {
                present,
                ..DefenderFacts::default()
            },
            // Which axes are available does not depend on the power state, so
            // these stay at their default rather than being varied here.
            ..HostFacts::default()
        }
    }

    #[test]
    fn unreadable_defender_state_is_not_treated_as_absent() {
        let unknown = defender_axis(&facts(None, Some(true)));
        let absent = defender_axis(&facts(Some(false), Some(true)));
        assert!(!unknown.available);
        assert!(!absent.available);
        assert_ne!(unknown.note, absent.note);
    }

    #[test]
    fn unelevated_loses_the_axis_even_with_defender_running() {
        assert!(!defender_axis(&facts(Some(true), Some(false))).available);
        assert!(!defender_axis(&facts(Some(true), None)).available);
    }

    #[test]
    fn disabled_realtime_protection_stays_available_but_carries_a_caveat() {
        let mut host = facts(Some(true), Some(true));
        host.defender.realtime_protection = Some(false);
        let status = defender_axis(&host);
        assert!(status.available);
        assert!(status.note.is_some());
    }
}
