# Measurements

What the instrument found, in the order it found it. Each record holds the numbers, the machine they came from, and the commit that produced them — so a figure quoted from here can be traced back to a run rather than to a memory.

## What M0 concluded

Five findings, on 16 cores / 31 GiB / Windows 11. Each links to the record that owns it; nothing here is stated that is not measured there.

1. **None of the costs this project was designed around is the bottleneck.** Processes are cheap in memory — [dropping conhost returns 3.9% of the budget at a hundred sessions](2026-07-30-first-redlines.md#what-dropping-conhost-is-worth-at-any-session-weight). Defender was [overstated by two orders of magnitude](2026-07-30-defender-at-scale.md). Output has [three to four orders of headroom](2026-07-30-output-path.md). [PLAN's own escape clause fired](../PLAN.md#residency-not-spawning) on that result.

2. **What binds is sessions competing for cores**, at `redline = 2ηC/d` — inversely with the share of its time a session computes, proportionally with the core count. [Derived rather than fitted](2026-07-30-duty-is-derivable.md), and checked against a rung predicted before it was run.

3. **`η` is memory, not scheduling.** Ten sessions received every core they asked for, ran a fifth slower anyway, and left six cores idle doing it — so `η` belongs to the workload as much as the machine and has to be measured rather than assumed.

4. **A redline from one ladder is worth ±12.5%**, from rungs that each reproduce within 2%. [The search was the noisy part, not the measuring](2026-07-30-redline-reproducibility.md); solving the budget on a fitted slope brings the same seven runs to 2.3%.

5. **A ramp changes the machine it measures.** A rung ran 4.8% slower at the end of one ladder than at its start, dragging the fitted redline down 3.9%. Every ramp now repeats a rung as a control, and [the rule that follows](../../CLAUDE.md) is to do nothing else on the box while one runs.

**What none of this settles is [G0](../../ROADMAP.md#current-priority-m0--attribution)**, which wants a real generation session this machine has never had. It is now two scalars and a ladder rather than a whole measurement — [the steps are written down](../../sessionbench/README.md#running-gate-g0-on-a-configured-machine).

| Record | What it answered |
|---|---|
| [conhost and Defender](2026-07-30-conhost-and-defender.md) | The first session, end to end. A pseudoconsole costs a second process, and it belongs to whoever created it rather than to the session it serves — which is why killing a session does not take its console with it. |
| [The first redlines](2026-07-30-first-redlines.md) | The ladder's first numbers, and pipes against pseudoconsoles at a hundred sessions. |
| [Duty and the redline](2026-07-30-duty-and-redline.md) | Why one redline is not a property of the machine: `redline × duty ≈ 25` here, so a session that computes half the time doubles the count. |
| [The exclusion delta](2026-07-30-exclusion-delta.md) | What a Defender path exclusion buys — at one session, nothing measurable, and the record is mostly about why the first answer looked otherwise. |
| [Defender at scale](2026-07-30-defender-at-scale.md) | **Withdraws the cost estimated in the first record.** Fifty sessions writing 1,875 MiB a minute used 0.9 cores where the earlier figure demanded 51. |
| [The output path](2026-07-30-output-path.md) | A hundred streams at half a gigabyte a second cost 1.7 cores of sixteen. The ceiling is about 7 GiB/s aggregate, and it is bandwidth rather than processors. |
| [The duty relation is derivable](2026-07-30-duty-is-derivable.md) | **Supersedes the 84% share above.** The constant 25 was `2ηC` — with the core count in it, which is what lets the relation speak for other hardware. Two ramps at different duties agree on `η` to within 0.4%, and a rung predicted in advance landed. |
| [How much the redline moves between identical runs](2026-07-30-redline-reproducibility.md) | **Puts an error bar on every number above.** Seven identical runs gave 30, 31, 31, 31, 33, 34, 34 — the ladder is reproducible to ±13% from rungs that each reproduce within 2%. Solving the budget on a fitted slope instead brings the same seven to 2.3%. |

## The pattern these share

**Every axis returned a wrong answer the first time it was actually exercised, and every wrong answer was a defect in the instrument rather than a fact about the machine.** Sampling that starved under load. A membership test that pid reuse turned into an undercount. A drop detector that counted a pseudoconsole's own startup sequences as lost output, and then — once that was fixed — counted a fast session's own counters against each other. A Defender cost that two workloads agreed on because both were measuring the same background noise.

A condition that has been passing is not a condition that works; it may only be a condition that has never been reached. That is the argument for pushing each axis until something breaks, and it is why these records are worth more than the numbers in them.

## Reading one

Figures are what the run emitted, not what was typed up afterwards — `ramp.md` and `run.md` are generated. The two earliest records predate that and were transcribed by hand; both carry corrections for exactly the reason that practice stopped.

A redline quoted without its session count, workload, mode, and hardware is not quotable. Every headline here carries all four.
