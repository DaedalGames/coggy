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
    /// The reported count: the fitted crossing where there was one, and the
    /// last rung observed to hold where there was not.
    pub sessions: u32,
    /// The condition that broke first at the next ramp step.
    pub limited_by: LimitingCondition,
    /// Where the budget met a line drawn through the saturated rungs.
    pub fitted: Option<Fit>,
}

/// The budget solved on a line through the rungs, rather than searched for.
///
/// Bisection asks a noisy curve where it crosses a line and lets one reading
/// of one rung decide which way to search next. Near the budget the verdicts
/// are neither monotone nor exact, so a rung landing a percent to either side
/// sends the search in opposite directions — [six identical
/// runs](../../docs/measurements/2026-07-30-redline-reproducibility.md)
/// returned 30, 31, 31, 31, 33, 34 and 34, a spread of 12.5% from measurements
/// whose own rungs reproduced within 2%.
///
/// Fitting instead uses every saturated rung, so no single one can drag the
/// answer, and it is what the model already claimed: `slowdown = N·d/(η·C)` is
/// linear in `N` through the origin, so solving `b·N = 2` for a fitted `b` is
/// solving `N = 2ηC/d`. **The slope is `η`.** The same seven fit to within
/// 2.3%.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Fit {
    /// The crossing itself, before it is floored into a session count.
    pub crossing: f64,
    /// Slowdown gained per added session — the line's slope.
    pub slowdown_per_session: f64,
    /// How many rungs the line was drawn through.
    pub rungs: usize,
    /// What the ladder's own search returned.
    ///
    /// Kept because the two disagreeing is information about how flat the
    /// curve is where the budget cuts it, not a discrepancy to hide.
    pub ladder_sessions: u32,
}

/// Fits `slowdown = b·N` through the origin and returns where it meets
/// `budget`.
///
/// Through the origin because that is what the model says, and because letting
/// the intercept float made the answer *less* reproducible — the sign of a
/// parameter absorbing noise rather than describing anything.
///
/// `None` when too few rungs are saturated to draw a line worth trusting.
pub fn fit_crossing(rungs: &[(u32, f64)], budget: f64, ladder_sessions: u32) -> Option<Fit> {
    let saturated: Vec<_> = rungs
        .iter()
        .filter(|(_, slowdown)| *slowdown >= FIT_FLOOR_SLOWDOWN)
        .collect();
    if saturated.len() < MIN_FIT_RUNGS {
        return None;
    }
    let numerator: f64 = saturated.iter().map(|(n, s)| f64::from(*n) * s).sum();
    let denominator: f64 = saturated.iter().map(|(n, _)| f64::from(*n).powi(2)).sum();
    let slope = numerator / denominator;
    (slope > 0.0).then(|| Fit {
        crossing: budget / slope,
        slowdown_per_session: slope,
        rungs: saturated.len(),
        ladder_sessions,
    })
}

/// Whether a rung is slow enough that the machine has run out of cores.
///
/// The fit needs this to know which rungs lie on the line. The drift check
/// needs it to pick a rung worth repeating, since a rung with cores to spare
/// would absorb a slower machine instead of showing it.
pub fn is_saturated(slowdown: f64) -> bool {
    slowdown >= FIT_FLOOR_SLOWDOWN
}

/// How slow a rung must run before it counts as saturated.
///
/// Below saturation every session gets the cores it asks for, so the slowdown
/// is flat rather than climbing and the rung sits well off the line. Including
/// those bends the fit toward the origin and reports a redline that is too
/// high. Stated as a slowdown rather than as a core count because the ramp
/// measures the one directly and infers the other.
const FIT_FLOOR_SLOWDOWN: f64 = 1.4;

/// How many saturated rungs a line needs before it is worth drawing.
const MIN_FIT_RUNGS: usize = 3;

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

#[cfg(test)]
mod tests {
    use super::*;

    /// A machine whose redline is exactly `at`, sampled without noise.
    fn clean(at: f64, counts: &[u32]) -> Vec<(u32, f64)> {
        counts
            .iter()
            .map(|n| (*n, WORK_RATE_BUDGET_FACTOR * f64::from(*n) / at))
            .collect()
    }

    #[test]
    fn a_clean_slope_returns_the_count_it_was_built_from() {
        let fit = fit_crossing(
            &clean(33.5, &[25, 31, 34, 37, 50]),
            WORK_RATE_BUDGET_FACTOR,
            31,
        )
        .expect("five saturated rungs are enough");
        assert!((fit.crossing - 33.5).abs() < 1e-9, "got {}", fit.crossing);
        assert_eq!(fit.rungs, 5);
    }

    #[test]
    fn unsaturated_rungs_are_left_out_of_the_line() {
        // Below saturation the slowdown is flat near 1 rather than climbing,
        // so those rungs sit well off the line. Fitting through them tilts it
        // and reports a ceiling higher than the machine has.
        let mut rungs = clean(33.5, &[25, 31, 34, 37, 50]);
        rungs.insert(0, (1, 1.00));
        rungs.insert(1, (10, 1.13));
        let fit = fit_crossing(&rungs, WORK_RATE_BUDGET_FACTOR, 34).expect("still enough");
        assert_eq!(fit.rungs, 5, "the two unsaturated rungs must not be fitted");
        assert!((fit.crossing - 33.5).abs() < 1e-9);
    }

    #[test]
    fn too_few_saturated_rungs_is_no_fit_rather_than_a_bad_one() {
        assert!(fit_crossing(&clean(33.5, &[34, 37]), WORK_RATE_BUDGET_FACTOR, 34).is_none());
        assert!(fit_crossing(&[], WORK_RATE_BUDGET_FACTOR, 0).is_none());
    }

    #[test]
    fn one_rung_landing_off_moves_the_fit_far_less_than_it_moves_a_search() {
        // The case that motivated this: rung 31 read 8% slow in one run of six
        // and the ladder answered 30 where its siblings answered 34. The fit
        // sees the same bad rung and barely flinches.
        let mut rungs = clean(33.5, &[25, 31, 34, 37, 50]);
        let bad = rungs.iter_mut().find(|(n, _)| *n == 31).expect("rung 31");
        bad.1 *= 1.08;
        let fit = fit_crossing(&rungs, WORK_RATE_BUDGET_FACTOR, 30).expect("five rungs");
        let drift = (fit.crossing - 33.5).abs() / 33.5;
        assert!(drift < 0.02, "one bad rung moved the fit by {drift:.3}");
    }

    #[test]
    fn the_slope_is_eta_over_the_cores_a_session_wants() {
        // slowdown = N·d/(η·C), so slope = d/(η·C) and the fit measures η.
        // Sixteen cores, three-quarter duty, η of 0.77.
        let (cores, duty, eta) = (16.0, 0.75, 0.77);
        let at = 2.0 * eta * cores / duty;
        let fit = fit_crossing(&clean(at, &[25, 31, 34, 37]), WORK_RATE_BUDGET_FACTOR, 33)
            .expect("four rungs");
        let recovered = duty / (fit.slowdown_per_session * cores);
        assert!((recovered - eta).abs() < 1e-9, "got η = {recovered}");
    }
}
