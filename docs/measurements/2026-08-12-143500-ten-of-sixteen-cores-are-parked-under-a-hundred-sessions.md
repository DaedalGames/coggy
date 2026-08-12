# Ten of sixteen cores are parked under a hundred sessions

**A hundred `cpu-spin --duty 0.27` sessions, tenant censused at zero processes: `Parking Status` reads 10 of 16 cores parked, twice, and per-core utilisation is 96, 96, 0, 0, 3, 3, 7, 3, 96, 96, 40, 46, 12, 4, 1, 0. The sessions were never descheduled off idle cores — eleven of the cores were not available to them. This is the mechanism behind every "on an idle box" reading taken tonight.**

> **2026-08-12 14:45 — the feedback loop below is refuted, and what replaces it is larger.** Parking on this box does not respond to load at all:
>
> | state | parked | machine |
> |---|---|---|
> | idle, no load | **12 of 16** | — |
> | 100 sessions at `--duty 1.0`, no sleep, tenant at 0 processes | **9–10 of 16** | **5.01 cores** |
>
> A hundred fully CPU-bound sleepless processes cannot unpark this machine. So the loop proposed below — a sleep-heavy load looks underused, the policy parks cores — **is wrong**: the saturating load looks nothing like underused and the cores stay parked anyway. Parking here is a standing state, not a response.
>
> **And the earlier reading that a sleepless hundred drove the machine to 92.5% was the tenant.** That window had `chrome-headless-shell` at 7.93 cores; with the tenant censused at zero the same workload reaches 5.01. The sessions never saturated anything.
>
> **What this replaces the loop with is the answer to a bigger question.** [The ceiling record](2026-08-12-114500-every-hundred-session-hold-today-is-a-third-of-what-the-box-used-to-do.md) asks why every hundred-session hold today sits at 3.34–4.81 job cores where 3 and 11 August reached 15.3–15.5, and names no cause. **A box pinned at five or six usable cores cannot produce fifteen.** The parked count is not a property of the workload, so it applies to every measurement taken in this state — which is what that record observed and could not explain.
>
> **What is still not established**: why the cores are parked, and whether they were unparked on the days that reached 15.5. The policy is hidden in this scheme and cannot be read without changing attributes on the machine, so this is an effect with a named shape and an unread cause.

> **2026-08-12 15:05 — the parked count is not a level, and every figure in this record is a point sample.** Reading it three times twelve seconds apart, within one arm:
>
> ```
> SOLO     sessions=1    parked = 4 / 12 / 11    machine 8.52 cores
> HUNDRED  sessions=100  parked = 0 /  7 /  9    machine 5.02 cores
> ```
>
> **It swings from 0 to 12 inside seconds, on both arms.** So *12 of 16 parked at idle* and *9-10 under a hundred sleepless sessions* are two and three point readings of a fast-moving quantity, and the words built on them — **a standing state**, **not load-responsive** — are withdrawn. A quantity sampled three times cannot support either.
>
> **This is the rule this repository already carries**: a point sample is not a windowed mean, and the failure is reading the nearest instrument rather than the one that decides. The parking counter is instantaneous by construction and was read as a level.
>
> **What survives.** The count is frequently high — 4 to 12 of 16 across every reading here — which is qualitatively consistent with a machine offering far fewer cores than it has. And the independent figure is steady where the parked count is not: `machine_cores` reads **5.0** under a hundred sessions across every hold tonight, by a counter that is a windowed mean rather than an instant. **The throughput ceiling is measured; the parked count is the candidate explanation and is not yet measured properly.**
>
> **What that costs the conclusions below.** The ceiling record's answer stands on the *ceiling*, which is windowed and reproducible, and on parking only as a mechanism. The ROADMAP correction saying `C` is five or six rather than sixteen rests on the same windowed figure and survives; the sentence quoting *12 of 16 parked* as a level does not, and needs the mean this record does not yet have.
>
> **What would measure it**: the parked count sampled at 1 Hz or faster across a full hold, reported as a mean with its spread, on both arms. The 30-second census now running polls too slowly to characterise something moving this fast, and will show the same aliasing.

