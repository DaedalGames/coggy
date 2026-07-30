# Measurements

What the instrument found, in the order it found it. Each record holds the numbers, the machine they came from, and the commit that produced them — so a figure quoted from here can be traced back to a run rather than to a memory.

| Record | What it answered |
|---|---|
| [conhost and Defender](2026-07-30-conhost-and-defender.md) | The first session, end to end. A pseudoconsole costs a second process, and it belongs to whoever created it rather than to the session it serves — which is why killing a session does not take its console with it. |
| [The first redlines](2026-07-30-first-redlines.md) | The ladder's first numbers, and pipes against pseudoconsoles at a hundred sessions. |
| [Duty and the redline](2026-07-30-duty-and-redline.md) | Why one redline is not a property of the machine: `redline × duty ≈ 25` here, so a session that computes half the time doubles the count. |
| [The exclusion delta](2026-07-30-exclusion-delta.md) | What a Defender path exclusion buys — at one session, nothing measurable, and the record is mostly about why the first answer looked otherwise. |
| [Defender at scale](2026-07-30-defender-at-scale.md) | **Withdraws the cost estimated in the first record.** Fifty sessions writing 1,875 MiB a minute used 0.9 cores where the earlier figure demanded 51. |
| [The output path](2026-07-30-output-path.md) | A hundred streams at half a gigabyte a second cost 1.7 cores of sixteen. The ceiling is about 7 GiB/s aggregate, and it is bandwidth rather than processors. |

## The pattern these share

**Every axis returned a wrong answer the first time it was actually exercised, and every wrong answer was a defect in the instrument rather than a fact about the machine.** Sampling that starved under load. A membership test that pid reuse turned into an undercount. A drop detector that counted a pseudoconsole's own startup sequences as lost output, and then — once that was fixed — counted a fast session's own counters against each other. A Defender cost that two workloads agreed on because both were measuring the same background noise.

A condition that has been passing is not a condition that works; it may only be a condition that has never been reached. That is the argument for pushing each axis until something breaks, and it is why these records are worth more than the numbers in them.

## Reading one

Figures are what the run emitted, not what was typed up afterwards — `ramp.md` and `run.md` are generated. The two earliest records predate that and were transcribed by hand; both carry corrections for exactly the reason that practice stopped.

A redline quoted without its session count, workload, mode, and hardware is not quotable. Every headline here carries all four.
