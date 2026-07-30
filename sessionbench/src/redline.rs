// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! The metric: how many concurrent sessions a machine sustains, and what
//! stopped it going further.

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
    /// Total RSS exceeded [`RSS_BUDGET_FRACTION`] of physical memory.
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

impl LimitingCondition {
    /// Whether the condition is an edge or a budget drawn across a slope.
    ///
    /// Only dropped output is an edge: its tolerance is zero, so it is absent
    /// and then present. The other three are budgets over quantities that
    /// degrade smoothly, which means a redline limited by one of them sits
    /// wherever that budget was drawn and moves when it moves. Load-testing
    /// practice prefers the edge as the abort trigger for exactly this reason,
    /// and a report that does not say which kind it hit invites its number to
    /// be read as sharper than it is.
    pub fn is_edge(self) -> bool {
        matches!(self, LimitingCondition::OutputDrop)
    }
}

/// Session counts the ramp climbs through to bracket the redline.
///
/// Coarse on purpose: these rungs find *which interval* the ceiling falls in,
/// and the ramp then refines inside that interval alone. Climbing is what
/// locates the bracket, since a search that started by bisecting the whole
/// range would be assuming the conditions behave monotonically across it —
/// but refining *within* an observed bracket assumes nothing of the kind, and
/// skipping that refinement is how a ceiling at sixteen gets reported as ten.
pub const RAMP_STEPS: [u32; 8] = [1, 10, 25, 50, 75, 100, 150, 200];

/// How tight the bracket must be before the redline is reported.
///
/// One session is exact and costs one hold per halving of the interval.
pub const DEFAULT_RESOLUTION: u32 = 1;

/// Share of physical memory total RSS may occupy before
/// [`LimitingCondition::Rss`] trips.
pub const RSS_BUDGET_FRACTION: f64 = 0.70;

/// How far per-session work rate may fall behind the same workload run alone
/// before [`LimitingCondition::WorkRate`] trips.
pub const WORK_RATE_BUDGET_FACTOR: f64 = 2.0;

/// How long a finished session may take to be replaced before
/// [`LimitingCondition::ReplacementLag`] trips.
pub const REPLACEMENT_BUDGET_SECS: u64 = 60;
