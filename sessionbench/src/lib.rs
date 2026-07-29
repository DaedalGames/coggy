// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Measurement library behind the `sessionbench` binary.
//!
//! Split from the CLI so backends stay testable without going through argument
//! parsing or report rendering. See `README.md` for the metric this produces.

use serde::{Deserialize, Serialize};

/// A redline reading: a session count paired with what stopped it.
///
/// Never a bare number. Without its limiting cause a redline cannot be
/// reproduced, and the cause is what says where to optimize next.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Redline {
    /// Last session count at which all four conditions still held.
    pub sessions: u32,
    /// The condition that broke first at the next ramp step.
    pub limited_by: LimitingCondition,
}

/// The four conditions that define a sustained session count.
///
/// Redline is their conjunction, so no single-axis optimization can raise it.
/// All four target residency: these sessions live for hours, so the machine
/// spends its time holding them rather than starting them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitingCondition {
    /// Per-session work rate fell past 2x the same workload run alone.
    ///
    /// The one that matters. Requesting N sessions and getting N processes that
    /// each crawl is not N-way concurrency, and redline is the count you got.
    WorkRate,
    /// Total RSS exceeded 70% of physical memory.
    Rss,
    /// A session dropped output. Tolerance is zero.
    OutputDrop,
    /// A finished session was not replaced within [`REPLACEMENT_BUDGET_SECS`].
    ///
    /// The only place spawn cost enters. Steady-state churn replaces a session
    /// roughly every 86 seconds, so what matters is whether replacement keeps
    /// up — not whether a cold start is fast.
    ReplacementLag,
}

/// Session counts the ramp steps through.
///
/// Monotonic increase rather than binary search: each step holds until memory
/// plateaus before advancing, so the breaking point is observed rather than
/// interpolated.
pub const RAMP_STEPS: [u32; 8] = [1, 10, 25, 50, 75, 100, 150, 200];

/// How long a finished session may take to be replaced before
/// [`LimitingCondition::ReplacementLag`] trips.
pub const REPLACEMENT_BUDGET_SECS: u64 = 60;
