# The output path · 2026-07-30

The fourth axis — bytes absorbed and bytes dropped — had never been tested. The two existing workloads emit about twenty bytes per unit, so every run reported zero drops without the path ever carrying anything. [`stdout-storm`](../../workloads/stdout-storm/) is the payload-as-output workload vtebench's format was adopted for, and this is what it found.

## At a realistic rate, nothing happens

Sessions throttled to roughly 5 MiB/s each:

| Sessions | Per-session rate | Against solo | Aggregate | Cores | Dropped |
|---|---|---|---|---|---|
| 1 | 636.83 units/s | 1.00× | 5 MiB/s | 0.2 | 0 |
| 25 | 634.62 units/s | 1.00× | 124 MiB/s | 0.7 | 0 |
| 50 | 652.11 units/s | 0.98× | 255 MiB/s | 1.1 | 0 |
| 100 | 652.55 units/s | 0.98× | **509 MiB/s** | **1.7** | 0 |

**A hundred sessions moving half a gigabyte a second cost 1.7 cores of sixteen**, ran at solo speed, and dropped nothing. No rung came near a condition.

## Unthrottled, the ceiling is bandwidth rather than cores

| Sessions | Per-session rate | Against solo | Aggregate | Cores | Verdict |
|---|---|---|---|---|---|
| 1 | 172,327 units/s | 1.00× | 1.35 GiB/s | 0.6 | held |
| 10 | 97,477 units/s | 1.77× | **7.4 GiB/s** | 6.5 | held |
| 11 | 74,887 units/s | 2.30× | 6.4 GiB/s | 6.7 | broke |
| 13 | 58,899 units/s | 2.93× | 7.3 GiB/s | 6.6 | broke |
| 17 | 43,454 units/s | 3.97× | 7.0 GiB/s | 6.8 | broke |
| 25 | 37,083 units/s | 4.65× | 7.1 GiB/s | 6.8 | broke |

```
redline: 10 sessions (WorkRate) · stdout-storm · pipe · 16C/31GiB
```

**Aggregate throughput is flat at about 7 GiB/s from ten sessions upward**, and per-session rate falls in exact inverse proportion after that. The shape is the same one the core ceiling produced — but **cores plateau at 6.8 of 16, not at 15**. The machine is not out of processors; it is out of the thing that moves bytes between a process and a reader.

Nothing dropped at any rung, unthrottled included. A pipe blocks rather than discarding, which is what makes the writer slow down instead of the reader losing data — and it is why this axis reads as work rate rather than as dropped bytes.

## What this means for a session that is not this one

The ceiling is 7 GiB/s across all sessions, so at a hundred sessions the budget is **70 MiB/s each**. Agent CLIs emit text — kilobytes a second, sometimes tens. That is three to four orders of magnitude of headroom.

**Absorbing a hundred output streams is not a bottleneck for this workload class.** It would become one for a hundred sessions each streaming video, and for nothing this project intends to run.

## The axis found its own bug first

Unthrottled, the very first rung reported **six dropped units and a redline of zero**. A pipe does not drop, so that was a counting error, and it was: the line count was batched to the end of each read while the highest ordinal advanced per line, leaving a reader between them looking at a maximum that had run ahead. The ramp reads those counters every tick.

Fixed by ordering rather than locking — the count goes up first, so the transient subtracts to zero instead of to a drop. **The condition that tolerates zero dropped output produced its first false positive the moment it first saw real output**, which is the argument for testing a condition rather than trusting that it has been passing.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| sessionbench | 0.0.0 at commit `289af30`, clean tree, release build |
| Workload | `stdout-storm --line 8`, throttled with `--interval 1` and unthrottled |
| Holds | 60 s per rung, first 30 s unmeasured · resolution 2 |
| Output kept | 4 MiB per stream; the rest is counted as it passes and discarded |
