# `η` folds two things together, and the footprint moves the larger one

Every `η` in these records is `N·d/(C · slowdown)` with `C` taken as the machine's sixteen logical processors. **The sessions never had sixteen.** Their own samples say how many they held, and it differs by more between runs than the effect being attributed to the footprint.

No run was made. This is arithmetic over three artifacts already on disk.

## What the jobs actually held

| | slowdown | cores the job held | `η` on `C = 16` | `η` on the cores held |
|---|---|---|---|---|
| 20 MiB rested | 2.3010 | **14.11** | 0.7334 | 0.8318 |
| 33 MiB rested | 2.0655 | **15.33** | 0.8170 | 0.8525 |
| 36 MiB rested | 2.0799 | **15.27** | 0.8113 | 0.8503 |

**The 20→33 rise falls from 11.4% to 2.5%** once the divisor is what the sessions received rather than what the machine has. The 33→36 plateau survives either way, at −0.7% and −0.3%.

## The decomposition, and a second route through it

Dividing each hold's total output by the cores it held gives units per core-second — how well the sessions used what they got, with occupancy divided out:

| | total | units per core-second |
|---|---|---|
| 20 MiB | 933.6 units/s | **66.19** |
| 33 MiB | 1054.8 | **68.79** |
| 36 MiB | 1049.9 | **68.77** |

So the footprint buys two separate things:

```
occupancy   14.11 -> 15.33 cores      +8.7%
efficiency  66.19 -> 68.79 units/core-s   +3.9%
product                                  +12.9%
measured slowdown ratio 2.3010/2.0655    +11.4%
```

The product overshoots by 1.3%, which is the two runs' solo rates differing slightly — the slowdown divides by each run's own solo and this arithmetic does not.

**And the two agree on where it stops.** 33 and 36 MiB are 0.03% apart on units per core-second and 0.4% apart on occupancy, which is the plateau seen from a second direction.

## Which of the two is a fact about the workload

**Only the efficiency.** Occupancy is what the machine had left after everything else on it, and all three runs peaked at the same place — 15.66, 15.71 and 15.66 cores. The machine could give 15.7 to any of them; the 20 MiB run averaged 14.11 because it spent more time below its own ceiling.

Why is unmeasured. What is measured is that **`η` as this project computes it is an occupancy term times an efficiency term**, and only the second belongs to the sessions. A governor admitting against `η` is admitting against a number that moves when something unrelated on the machine wakes up.

## What this does not change

**M1's verdict.** Slowdown is measured against each run's own solo baseline; it is not derived from `C`. 2.065 at 33 MiB against a condition of 2 stands, as does the plateau that [ends the footprint lever](2026-08-03-023009-the-footprint-lever-runs-out-before-the-budget-does.md).

**The relation's use as a redline.** `2ηC/d` is fitted from ladders where the same `C` convention applies throughout, so the convention cancels within a fit. It does not cancel across runs, which is where these three sit.

## What this cannot say

- **Whether the occupancy difference is background or workload.** Everything not in the job is unattributed here: the job's CPU is summed from its own members, and `16 − job` is an inference rather than a measurement of the rest.
- **How reliable a sum of 101 per-process CPU readings is.** `sysinfo` reports each process's share of one core; the sum has not been checked against a whole-machine counter.
- **Anything about the slow machine state.** All three runs are rested.
- **Anything past 36 MiB.** The memory budget stops a hundred sessions there.

## Provenance

| | |
|---|---|
| Inputs | `bench-out/1785586144-m1-daemon`, `1785687266-r33-rested-daemon`, `1785689778-r36-rested-daemon` |
| Machine | not used |
| Commit | `5d5fc30` |
