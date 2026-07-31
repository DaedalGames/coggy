# The duty relation at a quarter duty · 2026-08-01 08:01:58

`redline = 2ηC/d` was [derived rather than fitted](2026-07-30-154348-duty-is-derivable.md) and checked against a rung predicted in advance. Every ramp behind it ran at duty **1.00 or 0.75**. The measured duty of [a driven agent session is 0.27](2026-07-31-054657-the-driven-duty.md), and the 93-session core ceiling in [the G0 freeze](2026-07-31-150258-g0-frozen.md) sat on an extrapolation 2.8× below anything tested.

This tests it there.

## Both ramps held still, which took four attempts

| | Control, duty 1.00 | Test, duty 0.27 |
|---|---|---|
| Solo rate | 77.34 units/s | 19.11 units/s |
| Solo ratio | — | **0.247** against a requested 0.27 |
| **Solo check** | **+0.91%** | **−0.49%** |
| **Drift check** | −0.6% | **−0.0%** |
| Redline | **26**, fitted 26.93 | none — 100 sessions held at 1.83× |

Run back to back, same workload but for the duty knob, same hold, same resolution, on a machine `doctor` read at 17% background. **Every control inside a percent**, which is the first time today a duty pair has managed that: [three earlier attempts died on load shifting mid-ladder or on a lighter workload exposing core heterogeneity](2026-07-31-145412-the-cores-are-not-interchangeable.md).

## The relation holds to about ten percent

The test ladder ran out before the budget did, so its crossing is read from its own last two rungs:

| Sessions | Slowdown |
|---|---|
| 75 | 1.384× |
| 100 | 1.830× |

That is **0.01783 of slowdown per session**, meeting the 2× budget at **109.6**.

| | |
|---|---|
| Control's crossing | 26.93 at `d = 1.00` |
| Relation predicts at `d = 0.27` | `26.93 ÷ 0.27` = **99.7** |
| Measured | **109.6** |
| **Relation reads low by** | **9.9%** |

The slopes say the same thing from the other side: predicted `0.07427 × 0.27` = 0.02005 against a measured 0.01783, low by the same margin. **Two routes, one answer** — which is the check, since one of them could have been arithmetic on a mis-read rung.

**So `2ηC/d` survives a duty 2.8× below where it was built.** It is not exact: sessions at a quarter duty reach their ceiling about a tenth later than the relation says, so it is conservative in the direction that matters for admission.

## What that does to the core ceiling

The 93 sessions in the freeze came from `2ηC ≈ 25` and `d = 0.27`. Today's control puts `2ηC` at **26.93** for this workload, giving 99.7 — and the measurement says 109.6.

**The core ceiling was an extrapolation and is now a measurement with a ±10% error bar.** It does not move which condition binds: [memory stops a generation session at nine](2026-07-31-150258-g0-frozen.md), and nothing here comes within an order of magnitude of that.

## And the pair broke the tool that judges pairs

`compare` refused these two and gave the wrong reason — *the machine moved between these ladders* — when the machine had not moved at all. The duty knob moved the baseline by design, from 77.34 to 19.11, and both ramps held their own solo rungs to under a percent while it did.

**A solo rung is a machine fingerprint only across ramps that share a workload.** The tool now splits the verdict on whether the commands differ, and still refuses the pair, because two redlines measured against different baselines cannot be subtracted whatever moved them. Differing commands are not disqualifying on their own — [the shell-control trio varied its wrapper and its solo rates agreed to 3.2%](2026-07-31-141334-the-shell-costs-teardown.md).

## What this rests on

- **One pair, one workload.** `cpu-spin` at 80 MiB resident, which is memory-bound enough to hide [this machine's three core tiers](2026-07-31-145412-the-cores-are-not-interchangeable.md). A compute-bound workload would test the relation against a `C` that is not one number.
- **The test crossing is an extrapolation of 9.6 sessions** past the last rung measured, from two points. The ladder was capped at 100 because 150 sessions want 12.6 GiB against 10.2 free.
- **`d = 0.27` is the agent CLI's driven duty**, measured on one CLI driving one repository. The workload here only imitates that number.
- **17% background**, above the 10% `doctor` calls quiet. Both controls held anyway, which is what makes the run readable rather than the background figure.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Workload | `cpu-spin --units 9000000`, 80 MiB resident, pipes |
| Ramps | 60 s holds, resolution 2, capped at 60 and 100 sessions, run back to back |
| sessionbench | 0.0.0 at commit `29c532e`, release build |
| Defender | real-time protection on, no exclusions |
