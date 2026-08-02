# The core ceiling is four numbers, and two of them are a different workload

Tracing `redline × duty ≈ 25` back to see whether it shares a chain with the rest turned up something else: **the repository holds four values for `η` on this machine, and they split by workload rather than scattering around one answer.**

## The four

| source | workload | duty | `2ηC` | `η` | ceiling at `d = 0.27` |
|---|---|---|---|---|---|
| [The gate hold](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md) | `--resident 20` | 0.27 | 23.49 | **0.734** | **87** |
| [The corpus constant](2026-07-30-154348-duty-is-derivable.md) | mixed ladders | various | 25.00 | 0.781 | 92.6 |
| [Quarter-duty control](2026-08-01-080158-the-relation-at-a-quarter-duty.md) | **80 MiB resident** | 1.00 | 26.93 | 0.842 | 99.7 |
| [Quarter-duty measurement](2026-08-01-080158-the-relation-at-a-quarter-duty.md) | **80 MiB resident** | 0.27 | 29.59 | **0.925** | **109.6** |

Read as one quantity they span **26%**, which would make the whole relation shaky. Read by workload they are two tight pairs: the 20 MiB runs sit at 0.73–0.78 and the 80 MiB runs at 0.84–0.93.

**`η` belongs to the workload as much as the machine**, which [the harness already says](../../sessionbench/README.md#running-gate-g0-on-a-configured-machine) — *`cpu-spin --resident` is the knob for it if the real session's footprint is far from 20 MiB*. The figures were never contradicting each other; they were answering for different sessions.

## Which way the footprint pushes it, and why that reads backwards

The heavier workload has the **higher** `η` — sessions cost each other less at 80 MiB than at 20.

That is the opposite of the intuition that more memory means more contention, and it follows from what `η` measures. A memory-bound session is already waiting on memory when it runs alone, so its solo rate is low and adding neighbours takes proportionally less from it. A compute-bound one gets the core's full speed alone and loses more when the cache is shared. **`η` is the fraction of solo throughput that survives company, so a workload with less to lose keeps more of it.**

The quarter-duty record notes its 80 MiB workload is *memory-bound enough to hide this machine's three core tiers*. That is the same fact from the other side.

## What was being conflated

**87 and 93 were quoted as a range and they are both the 20 MiB workload** — the disagreement between one hold's slowdown and a constant fitted across ladders. Meanwhile 109.6 sits in its own record described as the ceiling now being *a measurement with a ±10% error bar*, and it is the 80 MiB workload. Nothing said so, so three numbers looked like a spread around one ceiling.

**None of them is the ceiling for a real session.** A generation session holds [2.39 GiB](2026-07-31-150258-g0-frozen.md), thirty times the heavier synthetic and a hundred and twenty times the lighter, and the trend across those two says a real session's `η` would be higher still. It does not matter: [memory stops it at nine](2026-07-31-150258-g0-frozen.md), an order of magnitude below any of these.

So the four numbers bound how far `η` moves with footprint on this machine, and none of them binds anything a governor decides.

## What this run cannot say

- **Nothing new was measured.** This is arithmetic over four existing records, and its value is the grouping rather than any figure.
- **Two points do not give a curve.** 20 MiB and 80 MiB, four figures between them; where `η` goes at 500 MiB or at 2 GiB is unmeasured, and the direction is inferred from two clusters.
- **The 20 MiB pair may not be a pair.** 0.734 comes from one hold's slowdown and 0.781 from ladders across several duties, and [every route to the first reduces to the same expression](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md). The 6.5% between them is one measurement against another, not a spread within a method.

## Provenance

Arithmetic over `docs/measurements/`, at commit `9ece615`. No run was made.

## 2026-08-03: footprint and duty covaried, and the controlled test reverses the sign

This record reads the four `η` values as two tight pairs split by footprint — 0.73–0.78 at 20 MiB and 0.84–0.93 at 80 — and offers a mechanism: a session already waiting on memory has less left to lose.

**A hold that moves footprint alone gets the opposite sign.** [At a fixed duty of 0.27, a hundred sessions at 33 MiB give `η = 0.518` against 0.733 at 20 MiB](2026-08-03-000430-the-footprint-lever-runs-backwards.md) — below both anchors rather than between them.

The split was never controlled. Its 80 MiB entry is a duty-1.00 run and its 20 MiB entries sit at 0.27 and at mixed duties, so footprint and duty moved together and the whole difference was assigned to one of them. The grouping is still the right question to ask of four numbers that spanned 26% as one quantity; the answer it gave is not supported.

What stands unchanged is the narrower claim that a ceiling quoted without its workload is not quoted. What does not stand is *which way* the workload moves it.
