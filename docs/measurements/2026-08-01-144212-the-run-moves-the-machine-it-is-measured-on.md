# The run moves the machine it is measured on · 2026-08-01 14:42:12

A work-rate condition is a ratio against the same workload held alone, so it needs a baseline. Taking one is not the hard part. **Taking one that describes the machine the concurrent run happened on** is, and the first attempt found out why.

## Three numbers, and the third is the finding

Two solo holds of `cpu-spin` bracketing eight concurrent sessions, all through `sessionbench hold --with-solo`:

| | units/s/session |
|---|---|
| solo, before | **33.645** |
| eight sessions, 24 s | 18.839 |
| solo, after | **31.413** |

**The two solo holds sit 6.9% apart**, against a 5% allowance. The run refused to produce a ratio.

Sixteen seconds separate the concurrent hold's end from the second solo hold's start. Nothing else ran. **The load moved the machine, and the machine had not come back.**

## It is the same effect a ramp already carries, at a heavier dose

[The drift control every ramp runs](../../CLAUDE.md) exists because a rung measured 40.02 units/s at the start of one ladder and 38.11 at the end — 4.8% across ten minutes of laddering. Here it is 6.9% across roughly one minute, because eight sessions at duty 1.0 saturate where a ladder's lower rungs do not.

So this is not a new phenomenon. What is new is where it lands: a ramp's drift check compares two rungs of the same *ladder* and reports a percentage, while a bracket uses its two solo holds as a *baseline* and divides by them.

## The allowance was borrowed, and it may be answering a different question

Five percent comes from [what a baseline is worth](2026-07-31-171719-what-a-baseline-is-worth.md), where it was measured rather than chosen: a solo rung reproduces to 0.37% over two and a half minutes, what grows with the interval is the machine, and across ramps the gaps form a band topping out at 4.2%.

`sessionbench compare` uses it to ask **is this one machine**, because two finished ramps cannot be averaged — either they saw the same afternoon or their redlines cannot be subtracted.

**A bracket is not doing that.** It averages its two baselines, which already corrects drift that is *linear* across the run. What its gap has to be small enough for is curvature, not sameness. Two checks now share a constant and ask different things, and the reuse was not examined for it.

## Which shape it is decides which way to move, and 6.9% does not say

The same number supports opposite conclusions:

- **A ramp** — the machine warming steadily through the run — puts the true baseline between the two solo holds. Averaging is the right estimator and the allowance is too tight.
- **A step** — the machine depressed only while loaded, recovering quickly after — makes the two holds samples of two states whose average is neither. Averaging is then wrong at *any* gap, and the fix is a cooldown rather than a number.

One trailing solo hold cannot tell a recovering point from a settled one. **Three can**, and that is the measurement this run earns rather than performs.

## What this does not claim

- **One bracketed run**, at eight sessions for 24 seconds with 16-second solo holds either side. Short holds are the only way to resolve a transient and they are the noisiest: a triple of twenty-second solo holds spread 2.4% to 3.5% on this machine, against the 6.9% being read.
- **Nothing about the direction of causation beyond timing.** The load ran, the machine was slower afterwards, and nothing else was running. Thermal, scheduler and cache effects are not separated here and this run has no way to separate them.
- **The allowance is not shown to be wrong.** It is shown to be answering a question nobody checked it against. Refusing is the safe failure and the tool refuses; what is open is whether it should have.
- **`ping` would have hidden all of it.** Three solo holds of it return the same count by construction, because it emits one line a second whatever the machine is doing. This is measurable only with a workload that competes for a core.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 26% before the run |
| Harness | `sessionbench hold --with-solo` at commit `0626e0d`, debug build |
| Daemon | `coggyd` 0.0.0, release build |
| Workload | `cpu-spin --units 100000 --duty 1.0 --resident 20` |
| Shape | solo 16 s · 8 sessions 24 s · solo 16 s, sampled every 4 s, back to back |

## 2026-08-01: one candidate ruled out, and it was in the instrument

Reading this run's own arithmetic turned up a real defect in `hold`, and it is
not big enough to be the answer.

The rate divided the daemon's cumulative counter by the hold's whole length.
Those are different windows: the daemon emits its final report and *then* clears
the pool, so the count stops where teardown begins while the clock runs through
it. A comment in `daemon.rs` had asserted the two spanned the same window and so
cancelled — they do not, and the leftover is the one part of a hold that grows
with the session count. A bracket divides a one-session hold by an
eight-session one, so it does not cancel there either.

Measured on this machine at twenty seconds a hold: **one session left 14 ms
uncounted of 20449, eight left 43 of 20500** — 0.068% against 0.210%, so the
slowdown carried 0.14% of bias, reading high. Fixed by dividing over what the
counter covers, and both numbers are now printed and in `hold.json` as
`counted_ms` beside `duration_ms`.

**Two orders of magnitude short of the 6.9%, so the shape question above stands
untouched.** What it removes is a candidate, which is the cheaper half of an
open question. The visible window line is worth more than the correction: a gap
much wider than a teardown is a final report that never arrived, which until now
looked like a slow hold.

## 2026-08-01: the shape question, asked three times, found nothing to shape

The section above leaves open whether the deficit is a ramp or a step, and says
one trailing hold cannot tell. Three repeats of the shape with three trailing
holds each [find no deficit to classify](2026-08-01-173927-the-baseline-is-the-noisy-term.md):
every trailing point straddles zero across repeats.

What the same run found instead is that this record's own baseline is the noisy
term. Eight sessions averaged inside one hold reproduce to 0.07% across three
runs while one session, twelve times, spans 4.54% — at a CPU share spanning
1.95%, so the sessions were given the same slice and did different work with it.

**That does not retract the 6.9% above**, which is larger than anything seen
here and cannot be re-measured now. It removes the assumption that a gap that
size needs a load to explain it, and it moves the open question: not *which
shape is the drift*, but *why a bracket's allowance is calibrated on a
population quieter than the one it judges*.
