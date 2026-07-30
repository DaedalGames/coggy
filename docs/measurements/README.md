# Measurements

What the instrument found, in the order it found it. Each record holds the numbers, the machine they came from, and the commit that produced them — so a figure quoted from here can be traced back to a run rather than to a memory.

## What M0 concluded

Six findings, on 16 cores / 31 GiB / Windows 11, ordered by what they decide rather than when they arrived. Each links to the record that owns it; nothing here is stated that is not measured there.

1. **An engine session costs what it costs only while cooking, and that is the state to govern.** [At rest it holds 8.97 MiB](2026-07-31-035111-between-builds.md), less than the synthetic workload that stood in for it. [While building it takes 3.27 GiB, but only one session per engine installation can build at a time](2026-07-31-034150-unreal-builds-serialise.md), so scheduling cannot change that number. [Cooking is both heavy and concurrent](2026-07-31-040348-cooking-is-the-governed-state.md) — 5.02 GiB at peak, two editors side by side, and a hundred of them wanting 193 GiB against a 22 GiB budget. It [takes the build lock on the way in](2026-07-31-042802-the-lock-reaches-cooking-too.md) and runs freely past it, so two sessions alternate through and ten queue. **The redline for an engine session is about eleven, and cooking is what sets it.** [Concurrent cooks sum to their steady figure rather than their peak](2026-07-31-043156-cook-peaks-scatter.md), so the governor counts sessions rather than scheduling them apart; [that figure is the editor loading itself rather than the content](2026-07-31-043951-the-editor-is-the-cost.md), so eleven holds for any project the editor still dominates; and [a Defender exclusion does not move the ceiling either](2026-07-31-041923-what-an-exclusion-buys-a-cook.md) — it is memory, and an exclusion returns CPU.

2. **For sessions that compute continuously, cores bind and the ceiling is `redline = 2ηC/d`** — inversely with the cores a session demands, proportionally with the machine's logical processors. [Derived rather than fitted](2026-07-30-154348-duty-is-derivable.md), and checked against a rung predicted before it was run. `d` above 1 is ordinary: [an agent turn's tool work takes 2.63 cores](2026-07-31-015246-what-a-session-costs.md) while it builds.

3. **A workload decides which condition you find.** Twenty megabytes of synthetic session put memory three orders clear and made cores the answer; [1.69 GiB of compiling Unreal](2026-07-31-022200-an-unreal-session.md) put memory first at thirteen sessions; nine megabytes of idle Unreal put it clear again. All three are correct about what they measured, and only the third is what a session mostly is.

