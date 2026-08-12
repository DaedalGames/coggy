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

> **2026-08-12 15:26 — the direction is settled: this box parks HARDER under load.** 161 consecutive samples, every one taken with 90 or more sessions runnable:
>
> | | under a hundred sessions | idle (353 samples) |
> |---|---|---|
> | parked mean | **9.56** | 5.48 |
> | parked minimum | **7 — never reaches 0** | **0 for 39% of samples** |
> | samples with >= 8 parked | **99%** | 38% |
> | machine cores | 5.09 | 8.4 |
>
> **Idle, the box spends 39% of its time fully unparked. Under a hundred CPU-bound sessions it never once drops below 7 parked cores in eight minutes of sampling.** So cores are not parking because the machine went quiet — it is loudest here, and the parking is at its most persistent.
>
> **That refutes the idleness direction and leaves the causal one standing**: on this box the delivered core count falls when parking rises, parking rises under load, and nothing in the load can push it back. Whatever governs it is not reading the runnable-thread count, or is reading something that a hundred short-cycle sleepers present differently than a scheduler queue length would suggest.
>
> **The remaining caveat is stated before the follow-up, not after.** At `--duty 0.27` a session is runnable only about a quarter of the time, so the box does have brief gaps to park into. That cannot explain a floor of 7 — a gap-driven policy would unpark during the compute phase and produce the bimodal pattern the idle machine shows, not a pinned high level — but it is not eliminated. **The clean arm is `--duty 1.0`, where there is no gap at all**, and it follows immediately.

> **2026-08-12 15:40 — the sleepless arm reverses it, and the original loop was right.** A hundred sessions at `--duty 1.0`, 113 samples, all with 90+ sessions alive:
>
> | | duty 0.27 (161 samples) | **duty 1.0 (113 samples)** |
> |---|---|---|
> | parked mean | 9.56, minimum 7 | **0.24** |
> | samples at 0 parked | **0%** | **97%** |
> | machine cores | 5.09 | **12.36** |
>
> **A sleepless load fully unparks this box. A gappy one does not.** So *parks harder under load*, written an hour ago from the 0.27 arm alone, is **withdrawn**: parking tracks the CONTINUITY of demand rather than its size, and a hundred runnable-but-sleeping sessions do not read as demand.
>
> **That reinstates the feedback loop this record withdrew at 14:45.** A `--duty 0.27` workload leaves gaps; the gaps invite parking; parking cuts the machine to ~5 cores; the starved sessions then present as even less demand. The loop was proposed on reasoning, withdrawn on three point samples, and is now supported by a controlled contrast with the load held fixed and only its continuity varied. **Being withdrawn once is not evidence against a claim** — what settled it was measuring the thing rather than arguing about it, in both directions.
>
> **And it completes the arithmetic the record opened with.** At 0.27 the sessions reach 12.7% of requested duty on a 5-core machine; at 1.0 the same hundred get 12.36 cores. The sessions were never being descheduled off idle cores — their own idleness was switching the cores off.
>
> **The grade on point sampling is the sharpest lesson here.** The three-sample duty-1.0 reading taken earlier gave 9, 10, 10 — and against this distribution those sit at the **97th, 100th and 100th percentiles**. Three consecutive samples landed in a 3% tail and became a published claim that stood for an hour. **A triple is not a small sample of this quantity; it is a sample of one moment**, and the parked count moves faster than the interval between reads.

> **2026-08-12 15:51 — the machine-level prediction confirms; the job-level one is confounded and says so.** A daemon-backed `sessionbench hold`, 100 sessions, `--duty 1.0`, 60 s:
>
> | | |
> |---|---|
> | job `median_cores` | **5.07** |
> | `rest_cores_median` | **7.57** |
> | `observer_cores_median` | 0.09 |
> | machine total | **12.64** |
> | work rate | 2.023 units/session/s |
>
> **The box delivered 12.64 cores**, against ~5 in every `--duty 0.27` hold today, which is the unparking result reproduced in `sessionbench`'s own instrument rather than an ad-hoc counter.
>
> **But the job took only 5.07 of it, because a neighbour held 7.57.** So this run cannot say whether unparking restores THROUGHPUT: the work rate came back at 2.023 units/session/s, inside today's low band of 1.42-2.19, and tenancy alone accounts for that. **The column that would have let this be over-claimed is the one that catches it** — reading `rest_cores_median` before reaching for a throughput reference costs nothing and is already written.
>
> **What it establishes and what it does not.** Established: a sleepless hundred unparks this box, in the archive's own metric, with the observer at 0.09 cores and therefore not a factor. Not established: that the ceiling record's 15.3-15.5 job cores are recoverable, since no run tonight has had a sleepless load AND an absent tenant at the same time. That pairing is the outstanding measurement, and it needs the named census gate in front of it.

