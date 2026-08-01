# The wait mechanism cancels, and the observation that said otherwise did not reproduce

`cpu-spin` can pause two ways: `--duty` derives each wait from how long that unit actually took, and `--wait-ms` holds it at a wall-clock constant. [The workload's own source claims the choice does not matter](../../workloads/cpu-spin/src/main.rs) — *solving both cases gives `slowdown = N·d/C` either way, the mechanism cancels, provided the wait really releases the core* — and says the flag exists to test that. [It was tested once at duty 0.75](2026-07-30-164912-redline-reproducibility.md) and the two configurations interleaved completely.

**Run back to back at duty 0.172 and a hundred sessions, they still cancel.**

## The pair

| | solo | concurrent | slowdown | tree CPU | peak RSS |
|---|---|---|---|---|---|
| `--duty 0.172` | 13.99 | 10.33 | **1.354** | 15.32 cores | 2.391 GiB |
| `--wait-ms 59` | 13.94 | 10.52 | **1.325** | 15.57 cores | 2.392 GiB |

**The solo fingerprints sit 0.33% apart**, which is what makes this a comparison rather than an afternoon: [two runs are comparable when their solo rungs agree](../../sessionbench/README.md#comparing-two-ramps), and these agree twelve times better than the allowance asks. The slowdowns differ by **2.16%**, inside what a single hold's noise covers.

Both saturate. Both hold a hundred sessions to the last report. Both land on the same peak RSS to a mebibyte, which is the workload's footprint rather than either flag's.

Each side's own baseline spread, which is what says the numbers are worth comparing at all:

| | before | after | gap |
|---|---|---|---|
| `--duty` | 0.33% | 0.25% | 0.67% |
| `--wait-ms` | 0.16% | 0.03% | 0.16% |

Quieter than anything measured earlier today, on a machine whose background was 8%. The eviction invariant holds exactly on both — `evicted = read − 100 × 2,000`, difference zero on each side.

## The observation this was opened to chase did not reproduce

Earlier the same day, a run at **`--duty 0.27` with a hundred sessions** was read mid-flight as backing off rather than competing: 0.0655 cores a session against 0.27 requested, and 6.7 of sixteen cores idle. That reading aborted a gate run and moved it to `--wait-ms`.

**At duty 0.172 there is no backoff.** `--duty` takes 15.32 of sixteen cores, against `--wait-ms`'s 15.57. Whatever produced the earlier figure, it is not a general property of the proportional wait.

What is left is a contradiction rather than an explanation, and both candidates are stated because neither is separated:

- **The machine was in a different state.** That reading came minutes into a run on a box that had spent the previous quarter hour under full load, and this one follows a cold boot and a quiet evening.
- **The cycle's non-work terms scale against duty.** The write and the resident touch sit outside the interval `--duty` measures, so they lengthen the cycle without lengthening the wait. A duty-0.27 cycle is shorter than a duty-0.172 one, so that fixed cost is a larger share of it — which would depress CPU more at the higher duty. It fits both numbers and nothing here isolates it.

**A figure that decided something and lives only beside the decision is how this stayed unexamined for four hours.** The 6.55 cores was written into a script comment as the reason for a flag and into a task, and never into a record, so nothing pointed at it when the next run disagreed.

## Which flag the gate should use, which reverses this afternoon

The gate's work-rate condition [is stated in duty](../../ROADMAP.md#m1--headless-daemon), and the two flags deliver a stated duty differently:

- **`--duty` is self-calibrating and holds the duty on any machine.** That is the whole of its design.
- **`--wait-ms` holds a wall-clock constant, so its duty moves with the machine's speed.** [The gate run was calibrated on a thermally loaded box and ran on a cold one, and delivered 0.172 where 0.271 was asked for](2026-08-01-202112-gate-m1-at-twenty-minutes.md) — a 79% change in compute speed arriving as 13.6% of rate, which is quiet enough to miss.

Since the slowdown is the same either way, **the flag that reliably delivers the duty is the better one for a gate stated in duty.** `--wait-ms` remains the more faithful shape of a session waiting on a model, whose duty really does climb as its compute slows; it is the right flag for realism and the wrong one for a stated parameter.

The afternoon's switch to `--wait-ms` rested on two arguments. The stronger one — that it matches the subject — survives. The one that actually drove it, that `--duty` could not saturate, does not.

## What this run cannot say

- **It does not explain the 6.55 cores.** It shows the behaviour is not general. Reproducing it would need a `--duty 0.27` hold at a hundred sessions with the achieved duty read from samples, on a machine in a stated thermal state.
- **One duty, one count.** 0.172 and a hundred sessions. The earlier pairing covered 0.75; between and beyond is unmeasured.
- **Five minutes a hold.** Long enough for the rate to settle and far short of the hour the gate asks for, so it says nothing about drift.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 1.34 of 16 logical before the pair |
| Harness | `sessionbench hold --with-solo --solo-repeats 3` at commit `60c2f2a`, release build |
| Daemon | `coggyd` 0.0.0, release build |
| Workload | `cpu-spin --units 100000000 --resident 20`, once with `--duty 0.172` and once with `--wait-ms 59` |
| Shape | each half: 3 × 120 s solo · 100 sessions for 300 s · 3 × 120 s solo, sampled every 5 s |
| Order | the two halves back to back, nineteen minutes apart |