4. **None of the four costs this project was scoped around is one.** Processes are cheap in memory — [dropping conhost returns 3.9% of the budget at a hundred sessions](2026-07-30-120002-first-redlines.md#what-dropping-conhost-is-worth-at-any-session-weight). Defender was [overstated by two orders of magnitude](2026-07-30-142218-defender-at-scale.md). Output has [three to four orders of headroom](2026-07-30-145414-output-path.md). [PLAN's own escape clause fired](../PLAN.md#residency-not-spawning) on that result.

5. **A redline from one ladder is worth ±12.5%**, from rungs that each reproduce within 2%. [The search was the noisy part rather than the measuring](2026-07-30-164912-redline-reproducibility.md), and solving the budget on a fitted slope brings the same seven runs to 2.3%. The fitted slope is `η`, which is memory contention between sessions rather than scheduling: ten sessions received every core they asked for, ran a fifth slower anyway, and left six idle doing it.

6. **A ramp changes the machine it measures.** A rung ran 4.8% slower at the end of one ladder than at its start, dragging the fitted redline down 3.9%. Every ramp now repeats a rung as a control, and [the rule that follows](../../CLAUDE.md) is to do nothing else on the box while one runs.

**What none of this settles is [G0](../../ROADMAP.md#current-priority-m0--attribution)**, which wants a real generation session this machine has never had. It is now two scalars and a ladder rather than a whole measurement — [the steps are written down](../../sessionbench/README.md#running-gate-g0-on-a-configured-machine).

| Record | What it answered |
|---|---|
| [What fixing the instrument was worth](2026-07-31-001314-instrument-corrections.md) | Before it could measure the machine it had to stop measuring itself: teardown 400× faster, a sampler tick 269×, a process refresh 2,346×. Every one of those was first read as a fact about Windows. |
| [conhost and Defender](2026-07-30-101141-conhost-and-defender.md) | The first session, end to end. A pseudoconsole costs a second process, and it belongs to whoever created it rather than to the session it serves — which is why killing a session does not take its console with it. |
| [The first redlines](2026-07-30-120002-first-redlines.md) | The ladder's first numbers, and pipes against pseudoconsoles at a hundred sessions. |
| [Duty and the redline](2026-07-30-130619-duty-and-redline.md) | Why one redline is not a property of the machine: `redline × duty ≈ 25` here, so a session that computes half the time doubles the count. |
| [The exclusion delta](2026-07-30-140251-exclusion-delta.md) | What a Defender path exclusion buys — at one session, nothing measurable, and the record is mostly about why the first answer looked otherwise. |
| [Defender at scale](2026-07-30-142218-defender-at-scale.md) | **Withdraws the cost estimated in the first record.** Fifty sessions writing 1,875 MiB a minute used 0.9 cores where the earlier figure demanded 51. |
| [The output path](2026-07-30-145414-output-path.md) | A hundred streams at half a gigabyte a second cost 1.7 cores of sixteen. The ceiling is about 7 GiB/s aggregate, and it is bandwidth rather than processors. |
| [The duty relation is derivable](2026-07-30-154348-duty-is-derivable.md) | **Supersedes the 84% share above.** The constant 25 was `2ηC` — with the processor count in it, which is what lets the relation speak for other hardware. Two ramps at different duties agree on `η` to within 0.4%, and a rung predicted in advance landed. |
| [The editor is the cost, not the content](2026-07-31-043951-the-editor-is-the-cost.md) | Ten times the assets moved steady RSS by 1.5%, which is below what the measurement resolves. Also locates the nineteen conhosts: they came from the target check on the way into a cook, and a flag avoids them. |
| [Concurrent cook peaks scatter](2026-07-31-043156-cook-peaks-scatter.md) | Four cooks reached 7.76 GiB where aligned peaks would have been 20, and the busiest sample divides to the single-session steady figure. Eleven sessions rather than six, and a governor that counts rather than schedules. |
| [The build lock reaches cooking too](2026-07-31-042802-the-lock-reaches-cooking-too.md) | Where the concurrency is: at startup the editor checks its target through the same per-installation lock a build takes, and runs freely once past it. Also the first measurement this machine was ruled out of in advance, with 8.7 GiB free against the 19 it needed. |
| [What a Defender exclusion buys a cook](2026-07-31-041923-what-an-exclusion-buys-a-cook.md) | 0.034 cores a session, against the write pattern scanning charges most for. Real, small, and irrelevant to a ceiling made of memory — which is the first verdict the sixth axis has produced rather than an inconclusive. |
| [Cooking is the state a governor has a job in](2026-07-31-040348-cooking-is-the-governed-state.md) | 5.02 GiB at peak, 47 processes, and two editors running side by side — unlike a build, it takes no per-installation lock. A hundred cooking sessions want 193 GiB, which puts the engine redline near eleven. |
| [Concurrent Unreal builds serialise](2026-07-31-034150-unreal-builds-serialise.md) | Ten sessions, one `cl.exe`. Two locks — `Build.bat`'s script guard, bypassable, and UnrealBuildTool's per-installation one, not. Also six defects on the way there, two of them the instrument breaking its own workload contract. |
| [An Unreal session, and the condition that actually binds](2026-07-31-022200-an-unreal-session.md) | The engine was installed the whole time. A build holds 1.69 GiB steady against a synthetic session's 20 MiB, so memory trips at thirteen sessions and cores would not until twenty — the reverse of what every earlier record concluded. |
| [What an agent session actually costs](2026-07-31-015246-what-a-session-costs.md) | The first reading of `d` from something other than a workload built to demand exactly one. A building session takes 2.63 cores, so the relation runs the other way from where it was exercised — and a hundred such sessions break both the core and the memory condition. |
| [How much the redline moves between identical runs](2026-07-30-164912-redline-reproducibility.md) | **Puts an error bar on every number above.** Seven identical runs gave 30, 31, 31, 31, 33, 34, 34 — the ladder is reproducible to ±13% from rungs that each reproduce within 2%. Solving the budget on a fitted slope instead brings the same seven to 2.3%. |

## The pattern these share

**Every axis returned a wrong answer the first time it was actually exercised, and every wrong answer was a defect in the instrument rather than a fact about the machine.** Sampling that starved under load. A membership test that pid reuse turned into an undercount. A drop detector that counted a pseudoconsole's own startup sequences as lost output, and then — once that was fixed — counted a fast session's own counters against each other. A Defender cost that two workloads agreed on because both were measuring the same background noise.

A condition that has been passing is not a condition that works; it may only be a condition that has never been reached. That is the argument for pushing each axis until something breaks, and it is why these records are worth more than the numbers in them.

## Reading one

**A record's name and title carry the moment it was written, not the moment the run happened.** `YYYY-MM-DD-HHMMSS-<slug>.md`, taken from the commit that added it, so the directory sorts into the order the findings arrived and does so identically on any clone. Where a run predates its write-up the provenance table says so — one record here is stamped past midnight for measurements taken the previous afternoon.

Figures are what the run emitted, not what was typed up afterwards — `ramp.md` and `run.md` are generated. The two earliest records predate that and were transcribed by hand; both carry corrections for exactly the reason that practice stopped.

A redline quoted without its session count, workload, mode, and hardware is not quotable. Every headline here carries all four.
