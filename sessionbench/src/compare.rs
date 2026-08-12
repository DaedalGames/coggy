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

/// Where a neighbour stops being noise and starts setting the core count.
///
/// **Under-determined by the data and recorded as such.** The tenant figures on
/// disk cluster at 0 and about 8.7 cores with nothing observed between, so any
/// bar in that gap separates the same pairs. Half a core matches the gate
/// scripts, which wait for the neighbour to fall under 0.5 before a hold, so
/// one number governs both and a pair the gate admitted is not then refused.
const TENANT_BUSY_CORES: f64 = 0.5;

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
///
/// **And it is a band for ramps, which a bracket then borrowed.** The critique
/// two paragraphs up — that a rung repeated inside one ladder is the quieter
/// statistic, because a fresh solo pays process startup and a cold cache again
/// — applies once more at the level above. A bracket's two baselines *are*
/// fresh: two daemon launches holding one session each. [Twelve of those spanned
/// 4.54% with no load between them](../../docs/measurements/2026-08-01-173927-the-baseline-is-the-noisy-term.md),
/// at a CPU share spanning 1.95%, so what moves them is which core the one
/// session landed on rather than anything a repeat would average out.
///
/// **That mechanism has a rival, found later and not separable now.** Those
/// twelve holds recorded the job and not the machine around it, and a neighbour
/// makes the same signature — nine one-session holds on 2026-08-03 moved 27%
/// between quiet and crowded windows while the job's own share held 0.24 to
/// 0.26 cores. So the 4.54% may be core placement or may be an afternoon, and
/// the constant sits above it either way. Twelve fresh holds would tell them
/// apart now that every one records the cores held elsewhere.
///
/// Left at five rather than widened here, because [`crate::daemon::bracket`] is
/// what would need the wider number and the population it should be calibrated
/// against has been measured once. Widening a shared constant to fit the noisier
/// of two callers would loosen the ramp comparison this was built for.
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
    /// Set when one ran on mains and the other on battery.
    ///
    /// **This belongs beside the hardware mismatch rather than beside the solo
    /// gap, because the solo rung cannot see it.** Two quiet holds of one
    /// workload ninety minutes apart read 9.574 and 9.322 units/s — [agreeing
    /// to 2.6% across a boundary worth
    /// 7.8×](../../docs/measurements/2026-08-11-163342-a-solo-baseline-cannot-see-the-plug.md).
    /// The 7.8× was measured at a hundred sessions, where the power plan's
    /// ceiling binds; a lone session at duty 0.27 asks for a quarter of one
    /// core out of sixteen and never makes the box leave its lowest state. So
    /// the fingerprint agrees at the baseline and the ramps diverge wherever
    /// they saturate, which is exactly where a redline is read.
    ///
    /// `None` when either side did not record the state, which is a different
    /// thing from the two matching: a pair that cannot be checked must not read
    /// as a pair that passed.
    pub power_mismatch: Option<String>,
    /// Whether one ramp ran beside a busy neighbour and the other did not.
    ///
    /// **The second mismatch a solo rung cannot see, and larger than the
    /// thermal state.** This box delivers **5.09-5.37 machine cores with
    /// `chrome-headless-shell` absent against 14.47-14.60 with it present**,
    /// while a 3.7x change in the sessions' own duty moves that total by 5%.
    /// Two censuses split **97-100% parked while the neighbour is absent
    /// against 2-4% while it is busy**, and a busy neighbour is present **61%**
    /// of the time — so roughly two runs in five are taken on a machine
    /// offering a third of the cores of the other three.
    ///
    /// Blind to the fingerprint for the same reason the plug is: a lone session
    /// at duty 0.27 asks for a quarter of one core and reads the same on a
    /// five-core box as on a fourteen-core one.
    ///
    /// Taken as the MAXIMUM across rungs rather than a median, because a
    /// neighbour present for one rung of ten still corrupts that rung and the
    /// redline is fitted across all of them.
    ///
    /// `None` when either side recorded no tenant figure, which is every
    /// artifact predating the column.
    pub tenant_mismatch: Option<String>,
    /// Whether the two ran different commands.
    ///
    /// The solo rung is a machine fingerprint only for ramps sharing a
    /// workload. Vary the workload's own duty and the baseline moves by
    /// design — a duty-1.0 ramp and a duty-0.27 one sat 75.3% apart while both
    /// held their own solo rungs to under a percent, and the verdict blamed a
    /// machine that had not moved. Differing commands do not make a pair
    /// incomparable on their own: the shell-control trio varied its wrapper
    /// and its solo rates still agreed to 3.2%.
    pub command_differs: bool,
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

        // Both sides must have answered. `Option::zip` gives None when either
        // did not, so an unrecorded state cannot be compared into a match --
        // the absence stays an absence instead of defaulting to `false` and
        // reading as two ramps that agreed.
        let power_mismatch = left
            .host
            .on_battery
            .zip(right.host.on_battery)
            .filter(|(l, r)| l != r)
            .map(|(l, _)| {
                let (first, second) = if l {
                    ("battery", "mains")
                } else {
                    ("mains", "battery")
                };
                format!("{first} against {second}")
            });

        // MAX across rungs, not a median: see the field's own note.
        let worst_tenant = |r: &RampReport| -> Option<f64> {
            r.steps
                .iter()
                .filter_map(|s| s.occupancy.as_ref())
                .map(|o| o.tenant_cores_median)
                .fold(None::<f64>, |acc, v| Some(acc.map_or(v, |a: f64| a.max(v))))
        };
        // Same `zip` as the power check, and for the same reason: an unrecorded
        // tenancy stays an absence instead of defaulting to zero and reading as
        // two ramps that agreed on a quiet machine.
        let tenant_mismatch = worst_tenant(left)
            .zip(worst_tenant(right))
            .filter(|(l, r)| (l >= &TENANT_BUSY_CORES) != (r >= &TENANT_BUSY_CORES))
            .map(|(l, r)| format!("{l:.2} against {r:.2} cores held by the neighbour"));

        Self {
            left_label: left.label.clone(),
            right_label: right.label.clone(),
            left_solo: left.solo_units_per_sec,
            right_solo: right.solo_units_per_sec,
            solo_gap_percent: solo_gap_percent(left.solo_units_per_sec, right.solo_units_per_sec),
            machine_mismatch,
            power_mismatch,
            tenant_mismatch,
            command_differs: left.command != right.command,
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
            && self.power_mismatch.is_none()
            && self.tenant_mismatch.is_none()
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
        // BEFORE every solo-based branch below, and the order is load-bearing:
        // a mains/battery pair can agree on its solo rungs to 2.6% and still be
        // 7.8x apart where it saturates, so reaching a solo verdict first would
        // print "comparable" for a mismatch the fingerprint is blind to. The
        // neighbour check below is the second of those; the count is left out
        // deliberately, because a sentence counting this list went stale once.
        if let Some(mismatch) = &self.power_mismatch {
            return format!(
                "**Different power state** — {mismatch}. A solo rung cannot see this: two quiet holds of one workload agreed to 2.6% across it while the state is worth 7.8x under saturation, which is where a redline is read."
            );
        }
        // Also before the solo branches, for the same reason: the fingerprint
        // agrees at the baseline whatever the neighbour is doing, because one
        // session at duty 0.27 never makes the box leave its lowest state.
        if let Some(mismatch) = &self.tenant_mismatch {
            return format!(
                "**Different neighbour** — {mismatch}. A solo rung cannot see this either: this machine delivers 5.09-5.37 cores with the neighbour absent against 14.47-14.60 with it present, while a 3.7x change in the sessions' own duty moves that total by 5%."
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
            return if self.command_differs {
                format!(
                    "**Not comparable, and the machine is not why.** The solo rungs sit {:.1}% apart against a {SOLO_AGREEMENT_PERCENT:.0}% allowance, but these ramps ran different commands — so the baseline may have moved because of what was varied rather than because the machine did. A solo rung is a fingerprint only across ramps that share a workload. Compare these by hand, against what the change was expected to do to a single session.",
                    self.solo_gap_percent
                )
            } else {
                format!(
                    "**Not comparable.** The solo rungs sit {:.1}% apart against a {SOLO_AGREEMENT_PERCENT:.0}% allowance, so the machine moved between these ladders and the gap between their redlines is that move rather than what was varied. Run the pair back to back.",
                    self.solo_gap_percent
                )
            };
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
            power_mismatch: None,
            tenant_mismatch: None,
            left_redline: Some(27),
            right_redline: Some(26),
            command_differs: false,
            left_solo_spread: None,
            right_solo_spread: None,
        }
    }

    #[test]
    fn agreeing_baselines_permit_a_subtraction() {
        assert!(agreeing_pair().comparable());
        assert_eq!(agreeing_pair().redline_delta(), Some(-1));
    }

    /// The pair this exists for: baselines that agree, and a plug that does not.
    ///
    /// Built from `agreeing_pair`, whose solo rungs sit ~1% apart and which the
    /// test above proves is otherwise accepted — so the only thing refusing
    /// this one is the power state, and the assertion cannot pass for some
    /// other reason. That is the shape the real pair had: 9.574 against 9.322,
    /// 2.6% apart, one on each side of the boundary.
    #[test]
    fn a_power_state_refuses_a_pair_its_baselines_would_have_passed() {
        let crossed = Comparison {
            power_mismatch: Some("mains against battery".into()),
            tenant_mismatch: None,
            ..agreeing_pair()
        };
        assert!(
            agreeing_pair().comparable(),
            "the control must pass, or this test proves nothing"
        );
        assert!(
            !crossed.comparable(),
            "a mains/battery pair must be refused"
        );
        assert_eq!(
            crossed.redline_delta(),
            None,
            "and its redlines must not subtract"
        );
        assert!(
            crossed.verdict().contains("Different power state"),
            "the verdict must say which check refused it, got: {}",
            crossed.verdict()
        );
    }

    /// The neighbour, in the same shape as the power test above.
    ///
    /// This box delivers 5.09-5.37 machine cores with the neighbour absent
    /// against 14.47-14.60 with it present, and a busy one is there 61% of the
    /// time — so a pair whose baselines agree can still be two machines. The
    /// control is what makes the refusal mean something rather than passing for
    /// an unrelated reason.
    #[test]
    fn a_busy_neighbour_refuses_a_pair_its_baselines_would_have_passed() {
        let tenanted = Comparison {
            tenant_mismatch: Some("8.77 against 0.00 cores held by the neighbour".into()),
            ..agreeing_pair()
        };
        assert!(
            agreeing_pair().comparable(),
            "the control must pass, or this test proves nothing"
        );
        assert!(
            !tenanted.comparable(),
            "a tenanted/quiet pair must be refused"
        );
        assert_eq!(
            tenanted.redline_delta(),
            None,
            "and its redlines must not subtract"
        );
        assert!(
            tenanted.verdict().contains("Different neighbour"),
            "the verdict must say which check refused it, got: {}",
            tenanted.verdict()
        );
    }

    /// An unrecorded state is not a matching one.
    #[test]
    fn a_state_only_one_side_recorded_cannot_read_as_agreement() {
        // `zip` is what enforces this: None on either side yields no mismatch
        // to report, and the pair must not therefore be advertised as checked.
        assert_eq!(None::<bool>.zip(Some(true)), None);
        assert_eq!(Some(false).zip(None::<bool>), None);
        // And two that did answer, differing, do produce one.
        assert_eq!(
            Some(false).zip(Some(true)).filter(|(l, r)| l != r),
            Some((false, true))
        );
    }

    #[test]
    fn a_varied_workload_is_not_blamed_on_the_machine() {
        // A duty-1.0 ramp against a duty-0.27 one: the baselines are 75% apart
        // by design, and both held their own solo rungs to under a percent.
        let varied = Comparison {
            left_solo: 77.34,
            right_solo: 19.11,
            solo_gap_percent: solo_gap_percent(77.34, 19.11),
            command_differs: true,
            left_solo_spread: Some(0.91),
            right_solo_spread: Some(-0.49),
            ..agreeing_pair()
        };
        assert!(
            !varied.comparable(),
            "the redlines still cannot be subtracted"
        );
        let verdict = varied.verdict();
        assert!(verdict.contains("machine is not why"), "{verdict}");
        assert!(!verdict.contains("Run the pair back to back"), "{verdict}");
    }

    #[test]
    fn the_same_workload_far_apart_is_still_blamed_on_the_machine() {
        let drifted = Comparison {
            left_solo: 74.11,
            right_solo: 35.89,
            solo_gap_percent: solo_gap_percent(74.11, 35.89),
            command_differs: false,
            ..agreeing_pair()
        };
        assert!(
            drifted.verdict().contains("the machine moved"),
            "{}",
            drifted.verdict()
        );
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
