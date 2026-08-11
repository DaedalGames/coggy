# Adding a neighbour doubles a lone session

**Injecting 1.47 cores of load onto an otherwise quiet box took the same session from 9.678 to 18.913 units/s — +95.4% — while its own CPU stayed at 0.27 cores.** Same CPU, ninety-five percent more work.

> **2026-08-11 14:19: the clock half of this sentence is withdrawn.** It read "and its core clock stayed at 205.1% against 203.7% … same CPU, same clock". That column is a single point sample taken at the *end* of a hold and must not be used this way; see [the section below](#the-clock-reading-in-this-record-is-not-evidence) and [the copied-binary control](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md). The CPU figure and the result stand.

## What was run

[`inject-tenant.ps1`](../../sessionbench/scripts/inject-tenant.ps1), 2026-08-11 13:32. It waits for quiet, holds 30 s for a baseline, starts six `cpu-spin --duty 0.27 --resident 1` co-tenants, holds 30 s again, and stops them. One `coggyd` session per hold running `cpu-spin --units 100000000 --duty 0.27 --resident 20`. Mains.

| | rate | rest cores | job cores | max-core clock | units |
|---|---|---|---|---|---|
| before | 9.678 | 0.78 | 0.27 | 205.1% | 293 |
| after | **18.913** | 2.25 | 0.27 | 203.7% | 571 |

**+95.4%, with tenancy moving 1.47 cores** — inside the band six spinners are measured to add.

## Why this is causal where everything before it was not

Every low-tenancy hold in this repository got that way because [the browser left](2026-08-11-103621-the-step-is-at-one-and-a-half-cores.md), never because anything was added. So the +10% to +49% step measured across four sittings could not distinguish a neighbour *causing* a faster session from a neighbour *coinciding* with whatever else makes one. Here the load is ours, its arrival is the only thing that changed, and the change is measured rather than assumed.

Three guards had to pass, all written before this run and each after a failure it prevents:

- **The baseline must be below the transition.** 0.78 cores. An earlier attempt reported a plausible −9.6% from a baseline carrying 12.09 cores of browser, and only the artifacts revealed both arms sat in the collapse region.
- **The injection must be what changed.** Delta 1.47 against an expected ~1.6. Another attempt reported a believable +62.2% where the delta was **11.42** — the browser arriving during the second hold, not the spinners.
- **A rate that cannot be parsed voids rather than defaulting.** A zero would read as the slowest possible machine.

## The job did nothing differently

`median_cores` reads **0.27 in both arms**. The session was not given more CPU, was not starved, and was not descheduled. What changed is how much work a core-second buys, and it changed by a factor of nearly two across 1.47 cores of somebody else's load on a sixteen-core machine.

## The clock reading in this record is not evidence

> **Appended 2026-08-11 14:19.** The paragraph below is withdrawn. It is kept because the [copied-binary control](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md) made the *opposite* error with the same column, and the pair is the point.

**The withdrawal.** `host.processor_performance_cores` is [queried once, at the *end* of a hold](../../sessionbench/src/host.rs), and its own documentation says: *"One point sample per report, and it is a poor summary of a hold … Do not derive a rate, a ratio or a normalisation from this column."* Two point samples cannot refute anything, and the correlation this paragraph called weaker averages twenty of them. The clock is still eliminated — on **r = +0.096**, which is where it always rested. What was wrong is the sentence claiming a better source for it.

The control an hour later read 100.8% before against 201.9% after and nearly wrote the doubling up as the mechanism. **Two baselines whose rates differ by 1.1% carry clocks of 205.1% and 100.8%**, which settles the column's usefulness here from both ends at once.

**The withdrawn paragraph:**

> **And the clock is eliminated for this transition outright**: 205.1% before against 203.7% after, on the per-core maximum. Not a small difference — none at all, in the direction of slightly lower. That is a stronger statement than [the r = +0.096 across twenty holds](2026-08-11-103621-the-step-is-at-one-and-a-half-cores.md), because it is one session, one minute, one change.

## The within-hold contradiction was not real, and neither was the reconciliation

> **Corrected within the hour.** The section below explains a disagreement between this result and a within-hold measure, via browser ramp transients. **There was no disagreement**: the within-hold measure is invalid. Per-tick deltas in a **daemon-backed** hold's `concurrent-samples.jsonl` alternate between zero and ~28.5 — `28.53, 0.00, 0.00, 28.72, 0.00, 29.31, 0.00, 28.53…` — because [`DaemonWatch::units`](../../sessionbench/src/daemon.rs) returns the *latest report's* count and `coggyd`'s `REPORT_EVERY` is 10 s against a 5 s sampling interval. **This is `--daemon`-only**: a directly-run session owns a live atomic counter fed by its stdout drain, so an `observe` run's samples have genuine per-sample resolution. Every hold analysed here is daemon-backed. A per-tick "rate" measures whether a burst landed inside that interval. **Only the hold-level average over `counted_ms` is a rate.** The −37.7%, a +9.6% "drift" between halves, and a +30%/−21.6% "hump" across thirds were all the same artifact, and the hump's near-perfect symmetry (first third ≈ last third) was the tell. This result stands unopposed rather than reconciled, and the paragraphs below are kept as the reasoning that was in play.

A within-hold measure taken an hour earlier said the opposite: across 36 holds where tenancy crossed 1.4 cores mid-hold, the rate **fell** — median −17.4%, and −37.7% restricted to the cleanest arms, worse still after correcting for a **+9.6%** first-half-to-second-half drift measured on six constant-tenancy holds.

Those ticks were binned at 1.4–4 cores, but **the browser does not sit at 1.4–4 — it ramps through it** on the way to eleven. So they captured arrival transients, not a steady 1.4–4 core machine. This injection holds 1.47 cores steadily for thirty seconds, which is a different condition.

So both stand: a **steady** one-and-a-half cores of neighbour is worth a large speedup, and a browser **ramping** to eleven and staying there is the collapse limb, where a hundred sessions also live at `r = −0.950`.

## What this does not establish

- **n = 1.** One valid pair, after two voids and several aborted attempts.
- **The co-tenants are `cpu-spin`, the same binary as the measured session.** Six copies share code pages, and a warm instruction cache could help in a way a browser never would. **A repeat with `file-write` or `stdout-storm` is required before this means "a neighbour" rather than "six copies of myself"** — this was written into the reading rules before the run, and it is the single most important follow-up.

  > **Answered in part, 2026-08-11 14:15.** [Injecting a byte-identical *copy* at a different path gave +100.1%](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md) against this run's +95.4%, at baselines agreeing to 1.1%. A different file is a different section object, so **shared pages and a warm instruction cache are out.** The behavioural half is untouched — the co-tenants are still `cpu-spin --duty 0.27`, waking on the same cadence — so the follow-up stands, narrowed to needing a workload of a *different shape* rather than merely a different path.
- **The injected load is bursty**, duty 0.27, where the browser is steady.
- **One duty.** All 130 solo mains holds on disk are duty 0.27, so this remains a statement about a *sleeping* workload — one that wakes, works 27% of the time, and sleeps. Nothing here separates running from waking, which is where the untested mechanisms live.
- **The size exceeds every correlational estimate** (+10 to +49%), which is unexplained and could mean the injection differs from the browser in more than magnitude.
- **No mechanism.** Core clock, uncore, core parking and core placement are all eliminated. Nothing named remains.

## Provenance

| | |
|---|---|
| Run | `inject-tenant.ps1 -Tenants 6 -Duration 30`, 2026-08-11 13:32, after 0 voids in this attempt |
| Artifacts | `bench-out/*inject-before*/`, `bench-out/*inject-after*/`, transcript `bench-out/inject-*.log` |
| Read from | `hold.json` — `units_per_session_per_sec`, `occupancy.rest_cores_median`, `occupancy.median_cores`, `host.processor_performance_cores` |
| Machine | 16 logical / 31 GiB / Windows 11, mains |
