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

> **2026-08-12 18:03 — the idle base rate, and the box is unparked most of the time.** 492 samples over 25 minutes with nothing of mine running:
>
> | | |
> |---|---|
> | fully unparked (0 cores parked) | **84%** of samples |
> | parked at 8 or more | **11%** |
> | machine when parked >= 8 | **2.72 cores** |
> | machine when parked < 8 | **12.15 cores** |
> | `chrome-headless-shell` present | **5 processes in 439 of 492 samples** |
>
> **A prediction was registered before this ran and it failed.** The expectation was that an idler machine would park MORE than the 39%-unparked reading taken earlier while the agent was working. It parks far less. **The reason is the last row**: the tenant was running for 89% of the window, so "idle" was never idle in the sense that matters — the prediction was about load and this machine answers to the neighbour.
>
> **The 4.5x split sits inside one census with nothing of mine running**: 2.72 cores while parked against 12.15 while not. That is the same effect the 2x2 found, measured without any workload at all.
>
> **And it settles what the archive's high holds were.** This box is unparked most of the time, so the 15.3-15.5 job cores of 3 and 11 August were **normal** rather than lucky. Today's readings are the exception, and what made them exceptional is that the tenant happened to be away — the opposite of the reading this record opened with, where a parked box looked like the machine's resting state.

> **2026-08-12 18:37 — the mechanism, from an observation nobody intervened in.** 590 samples over 30 minutes, machine otherwise idle, split by whether the neighbour was present in the SAME sample:
>
> | | |
> |---|---|
> | parked >= 8 while tenant **absent** | **98%** (n=206) |
> | parked >= 8 while tenant **present** | **4%** (n=384) |
> | distinct absence **episodes** | **5** |
> | arrival -> unpark | 2-3 samples, **6-9 s** |
> | departure -> park | 0 samples, **immediate** |
>
> **This is the strongest form of evidence available here**, because the variable moved on its own rather than because it was moved, inside one window, with no comparison across sittings. 98% against 4% is the 2x2 confirmed without any intervention.
>
> **The lags reverse the expectation registered before the run.** A fast unpark and a slow park were predicted — a response and a timeout. Measured: **parking within one sample of the neighbour leaving, unparking taking 6-9 s.** This policy releases cores eagerly and returns them reluctantly, which is the opposite of how a demand-following governor is usually described.
>
> **And it reconciles the hysteresis reading that looked contradictory.** In the five-minute hold above the tenant left and the box stayed unparked; here, idle, it parks instantly. The difference is that a hundred sessions were running there. **So the sessions can HOLD the unparked state once something else has triggered it, and cannot trigger it themselves** — which is why they read as irrelevant in the 2x2 and as load-bearing in the hold.
>
> **The episode count is stated because it is the real sample size.** The split rests on **five** absences rather than 206 independent observations, and each lag figure is four measurements. A 3-second series is autocorrelated, so counting rows would overstate the confidence by an order of magnitude.

> **2026-08-12 18:58 — sessions do NOT hold the unparked state. Withdrawn.** A 17-minute hundred-session hold with the census running alongside, 334 samples:
>
> | | during the hold | idle reference |
> |---|---|---|
> | departure -> park | **0 samples, all 7 episodes** | 0 samples, 4 episodes |
> | parked >= 8, tenant absent | **100%** (n=149) | 98% |
> | parked >= 8, tenant present | **52%** (n=185) | 4% |
>
> **Seven departures, seven immediate parks**, indistinguishable from an idle machine. The claim that a hundred sessions can hold a state they cannot trigger rested on ONE five-minute hold where the tenant left and the machine total did not move; that hold caught something else. **The failure condition was written into the task before this ran**, which is the only reason the withdrawal is quick rather than argued.
>
> **The 52% row is not yet a claim.** The census counts tenant PROCESSES, not tenant load, and a browser that is present but idle produces exactly this reading. Presence is a proxy for the demand that drives parking, and a process count cannot tell a busy Chromium from a dormant one. What would separate them is the tenant's own CPU per sample, which the census records and which this pass did not use.
>
> **And the hold's own tenant column was unusable here** — `tenant_cores_median` read **-0.000** while the census saw the browser throughout. That is the frozen-discovery defect: the column is populated only at `Sampler::new()`, so a neighbour already running when the sampler starts is caught and one that arrives later is not. The census and the hold disagreed, and the census was right.

