# Gate M1 misses its work-rate condition by 3.3%, and the two conditions close 372 MB apart

> **Read with [the interval-level re-reading](2026-08-03-024222-the-footprint-never-mattered.md).** The 20 MiB run lost the machine in 18% of its samples, which its mean absorbs; compared on intervals where all three held 15+ cores, the footprint is worth about 2% rather than 11%, and the three slowdowns are 2.065, 2.057 and 2.089 — the same within 1.5%.

> **The 372 MB is withdrawn; the 3.3% stands.** It came from extrapolating a two-point `η` slope four megabytes past its second point, and [a third point at 36 MiB shows the slope has already stopped](2026-08-03-023009-the-footprint-lever-runs-out-before-the-budget-does.md) — `η` 0.811 there against 0.817 at 33. So no footprint reaches the passing value, the memory budget never becomes the binding condition, and **work rate binds at every weight this machine can hold.**

The gate figure of record is **2.301× at 20 MiB a session**, 15% past the 2 it asks for. Run at the heaviest session the memory budget allows, on a rested box, both brackets clean:

**2.065×.** Three point three percent.

## The run

`sessionbench hold --sessions 100 --duration 1200 --with-solo --solo-repeats 3 --solo-duration 120`, workload `cpu-spin --units 100000000 --duty 0.27 --resident 33`, under `coggyd`, on mains, from a box confirmed rested by a solo hold at 21.555 units/s beforehand.

| | resident 20 (rested) | **resident 33 (rested)** |
|---|---|---|
| solo before | — | 21.809, 21.787, 21.902 |
| solo after | — | 21.708, 21.656, 21.858 |
| solo spread | 8.48% before · 0.35% after | **0.53% before · 0.93% after** |
| solo gap | 2.97% | **0.42%** |
| concurrent, 100 sessions | 9.3363 | **10.5480** |
| **slowdown** | 2.301 | **2.065** |
| `η` | 0.733 | **0.817** |
| implied ceiling `2ηC/d` | 87 sessions | **97** |
| peak RSS | 2.393 GiB | **3.651 GiB of 3.725** |
| `failed_reads` | — | **0 — held** |

**Two clean brackets in one machine state**, which no previous pair of these has managed: the 20 MiB run's own bracket refused, and last night's 33 MiB pair straddled two states. This is the first footprint comparison where both sides are published figures.

## Where the two conditions now meet

`η` rises 0.733 → 0.817 across 13 MiB, a slope of **0.00643 per MiB**. Passing the work-rate condition needs `η ≥ 0.844`, which that slope reaches at **37.2 MiB** a session.

At 37.2 MiB, with the [measured additive overhead of 4.06 MiB a session plus 0.44 for the daemon and harness](2026-08-02-225348-the-two-failing-conditions-are-not-independent.md):

```
100 × (37.2 + 4.06 + 0.44)  =  4170 MiB  =  4.072 GiB
budget                                    =  3.725 GiB
over by                                   =  372 MB   (9.3%)
```

**So the two failing conditions close on each other 372 MB apart.** [That was the shape claimed a day ago](2026-08-02-225348-the-two-failing-conditions-are-not-independent.md) with the direction of `η` inverted and the gap read as unbridgeable; measured properly the gap is 9.3% of one budget rather than a factor of two, and the direction is the one that helps.

## What this changes about M1

**The gate is much closer than any document said.** *Nowhere near the 2* was written into ROADMAP tonight on the 33 MiB figure taken in the slow machine state. Rested, it is 3.3% short.

The sizing follows: `C ≥ N·d/(2η)` at `η = 0.817` gives **16.5 logical processors**, against sixteen. The earlier figures of 17.3 and 18.4 came from the lighter session's `η`, and nineteen was 18.4 rounded up.

**RSS is the binding condition now, not work rate.** The footprint that would clear the ratio does not fit the memory budget, and the footprint that fits misses the ratio by 3.3%. A gate whose 4GB were the binary 4 GiB rather than decimal would allow 36.5 MiB, which the same slope puts at `η = 0.840` and a slowdown of **2.010** — a tenth of a percent short. **The gate's own choice of units is now the difference between passing and failing.**

## And twenty minutes of saturation did not move the machine

Solo rungs before: 21.809, 21.787, 21.902. After: 21.708, 21.656, 21.858. **Gap 0.42%.**

[Three minutes did not induce the slow state](2026-08-03-004512-a-saturating-burst-halves-the-box-for-an-hour.md) and neither does twenty. Duration was the one hypothesis left after the three-minute attempt, and a load seven times longer leaves the box exactly where it started — so the slow state is not a function of how long this workload has been running, and what remains are causes outside the run.

## What this cannot say

- **One run at one footprint.** 33 MiB at duty 0.27, rested.
- **The 37.2 MiB crossing is a two-point extrapolation.** `η` is measured at 20 and 33; the slope between them is assumed linear over four more megabytes. It is the weakest figure here and it decides whether the gate is 3.3% or 0.1% away.
- **`η` may not be linear in footprint at all.** Nothing here gives it a shape, and [the 80 MiB reading remains a redline fitted from a ladder](2026-08-02-202535-the-core-ceiling-is-four-numbers-for-two-workloads.md) rather than a slowdown at a hundred sessions.
- **Neither counter moved usefully.** `thermal_c` reads 39.1 °C, as it has all night in both machine states; `processor_performance` reads 144.3 here against 170–173 on an idle box, which is load rather than state.
- **The hour is still unmeasured.** This is twenty minutes, because [the box stops dead at forty-one](2026-08-01-202112-gate-m1-at-twenty-minutes.md).

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical · 31 GiB · Windows 11 (26200) |
| Power | on mains, SAMSUNG MODE, 100% |
| State | rested — a 40 s solo returned 21.555 units/s immediately beforehand |
| Background | 21% before the run |
| Harness | `sessionbench` 0.0.0 at `7ca493a`, release, `hold --with-solo --solo-repeats 3 --solo-duration 120` |
| Daemon | `coggyd` 0.0.0, release |
| Shape | 3 solo × 120 s, 100 sessions × 1200 s, 3 solo × 120 s, 30 s uncounted warm-up before each |
| Artifact | `bench-out/1785687266-r33-rested-daemon` |