> **2026-08-12 15:20 — sampled properly, the parked count is BIMODAL rather than noisy.** 26 consecutive samples:
>
> ```
> parked       mean 5.88   sd 5.54   min 0   max 12
> distribution 0 x11   4 x2   5 x1   10 x2   12 x10
> ```
>
> **Twenty-one of twenty-six samples sit at either 0 or 12**, with almost nothing between. So the mean of 5.88 describes no state this machine is ever in — it is the average of a switch, and reporting it as a level would be a third version of the same mistake.
>
> **This is the shape that explains two operating points 4.7x apart.** A box alternating between roughly sixteen usable cores and roughly four is not a box with a variable speed; it is two machines taking turns. It also explains why every point sample tonight looked like a confident level: each one caught the switch in one position, and 12 and 0 are both common enough to be sampled repeatedly.
>
> **And it retro-fits the earlier readings without needing them to be wrong.** *12 of 16 at idle* and *0 under a hundred sessions* are both real observations of a bimodal quantity; what was wrong was the word *state* attached to each.
>
> **The instrument does not sample as fast as it asks.** The census requests 1 s and delivers **3.1 s** (min 3.0, max 3.1), because `Get-Counter` carries its own ~1 s floor and the loop makes two calls. That is recorded rather than fixed: 3.1 s resolves a switch this coarse, and the artifact now states its achieved interval instead of its requested one — the same reason a hold reports achieved duty rather than the flag it was given.
>
> **What is still not established**: the switch's period and what drives it. 26 samples over ~80 seconds is enough to show bimodality and not enough to time it, and nothing here relates the position to load, to the tenant, or to the clock.

> **2026-08-12 15:16 — 353 samples over 18 minutes, and the picture closes.**
>
> | | |
> |---|---|
> | samples | 353 over 18.0 min, achieved interval **3.06 s** |
> | parked distribution | **0: 39%**, 12: **27%**, everything else spread thin — **67% at the two extremes** |
> | dwell means, across cuts of 4, 6, 8 and 10 | **18-35 s** |
> | dwell **maxima** | **178-257 s** |
> | machine cores, parked side vs unparked | **3.13 vs 11.28 — 3.6x** |
>
> **The bimodality survives.** The 37-sample read that looked like it was dissolving was itself the small sample, and this record came within one tick of withdrawing a correct claim. **The rule against asserting from few samples applies equally to retracting from them** — a shape that weakens at 37 and holds at 353 was never weakening.
>
> **The decisive figure is the maximum dwell, not the mean.** A mean of 25 s would average out inside any hold and could not produce two operating points. **A maximum of 178-257 s is several times longer than the 25-60 s holds taken all night**, so a hold can sit entirely inside one state — which is what makes a bimodal machine produce two reproducible numbers instead of one blended one.
>
> **The dwell figures are threshold-dependent and the conclusion is not.** Cuts at 4, 6, 8 and 10 give above-side means of 35, 25, 18 and 18 s and maxima of 257, 178, 178 and 178 s. The mean moves by a factor of two with the cut; every cut agrees the maximum is minutes rather than seconds. **That is the check the threshold sensitivity was run for** — a run-length statistic on a split series is a fact about the machine only where it survives its own cut.
>
> **And the two sides differ by 3.6x in delivered work**, 3.13 cores against 11.28, which is the same order as the 4.7x separating the two operating points.
>
> **What is still not established, and it is the direction.** Cores may park *because* the machine went idle, rather than the machine delivering less *because* cores parked. This correlation cannot distinguish them, and both readings are consistent with either. Settling it needs the parked state recorded *alongside* a controlled load rather than sampled beside it.

## What was measured

