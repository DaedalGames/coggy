# Three solo holds spread 30%, and the machine column said it was a tenant rather than a state

`doctor` read **1.35–1.89 cores** of background immediately before and after, so by the reading this project has always used, the window was open. Three back-to-back 120-second solo holds of the same workload then returned figures 30% apart.

This is the first use of [the whole-machine CPU column since it was put on the job's scale](2026-08-03-061911-the-machine-column-was-narrower-than-the-job.md), and it answers in one table what would otherwise have been read as a machine state.

## The three holds

`hold --sessions 1 --interval 5 --duration 120 -- cpu-spin --units 100000000 --duty 0.27 --resident 20`, run back to back.

| | rate | job (median) | machine (median) | **rest of the machine** |
|---|---|---|---|---|
| solo 1 | 14.543 units/s | 0.24 cores | 11.69 | **11.46 cores** |
| solo 2 | **17.561** | 0.26 | 3.26 | **2.99** |
| solo 3 | 13.002 | 0.25 | 11.83 | **11.60** |

**The job is the same in all three** — 0.24 to 0.26 cores, a single session at duty 0.27. What moved is everything outside it.

Spread on rate: **30.3%** of the mean. Spread on the rest of the machine: **11.46, 2.99, 11.60**.

## What that settles, and what it does not

**It is not the slow machine state.** That state is identified by a solo's own rate, and these three cannot identify anything because they disagree — but they disagree *with* the background and not independently of it. The fast one is the quiet one.

**A solo is not immune to a tenant, which is what this contradicts.** The gate script gained a guard hours earlier whose stated reasoning was that a solo needs one core, so probes run at full speed while a tenant ruins the concurrent hold. Measured here: taking 11.5 of 16 cores costs a *single* session **26%** (17.561 → 13.002). The guard is still worth having and its reason was wrong.

**And the level check cannot tell the two apart.** `m1-hour.ps1` notes a post-saturation machine when its probes mean under 15 units/s. These three mean 15.0. A tenant would trip that note and be reported as the thermal state, which is a different cause with a different remedy — waiting an hour fixes one and not the other. What separates them is the row above: the slow state leaves the machine idle, a tenant does not.

**Nothing here names the tenant.** It appeared and left inside a seven-minute window; `doctor` caught 1.89 cores before, 1.35 after, and 2.99 as its worst sample once it was gone. Earlier the same day this box ran `chrome-headless-shell` at 7.6–8.5 cores for over forty minutes, but nothing recorded what held the machine during these three holds, and a point sample after the fact cannot say.

**And the quiet hold is still 18% under the rested figure.** 17.561 against the ~21.5 this box gives solo when rested, with 2.99 cores of background. Either three cores of background costs a solo that much, or the box is partly degraded, and these artifacts cannot separate the two.

## What it means for the gate hour

The bracket takes its solo baselines at both ends of a run. **An intermittent tenant of this size is the case the bracket is least able to refuse**: two baselines that both land inside a burst agree with each other and are both wrong, and two that straddle one disagree and refuse a run that was fine. Neither outcome is the measurement.

The remedy is not another pre-screen. It is that every hold now records the machine beside the job, so a bracket's baselines can be asked what else was running rather than only whether they agree.

## Provenance

| | |
|---|---|
| Inputs | `bench-out/1785707434-fingerprint-1-daemon`, `1785707593-fingerprint-2-daemon`, `1785707752-fingerprint-3-daemon` |
| Machine | on mains, `doctor` 1.89 cores before and 1.35 after |
| Commit | `76ae21e` |

## The reading was fixed before the run

Three outcomes were written down before launching: near 21.5 means rested, near 9.4 means the slow state, and a spread past 5% means neither is claimed. The first two were the interesting ones, which is why the third was written first. What arrived was the third, and the first two values — 14.5 then 17.6 — looked like a recovery curve for about four minutes until the third came back lowest of all.
