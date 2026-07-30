// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! The hardware a reading belongs to.
//!
//! Different machines yielding different redlines is correct behaviour rather
//! than noise, which is exactly why no result travels without this. Nothing
//! identifying the machine is collected — reports are meant to be committed.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sysinfo::System;

use crate::Rows;

const BYTES_PER_GIB: f64 = (1024 * 1024 * 1024) as f64;

/// Hardware and operating system a run was taken on.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Machine {
    /// `None` on hardware where the physical count cannot be determined; the
    /// logical count is always available.
    pub physical_cores: Option<usize>,
    pub logical_cores: usize,
    /// Memory the operating system reports as usable, which is short of the
    /// installed total by whatever firmware and integrated graphics reserved.
    /// Usable is the right figure here: the RSS condition is a share of what
    /// processes can actually occupy.
    pub total_memory_bytes: u64,
    pub os: String,
    /// Carries the build number on Windows, so no separate kernel field
    /// exists to disagree with it.
    pub os_version: String,
}

/// What the machine is already doing before a session starts.
///
/// A desktop is never idle: this one ran a quarter of its sixteen cores busy
/// with nothing asked of it, spread across four hundred processes with no
/// single owner. That is a fact about whether a reading can be trusted, and
/// this is where a run says so before producing one.
///
/// **It is not a smaller `C`.** Subtracting it looks right and is not: solving
/// the relation with the free count puts `η` above 1, which cannot happen. The
/// reason is in the ramps themselves — sessions at the redline measured 15.1
/// of 16 cores, so the background that idles at four collapses to under one as
/// soon as something wants the machine. Background load is contention, not a
/// reservation, and busy sessions win it.
///
/// What it leaves is a question about `η`. That term was read as memory
/// contention between sessions, on the evidence that ten sessions with cores
/// to spare still ran a fifth slower. Whatever background survives under load
/// is a second candidate for part of that gap, and the two have not been
/// separated.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackgroundLoad {
    pub samples: usize,
    /// Cores busy on average with nothing of ours running, this process aside.
    pub mean_cores: f64,
    /// The worst single reading, which is what a rung unlucky in its timing
    /// competes against.
    pub peak_cores: f64,
    pub logical_cores: usize,
}

impl BackgroundLoad {
    /// Samples the whole machine over `window`, which has to outlast a few of
    /// sysinfo's minimum refresh intervals to mean anything.
    pub fn measure(window: Duration) -> Self {
        let mut sys = System::new();
        let logical_cores = {
            sys.refresh_cpu_list(sysinfo::CpuRefreshKind::nothing());
            sys.cpus().len()
        };
        // The first refresh has no interval behind it and reads as zero, so it
        // is taken and discarded before the window opens.
        sys.refresh_cpu_usage();

        let mut readings = Vec::new();
        let deadline = Instant::now() + window;
        while Instant::now() < deadline {
            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
            sys.refresh_cpu_usage();
            readings.push(f64::from(sys.global_cpu_usage()) / 100.0 * logical_cores as f64);
        }

        Self {
            samples: readings.len(),
            mean_cores: readings.iter().sum::<f64>() / readings.len().max(1) as f64,
            peak_cores: readings.iter().copied().fold(0.0, f64::max),
            logical_cores,
        }
    }

    /// Cores not already spoken for while the machine sits idle.
    ///
    /// A measurement condition rather than a capacity: sessions reclaim most
    /// of what this reports as soon as they start competing for it.
    pub fn idle_free_cores(&self) -> f64 {
        (self.logical_cores as f64 - self.mean_cores).max(0.0)
    }

    /// Whether the machine is quiet enough that a redline describes it rather
    /// than whatever else is running.
    pub fn is_quiet(&self) -> bool {
        self.mean_cores < self.logical_cores as f64 * QUIET_FRACTION
    }

    pub fn rows(&self) -> Rows {
        vec![
            (
                "busy before we start",
                format!(
                    "{:.2} of {} logical ({:.0}%)",
                    self.mean_cores,
                    self.logical_cores,
                    self.mean_cores / self.logical_cores as f64 * 100.0
                ),
            ),
            ("worst sample", format!("{:.2} cores", self.peak_cores)),
            (
                "free while idle",
                format!(
                    "{:.2} cores — a condition on the reading, not the C the relation takes",
                    self.idle_free_cores()
                ),
            ),
        ]
    }
}

/// How much of the machine may be busy before a reading stops describing it.
///
/// A tenth is where the background starts to move a redline by more than the
/// [spread the metric already carries](../../docs/measurements/2026-07-30-164912-redline-reproducibility.md).
const QUIET_FRACTION: f64 = 0.10;

impl Machine {
    pub fn detect() -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        sys.refresh_cpu_list(sysinfo::CpuRefreshKind::nothing());

        Self {
            physical_cores: System::physical_core_count(),
            logical_cores: sys.cpus().len(),
            total_memory_bytes: sys.total_memory(),
            os: System::name().unwrap_or_else(|| "unknown".into()),
            os_version: System::os_version().unwrap_or_else(|| "unknown".into()),
        }
    }

    /// Usable memory in GiB, rounded to the nearest whole unit.
    ///
    /// GiB rather than GB throughout, because the RSS condition is a fraction
    /// of this number and the two units differ by 7% — enough to move a
    /// verdict. A machine sold as 32GB reports about 31 GiB here.
    pub fn memory_gib(&self) -> u64 {
        (self.total_memory_bytes as f64 / BYTES_PER_GIB).round() as u64
    }

    /// The hardware fragment of the headline, as in `16C/31GiB`.
    ///
    /// Physical cores, because the claim being made is about how many sessions
    /// fit — hyperthreads flatter that number without adding capacity.
    ///
    /// **Not the `C` in `redline = 2ηC/d`.** That one is the logical count,
    /// because it is compared against measured CPU usage, which arrives in
    /// logical-processor units. On a 2-physical/4-logical runner the two give
    /// η of 1.75 and 0.875, and only the second can be a fraction of anything.
    pub fn label(&self) -> String {
        let cores = self.physical_cores.unwrap_or(self.logical_cores);
        format!("{cores}C/{}GiB", self.memory_gib())
    }

    /// Rows for the human-readable report.
    pub fn rows(&self) -> Rows {
        let cores = match self.physical_cores {
            Some(physical) => format!("{physical} physical / {} logical", self.logical_cores),
            None => format!(
                "{} logical (physical count unavailable)",
                self.logical_cores
            ),
        };

        vec![
            ("cores", cores),
            ("memory", format!("{} GiB usable", self.memory_gib())),
            ("os", format!("{} {}", self.os, self.os_version)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn machine(total_memory_bytes: u64) -> Machine {
        Machine {
            physical_cores: Some(16),
            logical_cores: 16,
            total_memory_bytes,
            os: "Windows".into(),
            os_version: "11 (26200)".into(),
        }
    }

    #[test]
    fn usable_memory_rounds_rather_than_truncating() {
        // What Windows reports on a 32 GiB machine once firmware and the
        // integrated GPU have taken their reservation: 31.39 GiB.
        assert_eq!(machine(33_707_749_376).memory_gib(), 31);
        // Truncation would call this 31 and lose most of a gigabyte.
        assert_eq!(machine(34_200_000_000).memory_gib(), 32);
    }

    #[test]
    fn label_matches_the_headline_format() {
        assert_eq!(machine(33_707_749_376).label(), "16C/31GiB");
    }
}
