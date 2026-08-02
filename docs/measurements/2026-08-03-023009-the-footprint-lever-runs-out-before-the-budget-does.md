# `η` stops rising at 33 MiB, so the footprint lever ends 3.7% short of the gate

[Forty minutes ago](2026-08-03-014841-the-gate-misses-by-three-percent.md) `η` was measured at 0.733 and 0.817 for sessions holding 20 and 33 MiB, a slope of 0.00643 per MiB, and the gate's passing value of 0.844 was projected to arrive at **37.2 MiB** — a footprint whose hundred sessions need 372 MB more memory than the budget allows. That made RSS the binding condition.

**A third point was predicted before it was run: 36 MiB should give a slowdown of 2.018.** It gives **2.0799**, off by 3.07% and outside the ±1% this instrument supports. `η` there is 0.811 — **below the 33 MiB value, not above it.**

## Three points

| footprint | slowdown | `η` | ceiling `2ηC/d` |
|---|---|---|---|
| 20 MiB | 2.301 | 0.733 | 87 |
| **33 MiB** | **2.0655** | **0.817** | **97** |
| 36 MiB | 2.0799 | 0.811 | 96 |

```
20 -> 33   eta +11.4% over 13 MiB    +0.00643 per MiB
33 -> 36   eta  -0.7% over  3 MiB    -0.00189 per MiB
```

The 0.7% between the last two sits at the edge of what a bracket with ±0.2–0.3% standard errors can separate, so the honest reading is **flat, not falling**. What is not in doubt is that it stopped rising.

**So the projection was wrong about the shape, and the shape is what decided.** Extrapolating a two-point slope four megabytes past its second point was named as the weakest figure in that record; it was weak in the direction that mattered.

## What that changes

**The footprint lever ends before the memory budget does.** The plateau sits near `η = 0.814` and the gate needs 0.844 — **3.7% short**, and no session weight reaches it because weight has stopped buying anything. The 372 MB shortfall computed against the RSS budget describes a footprint that would not have passed anyway.

So the earlier sentence *RSS is the binding condition now, not work rate* is withdrawn. **Work rate binds, at every weight this machine can hold**, and the best it does is 2.065 at 33 MiB against a condition of 2.

**And the gate's own units stop mattering.** The decimal-versus-binary reading of *4GB* was worth 3 MiB of headroom and, on the projected slope, the difference between 2.010 and failing. Measured, 36 MiB is *worse* than 33, so the binary reading buys a footprint nobody would choose. This run used a 4 GiB budget deliberately — RSS came back at 3.944 GiB, 98.6% of it, which the decimal reading would have refused.

## What the run itself says

| | |
|---|---|
| solo before | 21.705, 21.895, 21.911 |
| solo after | 21.761, 21.896, 21.854 |
| **solo gap** | **0.001%** |
| concurrent, 100 sessions | 10.4990 units/s/session |
| peak RSS | 3.944 GiB of 4.000 — **held** |
| `failed_reads` | 0 — **held** |
| work rate | **broke**, 2.0799 |

**The tightest bracket this instrument has produced**, and the third in a row where twenty minutes of saturating a hundred sessions left the machine where it started — the before and after means agree to a thousandth of a percent.

## What this cannot say

- **Three points, one duty, one machine.** 20, 33 and 36 MiB at duty 0.27, all rested.
- **The plateau is two points.** Whether `η` stays flat, falls, or resumes climbing past 36 MiB is unmeasured, and this machine's memory budget cannot hold a hundred sessions heavier than about 36 to find out.
- **Flat and slightly-falling are not separated.** 0.7% against standard errors of 0.2–0.3% is close to the edge; a repeat would settle it and would not change the conclusion, which turns on the rise having stopped.
- **Nothing here explains why.** `η` gained 11.4% between 20 and 33 MiB and nothing between 33 and 36, and a slowdown does not say what changed.
- **The hour is still unmeasured.** Twenty minutes, because [the box stops dead at forty-one](2026-08-01-202112-gate-m1-at-twenty-minutes.md).

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical · 31 GiB · Windows 11 (26200) |
| Power | on mains, SAMSUNG MODE, 100% |
| State | rested — a 40 s solo returned 21.532 units/s immediately beforehand |
| Background | 20% before the run |
| Harness | `sessionbench` 0.0.0 at `7c2377c`, release, `hold --with-solo --solo-repeats 3 --solo-duration 120 --rss-budget-gb 4.294967296` |
| Daemon | `coggyd` 0.0.0, release |
| Shape | 3 solo × 120 s, 100 sessions × 1200 s, 3 solo × 120 s, 30 s uncounted warm-up before each |
| Artifact | `bench-out/1785689778-r36-rested-daemon` |
