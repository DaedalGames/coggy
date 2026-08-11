# The scatter was the step, measured six times

**Six identical injections gave rate changes from −26.5% to +22.7%, and the 49-point spread is not noise: it tracks how much tenancy actually landed on the box, `r = +0.771`, at +19.7% per core.** The injection was constant; the browser was not; and the browser's wandering was the independent variable all along.

## The set

[`inject-tenant.ps1`](../../sessionbench/scripts/inject-tenant.ps1) with `-AnyBaseline`, run six times with one command between 00:11 and 00:53 on 2026-08-12, mains. Six `cpu-spin --duty 0.27 --resident 1` co-tenants around a 30-second hold; the measured session unchanged at `cpu-spin --duty 0.27 --resident 20`. Baselines all within 8.17–8.88 cores held.

| baseline rest | tenanted rest | delta | rate | change |
|---|---|---|---|---|
| 8.88 | 8.53 | **−0.35** | 3.910 → 3.251 | −16.9% |
| 8.23 | 8.54 | 0.31 | 3.723 → 3.063 | −17.7% |
| 8.17 | 8.75 | 0.58 | 3.766 → 3.719 | −1.2% |
| 8.28 | 9.18 | 0.90 | 5.320 → 3.912 | **−26.5%** |
| 8.34 | 10.01 | 1.67 | 3.546 → 4.352 | +22.7% |
| 8.24 | 10.00 | **1.76** | 3.554 → 4.271 | **+20.2%** |

Sorted by delta, the sign flips exactly once, and the two positives are the two largest deltas.

| | r | slope |
|---|---|---|
| change vs **achieved delta** | **+0.771** | **+19.7% per core** |
| change vs tenanted arm's absolute tenancy | +0.809 | +24.4% per core |

## What this corrects

**The first of these pairs was written up three hours earlier as +22.7% with the caveat that [its per-core-second ratio was unreadable](2026-08-12-001842-the-collapse-limb-starves-the-session-below-a-readable-ratio.md).** That caveat was right and insufficient: the raw rate was treated as solid, and it is one draw from a distribution whose mean is −3.2%. A single pair at this tenancy has no sign.

**Then the spread itself was nearly written off.** With five pairs in hand the reading was "the design cannot resolve a step, the noise is ±25 points". The sixth pair arrived with delta 1.76 and +20.2%, and the ordering became visible. Five points supported *the method fails*; six supported *the method measures something else than intended*.

## The variable was never the injection

Every run injected the same six co-tenants, verified holding 0.847–1.509 cores of their own. What differed between runs is **how much tenancy the box ended up carrying**, because the browser moved during the holds — sometimes arriving, once leaving.

So the delta guard was treating the independent variable as a validity check. It exists to prove the injection is what changed, and on a loaded box that job splits in two: the co-tenants' own CPU proves they ran, and nothing available proves the browser held still. Rather than demanding each pair's delta match the injection, **the analysis regresses rate change on achieved delta across many pairs** — which turns the browser's wandering from contamination into the thing being varied.

## Direction agrees with the mains archive by a second route

[Eighteen `inject-before` baselines](2026-08-11-173433-the-tenancy-step-is-on-mains-and-absent-on-battery.md) gave +60.9% per core-second from below 2 cores to above 8, comparing holds taken across an afternoon. These six pairs give +19.7% per core measured **inside single windows**, at 8–10 cores.

The two are not the same quantity — one is per core-second across a wide tenancy range, the other raw rate across a narrow one — and they are not independent of each other, since both are this box on mains. What they share is the sign, which the battery sweep did not have.

## What this does not establish

- **Six pairs, one sitting, one tenancy band.** The slope is fitted across 8.17–10.01 cores held and says nothing outside it.
- **`r = +0.771` on n = 6** is one point away from much weaker. The ordering is the more robust claim than the coefficient.
- **The mechanism is untouched.** Core clock, uncore, parking and placement remain eliminated; nothing here names a cause.
- **Nothing about a differently-shaped neighbour.** The `file-write` arm still has no pair, and this set says why it was premature: comparing two injectors on single pairs would have compared two draws from a 49-point distribution.
- **The baselines scatter too** — 3.546 to 5.320 units/s at nearly identical tenancy, one 40% above the rest — so both arms of every ratio are unstable on the collapse limb.

## Provenance

| | |
|---|---|
| Runs | `inject-tenant.ps1 -Tenants 6 -ExpectedPerTenant 0.16 -AnyBaseline -QuietBelow 99 -QuietMachineBelow 99 -Duration 30`, ×6, 00:11–00:53 |
| Artifacts | `bench-out/*inject-before-daemon/`, `bench-out/*inject-after-daemon/`; transcripts `bench-out/inject-20260812-*.log` |
| Read from | the runs' own `RESULT` lines, each computed from `hold.json`'s `units_per_session_per_sec` and `occupancy.rest_cores_median` |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, 0 survivors after teardown |