> **2026-08-12 19:23 — it is the neighbour's LOAD, not its presence, and the answer survives every threshold.** 307 samples on an idle machine, now recording the tenant's own cores:
>
> | tenant state | parked >= 8 |
> |---|---|
> | absent | **100%** (n=48) |
> | present, load **below** the cut | **75 / 80 / 83 / 78%** at cuts of 0.5, 1, 2 and 5 cores |
> | present, load **above** the cut | **4 / 3 / 3 / 2%** |
>
> **A present-but-idle browser parks this box almost exactly like an absent one.** So presence was the wrong predicate, and the earlier puzzle — 52% parked while the tenant was PRESENT during a hold against 4% when idle — dissolves: the browser was there and not working.
>
> **The threshold sweep is the check that makes this safe to build on.** Moving the cut by a factor of ten changes the below-side figure from 75% to 83% and the above-side from 4% to 2%, so the split is a fact about the machine rather than about where the line was drawn — the same discipline that turned a threshold-dependent dwell mean into a threshold-independent maximum above.
>
> **The honest weakness is n on the below side**: 4 to 9 samples, because this browser is almost always busy when it exists (median load **10.03 cores**). The direction is clean and the magnitude is thin, and a run that deliberately idles the tenant would fix that.
>
> **What it changes.** Every predicate written as *tenant present* is measuring the wrong thing — including `parking-under-load.ps1` and the census's own `tenant_processes` split. `ceiling-gated.ps1` already gates on the tenant's measured cores rather than its existence and needs no change. And a `compare` check should refuse on `tenant_cores_median`, a load figure, rather than on presence.
>
> **The census's achieved interval is now 3.92 s against the 1.0 asked for**, because measuring the tenant's rate inserts a one-second read. The parameter has been wrong twice in this script for two unrelated reasons, so the timestamps are the only trustworthy source of it.

> **2026-08-12 19:46 — this workload cannot unpark this box at any count, and that is a fact about the instrument.** Each rung gated on a quiet neighbour and refused if one arrived:
>
> | sessions | parked median | >= 8 | machine | tenant |
> |---|---|---|---|---|
> | 0 | 12 | **100%** | 2.54 | 0.00 |
> | 1 | 10 | **100%** | 4.93 | 0.00 |
> | 5 | 9 | **100%** | 5.09 | 0.00 |
> | 20 | *skipped, no quiet window* | | | |
> | 60 | 10 | **100%** | **5.09** | 0.00 |
>
> **Nine to twelve cores stay parked in every sample from zero to sixty sessions**, and the machine plateaus at **5.09 cores from five sessions onward** — sixty sleepless processes receive exactly what five do.
>
> **Against Chromium at ~10 cores unparking the same box to 14+, the difference is not demand.** Sixty processes asking for sixty cores get five; one browser asking for ten gets fourteen. Whatever the policy responds to, this workload does not have it.
>
> **The consequence is about `sessionbench`, not this laptop.** Its own workload cannot unpark the machine it benchmarks, so **every figure in the archive was taken in the low state unless something unrelated happened to be running** — which makes a browser a hidden input to the whole comparison set. That is why the 15.3-15.5 holds of 3 and 11 August look exceptional: they are the ones that happened to coincide with a busy neighbour.
>
> **Both defects that voided the first attempt are visible as fixed here.** Every rung reads `tenant 0.00`, where attempt 1 ran at 6.4-9.9 throughout; and `parked 12` beside `machine 2.54` is arithmetic that holds, where attempt 1's `parked 11` beside `machine 11.40` could not be true of one moment. **The skipped rung is the design working** — twelve minutes of waiting that produced no data beats a rung that measured the browser.

> **2026-08-12 19:57 — what the browser has that sixty processes lack, queried in one window with both present.**
>
> ```
> chrome   : ControlMask=0x4  StateMask=0x0
> cpu-spin : ControlMask=0x0  StateMask=0x0
> ```
>
> | candidate | verdict |
> |---|---|
> | process priority | **eliminated** — both `Normal` |
> | raw thread count | **eliminated** — 60 sessions carry **180** threads against Chromium's 134, and the box declines to unpark for more threads than the browser has |
> | job-object membership | **eliminated** — the ladder's processes were in no job and behaved identically to the job-managed hundred |
> | EcoQoS execution-speed throttling | **eliminated** — bit `0x1` is set on neither |
> | **timer resolution** | **the surviving lead** |
>
> **Chromium sets `0x4` — `IGNORE_TIMER_RESOLUTION` — with StateMask 0, an explicit request that its timer-resolution changes BE HONOURED. `cpu-spin` declares nothing.** So the browser shortens the system tick and this workload does not.
>
> **That fits the asymmetry rather than merely being available.** Core-parking decisions are evaluated on a timer, this box parks within one 3-second sample of a departure and takes 6-9 s to unpark, and [`timeBeginPeriod` is already recorded here as per-process on this build with a 15.64 ms floor](2026-08-12-114500-every-hundred-session-hold-today-is-a-third-of-what-the-box-used-to-do.md). A workload that never shortens the tick may never be re-evaluated often enough to earn cores back.
>
> **What this does not establish**: that timer resolution is the cause. It is one declared difference between two processes that behave differently, which is a lead rather than a mechanism — the same standing the parking correlation had before the 2x2. Testing it means making a workload raise the timer and re-running the ladder, and `unsafe_code = "forbid"` at the workspace root puts `timeBeginPeriod` out of reach of any crate inheriting the lints.

