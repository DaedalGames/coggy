# conhost and Defender, per session · 2026-07-30

**This is not the M0 baseline.** [Gate G0](../../ROADMAP.md#current-priority-m0--attribution) freezes an as-is redline taken against a real generation session, and [the first integrity rule](../../sessionbench/README.md#keeping-it-honest) allows that freeze exactly once. The harness does not exist yet, so spending the freeze on a synthetic workload would burn it.

What this record holds instead are the two figures that do not need the harness: **what a pseudoconsole costs**, which is a property of conhost rather than of the session, and **what Defender charges per megabyte written**, which is a rate the harness can later be multiplied against.

## Method

One workload, run four times — twice on pipes, twice under a pseudoconsole, alternating. Each run writes sixty 64 KiB files, one every 0.9 seconds, into a freshly generated directory, and prints a line per file.

```
sessionbench observe --label conhost-rN --interval 1 [--pty] -- \
  powershell.exe -NoProfile -ExecutionPolicy Bypass -File <workload>
```

The fresh directory is load-bearing. An earlier pair reused one path and reported Defender costing 4.72 s/min on pipes against 1.44 s/min under a pseudoconsole — a threefold difference that was entirely the second run meeting files Defender had already scanned. That pair is discarded and the rule is now [rule 4](../../sessionbench/README.md#keeping-it-honest).

## Results

| Run | Steady RSS | Processes | conhost | Defender startup | Defender steady |
|---|---|---|---|---|---|
| pipe · 1 | 78.33 MiB | 1 | 0 | 2.97 s | 7.28 s/min |
| pty · 1 | 88.43 MiB | 2 | 1 | 3.34 s | 5.78 s/min |
| pipe · 2 | 78.36 MiB | 1 | 0 | 3.18 s | 6.42 s/min |
| pty · 2 | 88.09 MiB | 2 | 1 | 3.12 s | 7.37 s/min |

Steady RSS is the median over each run's final quarter. Runs lasted ~55 s and produced 51 samples each.

**conhost costs 9.9 MiB resident per session** (10.10 and 9.73 across the two pairs). Within-mode spread is 0.03 MiB on pipes and 0.34 MiB under a pseudoconsole, so the difference is the pseudoconsole and not noise.

**Defender charges about 1.6 CPU-seconds per MiB written.** The workload wrote 3.84 MiB over 55 seconds, or 4.19 MiB/min, against a mean steady cost of 6.71 CPU-s/min. Pipe and pseudoconsole runs are indistinguishable on this axis once paths are fresh, which is the expected result — Defender scans files, and both modes write the same ones.

## What a hundred of these would cost

Linear, and therefore a floor. On this machine (16 physical cores, 31 GiB usable, so a 22 GiB RSS budget):

| | Pipes | Pseudoconsole |
|---|---|---|
| Total RSS | 7.65 GiB (35% of budget) | 8.62 GiB (39% of budget) |
| Processes | 100 | 200 |
| Defender | ~11.2 cores of 16 | ~11.2 cores of 16 |

## What this says about Decision 1

[Decision 1](../PLAN.md#four-core-decisions) proposes defaulting to pipes so that a hundred conhost processes become none, and the plan calls it the most dangerous untested belief in the document. It is now partly tested.

The saving is real, reproducible, and **small relative to what else is happening**. Dropping conhost returns 0.97 GiB, which is 4.4% of the RSS budget on a machine where the projected total already sits at 39% of it. RSS is not close to being the limiting condition here, so removing a slice of it changes no verdict.

Defender, over the same runs, projects to roughly 70% of the machine's cores. That is the term worth attacking, and [M3 already owns it](../../ROADMAP.md#m3--resource-governor) — as a five-minute implementation, filed behind three weeks of daemon work that buys a twentieth as much.

**This does not yet trigger the attribution rule.** That rule fires on the as-is redline, and the write rate here is a guess at what generation does rather than a measurement of it. Two things would change the reading: a session far heavier than 78 MiB makes conhost's share smaller still, while one that writes far less makes Defender's term collapse and could put RSS back in front.

**What holds regardless of workload** is the 9.9 MiB figure, because it is conhost's own footprint. For it to be worth half the project, the limiting condition has to be RSS and the sessions have to be light enough that 9.9 MiB is a large share of each. Neither is established, and the first is currently false on this machine.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| sessionbench | 0.0.0 at commit `2fcff61d4d30`, clean working tree |
| rustc | 1.97.1 (8bab26f4f 2026-07-14) |
| Measurement crates | portable-pty 0.9.0 · sysinfo 0.37.2 |
| Defender | real-time protection on, engine 1.1.26060.3008, **no exclusions configured** |
| Membership | job object · elevated |

Raw `samples.jsonl` and `run.json` are not committed. They are reproducible from the command above at that commit.
