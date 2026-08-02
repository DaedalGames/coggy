# The control refused its own run, and the first hold is why

An attempt to find out **what `η` is a function of**. [It measured 0.733 at duty 0.27 and about 0.8 at 0.172](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md), where duty and the number of sessions awake at any instant moved together — so nothing said which one it followed. The test was to reach the same awake count two ways.

**It came back inconclusive, and the reason is worth more than the answer would have been.**

## What was run

| | sessions | duty | awake `N·d` | per session | **total** |
|---|---|---|---|---|---|
| A1 | 100 | 0.27 | 27 | 9.726 | **972.6** |
| B | 135 | 0.20 | 27 | 7.711 | **1041.0** |
| A2 | 100 | 0.27 | 27 | 10.338 | **1033.8** |

Three holds of 300 s, back to back, no solo baselines. All held every session to the last report; `dropped_output` came back **held** on each, at 100 and at 135.

## The solo cancels, which is why there is none

A slowdown is `solo ÷ concurrent`, and the solo baseline is [the noisiest term in the whole comparison](2026-08-01-173927-the-baseline-is-the-noisy-term.md) — a set taken an hour earlier the same day spread **25%** across six holds. Comparing two slowdowns through it means dividing the two quietest numbers by the loudest one, twice.

It is avoidable. At saturation a session's rate is `solo ÷ slowdown`, and with `solo = d/w` and `slowdown = N·d/(η·C)` that comes to

```
concurrent = η·C ÷ (w·N)        so    concurrent × N = η·C ÷ w
```

**Duty cancels, and so do the machine's core count and its unit time.** Two holds run back to back share `C` and `w`, so their *total* throughputs are equal exactly when their `η` are — one number per hold, no baseline, and nothing that cares whether the machine is fast or busy.

## The drift control refused it

A1 and A2 are the same configuration twenty minutes apart, and they differ by **6.29%**. The effect being chased is B against A, which is 3.77% against the two A's interpolated to B's midpoint. **The control is larger than the signal, so this run says nothing about `η`.**

That is the control working. Without A2 the run would have reported B sitting 7.0% above A1 and read as a result.

## The first hold is not a measurement

The shape underneath the drift: **A1 is the outlier and the two later holds agree to 0.70%.**

It is the second time. [A gate run's nine solo holds put its first 8% below the other eight](2026-08-01-202112-gate-m1-at-twenty-minutes.md), which was written off as *cold behind a build*. Twice in two runs, always the opening hold, is a systematic first-hold deficit rather than luck — a cold file cache, a background still settling from whatever produced the binaries, and a page-fault storm across a hundred fresh processes all land on whichever hold goes first.

**Averaging is the wrong instrument for it.** Repeats dilute a systematic deficit rather than removing it, and each costs a hold; three repeats leave a third of it. `hold --with-solo` now runs one short throwaway at the run's own session count before anything counted, and drops it.

**The rule was written from this run and is not applied to it.** Dropping A1 leaves B and A2 agreeing to 0.70%, which would say `η` follows the awake count — a conclusion produced by a rule invented after seeing the data, so it is not one. The next run decides, with the throwaway already in place.

## What this run cannot say

- **Nothing about `η`.** The question stands exactly where it stood.
- **The two configurations differ in more than the awake count.** A holds 2.37 GiB and B holds 3.20; a difference between them could be footprint rather than session count, and separating that needs a third run varying `--resident` alone.
- **The drift's own cause is unattributed.** Background moved from 28% to 9% across the run and a directory scan was started during B's teardown by mistake — an observer that was not free. Either would do it, and neither was isolated.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 28% before the run, 9% after |
| Harness | `sessionbench hold` at commit `86f0e92`, release build, three holds from one detached script |
| Daemon | `coggyd` 0.0.0, release build |
| Workload | `cpu-spin --units 100000000 --resident 20` at `--duty 0.27` and `--duty 0.20` |
| Shape | 300 s a hold, A·B·A, no solo, `--rss-budget-gb 8` so the RSS verdict is not the gate's |
