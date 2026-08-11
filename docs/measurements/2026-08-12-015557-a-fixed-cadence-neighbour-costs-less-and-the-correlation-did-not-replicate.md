# A fixed-cadence neighbour costs less, and the correlation did not replicate

**Eight interleaved pairs say a `--wait-ms` neighbour costs a session 14.6 points less than a `--duty` one — −8.1% against −22.8%, `t ≈ 2.4`, suggestive and not established at four pairs a side. The same set refutes [the +0.771 correlation](2026-08-12-005518-the-scatter-was-the-step-measured-six-times.md) recorded ninety minutes earlier: the identical injection now gives `r = +0.25`.**

## The differently-shaped neighbour was a flag, not a program

[The open question](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md) is whether the injected step survives a neighbour that does not share the measured session's *wake cadence*. A byte-identical copy at a different path removed shared code pages and left behaviour untouched.

`cpu-spin` already carries two wake mechanisms, and its own documentation names the pairing: `--duty` stretches its pause to match a slower unit, so its cadence tracks the machine; `--wait-ms` sleeps a fixed span, so its cadence is fixed and its duty drifts. Same binary, same work loop, same cost model, different rhythm.

An evening went into sourcing a differently-shaped neighbour from a different *program*. `file-write` and `stdout-storm` are both disqualified — one blocks without a reader, the other's cost is disk-shaped and unsizeable — and the difference that matters was a flag.

**Sized two ways that agree.** Six at `--duty 0.27` hold 1.347/1.283 cores of their own; six at `--wait-ms 76` hold 1.290/1.168, within 7%. Derived independently: `duty = compute/(compute+wait)`, and at `wait 30` six held 2.02 → duty 0.337 → compute 15.2 ms, so matching duty 0.167 wants `wait 76`.

## The set

Four rounds, each a duty pair then a wait-ms pair, **interleaved** so the arms cannot differ by sitting. 01:24–01:53, mains, `-AnyBaseline`, 30-second holds.

| arm | delta | rate | change |
|---|---|---|---|
| duty | 0.70 | 9.592 → 7.933 | −17.3% |
| duty | 0.88 | 9.204 → 7.845 | −14.8% |
| duty | 1.59 | 6.443 → 5.085 | −21.1% |
| duty | 0.67 | 9.972 → 6.200 | **−37.8%** |
| wait-ms | 1.05 | 8.522 → 8.376 | −1.7% |
| wait-ms | 0.49 | 6.255 → 5.913 | −5.5% |
| wait-ms | 2.25 | 9.489 → 7.835 | −17.4% |
| wait-ms | 1.29 | 9.138 → 8.412 | −7.9% |

| arm | mean | sd | r(delta) |
|---|---|---|---|
| duty | **−22.8%** | 10.4 | +0.25 |
| wait-ms | **−8.1%** | 6.7 | −0.85 |

`t ≈ 2.4` on the difference of means, `p ≈ 0.06`. The arms overlap: duty's −14.8 sits above wait-ms's −17.4.

**Stopping at round two would have produced a clean result and a wrong one.** After two rounds the arms were −16.1% against −3.6%, non-overlapping, each tight. Two rounds later the spreads had grown to 10.4 and 6.7 and the arms crossed.

## The +0.771 does not replicate

The duty arm is the same injection and the same command as [the six-pair set](2026-08-12-005518-the-scatter-was-the-step-measured-six-times.md), ninety minutes later.

| set | n | mean | sd | r(delta) | slope |
|---|---|---|---|---|---|
| six-pair, 00:11–00:53 | 6 | −3.2% | 20.8 | **+0.771** | +19.7%/core |
| duty arm, 01:24–01:53 | 4 | −22.8% | 10.4 | **+0.25** | +6.0%/core |
| pooled | 10 | −11.0% | 19.4 | **+0.53** | +15.7%/core |

**And one pair contradicts another at matched tenancy.** The six-pair set gave **+22.7% at delta 1.67, rest 8.34 → 10.01**; this series gave **−21.1% at delta 1.59, rest 8.42 → 10.01**. Same injection, near-identical tenancies, opposite signs, 44 points apart.

So the earlier record described a **within-sitting artifact**, not a property of the machine. Its own hedge — *the ordering is the more robust claim than the coefficient* — was insufficient, because the ordering did not survive either.

## What this does not establish

- **Four pairs a side.** `p ≈ 0.06` is one pair from either conclusion, and the arms overlap.
- **The `−37.8%` duty outlier** drives much of the gap: without it the duty mean is −17.7% and the gap falls to 9.6 points.
- **The wait-ms arm's `r = −0.85`** runs opposite to the duty arm's `+0.25`, on four points each. Neither coefficient is worth anything at that n; they are reported so a later set can contradict them.
- **A confound remains in the flag itself.** `--wait-ms`'s duty *drifts* as the machine slows, so across a 30-second hold the two arms may not have delivered identical mean load even though they were sized to.
- **No mechanism.** Core clock, uncore, parking and placement remain eliminated, and wake cadence is now a candidate rather than an answer.

## Provenance

| | |
|---|---|
| Series | `wake-shape.ps1`, four interleaved rounds, 01:24–01:53 |
| Each pair | `inject-tenant.ps1 -Tenants 6 -AnyBaseline -QuietBelow 99 -QuietMachineBelow 99 -Duration 30`, injector args differing only in `--duty 0.27` against `--wait-ms 76` |
| Artifacts | `bench-out/inject-20260812-01*.log`, `bench-out/*inject-*-daemon/` |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, 0 survivors after teardown |