> **2026-08-12 20:10 — the timer lead is weakened by re-reading the report it rested on.** Every `chrome-headless-shell.exe` mention in the full energy report sits in the **CPU-utilisation** section (PID 36476 at 44.72%), not a timer section, and the only millisecond figure in the whole document is **`15.6ms`** — the unraised default.
>
> **So the platform timer appears to sit at its floor while Chromium runs and the box is unparked.** If that figure is the platform timer resolution, the browser unparks this machine WITHOUT shortening the tick, and the mechanism promoted an hour ago is wrong.
>
> **What my earlier count actually measured**: 27 hits of the Korean word for *timer* and 23 for *chrome*, in one 12,000-character document, which I read as the same section. They are not. **Counting two terms in one file and concluding they co-occur is a co-location error**, and it survived because both counts were large enough to feel like evidence — the localisation fix corrected the search and not the inference built on it.
>
> **Status: weakened, not eliminated.** `ControlMask=0x4` says Chromium wants its timer-resolution requests honoured, which is not the same as currently making one. Confirming the 15.6ms figure IS the platform timer resolution, rather than something else the report mentions once, is what would close it — and that is a re-read rather than a run.
>
> **Which leaves the mechanism open again**: four candidates eliminated, the fifth weakened, and a browser that unparks a box sixty CPU-bound processes cannot.

> **2026-08-12 20:15 — timer resolution is ELIMINATED, from the report's own Platform Timer Resolution section.**
>
> ```
> default platform timer resolution : 15.6ms (15625000ns)
> current timer resolution (100ns)  : 156250   ->  15.625 ms
> ```
>
> **The current platform timer equals the default while Chromium runs and the box is unparked.** So the browser unparks this machine with the tick at its floor, and the candidate promoted an hour ago is not merely weakened but wrong.
>
> **What makes this conclusive where the earlier read was not**: the section states the DEFAULT and the CURRENT value separately and they are equal. A lone `15.6ms` was ambiguous — a description of the default, or a measurement of now, and no way to tell. **Two numbers that happen to match say something one number cannot.**
>
> **The tally after five rounds**: process priority, raw thread count, job-object membership, EcoQoS execution-speed throttling and timer resolution are all eliminated. One structural difference remains untested — **concentration**, 27 threads per process against 3, and 5 processes against 60 — which is the only candidate a scheduler classifying per process would still see, and the only one with no cheap query behind it. `cpu-spin` is single-threaded, so testing it needs a workload that does not yet exist.

> **2026-08-12 20:17 — the load split replicates, and the base rate prices every measurement on this box.** An independent 402-sample census:
>
> | | this census | the earlier one |
> |---|---|---|
> | tenant absent | **97%** parked (n=151) | 100% |
> | present, **idle** | **60%** (n=5) | 75-83% |
> | present, **busy** | **4%** (n=246) | 2-4% |
> | **busy neighbour present** | **61% of the window** | not measured |
>
> **Two independent windows agree within a few points on all three arms**, which is a replication rather than a re-analysis of one dataset. The idle arm stays thin at 5 samples in both, so its 60-83% spread is the weak leg either way.
>
> **The new figure is the last row.** A busy neighbour is present **61%** of the time, so roughly two runs in five land on a machine offering ~5 cores and three in five on one offering ~14 — decided by a browser. That is why the archive's hundred-session holds fall into two clusters rather than scattering, and why any future comparison needs the tenancy gate rather than luck.

> **2026-08-12 21:10 — the idle arm settles, and the busy arm splits by whether I was working.** A fourth census, 138 samples, taken while the agent ran builds and gate runs:
>
> | | this window | windows with the machine otherwise idle |
> |---|---|---|
> | tenant absent | 100% parked (n=38) | 97-100% |
> | tenant present, **idle** | **100%** parked (**n=15**) | 60-83% (n=5) |
> | tenant present, **busy** | **55%** parked (n=83) | **2-4%** |
>
> **The idle arm is now unambiguous.** Fifteen samples at 100% parked, where every earlier window had five and spread 60-83%. A present-but-idle browser parks this box exactly like an absent one, which is the load-not-presence finding with the thin leg finally filled.
>
> **The busy arm is a factor of thirteen apart, and it is reproducible.** 55% here; **52%** in the census taken during a hundred-session hold. Both windows where the agent's own work overlapped read 52-55%; both windows where the machine was otherwise idle read 2-4%. Two observations a side, split by the same variable.
>
> **So the agent's own activity appears to push this box TOWARD parked while the neighbour is busy** — the opposite of a workload demanding cores, and the same direction as a hundred sessions failing to unpark it at any count. It points back at the concentration question rather than away from it: whatever the scheduler classifies, my processes seem to be counted on the side that argues for fewer cores.
>
> **Not yet a finding.** Two windows a side, the split is by "was the agent working", which is not a controlled variable, and both busy-arm windows contain different work (builds and gates here, a session hold there). What would settle it is the census run twice in one sitting with a fixed synthetic load present in one arm and absent in the other — the same within-window discipline that settled tenancy itself.

