// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether two ramps may be set against each other.
//!
//! Every comparison this project has published spans two ladders — pipes
//! against pseudoconsoles, `cmd` against `pwsh`, one duty against another — and
//! each assumes the two saw the same machine. A ramp's own drift control tests
//! that assumption *within* a ladder and says nothing about it *between* two.
//!
//! It has already cost a result. A pseudoconsole ramp run nine hours after its
//! pipes counterpart returned 14 against 27, and the difference was not the
//! transport: the solo session ran at half the rate and ten sessions received
//! seven cores where they had received ten. Both ramps passed their own drift
//! checks, because both machines held still — they were just not the same
//! machine.
//!
//! The fix reuses what the drift control already relies on. **The solo rung is
//! a machine fingerprint**: one session, no contention, the same work. Two
//! ramps whose solo rungs disagree by more than a rung reproduces are measuring
//! different afternoons, and their redlines cannot be subtracted.

use serde::{Deserialize, Serialize};

use crate::ramp::RampReport;

/// How far two solo rungs may sit apart and still be one machine.
///
/// **This is the weakest number in the file.** Two percent was the first
/// guess, taken from [what a rung reproduces
/// within](../../docs/measurements/2026-07-30-164912-redline-reproducibility.md),
/// and it refused a pair that is known to be sound: the shell-control trio ran
/// back to back inside twenty minutes and its solo rungs still spanned 3.1%
/// (74.11, 76.44, 76.45). That 2% is the wrong statistic — it measures a
/// saturated rung repeated inside one ladder, where a fresh solo rung pays
/// process startup and a cold cache again.
///
/// **[Since measured](../../docs/measurements/2026-07-31-171719-what-a-baseline-is-worth.md),
/// and five survives for a better reason than it was chosen for.** The solo
/// rung reproduces to **0.37%** over two and a half minutes, so measurement
/// noise is well under a percent; what grows with the interval is the machine,
/// reaching 2.72% across a ten-minute ladder. Across ramps the gaps form a band
/// rather than a trend — 0.0%, 1.0%, 1.0%, 3.1%, 3.2%, 4.2% — and ten hours
/// apart is no worse than ten minutes.
///
/// So five sits just above the widest gap this machine produces while being
/// itself, and more than ten times below the pair this exists to refuse, whose
/// solo rungs sat 51.6% apart. The first justification — *the spread is 3.1%,
/// so allow 5* — read a single offset as noise and was wrong about both.
///
/// Still one machine on one afternoon: the band is this laptop's, not the
/// metric's.
pub const SOLO_AGREEMENT_PERCENT: f64 = 5.0;

/// Two ramps, and whether their redlines mean anything set side by side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    pub left_label: String,
    pub right_label: String,
    pub left_solo: f64,
    pub right_solo: f64,
    /// Signed, as a percentage of the left ramp's solo rate.
    pub solo_gap_percent: f64,
    /// Set when the two ran on hardware that does not match, which no solo
    /// agreement could excuse.
    pub machine_mismatch: Option<String>,
    pub left_redline: Option<u32>,
    pub right_redline: Option<u32>,
    /// How far each ramp's own solo rung moved when it was held again.
    ///
    /// A ramp whose baseline does not reproduce cannot lend that baseline to a
    /// comparison, however close the two happen to land.
    pub left_solo_spread: Option<f64>,
    pub right_solo_spread: Option<f64>,
}

impl Comparison {
    pub fn of(left: &RampReport, right: &RampReport) -> Self {
        let machine_mismatch = (left.machine.label() != right.machine.label())
            .then(|| format!("{} against {}", left.machine.label(), right.machine.label()));

        Self {
            left_label: left.label.clone(),
            right_label: right.label.clone(),
            left_solo: left.solo_units_per_sec,
            right_solo: right.solo_units_per_sec,
            solo_gap_percent: solo_gap_percent(left.solo_units_per_sec, right.solo_units_per_sec),
            machine_mismatch,
            left_redline: left.redline.as_ref().map(|r| r.sessions),
            right_redline: right.redline.as_ref().map(|r| r.sessions),
            left_solo_spread: left.solo_spread_percent(),
            right_solo_spread: right.solo_spread_percent(),
        }
    }

    /// The worst either ramp's own baseline moved, when both measured it.
    pub fn worst_solo_spread(&self) -> Option<f64> {
        match (self.left_solo_spread, self.right_solo_spread) {
            (Some(l), Some(r)) => Some(l.abs().max(r.abs())),
            (Some(one), None) | (None, Some(one)) => Some(one.abs()),
            (None, None) => None,
        }
    }

    /// Whether the two redlines may be subtracted.
    ///
    /// Two tests, and the second is the one a measured ramp brings with it: the
    /// gap between the baselines has to fit the allowance, **and** each ramp's
    /// own baseline has to reproduce at least that well. A ramp whose solo rung
    /// moved 8% on a repeat cannot lend that rung to a 5% judgement — the
    /// allowance would be finer than the thing it is measuring.
    pub fn comparable(&self) -> bool {
        self.machine_mismatch.is_none()
            && self.solo_gap_percent.abs() <= SOLO_AGREEMENT_PERCENT
            && self
                .worst_solo_spread()
                .is_none_or(|spread| spread <= SOLO_AGREEMENT_PERCENT)
    }

