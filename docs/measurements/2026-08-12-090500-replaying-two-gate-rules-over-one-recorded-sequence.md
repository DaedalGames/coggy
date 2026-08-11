# Replaying two gate rules over one recorded sequence

**The quiet gate was changed from "two consecutive polls under the bar" to "a mean of three", and the obvious test — did clearances become more frequent — varies the hour as well as the rule. Replaying both rules over one recorded 141-poll sequence removes the hour entirely. The mean rule cleared once where the streak rule cleared none; that single extra clearance came from a window averaging 0.9967 with two of its three readings above the bar, and the baseline it admitted voided immediately at 3.43 cores.**

## Why a replay rather than a rerun

Clearance rate is a property of what the tenant is doing. Comparing this hour's rate against runs at 04:49, 05:45 and 07:08 varies the hour and the rule together — the same confound that made a tenancy comparison worthless until it was varied inside one window.

The gate prints **every** poll's reading whether or not it fires. That line exists so a reader can see the census counter sitting at zero beside a live machine figure. It also makes the transcript a recorded input both rules can be run against, which turns a confounded before-and-after into a paired test on identical data.

## The sequence

141 polls, 08:20:39 to 08:48:31, from a run whose gate was working for that stretch.

| | |
|---|---|
| polls under the 1.0 machine-wide bar | **2 of 141 — 1%** |
| the two readings | `0.81`, `0.52`, separated by `1.31` and `1.16` |
| **streak-of-2 clearances** | **0** |
| **mean-of-3 clearances** | **1** |

The streak rule cannot fire here at all: its two candidate readings are not adjacent. The mean rule fires on `(1.31 + 1.16 + 0.52) / 3 = 0.9967`, three thousandths under the bar.

## The extra clearance was not a good one

The baseline admitted by that clearance read **3.43 cores held** — well above the 1.3 guard, and voided. So on this sequence the mean rule bought one clearance and zero usable baselines.

That is the finding, and it cuts against the change rather than for it. A rule that fires on a mean of 0.9967 containing two readings above the bar is describing a machine that is *nearly* quiet on average, and the thing being gated needs the machine to be quiet **for the next thirty seconds** — which a backward-looking average of a wandering quantity does not predict any better than a streak does.

## What this does not establish

- **One clearance is not a comparison.** The window contained two sub-bar readings out of 141. Almost all of this sequence is a busy machine, on which both rules correctly refuse everything, so it holds very little information about the question.
- **Nothing about the streak rule's false negatives.** The case the change was made for — a genuinely quiet machine refused because one sample spiked — does not appear in this sequence, because the machine was not quiet.
- **It does not restore the streak rule.** One void is not evidence that the old rule was better; it is evidence that this particular extra clearance was worthless.
- **The replay is a method, not a result.** Its value is that it can be run against any future transcript at no cost, and it should be, before either rule is judged on clearance counts gathered in different sittings.

## What it changes

The gate rework should be judged on a sequence that actually contains quiet, and none of tonight's transcripts do. Until then the change stands on its stated reasoning — that a machine at ~0.8 with sampling excursions was being refused for variance rather than level — and **not** on any measured improvement, because there is none yet.

## Provenance

| | |
|---|---|
| Sequence | `bench-out/inject-20260812-082038.log`, 141 poll lines, 08:20:39–08:48:31 |
| Replay | both rules evaluated in one pass over the parsed `machine-wide` readings; a clearance resets its own state, as in the script |
| The run itself | ended early and its later baselines are ungated, [for a defect recorded separately](2026-08-12-064500-the-idle-floor-sits-on-top-of-the-transition-it-is-measured-across.md) |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, 0 survivors after teardown |
