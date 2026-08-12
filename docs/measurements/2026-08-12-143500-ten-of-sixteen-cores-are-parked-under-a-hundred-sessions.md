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
