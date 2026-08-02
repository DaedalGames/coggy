# η follows the awake count, and the warm-up hold is what made it visible

`redline = 2ηC/d` treats the contention term as a constant, and [it is not](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md) — it measured 0.733 where twenty-seven sessions were awake at any instant and about 0.8 where seventeen were. But duty and the awake count moved together there, so nothing said which one `η` followed. **A hundred sessions at duty 0.27 and a hundred and thirty-five at 0.20 both leave twenty-seven awake, and they agree to within the run's own noise.**

## The four holds

| | sessions | duty | awake | per session | **total** | RSS |
|---|---|---|---|---|---|---|
| A1 | 100 | 0.27 | 27 | 10.530 | **1053.0** | 2.36 GiB |
| B1 | 135 | 0.20 | 27 | 7.789 | **1051.5** | 3.19 GiB |
| A2 | 100 | 0.27 | 27 | 10.569 | **1056.9** | 2.36 GiB |
| B2 | 135 | 0.20 | 27 | 7.677 | **1036.3** | 3.19 GiB |

240 s a hold, back to back, one uncounted warm-up before each. Every session alive at every report; `dropped_output` held on all four.

**Total throughput is the whole comparison and it needs no baseline.** At saturation a session's rate is `η·C ÷ (w·N)`, so `total = concurrent × N = η·C ÷ w`. Duty cancels; so do the machine's core count and its unit time, which two holds run back to back share. Their totals are equal exactly when their `η` are — which matters because a solo baseline is [the noisiest term available](2026-08-01-173927-the-baseline-is-the-noisy-term.md) and a set taken earlier the same day spread 25% across six holds.

## Read against its own controls

| | |
|---|---|
| Drift — A1 against A2 | **0.37%** |
| Reproducibility — B1 against B2 | **1.46%** |
| Effect — A against B | **1.05%** |

**The effect sits inside the noise floor, and that is the result rather than a failure to find one.** Detecting a difference needs it to clear both controls; bounding one needs the controls to be small, and they are. Any difference in `η` between the two configurations is **under about 1.5%** — while the configurations differ by 35% in session count and 26% in duty.

Neither of those can hide inside 1.5%. So **`η` is a function of the product `N·d` and not of either term alone**, at least around twenty-seven awake sessions.

**It also closes the confound the earlier attempt could not.** A holds 2.36 GiB and B holds 3.19, a 35% wider footprint, and the same bound covers it: in this range memory pressure reaches `η` only through the awake count too.

## The warm-up hold is why the controls are small

[The same comparison a day earlier drifted 6.29%](2026-08-02-114542-the-control-refused-its-own-run.md) and its control correctly refused it. The difference is one uncounted hold at the run's own session count, added because the opening hold had come back the outlier twice — 8% below its eight siblings in a gate run, and 6.3% under a repeat of itself in that control.

**Drift fell from 6.29% to 0.37%, seventeen-fold.** The rule was written from the run that refused itself and deliberately not applied to it; this is the new run that tests it, and it passes.

What the earlier attempt read as a 7.0% effect was drift. Under the same procedure with warm-ups it is 1.05% against a 1.46% floor.

## What this run cannot say

- **One awake count.** Twenty-seven, reached two ways. Whether the product holds at five or at fifty is unmeasured, and the curve of `η` against the awake count still rests on two points from runs of uncertain comparability.
- **The reproducibility is worse at 135 than at 100** — 1.46% against 0.37% — and nothing here separates whether that is the session count, the wider footprint, or where the two B holds fell in the sequence.
- **Saturation only.** The algebra that cancels the baseline assumes every core is claimed; below saturation total throughput stops being `η·C ÷ w` and the comparison does not hold.
- **`--rss-budget-gb 8`**, so the RSS verdicts here are not the gate's. These holds measure `η`, not gate M1.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 13% before the run, 21% after |
| Harness | `sessionbench hold` at commit `23b0f4f`, release build, four holds from one detached script |
| Daemon | `coggyd` 0.0.0, release build |
| Workload | `cpu-spin --units 100000000 --resident 20` at `--duty 0.27` and `--duty 0.20` |
| Shape | A·B·A·B, 240 s a hold, 30 s uncounted warm-up before each, no solo baselines |
