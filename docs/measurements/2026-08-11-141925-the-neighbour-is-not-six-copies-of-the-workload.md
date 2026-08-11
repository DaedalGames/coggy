# The neighbour is not six copies of the workload

**Injecting 1.31 cores from a byte-identical copy of the workload, living at a different path, took a lone session from 9.574 to 19.155 units/s — +100.1% — reproducing the +95.4% measured an hour earlier with the same binary.** Shared code pages are out.

## What was run

[`inject-tenant.ps1`](../../sessionbench/scripts/inject-tenant.ps1) with `-Injector target\release\cpu-spin-b.exe`, 2026-08-11 14:15, after two voids. `cpu-spin-b.exe` is a byte-identical copy of `cpu-spin.exe` at a different path. Windows creates a section object per *file*, so two different files cannot share physical pages however identical their bytes — the copy removes page sharing and the warm instruction cache with it, and changes nothing else about the load.

| | rate | rest cores | job cores | units | counted |
|---|---|---|---|---|---|
| before | 9.574 | 0.75 | 0.265 | 289 | 30185 ms |
| after | **19.155** | 2.06 | 0.277 | 578 | 30174 ms |

## The pair reproduces

| run | injector | before | after | step | tenancy delta |
|---|---|---|---|---|---|
| 13:32 | `cpu-spin` — the session's own image | 9.678 | 18.913 | +95.4% | 1.47 |
| 14:15 | `cpu-spin-b` — a copy | 9.574 | 19.155 | **+100.1%** | 1.31 |

The two baselines agree to **1.1%** at near-identical tenancy, 0.78 against 0.75 cores, taken forty-five minutes apart. [CLAUDE.md's four scales](../../CLAUDE.md#treat-the-machine-as-a-variable) put one run against another in the same state at up to 14%, so the pair is comparable and the two steps sit 4.7 points apart — well inside it.

This was the single most important follow-up the [first causal record](2026-08-11-133424-adding-a-neighbour-doubles-a-lone-session.md) named, and it was named before the run rather than after.

## What the copy eliminates, and what it leaves standing

**Eliminated: shared code pages and a warm instruction cache.** Six copies of the measured session's own image could plausibly have helped it in a way no browser ever would. They cannot, because a different file is a different section object, and the step is the same size.

**Not eliminated: the co-tenants are still `cpu-spin --duty 0.27`.** Same wake period, same work loop, same duty, same resident set. A *behavioural* resonance — six neighbours waking on the same cadence as the measured session — is untouched by copying the file. The copy changes the pages, not the shape.

So the follow-up survives, narrowed: **it needs `file-write` or `stdout-storm`**, a workload with a different shape, not merely a different path. Until then this says "not six copies of my image" rather than "a neighbour".

## The clock columns cannot be read here, and the earlier record read them

The `after` arm's snapshot reads 201.9% on the max core against the `before` arm's 100.8% — ×2.003, beside a rate that moved ×2.001. That coincidence is arresting and it is worthless, for a reason the code states in [`host.rs`](../../sessionbench/src/host.rs):

> **One point sample per report, and it is a poor summary of a hold.** Taken once when the report is built […] Do not derive a rate, a ratio or a normalisation from this column.

The struct is queried once, at the **end** of a hold. The tenanted hold ends while the six co-tenants are still running — they are stopped afterwards — so a high reading there is the injected load being re-reported by a second instrument at one instant. **The doubling is guaranteed by construction**, and it is not evidence of anything.

**The same applies to the 13:32 record's flat pair, which is quoted there as a refutation.** That record says the clock is *"eliminated for this transition outright"* on 205.1% against 203.7%, and calls it *"a stronger statement than the r = +0.096 across twenty holds, because it is one session, one minute, one change."* It is a **weaker** statement — two point samples on a column whose own documentation forbids exactly this use, where the correlation at least averages twenty of them. The two records would otherwise be a matched pair of opposite errors: one reading a flat clock as proof, the other reading a doubled clock as mechanism.

That the numbers are unusable is visible without any of this reasoning: **two baselines whose rates differ by 1.1% carry clocks of 205.1% and 100.8%.**

**The conclusion is unaffected.** The clock is still eliminated, on [r(rate, max-core clock) = +0.096 across twenty holds](2026-08-11-103621-the-step-is-at-one-and-a-half-cores.md), which is where it always rested. Only the evidence cited in one sentence was wrong.

## Two voided baselines are still points, in a band that was empty

The control voids a **pair** when the baseline starts above the ~1.4-core transition, because there is nothing left to inject into. The baseline hold is still a valid solo reading at a known tenancy, and both landed above the [17-hold correlational set's](2026-08-11-103621-the-step-is-at-one-and-a-half-cores.md) range of 0.99–2.44:

| rest cores | rate |
|---|---|
| 1.43 | 10.573 |
| **3.26** | **14.195** |

3.26 cores of neighbour reading fast is consistent with a **plateau** between the transition and the collapse, rather than with a turnover just past 1.4. The hundred-session curve at [r = −0.950](2026-08-11-085752-the-neighbour-helps-one-session-and-robs-a-hundred.md) only ever samples the far end of the collapse, so the middle of the curve has been inferred rather than measured, and a discarded arm measured a piece of it.

Two holds, one window, so this is a direction rather than a level.

## What this does not establish

- **n = 1 for the copy**, as for the original. Two valid pairs total, one per injector.
- **The behavioural half of the shared-workload objection is untouched**, which is now the leading open question rather than a secondary one.
- **One duty.** Both pairs are `--duty 0.27`. Nothing here separates running from waking, which is where the [only unexamined mechanism class](2026-08-11-103621-the-step-is-at-one-and-a-half-cores.md) lives.
- **The quiet gate passed two runs the guard then rejected**, at readings of 0.00 cores against holds measuring 3.26 and 1.43. The gate counts only per-process instances above half a core; the guard counts the machine. Whether the gate is blind or merely stale is unsettled.
- **No mechanism.** Core clock, uncore, core parking and core placement remain eliminated, and nothing named has replaced them.

## Provenance

| | |
|---|---|
| Run | `inject-tenant.ps1 -Tenants 6 -Duration 30 -Injector target\release\cpu-spin-b.exe`, 2026-08-11 14:15, after 2 voids |
| Artifacts | `bench-out/1786425285-inject-before-daemon/`, `bench-out/1786425356-inject-after-daemon/`; voids `1786425107-`, `1786425185-inject-before-daemon` |
| Read from | `hold.json` — `units_per_session_per_sec`, `occupancy.rest_cores_median`, `occupancy.median_cores`, `counted_ms` |
| Machine | 16 logical / 31 GiB / Windows 11, mains, 0 survivors after teardown |