    /// The difference in sessions, when there is one worth quoting.
    pub fn redline_delta(&self) -> Option<i64> {
        if !self.comparable() {
            return None;
        }
        match (self.left_redline, self.right_redline) {
            (Some(l), Some(r)) => Some(i64::from(r) - i64::from(l)),
            _ => None,
        }
    }

    /// What a reader needs before quoting either number.
    pub fn verdict(&self) -> String {
        if let Some(mismatch) = &self.machine_mismatch {
            return format!(
                "**Different hardware** — {mismatch}. Nothing about these two belongs in one table."
            );
        }
        if let Some(spread) = self.worst_solo_spread()
            && spread > SOLO_AGREEMENT_PERCENT
        {
            return format!(
                "**Not comparable.** One of these ramps moved {spread:.1}% when it held its own solo rung again, against a {SOLO_AGREEMENT_PERCENT:.0}% allowance. Its baseline is noisier than the judgement being asked of it, so the two redlines cannot be told apart however close they land."
            );
        }
        if self.solo_gap_percent.abs() > SOLO_AGREEMENT_PERCENT {
            return format!(
                "**Not comparable.** The solo rungs sit {:.1}% apart against a {SOLO_AGREEMENT_PERCENT:.0}% allowance, so the machine moved between these ladders and the gap between their redlines is that move rather than what was varied. Run the pair back to back.",
                self.solo_gap_percent
            );
        }
        match self.redline_delta() {
            Some(delta) => format!(
                "Comparable: the solo rungs agree to {:.1}%, and the redline moves {delta:+} session(s).",
                self.solo_gap_percent
            ),
            None => format!(
                "Comparable to {:.1}% on the solo rung, but at least one ladder reached no redline, so there is nothing to subtract.",
                self.solo_gap_percent
            ),
        }
    }
}

/// The gap between two solo rates, as a percentage of the first.
///
/// Zero when both are zero, so an unmeasured pair reads as agreeing rather than
/// dividing by nothing.
fn solo_gap_percent(left: f64, right: f64) -> f64 {
    if left <= 0.0 {
        return if right <= 0.0 { 0.0 } else { f64::INFINITY };
    }
    (right - left) / left * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shell_control_trio_that_ran_back_to_back_is_admitted() {
        // The real figures, and the reason the threshold is not 2%: these
        // three ran inside twenty minutes and still spanned 3.1%.
        for (left, right) in [(74.11, 76.44), (74.11, 76.45), (76.44, 76.45)] {
            let gap = solo_gap_percent(left, right);
            assert!(
                gap.abs() <= SOLO_AGREEMENT_PERCENT,
                "a back-to-back pair has to pass, got {gap} for {left} against {right}"
            );
        }
    }

    #[test]
    fn the_pty_and_pipes_pair_that_prompted_this_is_refused() {
        // The real figures: 74.11 units/s in the morning, 35.89 nine hours on.
        let gap = solo_gap_percent(74.11, 35.89);
        assert!(
            gap.abs() > SOLO_AGREEMENT_PERCENT,
            "a half-speed machine has to fail the check, got {gap}"
        );
    }

    /// Two ramps whose baselines agree, so only the spread can refuse them.
    fn agreeing_pair() -> Comparison {
        Comparison {
            left_label: "a".into(),
            right_label: "b".into(),
            left_solo: 74.11,
            right_solo: 74.90,
            solo_gap_percent: solo_gap_percent(74.11, 74.90),
            machine_mismatch: None,
            left_redline: Some(27),
            right_redline: Some(26),
            left_solo_spread: None,
            right_solo_spread: None,
        }
    }

    #[test]
    fn agreeing_baselines_permit_a_subtraction() {
        assert!(agreeing_pair().comparable());
        assert_eq!(agreeing_pair().redline_delta(), Some(-1));
    }

    #[test]
    fn a_ramp_whose_own_baseline_moved_cannot_lend_it() {
        // Baselines 1.1% apart, but one ramp's solo rung moved 8% on a repeat:
        // the allowance is finer than the thing it would be judging.
        let noisy = Comparison {
            left_solo_spread: Some(-8.0),
            ..agreeing_pair()
        };
        assert!(!noisy.comparable(), "a noisy baseline cannot be lent");
        assert_eq!(noisy.redline_delta(), None);
        assert!(noisy.verdict().contains("8.0%"), "{}", noisy.verdict());
    }

    #[test]
    fn a_baseline_that_reproduces_still_permits_it() {
        let steady = Comparison {
            left_solo_spread: Some(0.4),
            right_solo_spread: Some(-1.2),
            ..agreeing_pair()
        };
        assert!(steady.comparable());
        assert_eq!(steady.worst_solo_spread(), Some(1.2));
    }

    #[test]
    fn an_unmeasured_pair_does_not_divide_by_nothing() {
        assert_eq!(solo_gap_percent(0.0, 0.0), 0.0);
        assert!(solo_gap_percent(0.0, 40.0).is_infinite());
    }

    #[test]
    fn the_gap_is_signed_so_a_reader_can_tell_which_way_it_moved() {
        assert!(solo_gap_percent(40.0, 20.0) < 0.0);
        assert!(solo_gap_percent(20.0, 40.0) > 0.0);
    }
}
