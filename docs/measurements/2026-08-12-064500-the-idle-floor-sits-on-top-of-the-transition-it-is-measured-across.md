# The idle floor sits on top of the transition it is measured across

**Two waiting runs, one hour and forty-five minutes, eight baselines and no pair. The result is the population they fall into: four landed between 1.0 and 2.0 cores held — and the step's transition is at 1.36–1.46. This box's *idle floor* straddles the phenomenon. The rising limb is not blocked by a loud machine; it is blocked by a quiet one whose quiet overlaps the threshold.**

## What was being attempted

The rising-limb sign contradiction: [two pairs](2026-08-12-042000-longer-holds-do-not-cut-the-spread-and-they-halve-the-acceptance.md) at 1.02–1.22 cores held went **−7.0%** and **−26.5%**, where the controlled injections that established the step recorded **+95.4%** and **+100.1%**.

Both runs used the real quiet gate and **no `-AnyBaseline`**, so the baseline guard enforced a baseline below 1.3 cores — under the transition, which is what "rising limb" means here. Both sat waiting rather than being launched at a window, [because a window can be detected but not predicted](2026-08-12-045500-the-anonymous-neighbour-is-a-software-rasteriser-and-the-observer-is-bigger-than-the-treatment.md).

## Eight baselines, three populations

| baseline `rest` (cores) | n | what it is |
|---|---|---|
| 1.03 | 1 | floor, low end — **the only one that passed** |
| 1.78, 1.94, 1.94 | 3 | floor, just above the 1.3 guard |
| 5.00 | 1 | transitional |
| 11.75, 12.22, 13.39 | 3 | the browser |

Only three of eight are the tenant. **Half are the machine's own idle floor**, sitting between 1.0 and 2.0 cores — and drifting across the 1.36–1.46 transition within minutes.

> **2026-08-12 08:08: a third run tested this out of sample, and the band held.** The eight baselines below were taken before this record was written; a third waiting run (07:08–08:08, `-MaxVoids 20`, ended `NoQuietWindow`) added two more, at **1.76** and **1.45** — both in the floor band, the second sitting *inside* the 1.36–1.46 transition itself. Six of ten baselines are now the floor. The run also cleared the gate only **twice in sixty minutes**, against five in forty-five and four in sixty for the two runs below, so the clearance rate is not steady either.
>
> **Two figures follow, and both are conditional on the tenant rather than on the hardware.** At roughly one clearance per ten minutes and one usable baseline per eight clearances, a single rising-limb pair costs about **eighty minutes of idle waiting** in expectation — for one pair, where a claim needs several. That price is a fact about a box running Playwright automation, not about this machine, and it should never be quoted as *this box cannot do rising-limb work*.
>
> **And the third run optimised the wrong term.** Its void budget was raised from 8 to 20 because [the abort predicate](2026-08-12-053500-the-rising-limb-is-reachable-and-the-tenanted-arm-is-what-loses-it.md) had made a spoiled attempt cost five seconds instead of thirty. It used two. The scarce thing was never the cost of an attempt but the rate of opportunities to make one — the same mistake as lengthening a hold to buy precision and finding acceptance halved instead. In both cases the binding constraint had been named and never measured before the change was made.

> **2026-08-12 09:00: a fourth run added one gated baseline and five ungated ones, and only the first counts.** A defect introduced while reworking the quiet gate disabled it after the first clearance, so five baselines were taken with no quiet check at all — 13.18, 13.02, 14.19, 9.11 and one earlier 3.43. **Those are a sample of the machine at arbitrary moments, not at candidate-quiet ones, and they do not bear on this claim**, which is about what a baseline looks like *when the gate says quiet*. Counting them would be reading a different population into the same table.
>
> The one **gated** baseline from that run read **3.43**, after a clearance at a mean of 1.00 cores. It is above the band. Gated baselines now stand at eleven — 1.03, 1.45, 1.76, 1.78, 1.94, 1.94, 3.43, 5.00, 11.75, 12.22, 13.39 — of which **six are the floor**, the same share as before, with the transitional group growing from one to two.
>
> **What the ungated five do show is worth keeping separately**: between 08:48 and 08:53 this machine went 0.52 → 3.43 → 13.18 → 13.02 → 14.19 → 9.11 cores. That is the tenant's range inside five minutes, and it is why a gate clearance says nothing about the thirty seconds that follow it.

> **2026-08-12 10:15: the first uncontaminated baselines, and the finding survives at lower numbers.** Every baseline above was taken under a watcher that streamed clearances — and [each such event wakes the agent into the hold it announced](2026-08-12-092000-the-clearance-notification-wakes-the-observer-into-the-hold-it-announced.md). A run watched **only on exit**, with the new `observer_cpu_percent` column recording the contamination per sample, gives four baselines at **1.85, 1.50, 1.36 and 2.18** — mean **1.72** against the previous run's **2.39**.
>
> **The column confirms why.** It reads **0.18–0.19 cores on every sample of all four holds**, against 1.79–1.99 measured on a hold taken while the agent was actively working. That figure was predicted before the run: near zero if the observer theory was right, and unchanged if something else arrives when a hold starts.
>
> **The floor still straddles the transition.** The cleanest baseline is **1.36**, which is the lower edge of the 1.36–1.46 interval, and the guard sits at 1.3. So the claim holds with the contamination removed — the machine's own idle residual lands on the boundary the experiment must start below — and the numbers it holds at are lower than the eleven contaminated baselines suggested. **Those eleven cannot be corrected**, since no artifact of theirs carries the column; they are superseded rather than adjusted.