> **2026-08-12 21:22 — the observer split is WITHDRAWN, and the threshold gap turns out to be populated near zero.** A fifth census, 315 samples:
>
> | | |
> |---|---|
> | tenant absent | **96%** parked (n=84) |
> | present, **idle** | **97%** parked (n=29) |
> | present, **busy** | **25%** parked (n=199) |
> | `tenant_cores` strictly between 0 and 5 | **72 samples**, at 0.031-0.125 cores |
>
> **The busy arm is unstable across windows: 2-4%, 25%, 52%, 55%.** An hour ago two windows reading 52% and 55% were split from two reading 2-4% by whether the agent was working, and written up as reproducible. **This window was also worked through and reads 25%**, which is neither cluster. Two observations a side is not a reproduction, and the split is withdrawn — the honest statement is that the busy arm varies by an order of magnitude for reasons not yet identified.
>
> **What survives untouched is the pair the split was never about.** Absent and present-but-idle both park at 96-97%, now on 84 and 29 samples — the load-not-presence finding, with the idle arm no longer thin in any window.
>
> **And the threshold gap is populated, but not where it would matter.** 72 samples fall strictly between 0 and 5 cores, which had been empty in every earlier survey — and they sit at **0.031 to 0.125**, hugging zero rather than spreading through the range. So `TENANT_BUSY_CORES = 0.5` still separates the same pairs, and these land on the idle side, which their 97% parked rate says is right. The constant remains under-determined and is now known to be under-determined for a better reason: the population is bimodal at 0 and ~8.7 with a tail just above zero, not a gap waiting to be filled.

> **2026-08-12 21:26 — the busy arm was never unstable. It is a dose-response, and it indicts the threshold.** Pooling the busy samples of all five censuses:
>
> | tenant cores | parked | machine cores |
> |---|---|---|
> | 9.76 | **4%** | 12.05 |
> | 8.71 | **4%** | 10.84 |
> | 7.70 | 26% | 10.66 |
> | 7.45 | 25% | 10.48 |
> | 4.56 | **54%** | 6.59 |
>
> **Monotonic across every window: the more the neighbour holds, the less the box parks**, and the machine total follows continuously from 6.59 to 12.05 cores. The spread withdrawn an hour ago as unexplained instability was this relationship seen through a bucket — a graded quantity split into busy/not-busy, which then disagreed with itself window to window depending on how hard the browser happened to be working.
>
> **That is a better correction than the withdrawal it replaces.** The withdrawal was right that two points are not a reproduction; it was wrong to reach for the agent's activity, when the census already recorded the variable that explains it and nothing had pooled the windows.
>
> **And it indicts `TENANT_BUSY_CORES = 0.5` directly.** A ramp beside a 4.56-core neighbour and one beside a 9.76-core neighbour both classify as *busy* and pass `compare`, while running on machines offering **6.59 and 12.05 cores** — nearly a factor of two, which is most of the 2.8x the check exists to catch. The threshold is not merely under-determined, it is in the wrong place: a shared side of one bar does not mean a shared machine when the effect is graded.
>
> **The likely fix is a DIFFERENCE rather than a bar** — refuse a pair whose tenant loads differ by more than some margin, the way the solo-agreement check already works on rates. That has the same shape as a check this repository already trusts, and it needs the margin measured rather than chosen.