| | |
|---|---|
| Load | 100 × `cpu-spin --units 100000000 --duty 0.27 --resident 1`, stdout to `NUL` |
| Tenant | `chrome-headless-shell`, **0 processes** |
| `Parking Status` | **10 of 16 parked**, on two independent reads |
| Per-core `% Processor Time` | `96 96 0 0 3 3 7 3 96 96 40 46 12 4 1 0` |
| Sum of per-core busy | **5.02 cores** |
| Survivors after teardown | 0 |

Four cores carry essentially all the work at 96%, two more take 40–46%, and the remaining ten sit between 0 and 12%.

## Why this is the answer rather than another symptom

The same workload was pinned from both sides an hour earlier, in one window with the tenant at zero:

| the session's own clock | | the kernel | |
|---|---|---|---|
| computed | 50.19 ms | per-session CPU | **0.0341 cores** |
| slept | 138.87 ms | machine busy | 4.481 of 16 |
| oversleep | **1.023** | duty achieved | **0.0341** |
| duty it believes it achieved | **0.2655** | | |

So the pause was already eliminated — the sessions sleep 2.3% longer than they ask and their own arithmetic is self-consistent. What remained was that a session is on-CPU **12.8%** of the wall time it spends computing. Ordinary core-sharing predicts **1.6×** at this duty (≈27 of 100 computing on 16 cores) and the measurement showed **7.8×**, leaving a factor of ~4.9 with no owner.

**Parking supplies it.** With 10 cores parked, ~27 runnable threads contend for ~6 cores rather than 16 — and the sum of per-core busy, 5.02, matches the machine counter's 4.48–5.00 across every hold tonight. A box that appears 70% idle is a box whose idle-looking cores are switched off.

## The feedback loop this creates — REFUTED, see the append above

*Written before the load comparison and left as a log.* The reasoning was that core parking responds to low utilisation, that a sleep-heavy workload presents exactly that, and that the condition triggering parking is then sustained by parking. **A hundred sleepless sessions at `--duty 1.0` leave 9–10 cores parked**, so the policy is not responding to this machine's load in either direction and the loop does not exist.

What survives is the part that never depended on the mechanism: the box has two stable operating points at a hundred sessions, roughly 4.7× apart, with identical per-core efficiency in both. **Identical efficiency with different totals is what a changing core count looks like, not a changing core speed** — and a parked count that ignores load is exactly such a change.

## What this costs the project

[Gate M1's work-rate condition](../../ROADMAP.md) is a ratio of per-session rate at a hundred sessions against the same workload alone. A solo session does not trigger parking and a hundred sleep-heavy ones do, so **the two arms of that ratio can run on different numbers of cores**. The gate's headline number is then partly a measurement of this box's power policy.

It also reframes every hold taken tonight. The readings are correct; the phrase "on an idle box" attached to them is not, because the idleness was manufactured by the same thing that suppressed the throughput.

## What this does not establish

- **The policy was not read, only its effect.** `Parking Status` says cores are parked; nothing here queries which power scheme or `CPMINCORES` setting produced that, and nothing was changed to test it.
- **No unparked comparison.** The decisive experiment is the same hundred sessions with parking disabled, which is a machine-configuration change and is not made here.
- **One workload, one duty, one box.** `cpu-spin --duty 0.27 --resident 1`. A saturating workload keeps cores unparked, which is consistent with the sleepless run reaching 92.5% machine busy, but that is one reading rather than a series.
- **The 4.9× is accounted for, not derived.** Six available cores against sixteen is a factor of 2.7, and the measured residue was 4.9. Parking is the right order and does not close the arithmetic exactly.
- **It does not explain why parking persists** under a hundred runnable threads, which is the behaviour a load-responsive policy is supposed to prevent.

## Provenance

| | |
|---|---|
| Counters | `\Processor Information(0,*)\Parking Status`, `\Processor Information(0,*)\% Processor Time`, 15 s sample |
| Timing side | `sessionbench/scripts/oversleep-in-window.ps1`, 12 of 100 sessions with `--report-timing` |
| Tenant gate | named per-process census of `chrome-headless-shell`, not `doctor` and not a residual |
| Machine | 16 logical / 31 GiB / Windows 11, mains, 0 survivors after teardown |
