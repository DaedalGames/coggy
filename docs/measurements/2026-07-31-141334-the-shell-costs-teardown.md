# What wrapping a session in a shell costs · 2026-07-31 14:13:34

[The comparison set](../../sessionbench/README.md#what-we-measure-against) has carried a row for *pwsh 7 against cmd against the workload alone* since before anything in it was measured, and it was the last reachable one still empty. The expectation behind it was that a shell is a residency cost — [Decision 1](../PLAN.md#four-core-decisions) was fought over a conhost at 8.7 MiB a session, and a pwsh host is eight times that.

Three ramps, identical in every respect but the wrapper.

## The redline does not move

| | Redline | Fitted crossing | Slope | Solo rate | Drift |
|---|---|---|---|---|---|
| Workload alone | 27 | 27.6 | 0.0724 | 74.11 units/s | −0.0% |
| Under `cmd` | 26 | 26.6 | 0.0751 | 76.44 units/s | +1.3% |
| Under `pwsh -NoProfile` | 26 | 26.8 | 0.0746 | 76.45 units/s | −0.9% |

**Three fitted crossings inside 3.6% of each other**, against an instrument whose ladder search spans [±12.5%](2026-07-30-164912-redline-reproducibility.md) and whose fitted slope resolves 2.3%. All three drift checks held.

A shell adds a process and holds memory, and **demands no CPU while its child computes**. The condition that binds here is work rate, so the shell is invisible to it.

## Checked afterwards by the tool this run prompted

`sessionbench compare`, built later the same day, admits all three pairings — and the pattern inside them is not the one that was assumed:

| Pair | Solo rungs | Redline |
|---|---|---|
| bare against `cmd` | +3.1% | 27 → 26 |
| bare against `pwsh` | +3.2% | 27 → 26 |
| **`cmd` against `pwsh`** | **+0.0%** | **26 → 26** |

**The two wrapped ramps agree to nothing at all** — 76.44 against 76.45 units/s — while the bare ramp sits 3.1% from both. That is a consistent offset rather than scatter, and time does not explain it: bare and `pwsh` ran thirteen minutes apart and differ by 3.2%, `cmd` and `pwsh` ran twenty-two minutes apart and differ by 0.0%.

It matters for what the allowance in `compare` should be. The 3.1% was read as the spread a solo rung carries between runs, and used to set the threshold. **But 0.0% is equally observed**, so the rung's own measurement noise is far smaller than that, and whatever separates the bare ramp from the two wrapped ones is something else — the first ramp of a batch, or the wrapper genuinely changing how a single session is scheduled. Neither is established here.

## What it does cost is memory, and the ranking is not the interesting part

| | Per session | Processes |
|---|---|---|
| Workload alone | 84.01 MiB | 1 |
| Under `cmd` | 92.74 MiB | 2 |
| Under `pwsh -NoProfile` | 154.44 MiB | 2 |

`cmd` costs 9 MiB and `pwsh` 70. Both figures were also taken directly from a single host outside the benchmark and agree, which is [the check worth more than a repeat](../../CLAUDE.md).

**Against a real session it is small either way.** A cooking session holds [2.39 GiB across its engine and agent](2026-07-31-054657-the-driven-duty.md), so a `pwsh` wrapper is 2.9% of it and `cmd` is 0.4%. Neither moves a ceiling of nine.

## Teardown is where a wrapper is expensive, by two orders of magnitude

| Teardown at 50 sessions | | Stragglers |
|---|---|---|
| Workload alone | **313 ms** | 0 |
| Under `cmd` | **112,957 ms** | 50 |
| Under `pwsh` | **106,476 ms** | 50 |

**361× slower, and `cmd` is worse than `pwsh`** — so this is not PowerShell being heavy. It is the extra layer.

**The straggler count equals the session count exactly, in both.** Every wrapped session leaves precisely one process behind: killing the shell does not take its child with it, so the instrument waits out a 20 s grace and then reaps 50 survivors one at a time, at about 2.26 s a session.

This is [the conhost finding](2026-07-30-101141-conhost-and-defender.md) from the opposite side. There, a process the session did not own outlived it. Here, a process the session *did* own outlives the kill. Both say the same thing: **a session's lifetime is not the lifetime of the process that was spawned for it**, which is the assumption a governor would otherwise make about reclaiming a slot.

## The ramp reported a scaling result for a command that never ran

The first attempt at the `cmd` row produced this, and called it a floor of fifty sessions:

| Sessions | RSS | Cores | Processes | Replaced | Verdict |
|---|---|---|---|---|---|
| 1 | 0 B | 0.1 | 0 | 60 | held |
| 50 | 0 B | 0.7 | 0 | 3,000 | held |

Git Bash rewrites a leading-slash argument as a Windows path, so `cmd /c` reached the ramp as `cmd C:/`. Every session died at once and was respawned about once a second, and **`cmd`'s three lines of error output were counted as three completed work units** — 3.00 units/s against the 76 a working session gives.

That is [workload-contract rule 3](../../workloads/README.md#the-contract) — *a unit is work that was actually done* — broken from a direction the rule did not anticipate. The rule guards against a workload misreporting itself; here nothing ever reached the workload, and the **wrapper's failure output** was the thing being counted.

**`sessionbench` now refuses to grade a rung that held no process.** Zero processes resident across every sample, while sessions were asked for, is reported as inconclusive rather than held, and an inconclusive rung stops the ladder. Alongside it the decision moved out of `hold_rung` into a function a test can reach, because all three ways a rung can be unmeasurable were found by a run going wrong rather than by a test.

**The pre-flight that caught it was cheaper than the ramp by a factor of six hundred** — one 20 s `observe`, reading back the command as recorded rather than as typed.

## What this rests on

- **One workload at duty 1.0.** A shell that sleeps costs nothing in CPU; a wrapper that stays busy would not be free, and none was tested.
- **`pwsh -NoProfile`.** [PLAN's 300–700 ms profile autoload](../PLAN.md#residency-not-spawning) is excluded, so 154 MiB is the floor for a pwsh session rather than the stock-desktop figure.
- **The teardown figure is this instrument's teardown.** A governor that used job-object termination rather than killing the root process would not pay it, and that is the obvious next thing to test.
- **Sessions never exited on their own**, so nothing here measures shell *startup* cost, which is the other half of the row's stated role.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| Workload | `cpu-spin --units 1000000`, 80 MiB resident, duty 1.0 |
| Ramps | three, 60 s holds, ladder capped at 60 sessions, resolution 2, pipes |
| sessionbench | 0.0.0 at commit `9a23d0a`, release build |
| Defender | real-time protection on, no exclusions |

The first `cmd` ramp is not in the table above; it measured nothing and is described in its own section. The re-run replaced it under the same settings.
