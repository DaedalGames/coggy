# A neighbour costs the solo baseline 27%, and it is not core starvation

Nine one-session holds of the same workload, taken across one afternoon while a third party came and went, each recording the cores held outside its own job. The baseline the work-rate condition divides by moves **27%** with a neighbour present, and the size of the neighbour barely matters.

## The nine holds

`hold --sessions 1 --interval 5 --duration 120 -- cpu-spin --units 100000000 --duty 0.27 --resident 20`

| cores held elsewhere | rate | run |
|---|---|---|
| 1.24 | **20.295** | `fp3-1` |
| 2.99 | **17.561** | `fingerprint-2` |
| 10.47 | 13.418 | `fp3-3` |
| 11.46 | 14.543 | `fingerprint-1` |
| 11.50 | 13.336 | `fp2-3` |
| 11.60 | 13.002 | `fingerprint-3` |
| 12.34 | 13.449 | `fp2-1` |
| 13.20 | 13.085 | `fp2-2` |
| 13.40 | **15.709** | `fp3-2` |

| | n | mean | spread |
|---|---|---|---|
| under 4 cores held | 2 | **18.928** | 14.4% |
| over 9 cores held | 7 | **13.792** | 19.6% |

**27.1% between them.**

## It is presence, not size

The most crowded hold in the set — 13.40 cores held — ran **15.709**, faster than one at 10.47 cores. Across the seven crowded holds, three cores of extra tenancy buy no consistent slowdown. Whatever the cost is, it arrives with the neighbour and then stops scaling.

## And it is not the cores running out

At 13.40 cores held, **2.6 are free and the session needs 0.27**. A workload at duty 0.27 has ten times the core it asks for and still runs 23% slower than the same workload on a quiet box. Core count does not explain it, so the loss travels through what a core count does not measure — the same memory system `η` was introduced for.

This is the sharper form of something already known in the other direction: a hundred sessions cost each other through contention rather than through arithmetic. Here one session pays it against a stranger.

## Why this matters more than it looks

The gate's work-rate condition is `slowdown = solo ÷ concurrent ≤ 2`. The numerator is what these nine holds measure. **A neighbour during the baselines lowers it 27%, which lowers the slowdown, which makes the gate look closer to passing.** Nothing in the bracket sees it: [three baselines under a tenant agreed to 2.7% against a 5% allowance](2026-08-03-073401-three-baselines-agreed-to-under-three-percent-and-were-all-wrong.md).

Every hold now prints the figure, which is what makes this readable at all — and what makes the nine points here recoverable from artifacts taken for other reasons.

## What this cannot say

- **The quiet group is two holds.** 20.295 and 17.561, spreading 14.4%. That is a direction with an error bar wider than most things measured here, not a baseline. This box's rested figure of about 21.5 is two days old and today's quietest hold sits 5.6% under it, which is inside that spread and settles nothing.
- **A median smooths a burst.** Each row is the median rest over 120 seconds, so a hold whose neighbour came and went reads as moderately crowded throughout rather than as two states. `fp3-2` at 13.40 cores and 15.709 units/s is the row that most looks like it.
- **Nothing names the neighbour.** It held 7.6 to 13.4 cores in bursts of minutes across the afternoon, and `Get-Counter '\Process(*)\% Processor Time'` named `chrome-headless-shell` once, hours before most of these runs.
- **Whether the shape is a step or a curve.** Two points below four cores and seven above nine leaves the middle unsampled, and the middle is where a step and a curve differ.

## Provenance

| | |
|---|---|
| Inputs | `bench-out/` — `1785707434/1785707593/1785707752-fingerprint-*`, `1785709115/1785709273/1785709432-fp2-*`, and the three `fp3-*` holds |
| Machine | on mains; `doctor` read 1.17 to 15.80 of 16 cores across the afternoon |
| Commit | `5b817a0` |

## The three fingerprint attempts that produced this

None of the three sets was the measurement it was launched as. The first spread 30% and was read as a possible machine state until the column said tenant; the second agreed to 2.7% and was wrong; the third caught one quiet hold and lost two. **The set that answers a question here is the union of three runs that each failed at their own**, which was only assemblable because every hold records what was beside it rather than only what it did.