> **2026-08-12 21:30 — the neighbour's cores pass through to the machine almost one for one, across 1085 samples.** Fitting every census sample that carries both columns:
>
> ```
> machine_cores = 0.937 * tenant_cores + 3.119     r = 0.912   r2 = 0.831
> ```
>
> **A slope of 0.937 says the browser is not competing for cores, it is unlocking them** — nearly every core it holds is a core that would otherwise be parked. The intercept of **3.12** is what this box delivers with no neighbour at all, which is consistent with the ~5-core holds once the agent's own share is added.
>
> **r2 = 0.831 on 1085 samples: the neighbour's load explains 83% of the variance in what this machine delivers.** That is a stronger statement than the 2x2 that established the direction, and it is drawn from samples the censuses were already writing.
>
> **It also converts the outstanding threshold question from taste to arithmetic.** A tenant difference of 1 core costs 0.94 machine cores, 2 costs 1.87, 3 costs 2.81 — so a margin for a difference-based comparison check can be set against what the solo-agreement allowance already tolerates, rather than chosen and defended afterwards.
>
> **What this does not establish**: causation in the direction assumed. The fit is consistent with the neighbour unparking the box, and equally consistent with something unparking the box and letting the neighbour take more. The 2x2 and the arrival-lag data argue for the first — parking follows a departure within one sample — and the fit alone cannot distinguish them.

> **2026-08-12 21:53 — the fit predicts a window taken after it was derived.** A sixth census, 310 samples, 304 of them beside a busy neighbour:
>
> | | |
> |---|---|
> | mean tenant load | **8.02 cores** |
> | mean machine cores | **10.57** |
> | predicted by `0.937 x tenant + 3.119` | **10.63** |
> | parked | **13%** |
>
> **0.6% out, out of sample.** The fit was derived from the five earlier censuses and this window was recorded afterwards, so it is a prediction rather than a restatement of the points it was drawn through — the difference between a curve fitted to data and a curve that forecasts.
>
> **And its parked rate falls where the dose-response says it should**, at 13% between the 25% seen at 7.45-7.70 tenant cores and the 4% seen at 8.71-9.76. The relationship holds on a sixth window without being tuned to it.

> **2026-08-12 22:12 — the archive breaks the fit, and the out-of-sample check was not out of sample.** Two hundred-session holds on disk:
>
> | hold | job | rest | machine | the fit predicts |
> |---|---|---|---|---|
> | `statepair2-conc` | **15.505** | 0.495 | **16.00** | ~3.6 |
> | `slowstate-ratio` | **15.340** | 0.768 | **16.11** | ~3.8 |
>
> **Sixteen cores with no neighbour, where `0.937t + 3.119` predicts 3.6** — wrong by a factor of **4.4**, and wrong on the two best runs in the archive, which are the 3 and 11 August holds this whole question started from.
>
> **So the fit is a law of the state this box has been in today, not of the hardware.** It was built from six censuses taken within a few hours and confirmed by a seventh taken minutes later — which is out of sample in *time* and inside the sample in *state*. That is the shared-input trap in a new costume: two routes that agree because they share the thing that varies.
>
> **The intercept is the clearest casualty.** 3.12 cores was read as *what this machine offers with nothing else running*, and the archive says a quiet hundred-session hold once got 16.00. The intercept describes what the box offers **while parked**, and parking is the state, not the floor.
>
> **What survives**: within today, tenant load and delivered cores move together with r² 0.831 across 1085 samples, and the dose-response reproduced on six windows. That is a real relationship in a real state. **What does not**: any use of it as `C` in the gate's arithmetic, or of 3.12 as this machine's own core count.
>
> **And the archive is the control this needed all along** — 37 hundred-session holds with occupancy, spanning 4.52 to 16.11 machine cores. Eleven sit under 7. The question the fit cannot answer is what puts the box in one regime or the other, which is where this record started.

> **2026-08-12 22:18 — the archive splits by DAY, and the collapsed quantity is the JOB's share, not the machine's.** All 37 hundred-session holds with occupancy, grouped:
>
> | day | n | job cores | rest |
> |---|---|---|---|
> | 08-03 | 3 | 2.40 - **15.34** | 0.77 - 13.60 |
> | 08-11 | 4 | 4.34 - **15.51** | 0.49 - 10.24 |
> | **08-12** | **30** | 0.00 - **5.95** | 0.55 - 14.70 |
>
> **Every high hold on 3 and 11 August reached a machine total of 16.00-16.11. Today's highest is 14.70.** And the job's own share fell from 15.5 to a ceiling of **5.95 across thirty holds**, while `rest` still reaches 14.70 — so the box still delivers cores today; they stop reaching the sessions.
>
> **That is a second phenomenon, and the larger one.** Parking moves the machine TOTAL between about 5 and 14 and is now well characterised. It cannot explain this: on 11 August the job took **15.51 cores with `rest` at 0.49**, a quiet box handing everything to the workload, where today's best quiet hold gives the job **4.6**.
>
> **Neither the plug nor the thermal sensor separates the groups**: no hold in either was on battery, and the low group sits at a single thermal reading of 44.1 while the high group spans 39.1-63.1 — the sensor moves between days and not within them, which was already recorded.
>
> **So the night's work characterised the smaller effect.** What changed between 11 and 12 August, such that a hundred sessions can no longer take more than six cores on a machine still capable of delivering fourteen, is the question this record opened with and it is still open — now with the collapse located in the job's share rather than the machine's total, which is a different search.

