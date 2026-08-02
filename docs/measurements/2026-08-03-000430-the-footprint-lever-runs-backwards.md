# The footprint lever runs backwards: `η` falls from 0.733 to 0.518 between 20 and 33 MiB

[Six hours earlier](2026-08-02-225348-the-two-failing-conditions-are-not-independent.md) gate M1's two failures were shown to close on each other, and the way out was named: `η` rises with a session's footprint, the RSS budget allows 33 MiB a session, and `η` had never been measured between 20 and 80. One hold would settle whether M1 passes on this machine.

**It settled it. `η` falls.** At the same duty, the same count and the same machine, a hundred sessions holding 33 MiB slow down **3.261×** where 20 MiB gave 2.301 — so the gate fails harder at the footprint that was supposed to save it.

## The run

`sessionbench hold --sessions 100 --duration 1200 --with-solo --solo-repeats 3 --solo-duration 120`, workload `cpu-spin --units 100000000 --duty 0.27 --resident 33`, under `coggyd`, on mains.

| | resident 20 (2026-08-01) | **resident 33 (tonight)** |
|---|---|---|
| solo, mean of six | 21.484 units/s | **9.377** |
| concurrent, 100 sessions | 9.3363 units/s/session | **2.8753** |
| **slowdown** | 2.301 | **3.261** |
| `η = N·d/(C · slowdown)` | 0.7333 | **0.5175** |
| solo spread | 8.48% before · 0.35% after | **0.28% before · 0.68% after** |
| solo gap | 2.97% | **0.84%** |
| peak RSS | 2.393 GiB | **3.648 GiB of 3.725 — held** |
| `failed_reads` | not recorded then | **0 — held** |
| work rate | bracket refused | **broke** |

**This is the quietest bracket this instrument has produced.** Standard error 0.09% before and 0.20% after, against a 5% allowance, and the two sides agree to 0.84%. The 08-01 run's bracket refused itself; this one did not have to.

## Gate M1 at the footprint its own RSS condition allows

- **RSS: held.** 3.648 GiB of 3.725, 97.9% of the budget, 37.36 MiB a session.
- **Dropped output: held.** `failed_reads 0`.
- **Work rate: broke.** 3.261 against a 2.
- Replacement remains out of reach.

So the footprint that fits the memory condition fails the work-rate condition **worse than the light one did** — 63% over instead of 15%.

## What this contradicts

Today's [split of four `η` values by footprint](2026-08-02-202535-the-core-ceiling-is-four-numbers-for-two-workloads.md) read 0.73–0.78 at 20 MiB and 0.84–0.93 at 80 MiB and concluded that a heavier session keeps more, because one already waiting on memory has less left to lose. **Tonight moves footprint alone, at a fixed duty, and gets the opposite sign.**

The split's pairs were never a controlled comparison: its 80 MiB entry is a **duty-1.00** run and its 20 MiB entries are at 0.27 and at mixed duties. Footprint and duty covaried, and the reading assigned the whole difference to footprint. One deliberate variation of the one variable is worth more than four runs that each moved two.

**So the middle was worth measuring and the answer was not the one the interpolation offered.** A straight line between the two footprints predicted `η(33) ≈ 0.794`; the measurement is 0.518, below *both* anchors. There is no monotone curve through 0.733 at 20, 0.518 at 33 and 0.842 at 80, which means at least one of those three is measuring something other than footprint.

## The solo rung stopped being a fingerprint

Two runs are comparable when their solo rungs agree — one session, no contention, the same work. These differ by **2.29×**: 21.484 against 9.377.

**That is not the machine moving. It is the swept variable moving the solo.** A session holding 33 MiB strides more pages per unit than one holding 20, so a unit costs more before any second session exists. The fingerprint check assumes the solo is invariant across the pair, and a footprint sweep breaks that assumption by construction.

What survives is that `slowdown` is a ratio taken *within* each run, so each side divides by its own solo and the machine cancels. What is lost is the cross-run check that the machine did not move between them — and for this pair there is now no such check. **Sweeping a variable that moves the baseline costs the drift control**, and the honest reading is that the 2.301 belongs to one afternoon and the 3.261 to another.

## The background reading did not predict the noise

Four `doctor` readings before the run spread **26.7%** — 3.63 to 4.69 cores — and that was used to call the window shut for a ratio, twice today. The bracket that followed spread **0.28%**.

**So `doctor`'s pre-run background spread did not predict this bracket's noise at all.** A hundred sessions divide a disturbance a hundred ways, and a solo hold at 120 seconds averages over it rather than sampling it — the swing that looks fatal in four instantaneous readings is not what a two-minute hold sees. The rule as written — *for a ratio, read a few consecutive `doctor` lines and look at their spread* — is not supported by this run, and the run that would have been skipped on it is the one that answered the question.

## What this cannot say

- **One footprint pair, one duty.** 20 and 33 MiB at 0.27. Whether `η` keeps falling past 33 is unmeasured, and the 80 MiB figure sits at a different duty.
- **Two afternoons, and no fingerprint to bridge them.** For the reason above. A same-session pair would need both footprints back to back.
- **Nothing here explains the mechanism.** Why a heavier session should cost its neighbours *more* rather than less is not answered by a slowdown.
- **The mid-run rate excursion is a sampling beat, not work.** Five samples out of 239 carry about 10 300 units where the mean is 1 449, alternating with low neighbours. The reported total comes from the daemon's own cumulative counter rather than a sum of deltas, so the rate is unaffected.
- **The observer was not perfectly still.** One `tail` of the run log was issued while the after-side solos were running. Its effect would depress an after-side hold and inflate the slowdown; the after side spread 0.68% and sits 0.6% below the before side, which bounds it at well under a percent.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical · 31 GiB · Windows 11 (26200) |
| Power | on mains, SAMSUNG MODE, 100% |
| Background | four `doctor` readings before the run: 3.63, 3.67, 3.88, 4.69 cores |
| Harness | `sessionbench` 0.0.0 at `e1bd527`, release, `hold --with-solo` |
| Daemon | `coggyd` 0.0.0, release |
| Shape | 3 solo × 120 s, 100 sessions × 1200 s, 3 solo × 120 s, 30 s uncounted warm-up before each |
| Artifact | `bench-out/1785680927-eta-at-33-daemon/` |
