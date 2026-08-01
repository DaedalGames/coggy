# Gate M1 at twenty minutes

A hundred sessions under `coggyd`, held with a bracketed solo baseline on either side. **The two conditions this run can answer both hold**, and three separate things came close to leaving no number at all.

## The gate's own lines

| Condition | | |
|---|---|---|
| Total RSS under 4 GB | **2.392 GiB of 3.73** | Held |
| Work rate within 2× of solo | **1.257×** | Held |
| No dropped output | — | OutOfReach |
| Replacement under 60 s | — | OutOfReach |

The last two are `out_of_reach` rather than passed, for the reason [the harness states](../../sessionbench/README.md#running-it): under a daemon the reading end of each session's output belongs to the daemon, so nothing here can find a gap in the ordinals, and nothing in `coggyd` restarts a session that exited. Replacement is [the redline's fourth condition and not one of the gate's three](../../ROADMAP.md#m1--headless-daemon); it is reported because the same code answers both.

**Fewest running was 100 at every report across the whole hold**, and peak processes 101 — the daemon and one process a session, with nothing spawned and nothing lost.

## It is twenty minutes and the gate asks for sixty

The first attempt at the full hour **stopped the machine dead at forty-one minutes**: Windows event 41, `BugcheckCode 0`, no power button, no update, and no System log entries at all for the four minutes before it. On AC at 79%, so not a flat battery. A thin laptop holding sixteen cores at 100% reaches either the adapter's budget or its thermal limit, and both end exactly like that.

**A hundred `ping` sessions [held an hour on this same box](2026-08-01-103225-an-hour-of-a-hundred-sessions.md) the same morning**, so what this machine cannot sustain is the saturation the work-rate condition requires, not the session count. That is the distinction worth carrying: the gate's memory condition and its work-rate condition ask different things of the hardware, and only the second one is what stopped it.

So the measurement was shrunk rather than the machine blamed. RSS and work rate settle in minutes; what an hour buys is the sentence *held for an hour*, and that sentence is unearned here. **The failure is bracketed between twenty minutes, which completed, and forty-one, which did not.**

## The run happened at duty 0.172, and the gate is stated in duty

This is the caveat that outranks the verdict. [The gate's arithmetic](../../ROADMAP.md#m1--headless-daemon) makes the work-rate condition a function of the workload's duty before the daemon does anything, and this run did not hold the duty it was aimed at.

`--wait-ms 67` was calibrated by pre-flight against a unit measured at 24.9 ms, giving a solo cycle of 91.9 ms and a duty of 0.271 — [the measured driven duty of a real agent turn](2026-07-31-054657-the-driven-duty.md). That pre-flight ran minutes after forty-one minutes of full load. **After the crash and a cold boot the same unit takes 13.9 ms**, so the same fixed wait gives a solo cycle of 80.9 ms and a duty of **0.172**.

| | pre-flight, warm machine | this run, after reboot |
|---|---|---|
| unit's work | 24.9 ms | 13.9 ms |
| solo cycle | 91.9 ms | 80.9 ms |
| solo rate | 10.881 units/s | 12.356 units/s |
| duty | 0.271 | **0.172** |

**A 79% difference in compute speed showed up as a 13.6% difference in rate**, because sixty-seven of the eighty-odd milliseconds are a wall-clock constant no machine touches. That is the dilution a fixed wait produces, and it cuts both ways: it is why [a fixed-wait baseline is an order of magnitude quieter than a proportional one](2026-08-01-173927-the-baseline-is-the-noisy-term.md), and it is why the duty drifted here without the rate looking like it had.

So **`--wait-ms` must be recalibrated whenever the machine's state changes**, and a thermally loaded machine is a different machine. Scaling the observed result to the intended duty: 1.257 against a core-limited floor of `100 × 0.172 ÷ 16 = 1.075` gives `η ≈ 0.855`, which at duty 0.271 predicts **1.98× — inside 2, with 1% to spare.** The gate is marginal at the duty it is meant for and comfortable at the one it got, and this run does not settle it.

## Three repeats a side are what produced a ratio

The bracket's before side spread 3.98% and its after side 0.32%. One solo each side would have compared 11.828 against 12.549 — **a 5.92% gap, past the 5% allowance, and no ratio at all.** Averaged, the sides sit 3.36% apart and the run is judged.

| | rates | spread |
|---|---|---|
| before | 11.828, 12.305, 12.312 | 3.98% |
| after | 12.549, 12.553, 12.590 | 0.32% |

The outlier is the first hold of the run, cold behind a build. **[`--solo-repeats` was added this afternoon](2026-08-01-173927-the-baseline-is-the-noisy-term.md) from a finding about placement noise, and its first real outing is a run that would otherwise have refused itself.** The after side's 0.32% is the second reading of a fixed wait's baseline being quiet, after 0.47% earlier the same day.

## What the daemon costs, which is what this gate is for

[The gate measures the daemon rather than capacity](../../ROADMAP.md#m1--headless-daemon), so this is the headline rather than the total:

| | |
|---|---|
| `coggyd` resident, holding 100 sessions | **24.82 MiB** |
| Per session held | **254 KiB** |
| Share of what it holds | **1.0%** |
| `coggyd` CPU | 1.55% of one core |

**254 KiB against [363 KiB in the morning's `ping` hour](2026-08-01-103225-an-hour-of-a-hundred-sessions.md)**, and the difference is the workload's line length reaching the daemon through a cap that counts lines — which is [what a line-count cap was expected to do](2026-08-01-103225-an-hour-of-a-hundred-sessions.md). `cpu-spin` writes 21.06 B a line against `ping`'s ~44.

**The per-line overhead is not the constant the earlier record implied.** Subtracting the same 110 KiB of fixed cost leaves 144 KiB of scrollback over 2,000 lines, or 73.7 B a line against 21.06 B of content — 53 B of overhead where `ping`'s 2,000 lines implied 85. An allocator rounds a 21-byte string and a 44-byte string into different size classes, so the overhead is a function of the content rather than a constant beside it.

## The counters agree exactly

Registered before the run and checked after: the scrollback keeps 2,000 lines a session and drops the rest, so

```
evicted  =  read − 100 × 2,000
983,088  =  1,183,088 − 200,000        difference: 0
```

Two counters maintained in different places, agreeing to the line. Nothing else here checks the scrollback's eviction against the daemon's read count.

Output came to **24,917,336 bytes over 1,183,088 lines**, 21.06 B a line, with **truncated 0** — no line reached `MAX_LINE_BYTES`, so the byte cap never bound and the line cap did all the work.

## Saturation was real

The tree took **15.50 of 16 cores** at the last sample. A hundred sessions at duty 0.172 demand 17.2, so they were core-limited rather than backing off — which is the thing [`--duty` could not do](2026-08-01-173927-the-baseline-is-the-noisy-term.md) and the reason this run uses a fixed wait.

## What this run cannot say

- **It does not clear gate M1.** Two of four conditions are out of reach by construction, the duration is a third of what the gate asks, and the duty is not the one the gate is meaningful at. What it establishes is that nothing in the daemon breaks at a hundred sessions under saturation for twenty minutes.
- **The crash is diagnosed by elimination, not measured.** No bugcheck, no update, no power button, on AC — thermal and power delivery both fit and nothing here separates them. A third attempt was not made; two hard stops on someone's laptop in one day is enough.
- **`η ≈ 0.855` comes from one point.** It is a rearrangement of this run's own slowdown, not an independent fit, so the 1.98× it projects is arithmetic rather than a measurement.
- **The daemon's 254 KiB is a full scrollback at this line length.** A workload writing longer lines pays more, up to the byte cap; one writing fewer lines pays less. It is not a property of the daemon alone.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200), rebooted 19:21 after an unexpected shutdown |
| Background | 3.49 of 16 logical before the run |
| Harness | `sessionbench hold --with-solo --solo-repeats 3` at commit `54b092c`, release build, driven detached by `scripts/m1-hour.ps1` |
| Daemon | `coggyd` 0.0.0, release build |
| Workload | `cpu-spin --units 100000000 --wait-ms 67 --resident 20` |
| Shape | 3 × 120 s solo · 100 sessions for 1200 s · 3 × 120 s solo, sampled every 5 s, back to back |
| Samples | 238 on disk, written as they were taken |