> **2026-08-12 22:21 — the sessions are all there, so the collapse is not attrition.** Across all 37 hundred-session holds, on every day:
>
> | day | `fewest_running` | `peak_processes` |
> |---|---|---|
> | 08-03 | **100** | 101 |
> | 08-11 | **100** | 101 |
> | 08-12 | **100** | 101 |
>
> **Every hold kept every session.** Nothing dies, nothing fails to start, and the session mode is unchanged — so a hundred identical processes, spawned the same way on the same binary lineage, took **15.5 cores on 11 August and take 5.9 today**. The cheapest explanation is eliminated.
>
> **An eighth census confirms the fit and adds nothing.** Tenant 9.49 predicted 12.01 machine cores against a measured 12.21, +1.7% — which is another sample of today's state, and today's state already has seven. Recorded so the count is honest rather than to strengthen anything: the boundary this needs to cross is the day, and no run taken now can cross it.

> **2026-08-12 22:28 — the day boundary is `--resident`, and it was in every artifact the whole time.** The best hold of each day, read with its argv:
>
> | day | best job | workload |
> |---|---|---|
> | 08-03 | 15.53 | `--duty 0.27 --resident 20` |
> | 08-11 | 15.51 | `--duty 0.27 --resident 20` |
> | **08-12** | **5.95** | `--duty 1.0 --resident 1` |
>
> **All thirty of today's holds ran `--resident 1`. Every 15.5-core hold ran `--resident 20`.** Split across all 37, on QUIET holds only (`rest` < 1.5) where the machine caps nobody:
>
> | | job cores |
> |---|---|
> | `--resident 20` | **15.34, 15.51** |
> | `--resident 1` | **4.76, 3.12, 4.59, 4.55** |
>
> **A 3.3x lever, on a quiet box.**
>
> **And this was raised and withdrawn earlier the same day.** A back-to-back pair gave 4.675 against 4.811 and `--resident` was declared irrelevant — but that pair ran at `rest` ~ 9, where the machine holds both arms near 4.7 and no lever can show itself. The refutation was right about its regime and generalised past it, which is the invariant-exercised-only-where-it-cannot-break error. **The ten `--resident 20` holds that overturn it were on disk throughout.**
>
> **So the ceiling never collapsed.** Today's holds ran a different workload, and every reading in this record that compares today against 3 or 11 August compares two workloads as well as two days. The parking findings stand as measurements of the machine; what they cannot carry is the ceiling question, which was a workload difference.
>
> **What is still real and unaffected**: parking follows the neighbour's load, the dose-response, the arrival and departure lags, and that this workload cannot unpark the box at any count — all measured within today at fixed `--resident 1`.

> **2026-08-12 22:38 — the `--resident` correction is WITHDRAWN. It does nothing today, and the day difference is real.** The argv that produced 15.5 job cores on 11 August, run now on a quiet box:
>
> ```
> job 3.631   rest 2.148   tenant 0.000   machine 5.779
> units/session/s 1.379   peak_rss 2.36 GiB   fewest_running 100
> ```
>
> | | job cores |
> |---|---|
> | archive `--resident 20` (08-03, 08-11) | **15.34, 15.51** |
> | **today `--resident 20`** | **3.63** |
> | today `--resident 1` | 3.12 - 4.76 |
>
> **`--resident` makes no difference today.** 3.63 sits inside the band `--resident 1` already occupies, so the archive's resident split was itself confounded by day — every `--resident 20` hold on disk was taken on 3 or 11 August, and every `--resident 1` hold today. **The mirror of the confound it was raised to fix**, flagged before this ran and fired anyway.
>
> **So the original reading stands: the ceiling collapsed between 11 and 12 August, and it is the machine rather than the workload.** Three reversals on this axis in one evening — `--resident` matters, then it does not, then it does, then it does not — each correcting the last on evidence that could not separate the variables. What finally separated them was holding the day fixed and varying the flag, which is the only design here that ever could.
>
> **What this restores**: every parking finding, which was always measured within today at a fixed flag; the observation that today's box caps a hundred sessions near 5 cores whatever they ask for; and the open question of what changed between 11 and 12 August, which is where this record started and is still where it stands.
>
> **An instrument note worth keeping**: the verdict line of this run crashed on an em-dash, because the console encodes cp949 and cannot represent `—`. The figures printed only because they came first. **Put the numbers before the interpretation in any script that prints both** — a formatting crash after the data is an inconvenience, and before it is a lost run.

