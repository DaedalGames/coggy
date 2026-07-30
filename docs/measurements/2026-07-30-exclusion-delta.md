# The Defender exclusion delta, and why it moves to the ramp · 2026-07-30

The sixth axis is implemented and, on this machine, **it cannot answer the question it was built for at one session**. Recording that is the point: [doctor](../../sessionbench/README.md#running-it) had been calling this axis available for as long as it existed while nothing measured it, and the first thing measuring it did was produce three different answers.

## What it produced before the guards

Three attempts at a single watched-versus-excluded comparison, same workload, same machine, minutes apart:

| Attempt | Defender, watched | Defender, excluded | Apparent result |
|---|---|---|---|
| 1 | 11.87 s/min | 1.92 s/min | −84% |
| 2 | 9.13 s/min | 10.64 s/min | +17% |
| 3 | 7.48 s/min | 6.12 s/min | −18% |

None of these is about exclusions. **Defender is one machine-wide process**, and the run attributed all of its CPU to the session under test — so everything else touching the disk arrived as the workload's scanning cost. During the first attempt that included this project's own `git` and `gh` traffic, run by the same hand that had said it would leave the machine alone.

## What the guards did

Two changes, in the order the failures appeared.

**An idle baseline before each half**, so the room's share is visible and subtracted. It fires immediately: baselines here sit between 3.5 and 10.1 s/min with nothing running at all.

**Pairs, repeated and adjacent in time**, so slow drift lands on both halves. Three pairs:

| Pair | Saving after subtracting idle | Baseline drift within the pair |
|---|---|---|
| 1 | −5.18 s/min | 5.70 s/min |
| 2 | +0.39 s/min | 3.13 s/min |
| 3 | −6.28 s/min | 3.98 s/min |

```
mean -3.69 s/min · across pairs -6.28 to +0.39 · worst drift 5.70
verdict: inconclusive — the spread across pairs is larger than what separates them
```

The range crosses zero, so not even the direction is established. **Reporting the mean alone would have published "the exclusion costs 3.69 s/min"**, which is a confident sentence about noise.

## The one thing that did reproduce

Work rate, in all three pairs, to the last digit:

```
watched 4.94 units/s   excluded 4.94 units/s
```

**At one session the exclusion changes throughput by nothing at all**, and that is not a null result — it is the expected one. Defender was taking roughly a sixth of one core on a machine with fifteen idle. There is nothing for it to slow down.

## Which is why this axis belongs on the ramp

The measurement was built at the wrong scale, and its own output says so.

- At **one** session, Defender's cost is real but invisible: it competes with nobody, and the only visible term is its CPU rate, which is exactly the quantity buried in machine-wide noise.
- At **a hundred** sessions, the write volume is a hundredfold, so if Defender's demand is a large share of the sixteen cores it must show up in per-session work rate, which is [the condition the metric leans on](../../sessionbench/README.md#redline). ([It is not](2026-07-30-defender-at-scale.md) — but that is the answer the experiment returned rather than one it assumed.)
- A redline is an integer produced from deltas over a window. It does not average noisy per-sample rates, and it is exactly the robust statistic this axis lacked.

So the next form of this measurement is **two ramps, one with the sessions' scratch root excluded**, compared by their redlines. That is a different and better experiment, and it was only findable by building the wrong one and reading what it said.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| sessionbench | 0.0.0 at commit `70836f9`, clean tree, release build |
| Workload | `file-write --files 900 --size 128 --interval 200`, about 640 KiB/s |
| Halves | 60 s each, first 30 s unmeasured; 20 s idle baseline before each |
| Defender | real-time protection on; **exclusions verified back to zero after every run** |

The exclusion is added over a directory the benchmark created for that run, removed before the result is read, removed again on drop, and a failure to remove is printed rather than swallowed. Every run in this record ended with the machine's exclusion list back to empty.
