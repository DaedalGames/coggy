# How much the redline moves between identical runs · 2026-07-30

Every measurement in this repository quotes a redline to the session, and until now no configuration had ever been run twice. The reproducibility of the headline number was unmeasured while every other conclusion rested on it.

## What forced the question

A matched pair of ramps — one waiting proportionally, one waiting a fixed 4 ms, at the same solo duty — returned **31** and **34**. That looked like the waiting mechanism mattering. Overlaying the rungs said otherwise:

| Sessions | Proportional | Fixed wait |
|---|---|---|
| 25 | 1.50× | 1.50× |
| 31 | 1.89× | 1.81× |
| 34 | 2.01× | 1.99× |
| 37 | 2.20× | 2.19× |

The curves are the same. The redlines differ because rung 34 lands *on* the 2× budget, and 2.01 against 1.99 is a coin toss that the ladder then amplifies: one run bisected down into (31, 34) and stopped at 31, the other went up and stopped at 34.

## Seven runs

| Configuration | Ladder redline |
|---|---|
| Proportional duty 0.75 | 31, 34, 31, 33 |
| Fixed wait 4 ms | 34, 30, 31 |

**30, 31, 31, 31, 33, 34, 34 — mean 32.00, range 4 sessions, 12.5%.** The two configurations interleave completely, so the waiting mechanism does not move the redline and [the algebra that predicted it would not](2026-07-30-duty-is-derivable.md) survives.

**The redline as the ladder computes it is reproducible to about ±13%, not to the session.** Every figure in the records before this one carries that spread unstated.

## The instrument is precise; the estimator is not

The rungs themselves reproduce far better than the redline drawn from them:

| Quantity | Spread across runs |
|---|---|
| Rate at 25 sessions | 1.7% |
| Rate at 37 sessions | 2.1% |
| Solo rate | 1.9% |
| **Ladder redline** | **12.6%** |

So the noise is not in the measuring. It is in asking a noisy curve *where it crosses a line* and then trusting one reading of one rung to decide which way to search next. Bisection assumes the verdicts are monotone and exact; within a few percent of the budget they are neither.

## Fitting the slope instead of hunting the crossing

The [derivation](2026-07-30-duty-is-derivable.md) says `slowdown = N·d/(η·C)` — **linear in N, through the origin.** So the right estimator is not a search at all: fit the slope through the saturated rungs and solve it at the budget.

Fitting `slowdown = b·N` and solving `b·N = 2` *is* `N = 2ηC/d`, with `b = d/(ηC)`. Measuring the slope is measuring `η`. The estimator and the model are the same object.

Against the same runs, changing nothing but how the number is drawn from them:

| Estimator | Mean | Range | Spread |
|---|---|---|---|
| Ladder bisection | 32.00 | 4.00 | **12.5%** |
| Least squares, free intercept | 33.46 | 0.86 | 2.6% |
| **Least squares through the origin** | **33.59** | **0.76** | **2.3%** |

**Five times more reproducible from the same measurements.** The fit uses every rung, so no single one can drag the answer.

Run by run, through the origin: 33.31, 33.35, 33.46, 33.62, 33.65, 33.65, 34.06, against the ladder's 30, 31, 31, 31, 33, 34, 34.

The free intercept came out between +0.015 and +0.166 — unstable, and the derivation says it should be zero. Dropping it *improved* reproducibility, which is what happens when a parameter is absorbing noise rather than describing anything.

### What the change cost

| | |
|---|---|
| Runtime | unchanged — the ladder still climbs and refines, and no rung is added |
| Code | `fit_crossing` in `redline.rs`, about fifteen lines and five tests |
| Scope | work rate only. Dropped output is an edge with nothing between two rungs to interpolate; RSS and replacement lag are slopes but would each need their own quantity and budget |
| Compatibility | `Redline` gains a `fitted` field, so older `ramp.json` files stay readable and newer ones carry both numbers |

**Five times more reproducible at no measurement cost**, because the readings were always there — only one of them was being used.

The fitted value also agrees with what the duty relation predicts independently, `2 × 0.77 × 16 / 0.75 = 32.9`, to within 2.1% — closer than the ladder's mean managed.

> The first six runs gave a spread of 1.0%, and this record said so. A seventh — taken by the instrument after the fit shipped, rather than by hand afterwards — came in at 34.06 and widened it to 2.3%. Six samples were not enough to quote a spread to one decimal, which is the same mistake in miniature as quoting a redline to the session.

## An anomaly this does not explain

Rung 32 read **slower than rung 34** in every run that measured it: 2.06, 2.07, 2.06 against 2.01, 1.99, 2.01. Three independent runs agreeing is not noise.

Residuals against each run's own line, by position in the run, put the last-measured rung at **+0.075** where every earlier position sits near −0.02. Something about being measured late reads as slower.

**But 32 was always the last rung measured**, so position and session count are perfectly confounded here, and two of the six runs break the pattern. Thermal accumulation over a fifteen-minute ramp is the obvious suspect; Defender working through the logs earlier rungs left behind is another; neither is measured. This is [the same confusion that cost the Defender estimate](2026-07-30-defender-at-scale.md), and it is recorded as open rather than resolved.

The test that separates them: measure one rung at the start of a ramp and the same rung at the end. Same session count, different position.

**Every ramp now does this.** Rather than placing an arbitrary rung early — which the fixed ladder cannot do — it holds the *lowest saturated* rung a second time once the ladder has finished. One extra hold, on by default, kept out of `steps` so the same count does not enter the curve twice. It is also the only control the ramp has on itself for a reason that outlives this anomaly: the fitted slope averages noise away but carries drift straight through, so a machine slowing under its own ladder would report a ceiling too low with nothing anywhere to show it.

