// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! What produced a measurement, recorded alongside it.
//!
//! Nothing here is pinned — not the toolchain, not the measurement crates —
//! because going stale costs more than drift does. That trade only holds if
//! every artifact is labelled, so a frozen baseline stays comparable by being
//! identified rather than by having been held still.

use std::collections::BTreeMap;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::Rows;

/// The provenance block carried by every report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Crate version of the binary that produced the run.
    pub sessionbench_version: String,
    /// Short commit hash, or `None` when the binary ran outside a checkout.
    pub sessionbench_commit: Option<String>,
    /// Whether the checkout had uncommitted changes at measurement time.
    ///
    /// `None` travels with a `None` commit: not knowing is a distinct state
    /// from knowing the tree was clean.
    pub working_tree_dirty: Option<bool>,
    /// Full `rustc -V` of the compiler that built the binary.
    pub rustc: String,
    /// Resolved versions of the crates that can move a measured number.
    pub measurement_crates: BTreeMap<String, String>,
}

impl Provenance {
    /// Collects the block, half from build time and half from now.
    ///
    /// The split is deliberate. The compiler and the resolved dependency
    /// versions are properties of the binary, so asking at run time could name
    /// a toolchain that never touched it. The commit and the dirty flag are
    /// properties of the checkout being measured, and only run time knows
    /// whether someone edited a file since the last build.
    pub fn current() -> Self {
        let (commit, dirty) = match git_commit() {
            Some(commit) => (Some(commit), Some(git_tree_is_dirty())),
            None => (None, None),
        };

        Self {
            sessionbench_version: env!("CARGO_PKG_VERSION").to_string(),
            sessionbench_commit: commit,
            working_tree_dirty: dirty,
            rustc: env!("SESSIONBENCH_RUSTC").to_string(),
            measurement_crates: parse_deps(env!("SESSIONBENCH_DEPS")),
        }
    }

    /// Whether this run may be compared against another without a caveat.
    ///
    /// A dirty tree or an unknown commit does not invalidate a measurement, but
    /// it does mean no second run can be reproduced from the label alone.
    pub fn is_reproducible(&self) -> bool {
        self.sessionbench_commit.is_some() && self.working_tree_dirty == Some(false)
    }

    /// Rows for the human-readable report.
    pub fn rows(&self) -> Rows {
        let commit = match (&self.sessionbench_commit, self.working_tree_dirty) {
            (Some(c), Some(true)) => format!("{c} (dirty — not reproducible)"),
            (Some(c), _) => c.clone(),
            (None, _) => "unknown — ran outside a checkout".into(),
        };
        let deps = self
            .measurement_crates
            .iter()
            .map(|(name, version)| format!("{name} {version}"))
            .collect::<Vec<_>>()
            .join(", ");

        vec![
            ("sessionbench", self.sessionbench_version.clone()),
            ("commit", commit),
            ("rustc", self.rustc.clone()),
            ("measured by", deps),
        ]
    }
}

/// `name=version` pairs as emitted by the build script.
fn parse_deps(packed: &str) -> BTreeMap<String, String> {
    packed
        .split(',')
        .filter_map(|pair| pair.split_once('='))
        .map(|(name, version)| (name.to_string(), version.to_string()))
        .collect()
}

fn git_commit() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!hash.is_empty()).then_some(hash)
}

/// Treated as dirty when git cannot answer, so an unreadable checkout never
/// passes itself off as a clean one.
fn git_tree_is_dirty() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .is_none_or(|out| !out.stdout.is_empty())
}
