# conhost and Defender, per session · 2026-07-30 10:11:41

**This is not the M0 baseline.** [Gate G0](../../ROADMAP.md#current-priority-m0--attribution) freezes an as-is redline taken against a real generation session, and [the first integrity rule](../../sessionbench/README.md#keeping-it-honest) allows that freeze exactly once. The harness does not exist yet, so spending the freeze on a synthetic workload would burn it.

What this record holds instead are the two figures that do not need the harness: **what a pseudoconsole costs**, which belongs to conhost rather than to the session, and **what Defender charges per megabyte written**, which is a rate the harness can later be multiplied against.

## Method

Six runs of one workload, alternating pipes and a pseudoconsole. [`file-write`](../../workloads/file-write/) holds 80 MiB resident and writes sixty 64 KiB files at 900 ms intervals into a directory it creates for that run, printing one line per file.

```
cargo build --release -p file-write
cargo run -p sessionbench -- observe --label record --interval 1 [--pty] -- \
  ./target/release/file-write
```

Two details are load-bearing. The workload **holds memory** rather than merely allocating it, because a session that lets Windows trim its working set measures a machine nobody is running. And it writes into a **fresh directory**, because real-time scanning charges far less the second time a file is written — an earlier pair that reused one path reported Defender at 4.72 s/min against 1.44 s/min and the whole difference was the cache. That pair is discarded and the rule is now [rule 4](../../sessionbench/README.md#keeping-it-honest).

## Results

| Run | Steady RSS | conhost | Work rate | Defender startup | Defender steady |
|---|---|---|---|---|---|
| pipe · 1 | 84.11 MiB | 0 | 1.11 units/s | 3.02 s | 7.34 s/min |
| pty · 1 | 92.68 MiB | 1 | 1.09 units/s | 3.34 s | 8.22 s/min |
| pipe · 2 | 84.11 MiB | 0 | 1.10 units/s | 3.79 s | 7.48 s/min |
| pty · 2 | 92.67 MiB | 1 | 1.10 units/s | 3.27 s | 7.33 s/min |
| pipe · 3 | 84.11 MiB | 0 | 1.11 units/s | 3.38 s | 5.29 s/min |
| pty · 3 | 92.68 MiB | 1 | 1.10 units/s | 3.09 s | 5.21 s/min |

**A conhost costs 8.57 MiB resident.** Memory reproduces to 0.01 MiB across three runs in each mode, so the difference is the pseudoconsole and nothing else. An earlier pair driven by a PowerShell session instead put it at 9.9 MiB, which is worth keeping: conhost's footprint tracks what its client asks of the console, so **8.6 to 9.9 MiB** is the honest range and the low end is the one to plan against.

> **Withdrawn.** ~~Defender charges about 1.6 CPU-seconds per MiB written.~~ The workload wrote 4.18 MiB/min against a mean steady cost of 6.81 CPU-s/min, and two unrelated workloads agreed to within a few percent — which turned out to establish only that both runs were measuring the same background. **Defender is one machine-wide process, and attributing all of its CPU to the session under test attributes the whole machine to it.** [Fifty sessions writing 1,875 MiB a minute later used under one core in total](2026-07-30-142218-defender-at-scale.md), where this figure demanded fifty-one. It is wrong by more than two orders of magnitude and nothing downstream of it stands.

**Work rate is the same in both modes** — 1.105 against 1.098 units/s — which is the expected result at one session and the solo figure that [the work-rate condition](../../sessionbench/README.md#redline) will be compared against once the ramp exists.

Defender's steady cost varies between 5.2 and 8.2 s/min across otherwise identical runs. That spread is the machine, not the instrument: scanning arrives in bursts, and no amount of averaging inside one 55-second run removes it. Repetitions, not longer runs, are what tighten it.

## What a hundred of these would cost

Linear, and therefore a floor. On this machine — 16 physical cores, 31 GiB usable, so a 22 GiB RSS budget:

| | Pipes | Pseudoconsole |
|---|---|---|
| Total RSS | 8.21 GiB (37% of budget) | 9.05 GiB (41% of budget) |
| Processes | 100 | 200 |
| Cores wanted | 11.4 of 16 | 11.8 of 16 |

## What this says about Decision 1

[Decision 1](../PLAN.md#four-core-decisions) proposes defaulting to pipes so that a hundred conhost processes become none, and the plan calls it the most dangerous untested belief in the document. It is now partly tested.

The saving is real, reproducible, and **small next to what else is happening**. Dropping every conhost returns 0.84 GiB, which is 3.8% of the RSS budget on a machine whose projected total already sits at 41% of it. RSS is nowhere near limiting here, so removing a slice of it changes no verdict.

~~Defender over the same runs wants about 72% of the machine's cores.~~ **Withdrawn with the figure it rested on.** Defender's real cost at scale is small — see [Defender at scale](2026-07-30-142218-defender-at-scale.md) — so the conclusion that it was the term worth attacking, and that M3's exclusion work buys twenty times what the daemon does, has nothing behind it.

**This does not yet trigger the attribution rule.** That rule fires on the as-is redline, and this workload's write rate is a guess at what generation does rather than a measurement of it. Two things would change the reading: a session much heavier than 84 MiB makes conhost's share smaller still, while one that writes far less makes Defender's term collapse and could put RSS back in front.

**What holds regardless of workload** is the 8.6 MiB figure, because it is conhost's own footprint. For that to be worth half the project, the limiting condition has to be RSS and sessions have to be light enough for 8.6 MiB to be a large share of each. Neither is established, and the first is currently false on this machine.

## Do the redline conditions survive this?

Yes, which is the answer this run existed to give. [The conditions](../../sessionbench/README.md#redline) can be built into the ramp without rewriting them.

The shape found here is that memory has room and cores do not. Condition 2 would not trip at a hundred sessions, sitting at 41% of a 70% budget. Condition 1 would, because twelve of sixteen cores going to Defender leaves each session a fraction of the machine it had to itself. Condition 1 is already the one the metric leans on, so the instrument is pointed at the right thing.

One gap opens instead. `WorkRate` names a symptom rather than a cause, so a redline of `84 / WorkRate` does not say whether work rate fell to Defender, to contention between sessions, or to disk. Every run now records the session's cores beside Defender's for that reason — a redline has to name where the cores went, or the pair it is supposed to be collapses into a number with a label attached.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| sessionbench | 0.0.0 at commit `b3f50efdcc3c`, clean working tree |
| rustc | 1.97.1 (8bab26f4f 2026-07-14) |
| Measurement crates | portable-pty 0.9.0 · sysinfo 0.37.2 |
| Defender | real-time protection on, engine 1.1.26060.3008, **no exclusions configured** |
| Membership | job object · elevated |

Raw `samples.jsonl` and `run.json` are not committed. Both are reproducible from the commands above at that commit.
