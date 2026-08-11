# Wake shape adds nothing once the tenancy change is controlled

**Twice, in two series with different confounds, a large apparent difference between two neighbour shapes collapsed to nothing when what the machine's tenancy actually did was accounted for: 8.6 points → 1.5, and 25.2 points → 5.3, against within-arm spreads of 13 to 23.** What separates the arms is how much tenancy changed, never which workload caused it.

## What was compared

`cpu-spin --duty 0.27` against `cpu-spin --wait-ms N` as co-tenants. Same binary, same work loop, same cost model, **different wake cadence**: `--duty` stretches its pause to match a slower unit so its rhythm tracks the machine, `--wait-ms` sleeps a fixed span so its rhythm is fixed and its duty drifts. The flag's own documentation names the pairing.

This is [the behavioural-resonance question](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md) the copied-binary control left open: a different *file* removed shared code pages, and left the neighbour's rhythm identical to the measured session's.

Four rounds per series, each round a duty pair then a wait pair, **interleaved** so the arms cannot differ by sitting.

## Two series, two confounds, one answer

| series | raw arm gap | confound found | after correcting |
|---|---|---|---|
| 1 (`--wait-ms 76`, 8 pairs) | 8.6 pts | duty delivered **53% more load** (1.508 vs 0.987 cores) | **1.5 pts** |
| 3 (`--wait-ms 46`, 8 pairs) | 25.2 pts | wait moved tenancy **3.4× further** (delta 1.72 vs 0.50) | **5.3 pts** |

Series 1's arms delivered nearly disjoint loads — duty 1.386–1.636, wait 0.640–1.443 — so treatment and covariate were collinear and the correction was extrapolation. Re-sizing to `--wait-ms 46` fixed that: series 3's loads overlap at duty 0.650–0.962 against wait 0.878–1.396, which is what makes its correction honest.

Both residual gaps sit well inside the within-arm spreads (series 1: sd 18.7 and 5.8; series 3: sd 23.4 and 12.9).

## The variable that does predict

In series 3, rate change against **achieved tenancy delta** gives `r = +0.633` at **+16.3% per core** — a better predictor than delivered co-tenant load (`r = +0.416`).

That figure agrees closely with the **+19.7% per core** from [the six-pair set](2026-08-12-005518-the-scatter-was-the-step-measured-six-times.md) whose correlation was withdrawn hours earlier. **The withdrawal was of the coefficient's strength, not the relationship's magnitude** — `r` fell from +0.771 to +0.25 on replication, but the slope reappears here at a comparable value from an independent set. Neither is strong enough to build on; both point the same way.

## Neither arm has a stable sign

The same command at the same tenancy produced, across the night at rest 8.0–9.4 cores:

- duty: −27.4%, −21.1%, −6.9%, −5.5%, +0.1%, +17.4%, +42.4%
- wait: −17.4%, −8.5%, −7.9%, +17.6%, +28.4%, +36.8%, +48.0%

**A 70-point range at matched tenancy.** And the arms swap character between series — wait was the tight arm in series 1 (−1.7 to −7.9) and the tight arm in series 3 was neither, with duty scattering 48 points inside one window. Two series assigning "tight" and "scattered" to opposite workloads is the tell that neither pattern belongs to the treatment.

## What this costs the next attempt

With a within-arm standard deviation near 20 points, resolving a 10-point arm difference at conventional confidence needs roughly **100 pairs per arm** — about four hours an arm at this box's throughput, and that assumes the tenant holds still, which it does not.

> **2026-08-12 04:20: half of this was measured and is wrong.** Quadrupling the hold does **not** cut the spread — `sd 7.61` at 30 seconds against `12.81` at 120, with acceptance falling 7-of-8 to 4-of-8 for **7.0x the wall clock per usable pair**. [The measurement is here](2026-08-12-042000-longer-holds-do-not-cut-the-spread-and-they-halve-the-acceptance.md). The ~100-pair price below stands; of the two escapes named beside it, only **a quiet machine** survives.

**So the lever is not more pairs.** Variance falls with hold duration, and every pair here is 30 seconds. A quiet machine would remove the confound at its source: both corrections exist only because the browser moved during the holds.

## What this does not establish

- **No mechanism.** Core clock, uncore, parking and placement remain eliminated, and wake shape now joins them as unsupported rather than refuted — the design lacked the power to detect a small effect.
- **Absolute figures do not transfer between series.** Series 1's duty arm averaged −22.8%, series 3's +7.5%, at the same tenancy. The sitting moves the whole distribution; only the interleaved arm contrast is protected.
- **The `--wait-ms` confound is real.** Its duty climbs as the machine slows, so the arms cannot be held to equal delivered load across a hold by any fixed sizing — series 1 and series 3 needed different `N` and still diverged.
- **Eight pairs a side, one night, one box, mains.**

## Provenance

| | |
|---|---|
| Series 1 | `wake-shape.ps1`, `--wait-ms 76`, four interleaved rounds, 01:24–01:53 |
| Series 3 | same, `--wait-ms 46`, `-ExpectedPerTenant 0.25`, 02:22–02:49 |
| Each pair | `inject-tenant.ps1 -Tenants 6 -AnyBaseline -QuietBelow 99 -QuietMachineBelow 99 -Duration 30` |
| Read from | each run's `RESULT` line and its `co-tenants hold N cores` assertion, both computed from `hold.json` |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, 0 survivors after teardown |