> **2026-08-12 15:56 — the gate cleared and the tenant came back inside the hold, for the third time tonight.** A hundred sessions at `--duty 1.0` behind a named census gate:
>
> | | |
> |---|---|
> | gate cleared | 15:55:00, tenant under 0.5 cores |
> | `rest_cores_median` **during** the hold | **9.603** |
> | job `median_cores` | 4.852 |
> | machine total | **14.455** |
> | work rate | 1.963 units/session/s |
>
> **The machine figure is the strongest unparking evidence yet** — 14.455 cores against about 5 in every `--duty 0.27` hold today. Parking is settled as the ceiling on what this box *offers*.
>
> **The job figure measures the tenant again.** 9.603 cores went elsewhere, so the sleepless-and-quiet pairing is still unobtained after three attempts, and **no run tonight has held a sleepless hundred on a genuinely empty box.**
>
> **A pre-hold gate cannot fix this, by construction**: it tests an instant and the hold needs a minute, and this tenant returns within seconds. That is [the same conclusion the idle-floor record reached](2026-08-12-064500-the-idle-floor-sits-on-top-of-the-transition-it-is-measured-across.md) about a pre-baseline recheck, reached again from the other side.
>
> **The instrument for it already exists and was not used here.** `--abort-rest-above` makes a hold refuse itself mid-flight when the residual climbs, which turns an invaded window into a discarded run rather than a published number. Building a guard and then running without it is the more useful half of this entry: the three confounded holds above are all runs that could have refused themselves.

> **2026-08-12 16:06 — the guard fired on the load it protects, and the tell was a zero.** Four consecutive attempts aborted at 5-6 seconds reporting **11.5 to 14.7 cores held outside the job**, seconds after the tenant censused at 0.00. The first sample of one of them:
>
> ```
> procs=101   rss=540 MB   cpu=0.00018%   machine=1470.5% (14.7 cores)
> ```
>
> **The sessions were there — 101 processes, 540 MiB.** What was missing was their CPU, because `cpu_percent` is a delta between two refreshes and the first sample has no predecessor to difference against. `rest` is `machine - cpu`, so `14.7 - 0` made the measurement's own hundred sessions read as a neighbour.
>
> **That is the absence-wearing-a-zero rule**, in the one place where every consumer subtracts the field. The printed line said `occupancy 0.00 cores median ·· 14.61 cores held outside the job`, which cannot be true of a job holding 101 live `--duty 1.0` processes, and the two halves of that sentence disagree on their face.
>
> **Fixed by refusing to evaluate the predicate on a sample with no baseline** rather than by a time-based grace period. A grace window would have hidden the same defect behind a tuned constant, and the defect is not that the job is slow to start — it is that the first reading is not a measurement.
>
> **What it cost**: four aborted attempts, and nearly a conclusion. All-attempts-aborted was about to be recorded as *this box cannot hold a quiet minute while Playwright runs*, which is a plausible claim, matches three earlier confounded holds, and was going to be wrong — the aborts were the instrument, not the box.

> **2026-08-12 16:44 — the driver is the TENANT, not the workload's continuity. The finding above is withdrawn.** Three gated holds of a hundred sessions at `--duty 1.0`, 90 s each, with the abort predicate armed after the startup window:
>
> | attempt | job cores | rest | machine | tenant |
> |---|---|---|---|---|
> | 1 | 4.59 | 0.73 | **5.32** | absent |
> | 2 | 4.55 | 0.82 | **5.37** | absent |
> | 3 | **5.95** | 8.52 | **14.47** | **present** |
>
> **A sleepless hundred on a genuinely quiet box leaves the machine at ~5.3 cores** — the parked level. The same workload with Chromium present reaches **14.47**. So it is the tenant's continuous rasterisation that unparks this machine, and my sessions never did.
>
> **Every "sleepless unparks it" reading was tenanted.** The 12.36 from the parking-under-load run recorded `tenant_processes` 3-4; the 12.64 and 14.455 holds carried residuals of 7.57 and 9.60 and were written up above as *confounded by tenancy* on the job side while their MACHINE figure was quoted as clean. It was not clean — it was the confound. **The two arms of the continuity contrast were the same runs**, and tenancy was uncontrolled in both.
>
> **And the neighbour helps the job.** 5.95 cores with a tenant taking 8.52, against 4.55-4.59 with the machine otherwise empty. That is [the documented "neighbour helps one session"](2026-08-11-085752-the-neighbour-helps-one-session-and-robs-a-hundred.md) effect with a mechanism instead of a shrug: the neighbour unparks cores the workload cannot ask for.
>
> **What survives.** The parked count is real, bimodal, and worth 2.5-3.6x in delivered cores; the dwell maxima of 178-257 s still explain how a hold lands entirely in one state; and the box still delivers ~5 cores to a hundred sessions whatever their duty. **What does not**: that duty or continuity is the lever. `--duty 0.27` and `--duty 1.0` both give ~4.6-5.1 job cores when the box is quiet.
>
> **The measurement that would settle it** is the mirror image and has never been run: a `--duty 0.27` hundred held WHILE the tenant is present. Continuity predicts ~5 there and tenancy predicts ~14, and for the first time the two hypotheses disagree.

