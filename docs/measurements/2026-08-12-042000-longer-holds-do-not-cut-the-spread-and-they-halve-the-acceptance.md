# Longer holds do not cut the spread, and they halve the acceptance

**The lever named three hours earlier is not a lever. Quadrupling the hold left the spread no smaller — `sd 7.61` at 30 seconds against `12.81` at 120 — while the acceptance rate fell from 7 of 8 to 4 of 8, so a usable pair costs seven times the wall clock and buys nothing.** The remaining lever is a quiet machine, which this box does not supply on request.

## What was being priced

[The night's closing record](2026-08-12-025147-wake-shape-adds-nothing-once-the-tenancy-change-is-controlled.md) ends with a cost: within-arm `sd` near 20 points, so resolving a 10-point arm difference needs about 100 pairs a side, and **the lever is longer holds or a quiet machine, not more pairs**. That sentence prices or blocks every remaining injector contrast. It was never measured — every pair on disk was 30 seconds.

Variance was expected to fall with duration because the spread's dominant source is the tenant moving *during* a hold, and a longer hold integrates over that motion instead of being spoiled by it. That is the same reasoning that prefers a two-minute solo hold to four point readings of `doctor`.

## The set

Eight rounds, each a 30-second pair then a 120-second pair, **interleaved** so the arms cannot differ by sitting. Six `cpu-spin --duty 0.27 --resident 1` co-tenants, measured session unchanged, `-AnyBaseline`. 02:59–04:11, mains throughout, confirmed by a power watch running the whole span.

| arm | accepted | mean | sd | variance | range |
|---|---|---|---|---|---|
| 30 s | **7 / 8** | −4.23% | **7.61** | 57.98 | −13.2 .. +11.0 |
| 120 s | **4 / 8** | −8.97% | **12.81** | 164.00 | −26.5 .. +2.1 |

Variance ratio long/short is **2.83**, in the wrong direction. At the observed degrees of freedom (6 and 3) the `F(3,6)` 95% critical value is 4.76, so the long arm cannot be called *significantly* noisier — but a 2.5× `sd` reduction predicts a long-arm variance of 9.28 against 164.00 observed, a factor of 17.7, and that is refuted outright.

## The power claim does not apply, and that was pre-registered

The design fixed `n = 8` a side and `F(7,7) = 3.79` before the run, with the condition written down at 03:12: *if the long arm's accepted `n` falls materially below 8, `F(7,7)` is the wrong critical value and the honest output is the observed `n` and a wider bound.* It fell to 4. So the primary comparison is reported at its real degrees of freedom and no threshold was re-derived after the fact.

## Why the failures are the result rather than an obstacle

The four missing long pairs did not void out. **Every one died on `deadline reached`** — four minutes elapsed without the tenant holding still long enough for two consecutive 120-second holds. The one missing short pair died differently, on `gave up after 3 voids`.

That is the secondary result the design added after run 1: a longer hold gives the tenant four times the window to move in, so the arm that was supposed to be *cheaper per unit of precision* is dearer per unit of anything.

| | 30 s | 120 s |
|---|---|---|
| hold time per attempt | 60 s | 240 s |
| acceptance | 87.5% | 50.0% |
| **wall clock per accepted pair** | ~69 s of holding | ~480 s of holding — **7.0×** |

## The censoring bias runs against the finding, which strengthens it

The delta guard fires exactly when tenancy moved, and achieved tenancy delta is [the variable that predicts rate change](2026-08-12-025147-wake-shape-adds-nothing-once-the-tenancy-change-is-controlled.md). So accepted pairs are filtered *on the independent variable*, and the more heavily filtered arm should appear **tighter**.

The 120-second arm is the more heavily filtered one — half its attempts discarded against one in eight — and it is wider anyway. Whatever the true long-arm spread is, it is at least this.

## Two pairs on the rising limb went the wrong way

Rounds 6 and 7 caught a rare quiet window: `rest_cores_median` fell to 1.02–1.22 before the injection, which is the **rising limb** below the ~1.4-core transition where the controlled injections recorded **+95.4%** and **+100.1%**.

| arm | rest before → after | rate | change |
|---|---|---|---|
| 30 s | 1.22 → 2.73 | 8.552 → 7.954 | **−7.0%** |
| 120 s | 1.02 → 2.47 | 9.440 → 6.940 | **−26.5%** |

Both negative, on the limb where the step is supposed to be large and positive. This is **two pairs** and the night established that a single pair has no sign — six identical injections spanned 49 points. It is recorded because it is cheap to record and expensive to rediscover, not because it overturns anything.

## What this does not establish

- **The long arm rests on four pairs.** Its `sd` of 12.81 has a 95% interval running roughly 7.3 to 47.7, so the finding is *no reduction*, never *an increase*.
- **`sd` is a fact about the sitting.** The 30-second arm here reads 7.61 against the ~20 quoted from earlier sittings at the same duration. Both are 30-second spreads of the same injection; the machine differed.
- **The machine moved a long way inside the series.** `doctor` read 8.79 of 16 cores held at the start and 12.40 at the end, and the accepted pairs sampled three states — around 8 cores, a brief window near 1, then 11–13. Interleaving protects the arm contrast; nothing protects the absolute levels, and the arm means are not comparable with any other sitting's.
- **Only two durations.** Nothing here says a 300-second hold behaves like the 120-second one, only that the first quadrupling bought nothing while costing sevenfold.
- **No mechanism for why the spread does not integrate down.** If the tenant's motion were fast relative to the hold, a longer window would average it; that it does not suggests the motion is slow — excursions lasting minutes rather than seconds — which is consistent with the ±10-core arrival and departure the last two runs caught.

## What it changes

The `~100 pairs an arm` price stands, and the escape named beside it does not exist. **The lever is a quiet machine.** [#78](../../ROADMAP.md) is therefore blocked on quiet rather than on pair count, and any contrast needing better than ~8 points of resolution on this box in this state should not be attempted by adding pairs or by lengthening holds.

## Provenance

| | |
|---|---|
| Series | `dur-spread.ps1`, eight interleaved rounds, 02:59:33–04:11 |
| Each pair | `inject-tenant.ps1 -Tenants 6 -ExpectedPerTenant 0.22 -AnyBaseline -QuietBelow 99 -QuietMachineBelow 99 -GiveUpMinutes 4 -MaxVoids 3`, `-Duration` 30 or 120 |
| Read from | each run's own `RESULT` line and its `gave up` / `deadline reached` line, arm matched by the `around a N s hold` line in each transcript rather than by log order |
| Artifacts | `bench-out/inject-20260812-0259*.log` through `-0405*.log`, 16 transcripts |
| Machine | 16 logical / 31 GiB / Windows 11, **mains** throughout, 44.1 °C at both ends, 0 survivors after teardown |