> **2026-08-12 22:45 — the whole night in one table: the PARKING BEHAVIOUR changed between 11 and 12 August.** Every quiet hundred-session hold on disk, tenant absent in all six:
>
> | day | job | rest | machine |
> |---|---|---|---|
> | 08-03 | 15.34 | 0.77 | **16.11** |
> | 08-11 | 15.51 | 0.49 | **16.00** |
> | 08-12 | 3.12 | 1.40 | **4.52** |
> | 08-12 | 4.55 | 0.82 | **5.37** |
> | 08-12 | 4.59 | 0.73 | **5.33** |
> | 08-12 | 4.76 | 0.55 | **5.31** |
>
> **Same condition, same workload family, and the box delivers 16 cores then and 4.5-5.4 now.**
>
> **So tenancy was never the cause — it is the current TRIGGER for a behaviour that switched on between those days.** Today an absent neighbour means a parked box, measured at 97-100% parked across two censuses. On 11 August an absent neighbour meant sixteen cores. The `0.937t + 3.119` law describes today's policy, and on 11 August that policy was not in force, which is why the fit failed by 4.4x against the archive.
>
> **Everything measured tonight survives as a description of today** — the dose-response, the arrival and departure lags, the load-not-presence split, that this workload cannot unpark the box at any count, and that `--resident` is inert. All of it was taken after the switch.
>
> **And the question is narrower than when this record opened.** Not why a hundred sessions get few cores, but **why this box began parking cores at all between 11 and 12 August** — with every environmental candidate the artifacts record already eliminated: OS build, power plan, Defender engine, core count, elevation, the plug, the thermal sensor, session attrition, and eighteen instrument commits.

> **2026-08-12 22:48 — two Windows updates installed on 12 August, invisible to every artifact.**
>
> ```
> KB5121003   installed 2026-08-12
> KB5123304   installed 2026-08-12
> ```
>
> **The day the parking behaviour changed, and the OS build string did not move**: every hold in both regimes records `11 (26200)`. The machine changed underneath the only version field the host block carries.
>
> **This is a candidate, not a cause.** The install timestamp is date-only, so it cannot be ordered against the first low hold, and a correlation with a day is exactly what this evening has spent itself punishing. What makes it worth recording is that it is the first candidate with a plausible mechanism to survive the eliminations the artifacts already made — updates routinely carry power and scheduler policy, and nothing else that changed between the days does.
>
> **The durable finding is about the instrument.** `doctor` records an OS build string, and a build string is not a configuration fingerprint: the archive cannot tell two machines apart that report the same one. Every cross-day comparison in this repository rests on a field that did not notice the largest environmental change it has ever recorded.

> **2026-08-12 23:18 — what those updates are, checked locally and uninformative.**
>
> | KB | description | installed by |
> |---|---|---|
> | KB5121003 | Security Update | `NT AUTHORITY\SYSTEM` |
> | KB5123304 | Security Update | `NT AUTHORITY\SYSTEM` |
>
> **Neither supports nor undermines the candidate**, and that is worth recording rather than leaving the question looking unexamined. Windows files cumulative packages containing kernel and power-management changes under exactly this label, so the classification carries no signal about whether either touched core parking.
>
> **What it does add is that both were installed unattended by SYSTEM** — no prompt, no reboot anyone would have noticed. That is consistent with a machine whose behaviour changed between two mornings with nothing visible happening, and it is the best available explanation for why the change went unnoticed for a day while its effects were attributed to the workload, the daemon, the thermal state and the neighbour in turn.

> **2026-08-12 23:24 — the KB article names a power-management change, and it is the mechanism this needed.** KB5121003 is the August 2026 cumulative for Windows 11 25H2, taking the build to **26200.9168**, and its notes list under Power/battery:
>
> > *"sleep, display, and power setting changes now apply correctly across all power plans"*
>
> **That is a change that switches core parking on.** This box runs a VENDOR scheme, SAMSUNG MODE, whose parking values are hidden and have never been readable here. If power settings previously did not apply correctly and now do, values that were always present but inert begin taking effect — with no visible action, no reinstall, and no change to the scheme's name.
>
> **It fits every constraint the artifacts imposed.** The scheme name is identical across both regimes; the build string did not move because only the REVISION changed, 26200.**9168**, which the host block does not capture; nothing was reinstalled; and the switch happened overnight between two mornings with the update applied unattended by SYSTEM.
>
> **And KB5123304 is retired as a candidate**: it is the bundled servicing-stack update that ships alongside the cumulative, not a separate change.
>
> **What would confirm it** is reading the parking values inside SAMSUNG MODE and finding them non-default — which needs an attribute change to expose hidden settings and is a configuration change to this machine, not a measurement. **What this is**: a named mechanism from the vendor's own release notes that survives every elimination the archive made, which is as far as evidence goes without touching the box.
>
> **The instrument lesson sharpens.** `doctor` records `os_version` as `11 (26200)` and drops the revision — so even a field designed to catch this would have missed it. Recording the update IDs is what closes the gap, and it is now in every artifact.

