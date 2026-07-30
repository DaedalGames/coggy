# The duty relation is derivable · 2026-07-30

[The duty record](2026-07-30-duty-and-redline.md) holds two numbers it never connects: `redline × duty ≈ 25`, and sessions receiving 84% of a proportional core share. Connecting them turns a constant fitted to three points into a formula — and a formula carries to hardware this machine cannot speak for, which is what [G0](../../ROADMAP.md#current-priority-m0--attribution) is waiting on.

## The derivation

A session that computes a fraction `d` of its wall time needs `d` cores to run at solo speed. `N` of them demand `N·d` against `C` cores. Past saturation each receives `C/N`, so its rate falls to `η·C/(N·d)` of solo, where `η` is whatever is lost to running alongside others rather than alone.

    slowdown = N·d / (η·C)

The [WorkRate condition](../../sessionbench/README.md#redline) allows a 2× slowdown, so

    redline = 2·η·C / d

The constant 25 was `2·η·C` all along. What it hides is `C`, and a machine with twice the cores has twice the constant.

## The prediction, made before the run

`η = 0.84` and `C = 16` give `26.9/d`. Against the three existing points that read 7–11% high, so the corrected estimate was `24.7/d`:

- **control at duty 1.00 → [24, 27]**, to show that a 60 s hold reproduces the 25 taken at 20 s
- **test at duty 0.75 → [31, 36]**, a new point between two existing ones

Both landed. Both landed on the floor of their interval, which is the signature of a biased constant rather than a lucky one — and the bias is named below.

| | Predicted | Redline |
|---|---|---|
| duty 1.00 | [24, 27] | **24** |
| duty 0.75 | [31, 36] | **31** |

## What actually confirms it

Not the redlines. Solving the relation for `η` at each ramp's own redline:

| Ramp | Sessions | Slowdown | `η = N·d / (slowdown·C)` |
|---|---|---|---|
| duty 1.00 | 24 | 1.95× | **0.771** |
| duty 0.75 | 31 | 1.89× | **0.768** |

**Two independent ramps, different duties, different session counts, and the same constant to within 0.4%.** Across every saturated rung in both runs — seventeen sessions to fifty — `η` stays between 0.73 and 0.80, mean 0.77.

That is the difference between a formula and a curve through three points. `redline × duty ≈ 25` describes the points it was drawn from; `slowdown = N·d/(η·C)` predicts a rung nobody had measured.

## Against every duty measured so far

With `η = 0.77`, the formula gives `24.6/d`:

| Duty | Predicted | Observed | Ratio |
|---|---|---|---|
| 1.00 | 24.6 | 24 | 0.98 |
| 0.75 | 32.9 | 31 | 0.94 |
| 0.50 | 49.3 | 48 | 0.97 |
| 0.25 | 98.6 | 100 | 1.02 |

Within 6% across a fourfold range, reading about 2% high on average. The 0.50 and 0.25 rows come from the earlier 15 s-hold runs and are not held to the same standard as the two taken here.

## Why the earlier 84% was wrong, and why it mattered

It was measured on a hold whose spin-up was too short to clear session startup — [the earlier record says so about its own core figures](2026-07-30-duty-and-redline.md). At 60 s the figure is 0.77. Seven points of `η` is 9% of the redline, and it is the whole of why both predictions here came in at the floor rather than the middle.

**Which is the argument for the control ramp.** Without it, `24` at duty 1.00 against the earlier `25` would have read as drift, and there would have been no way to tell a changed relation from a changed hold.

## `η` is memory, not scheduling

The earlier record called cache pressure and scheduling the obvious suspects and measured neither. These runs separate them, using the rungs where cores are **not** yet saturated:

| Ramp | Sessions | Cores demanded | Cores received | Rate against solo |
|---|---|---|---|---|
| duty 1.00 | 10 | 10.0 of 16 | 9.9 | **0.80** |
| duty 0.75 | 10 | 7.5 of 16 | 7.5 | **0.88** |

Both got every core they asked for. Both ran slower anyway. **With six spare cores there is no scheduling contention to blame**, so the loss is what ten sessions do to each other through the memory system — 240 MiB of resident working set against a cache that holds a fraction of it. The loss is larger at duty 1.00, which touches memory more often per second, exactly as a bandwidth account predicts.

The unsaturated 0.80 is close to the saturated 0.77, so `η` is not a threshold effect that switches on at the redline. It is present throughout and merely becomes visible when cores run out.

**So `η` belongs to the workload as much as the machine**, and cannot be carried to a real session by assumption. It can be carried cheaply by measurement: one solo run and one saturated rung give it, without a ladder.

## What this changes for G0

The gate needs a redline for a session this machine cannot run. It now needs **two scalars and one rung**:

1. `d`, the fraction of wall time a real session computes — `observe` reports it.
2. `η`, from one rung held past saturation.
3. `redline = 2ηC/d`, checked against a ladder rather than trusted in place of one.

`C` being in the formula is the part that travels. The configured machine will not have sixteen cores, and `25/d` would have been wrong there by whatever ratio its core count differs by.

## What is still assumed

- ~~**Both ramps waited by sleeping proportionally.**~~ **Run, and it cancels.** `cpu-spin --wait-ms 4` gives the same solo duty with a pause that does not stretch under load, which is the shape of a session waiting on a model. Matched against the proportional ramp its rungs agree within 1.6% at every session count — 1.50× against 1.50× at 25, 1.99× against 2.01× at 34 — so the waiting mechanism does not move the curve. The two configurations' redlines came out 31 and 34, which looked like a difference until [three runs of each interleaved completely](2026-07-30-redline-reproducibility.md).
- **`η` was measured on one working-set size.** 20 MiB per session is a knob, and a session that fits in cache should show `η` nearer 1.
- **Memory never became the limiting condition.** At 100 sessions and quarter duty the ceiling was still work rate.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| sessionbench | 0.0.0 at commit `776b128`, clean tree, release build |
| Workload | `cpu-spin --units 100000 --resident 20 --duty {1.0, 0.75}` |
| Holds | 60 s per rung, first 20 s unmeasured · resolution 1 |
| Defender | real-time protection on, no exclusions |

Rung-level noise is about 2.5%: at duty 0.75, 32 sessions measured 2.06× while 34 measured 2.01×, which is out of order and is why the ladder stopped at 31 rather than 32.
