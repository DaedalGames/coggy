# What fixing the instrument was worth · 2026-07-31 00:13:14

Before the benchmark could measure the machine it had to stop measuring itself. These are the as-is and to-be figures for each defect that made a wrong answer right, taken on the same machine and the same release build either side of the change named beside it.

They are recorded because the sizes are the argument: a scaling benchmark's worst failure is the observer becoming the bottleneck, and none of these announced itself as one. Every figure below was first read as a fact about Windows.

## The ones that made a wrong answer right

| | As-is | To-be | Δ | What changed |
|---|---|---|---|---|
| Teardown, 100 pseudoconsole sessions | **238,714 ms**, then hung | **596 ms** | **400× and no hang** | Release the handles and wait for the hosts to leave, rather than terminating processes that were already terminating |
| Sampler tick, 25 saturating sessions | **15,084 ms** | **56 ms** | **269×** | Raise the sampling thread above the sessions it is watching |
| Process refresh, 25 sessions | **79,771 ms** | **34 ms** | **2,346×** | Refresh the job's own processes rather than every process on the machine |
| Ramp to 100 pseudoconsole sessions | never completed | completes | — | The three above, together |
| Redline resolution | **10** — the last coarse rung | **25** — bracket halved to one | a value rather than a floor | Climb to bracket, then refine inside it |
| Dropped output under a pseudoconsole | **false positive every run** | correct | a spurious redline of 0 removed | Count lines while tracking only the highest ordinal, so a console's own prologue cannot read as a gap |
| Sessions counted, 25 requested | **12** | **25** | — | Stop subtracting pids a dead helper once held, which Windows had since reused |
| Scratch directories left per ramp | **299** | **0** | — | The benchmark names the directory the workload may write to, and removes it |

**Three of these eight were the instrument competing with its own subject**, and each was found only by pushing a rung until something broke. The 79,771 ms refresh is the sharpest: `sysinfo`'s default is to walk every process on the machine, which is correct and cheap until the machine is holding fifty of your own.

## Developer loop

| | As-is | To-be | Δ |
|---|---|---|---|
| Incremental release build | 8.3 s | 5.4 s | −35% |
| Incremental `cargo check` | 0.8–1.4 s | unchanged | — |
| MSRV check sharing `target/` | 0.3 s warm | unchanged | no cache thrash — measured rather than assumed |

`codegen-units = 1` was removed after measuring that it moved the sampler's worst tick by nothing, 35 to 48 ms either way. The instrument's time goes into system calls, which codegen tuning does not reach. The MSRV row exists because the opposite was claimed first: a second toolchain sharing `target/` sounded like it would thrash the cache, and it does not.

## What the fixed instrument then measured

Same workload, same machine, pipes against a pseudoconsole — the comparison [Decision 1](../PLAN.md#four-core-decisions) rests on.

| Sessions | As-is · pseudoconsole | To-be · pipes | Δ RSS | Δ processes |
|---|---|---|---|---|
| 1 | 92.36 MiB · 2 proc | 84.16 MiB · 1 proc | −8.20 MiB | −1 |
| 10 | 923.59 MiB · 20 proc | 841.59 MiB · 10 proc | −82.0 MiB | −10 |
| 25 | 2.25 GiB · 50 proc | 2.05 GiB · 25 proc | −205 MiB | −25 |
| 50 | 4.51 GiB · 100 proc | 4.11 GiB · 50 proc | −410 MiB | −50 |
| 75 | 6.76 GiB · 150 proc | 6.16 GiB · 75 proc | −614 MiB | −75 |
| **100** | **9.02 GiB · 200 proc** | **8.15 GiB · 100 proc** | **−890 MiB** | **−100** |

Every rung held in both modes at solo work rate, so neither came near a condition. **Dropping conhost buys 0.87 GiB at a hundred sessions — 3.9% of the budget, on a ladder whose top rung uses 41% of it — and buys nothing at all in work rate.** What it costs to leave undone is a hundred extra processes, which is [the ground the decision now stands on](2026-07-30-120002-first-redlines.md#what-dropping-conhost-is-worth-at-any-session-weight).

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| Build | release, same toolchain either side of each change |
| Measured | 2026-07-30, each pair back to back on an otherwise idle machine |

Figures predating [the drift control](2026-07-30-164912-redline-reproducibility.md) carry no check on whether the machine held still between the two halves of a pair. The ratios here are large enough that it does not decide any of them — a 2,346× change survives a few percent of drift — but the developer-loop rows are close enough to it to be read as directional.
