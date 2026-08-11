# The tenancy step is on mains and absent on battery

**The same workload, the same box, the same thirty-second holds: on mains a lone session does +60.9% more work per core-second with twelve cores of neighbour than with one, and on battery it does 5.2% less.** The effect this repository has spent two days chasing does not exist unplugged.

## The two sets

Every hold is one `coggyd` session running `cpu-spin --units 100000000 --duty 0.27 --resident 20`, thirty seconds, sampled every five. `rate/job` is the comparison column — `job` is the **achieved** duty, and it falls under load as the session is descheduled, so raw rate would confound how much CPU it got with what it did per core-second.

| power | low tenancy (< 2 cores held) | high tenancy (> 8 cores held) | step |
|---|---|---|---|
| **mains** | **37.23** (n = 4) | **59.90** (n = 7) | **+60.9%** |
| **battery** | 34.41 (n = 3) | 32.62 (n = 4) | **−5.2%** |

The mains rows come from the baseline arms of the [injection experiments](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md), which were never read as a set — eighteen `inject-before` holds accumulated across the afternoon, most of them voided *as pairs* and kept as points. The battery rows are the duty-0.27 arm of [the sweep](2026-08-11-163342-a-solo-baseline-cannot-see-the-plug.md).

The mains curve, in tenancy order, with the injected arms marked:

| rest cores | rate | job | rate/job |
|---|---|---|---|
| 0.75 | 9.574 | 0.26 | 36.15 |
| 0.78 | 9.678 | 0.27 | 36.23 |
| 1.09 | 9.580 | 0.26 | 36.64 |
| 1.43 | 10.573 | 0.27 | 39.90 |
| 2.06 | 19.155 | 0.28 | 69.09 ← *injected* |
| 2.25 | 18.913 | 0.27 | 71.34 ← *injected* |
| 3.26 | 14.195 | 0.27 | 52.38 |
| 12.09–12.89 | 13.748–15.710 | 0.24–0.26 | 54.48–65.82 |

**The three lowest mains holds agree to 1.4%** — 36.15, 36.23, 36.64 — so the low end is not a noisy reading, and the low end is what the battery set matches almost exactly at 34.41.

## Why this matters more than the size of the step

**It explains a null that was about to be treated as a contradiction.** The battery sweep found no tenancy step at either duty across 0.76 to 12.55 cores, which sat awkwardly beside two controlled injections that had just doubled the same session. Those injections are mains. There is no contradiction: the two sets were measuring a box in two different states, and the state is the thing that decides whether the effect is there at all.

**And it puts a mechanism within reach for the first time.** Every named candidate has been eliminated — core clock, uncore, core parking, core placement — and the survivors were wake costs. A power-state dependence is a different kind of clue: whatever the neighbour frees up, the battery power plan is already withholding, so there is nothing left for the neighbour to release. That is a testable shape rather than another absence.

## What this does not establish

- **The two sets are different sittings**, 13:32–14:15 against 15:50–16:31, and [a comparison across sittings varies time as well as the thing you meant](../../CLAUDE.md#treat-the-machine-as-a-variable). What makes this worth stating anyway is that the *direction* differs rather than the magnitude, and a direction usually survives that objection where a size does not.
- **Nobody has run both states inside one window**, which is the measurement that would settle it. It needs someone to change the plug mid-experiment, which nothing here automates.
- **n = 4 and n = 7 on mains, n = 3 and n = 4 on battery.** Small, though the mains low end's 1.4% agreement is tighter than the effect by a factor of forty.
- **The mains high-tenancy holds are all browser**, so this does not separate "a neighbour" from "that particular neighbour" — which is [the open question the injectors were being calibrated for](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md).
- **This says nothing about which power state is "right".** Gate M1's figures are all mains, and this is a reason to keep them there rather than a reason to prefer either.

## A caution about the arithmetic

Four holds in the same directory read `rate/job` in the **millions**: five-second holds from an aborted test, where the startup window `sampler.rs` applies left `median_cores` at essentially zero. They are all battery and were excluded from both means, but they are exactly why [`duty-bands.ps1`](../../sessionbench/scripts/duty-bands.ps1) refuses anything below `job = 0.15` — a ratio's denominator decides its noise, and this one goes to zero.

## Provenance

| | |
|---|---|
| Mains | `bench-out/*inject-before-daemon/`, 2026-08-11 13:00–14:16, `host.on_battery: false` |
| Battery | `bench-out/*quiet-solo-*-d0.27-daemon/`, 15:50–16:31, `host.on_battery: true` |
| Read from | `hold.json` — `units_per_session_per_sec`, `occupancy.median_cores`, `occupancy.rest_cores_median`, `host.on_battery` |
| Machine | 16 logical / 31 GiB / Windows 11 |