### It fired on the first run that used it

```
redline: 32 sessions (WorkRate) · cpu-spin · pipe · 16C/31GiB · Defender on
  fitted at 32.3 through 6 saturated rungs, gaining 0.0619 slowdown per session
  drift check: 25 sessions ran 40.02 units/s early and 38.11 at the end (+4.8% slower)
```

| | Six quiet runs | This run |
|---|---|---|
| Fitted redline | 33.31 – 34.06 | **32.3** |
| Slope | 0.0554 – 0.0590 | **0.0619** |
| Cores at the saturated rungs | 15.0 – 15.2 | 13.0 – 15.1, falling through the run |

**The failure mode is exactly the predicted one.** The machine lost 4.8% across the ramp, the slope steepened, and the redline came out 3.9% low — and without the control, 32.3 would have looked as reasonable as 33.6.

**A large part of that drift was the observer.** During the six earlier runs nothing else touched the machine; during this one it was being edited alongside. So this run is contaminated as evidence about *what* makes the machine drift — thermal, Defender, or an agent writing files — and it is a clean demonstration of the thing the control is for. [The Defender estimate that had to be withdrawn](2026-07-30-defender-at-scale.md) was contaminated the same way, with no line anywhere in the output to show it.

The late-rung anomaly is a plausible casualty of the same effect — rung 32 was measured last in the runs where it read high — but that remains unproven, because those runs had no control on them.

## What this changes

- The fitted crossing replaces the bisection result when work rate is what broke. Dropped output stays a search, because it is an edge rather than a slope and nothing is being interpolated. RSS and replacement lag are slopes too and could each be fitted, but against their own quantity and their own budget.
- Redlines already recorded stand as measurements but should be read as ±13% unless they were taken far from their budget.
- `redline × duty ≈ 25` and everything derived from single ladders inherits that spread. The relation survives it, because the errors are scatter rather than bias — four samples at duty 0.75 average 32.25 against a prediction of 32.9, closer than any one of them.

### Which conclusions this could have overturned, and did not

A spread of ±13% swallows any finding that turned on two redlines differing by less than that. Every earlier record was checked for one:

| Record | Rests on | Touched |
|---|---|---|
| [Pseudoconsole against pipes](2026-07-30-first-redlines.md) | Per-session RSS, reproducible to 0.01 MiB, and arithmetic over it. **Every rung held in both modes**, so no redline was produced to compare | No |
| [Defender at scale](2026-07-30-defender-at-scale.md) | Work rate flat to three significant figures across a fiftyfold range. **Neither ramp reached a redline** | No |
| [The output path](2026-07-30-output-path.md) | An aggregate throughput ceiling, and a redline whose neighbouring rungs sat at 1.77× and 2.30× — far enough from the budget that a few percent cannot move it | No |
| [Duty and redline](2026-07-30-duty-and-redline.md) | A ratio across a fourfold range, where the spread is a tenth of the effect | No |

**No conclusion in this repository was drawn from a redline difference small enough for this to reach.** That is luck rather than discipline: the comparisons that mattered happened to be made on quantities measured directly, and the redline was mostly used as a headline rather than as evidence.

## The readings, so the fits can be checked

Per-session units/s, listed in the order each run measured them. The ladder climbs to bracket and then halves, so the counts are not in ascending order and the position of a rung is part of its record.

| Run | Solo | Rungs, in measurement order |
|---|---|---|
| prop-1 | 61.10 | 10: 53.61 · 25: 40.82 · 50: 20.88 · 37: 27.79 · 31: 32.31 · 34: 30.46 · 32: 29.64 |
| fixed-1 | 60.33 | 10: 54.66 · 25: 40.20 · 50: 20.53 · 37: 27.53 · 31: 33.34 · 34: 30.25 · 35: 27.69 |
| prop-2 | 60.81 | 10: 53.65 · 25: 40.77 · 50: 20.43 · 37: 27.74 · 31: 32.28 · 34: 30.50 · 35: 29.43 |
| fixed-2 | 61.33 | 10: 55.17 · 25: 41.06 · 50: 20.85 · 37: 27.82 · 31: 30.64 · 28: 36.88 · 29: 35.54 · 30: 35.11 |
| prop-3 | 60.90 | 10: 53.82 · 25: 41.27 · 50: 20.96 · 37: 28.10 · 31: 33.05 · 34: 30.24 · 32: 29.44 |
| fixed-3 | 61.46 | 10: 55.23 · 25: 40.58 · 50: 20.72 · 37: 28.12 · 31: 33.26 · 34: 30.59 · 32: 29.87 |
| prop-4 | 60.33 | 10: 53.23 · 25: 40.02 · 50: 20.86 · 37: 27.75 · 31: 33.18 · 34: 30.05 · 32: 31.55 · 33: 31.26 |

Slowdowns are each run's own solo divided by the rung, and the fits cover rungs from 25 upward. The ramps that produced these were pruned from `bench-out/` afterwards, so **this table is the record** — it is here rather than in a scratch script because the analysis it supports is quoted above.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| sessionbench | 0.0.0 at commit `a978904`, clean tree, release build |
| Workload | `cpu-spin --units 100000 --resident 20` with `--duty 0.75` and with `--wait-ms 4` |
| Holds | 60 s per rung, first 20 s unmeasured · resolution 1 |
| Order | The two configurations were interleaved so machine drift would land on both |
| Defender | real-time protection on, no exclusions |

Fits cover rungs from 25 sessions upward, which is where demand first exceeds the core count at this duty. Below that the machine is not saturated and the slowdown is not linear in N.