> **2026-08-12 17:00 — the 2x2 closes, and one arm contains its own control.** The mirror-image cell, a gappy hundred held WHILE the tenant is present, is the first configuration where the two hypotheses predicted opposite answers:
>
> | | tenant absent | tenant present |
> |---|---|---|
> | `--duty 0.27` | machine **5.09** | machine **14.53 / 14.60** |
> | `--duty 1.0` | machine **5.32 / 5.37** | machine **14.47** |
>
> **A 3.7x change in the sessions' own continuity moves the machine total by 5%. The presence of one browser moves it by 2.8x.** Continuity explains nothing; tenancy explains it.
>
> **And one arm varies the neighbour inside a single window**, which is stronger than a table assembled across a night. Three `--duty 0.27` holds minutes apart, same command, with the tenant leaving during the second:
>
> | attempt | job | rest | machine |
> |---|---|---|---|
> | 1 | 3.42 | 11.11 | **14.53** |
> | 2 | 3.01 | **2.14** | **5.15** |
> | 3 | 3.65 | 10.95 | **14.60** |
>
> The machine total follows the tenant and nothing else. **The job cores barely move** — 3.42, 3.01, 3.65 — because at duty 0.27 the sessions are limited by their own duty rather than by the machine. So the neighbour nearly triples what the box delivers and hands almost none of it to the job, which is [the "helps one session, robs a hundred" effect](2026-08-11-085752-the-neighbour-helps-one-session-and-robs-a-hundred.md) visible inside one arm and with a mechanism: **the neighbour keeps cores unparked, and a hundred self-limiting sessions cannot.**
>
> **A defect in the script that produced this, recorded because it will bite the next user of the switch**: `-RequireTenant` waits for a tenant ABOVE 5 cores while `--abort-rest-above` stays at 2.0, so every attempt aborts within ~30 s. The gate and the guard contradict each other by construction. Each attempt still yielded the machine total this arm needed, so the result stands, but the ceiling has to track the gate's own requirement.

> **2026-08-12 17:17 — the state outlives its cause, so naming the neighbour is not enough.** A five-minute hundred-session hold with the new `tenant_cores_median` column, 60 samples:
>
> | | |
> |---|---|
> | tenant cores, per sample | **0.00 to 9.26** |
> | machine cores, per sample | **13.50 to 15.32** |
> | correlation | **r = -0.641** |
> | attribution | job 3.65 + tenant 8.71 + observer 0.42, **1.36 unattributed** |
>
> **The tenant fell to zero mid-hold and the machine stayed unparked at 13.5-15.3 cores for the whole run.** So the box does not follow the neighbour instantaneously: once unparked it stays unparked, which is what the dwell maxima of 178-257 s already said — **the state outlives its cause**.
>
> **The negative correlation is composition rather than opposition.** The total is pinned by the unparked core count, so when the tenant releases cores the job takes them: `machine ~= job + tenant`, and the two trade instead of adding. A hold that reads a low tenant is therefore not a hold on a quiet machine, and a hold that reads a high one is not necessarily worse off.
>
> **This is why the parked count is still needed, and the reasoning was pre-registered.** The prediction was that a machine swinging under a steady tenant would prove a second switch; what happened is the inverse — a steady machine under a swinging tenant — with the same implication. **`tenant_cores_median` records the CAUSE and cannot record the STATE**, and the state is what decides which machine a hold measured. That promotes the PDH work from a generalisation to the only instrument that sees it.
>
> **What the column did earn**: three quarters of a formerly anonymous 10.5-core residual is now named, leaving 1.36 unattributed where the whole figure used to be a shrug.

> **2026-08-12 17:31 — the new tenant column misses a neighbour that arrives mid-hold, and its own count field caught it.** A ten-minute hundred-session hold, 119 samples:
>
> | | |
> |---|---|
> | `tenant_cores_median` | **0.000** |
> | `tenant_processes` | **0 in every sample** |
> | `rest_cores_median` | **10.411** |
> | machine, per sample | **4.60 to 14.38** |
> | `chrome-headless-shell` running afterwards | **5 processes** |
>
> **The tenant was there and the column read zero.** Discovery only runs on the full-table refresh, and a hold calls `refresh` with a tracked PID set on every tick, so the list is frozen at `Sampler::new()`. The observer read 2 throughout because it exists before the sampler does; the neighbour arrived later and was never enumerated. **The column misses exactly the case it was built for.**
>
> **`tenant_processes` is what made this visible within the hour.** Without it the artifact would have published `tenant_cores_median: 0.00` beside a 10.4-core residual, reading as *the machine was quiet and something unknown held ten cores* — the anonymous-residual failure wearing the name of the column that exists to prevent it. A zero with a count beside it cannot lie that way.
>
> **The fix is not obvious and must not be rushed.** Re-enumerating per tick is the thing the tracked-refresh path exists to avoid: the full table cost **eighty seconds at twenty-five sessions** and made the instrument the bottleneck it was measuring. Periodic re-discovery needs its own cost measurement before it lands.
>
> **What the hold did establish, from columns that work.** The machine ranged **4.60 to 14.38 cores inside a single artifact** — 20 samples below 8 and 99 at or above — which is the two-operating-points switch captured in one file for the first time, rather than inferred across sittings. The parallel census agrees: 153 of 205 polls at zero parked, the rest between 9 and 12.

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
