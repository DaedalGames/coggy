# η is flat where it was said to fall

[The day before, `η` was reported falling as more sessions were awake at once](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md) — 0.733 with twenty-seven awake against about 0.8 with seventeen — and that reading is why `2ηC/d` was called unusable with a constant term. **Measured directly, between twenty-seven awake and forty-five, it does not fall.**

## The run

A hundred sessions throughout, so the session count and the 2.4 GiB footprint are fixed and only the duty moves. Each hold bracketed by a reference at the duty the gate cares about.

| | duty | awake | **total throughput** |
|---|---|---|---|
| reference | 0.27 | 27 | **907.1** |
| **held** | **0.45** | **45** | **913.9** |
| reference | 0.27 | 27 | **900.9** |

| | |
|---|---|
| Drift between the two references | **0.69%** |
| The held point against their mean | **+1.10%** |

**Total throughput is `η·C ÷ w`**, so with `C` and `w` shared across back-to-back holds it moves only with `η`. The awake count rose 67% and it moved 1.10%, against a drift of 0.69% — and in the *opposite* direction to a term that worsens with load.

**So `η` is flat across this range to within about a percent**, and the honest reading of the sign is that a 1.10% rise sitting 0.4 points above its own drift is not a rise.

## Why the earlier reading disagreed, and which one to keep

Two differences, and both favour this run.

**The earlier `η` came through a solo baseline.** It was `N·d/C` divided by a slowdown, and a slowdown divides by a solo hold — the noisiest number this instrument produces, [spreading 25% across six holds on one afternoon](2026-08-02-114542-the-control-refused-its-own-run.md). The run that gave 0.733 had an 8% outlier in its own baseline. Total throughput needs no baseline at all.

**And the seventeen-awake point sits on the saturation boundary.** `N·d ÷ C` is 1.07 there against 1.69 and 2.81 for the two points here. The algebra behind `total = η·C ÷ w` assumes every core is claimed for the whole hold; a tenth above the line, some are idle part of the time, and what comes out is not the same quantity. **The apparent trend was a comparison between a saturated point and a boundary one.**

So the constant survives: **`2ηC/d` is usable with a fixed `η` in the saturated regime**, and the caution it was given a day ago applies to the boundary rather than to the relation.

## What this run cannot say

- **Nothing below twenty-seven awake.** Where the boundary effect ends is unmeasured, and the interesting region for a governor — deciding whether to admit one more session — is exactly there.
- **One machine, one footprint.** A hundred sessions holding 2.4 GiB. Whether `η` stays flat when the working set approaches the cache is a different question.
- **The sign is not resolved.** +1.10% against 0.69% of drift is compatible with flat, and with a slight improvement. Two more repeats would separate them, and nothing here needs them separated.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 13% before the run |
| Harness | `sessionbench hold` at commit `23b0f4f`, release build, three holds from one detached script |
| Daemon | `coggyd` 0.0.0, release build |
| Workload | `cpu-spin --units 100000000 --resident 20` at `--duty 0.27` and `--duty 0.45` |
| Shape | reference · held · reference, 240 s a hold, 30 s uncounted warm-up before each, no solo baselines |