## Why no guard tuning fixes this

The 1.3 bar is not mis-set. It is placed just below the transition on purpose, because crossing that transition is the thing being measured. Moving it up admits baselines *above* the transition, which measures nothing. Moving it down refuses nearly everything the machine offers.

The gate does not fix it either, and its own defect is real but separate: across the first run **every one of ten polls read below the 1.0 machine-wide bar** — 0.94, 0.88, 0.49, 0.78, 0.89, 0.90, 0.58, 0.83, 0.93, 0.62 — and most failed anyway, because the rule asks for two *consecutive* samples from a machine whose floor is ~0.8. The census half of the gate contributed nothing at all, reading `0.00` on every poll of both runs.

**The floor and the guard occupy the same region for unrelated reasons.** That is the finding, and it is not something the instrument can be tuned out of.

## The tenant reaches inside a thirty-second hold

Three of the four voids in run 2 were the browser arriving **after** the gate certified the machine under 1.0 core and **during** the 30-second baseline hold: 13.39, 11.75 and 12.22 cores.

So a pre-baseline re-check would be worthless — the gate has just read quiet, and an immediate second reading says the same. The arrival happens in the one window nothing observes. That is a second claimant for an abort predicate inside the hold itself, alongside [the arrival during a *tenanted* arm that cost the best baseline of the night](2026-08-12-053500-the-rising-limb-is-reachable-and-the-tenanted-arm-is-what-loses-it.md).

## Two operator errors, both recorded because both cost something

**The clock was estimated rather than read, for the second time in three hours.** The run was judged thirteen minutes past its deadline; the clock said 06:44:36 against a 06:45:04 deadline — twenty-eight seconds *short*. The first instance had already produced a written rule (*print the clock when the number matters*), and the rule did not fire. Tick-to-tick intervals feel uniform and the arithmetic inherits that silently; nothing contradicts it until a real timestamp is read for an unrelated reason.

**And that mistaken belief cost a hold.** The status query it prompted landed at 06:44:36, inside a baseline hold that had started at 06:44:05, adding roughly 1.6 cores of agent to a measurement whose entire purpose is a low residual. That baseline voided at 12.22 cores — browser-scale, so the observer was a small part of it and the void would have happened anyway. The contamination did not determine the outcome, which is worth stating rather than leaving as a suspicion.

## What this does not establish

- **No pair, so nothing about the step.** The sign contradiction now has four failed attempts against it rather than an answer — the 04:38 launch into an open window, and the three waiting runs.
- **Eleven gated baselines is not a distribution.** "Floor between 1.0 and 2.0" is a description of eleven readings on one night, not a characterisation of the machine — though six of the eleven are in that band and the last two were taken after this claim was written, which is the difference between a description and a survived prediction.
- **The 1.03 baseline that passed** shows the low end is reachable, and says nothing about how often.
- **Nothing here re-derives the +95.4% and +100.1% figures.** But it does raise a question about them that this record cannot answer: if a baseline under the transition is this rare, how were those two obtained, and what was the machine doing at the time?

## What it changes

The rising-limb design should not be retried on this box without something new. Three options, in order of cost:

1. **An abort predicate inside `hold`**, so a spoiled arm ends at its second sample rather than its sixth. It now has two independent claimants and is the only change that serves both arms.
2. **A shorter baseline hold**, which narrows the window in which the tenant can arrive — at the cost of a noisier rate, on a quantity whose spread is [already the binding constraint](2026-08-12-042000-longer-holds-do-not-cut-the-spread-and-they-halve-the-acceptance.md).
3. **Accept the collapse limb as the instrument.** `-AnyBaseline` pairs run whenever the box allows and were treated as a compromise. If the rising limb is reachable roughly once in eight attempts, that ordering is backwards.

## Provenance

| | |
|---|---|
| Run 1 | `inject-tenant.ps1 -Tenants 6 -ExpectedPerTenant 0.19 -Duration 30 -MaxVoids 4 -GiveUpMinutes 45`, 04:49:32–05:34, outcome `GaveUpOnVoids` |
| Run 2 | same with `-MaxVoids 8 -GiveUpMinutes 60`, 05:45:04–06:45, outcome `DeadlineReached` |
| Run 3 | same with `-MaxVoids 20`, 07:08:24–08:08, outcome `NoQuietWindow`, 2 voids, both floor baselines |
| Read from | `bench-out/inject-20260812-044932.json`, `-054504.json` and `-070825.json`, all `pair: null` |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, 0 survivors after teardown |
