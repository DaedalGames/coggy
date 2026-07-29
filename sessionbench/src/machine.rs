// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! The hardware a reading belongs to.
//!
//! Different machines yielding different redlines is correct behaviour rather
//! than noise, which is exactly why no result travels without this. Nothing
//! identifying the machine is collected — reports are meant to be committed.

use serde::{Deserialize, Serialize};
use sysinfo::System;

use crate::Rows;

const BYTES_PER_GIB: f64 = (1024 * 1024 * 1024) as f64;

/// Hardware and operating system a run was taken on.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