> **2026-08-13 00:05 — the new column validates against the quantity it explains.** The first three holds to carry `parked_fraction`:
>
> | `parked_fraction` | machine cores |
> |---|---|
> | 1.00 | 3.60 |
> | 0.81 | 5.90 |
> | 0.56 | 6.47 |
>
> **Monotonic, and in the direction the mechanism requires** — the more of a hold that ran with half the box parked, the fewer cores it was given. Three points is not a fit and is not offered as one; what it establishes is that the column moves with the thing it was added to explain, which is the cheapest check available on a new instrument and the one most easily skipped.
>
> **What every artifact before these three cannot do**: distinguish a machine that was busy elsewhere from one that was switched off. Both read as a low `machine_cpu_percent`, and separating them cost this evening four hours and three reversals.

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

## 2026-08-13 00:52 — the concentration test needs the gate, and the quiet arm is an n=18 reading

`cpu-spin --threads` shipped, so the concentration question was runnable. The first attempt ran it without the ladder's gate and learned nothing:

| | reading |
| --- | --- |
| tenant at start | 5 processes present |
| parked before | 0 of 16 |
| parked during 5 procs x 27 threads | 0 at all eight samples |
| machine during | 14.93 cores |
| threads per process | 29, so the shape was achieved |

The workload did what it was asked. The box was already at zero parked cores with the tenant present, so a treatment whose only possible effect is unparking had nothing left to unpark. **A control already sitting at the outcome's ceiling cannot fail**, which is why the dose ladder gates each rung on a quiet machine — and the ad-hoc run skipped the one part that makes the ladder a measurement. The gated version now takes `-Threads` and records it per rung, so a concentration arm and a count arm cannot be confused in one artifact.

The census that ran alongside it reports the two arms again:

| census | n | quiet n | quiet parked>=8 | busy parked>=8 | busy machine | busy tenant |
| --- | --- | --- | --- | --- | --- | --- |
| 23:43 | 291 | 95 | 89% | — | 11.78 | 8.22 |
| 00:19 | 71 | 18 | 83% | 4% | 13.51 | 8.83 |
| 00:48 | 318 | 18 | 44% | 7% | 12.52 | 8.61 |

The busy arm is stable across all three — a loaded box does not park, at 4% and 7%. **The quiet arm is not**: 89%, 83%, 44%, and the two that disagree most are the two with `quiet n = 18`. The 95-sample reading and the 18-sample reading are not the same quality of number, and nothing in the earlier appends said which was which. So the direction stands — quiet parks, busy does not — and **any specific quiet-side percentage in this record carries an error bar wide enough to hold 44 and 89 at once** until a census accumulates quiet samples in the hundreds. The busy arm gets there in one run because this box is busy; the quiet arm is rare here by construction, which is the same reason it is the harder one to measure.

## 2026-08-13 01:10 — SUPERSEDED: this workload CAN unpark the box, at five sessions

Four appends above say *this workload cannot unpark this box at any count*, and it is now false. It was true of every workload that existed when it was written — all of them one thread to a process. [Five processes of 27 threads take the box from 11 parked cores to 2](2026-08-13-011000-the-box-unparks-for-threads-not-for-processes.md), tenant absent, in the same window as a five-process one-thread arm that reaches 7.

**The count ladders were not wrong, they were answering the wrong question.** Sixty processes asking for more than Chromium consumes leave the box parked because size is not what the policy reads. What the sentence should have said is that this workload could not unpark the box **by adding processes**, which is a fact about the arrangement rather than about the amount — and the flag that varies arrangement did not exist until tonight.

The neighbour section above is affected the same way. *What the neighbour IS* is no longer the sharpest open question left; it is not a question at all, since 27 threads were picked to match `chrome-headless-shell` and reproduce its effect with no browser present.

## 2026-08-13 01:45 — the supersession above is itself withdrawn

The append at 01:10 says four earlier appends are false and this workload can unpark the box. [The arm it rests on did not replicate forty minutes later](2026-08-13-011000-the-box-unparks-for-threads-not-for-processes.md), reading 8 parked cores and 5.83 machine cores where it had read 2 and 9.12, tenant absent from both.

So *this workload cannot unpark this box at any count* goes back to standing, with its scope made explicit rather than restored silently: **it is established for adding PROCESSES, which is the only axis any replicated ladder here has varied.** Whether adding threads does something different is open and is the one live question, not a settled correction to it.

The neighbour's identity returns to being an open question with it.

