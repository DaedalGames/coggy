# Every hundred-session hold today is a third of what the box used to do

**Seventeen hundred-session holds are on disk. The four best reach 15.3–15.5 job cores at 7.6–10.5 units per session per second, and all four were taken on 3 and 11 August. Every one of the eight taken today sits between 3.34 and 4.81 cores at 1.42–2.19 units — across residuals from 0.55 to 10.7. The ceiling has moved by three to five times, and nothing in the workload, the parameters or the residual accounts for it.**

## The table that shows it

| job cores | when | `rest` | units/s/session | label |
|---|---|---|---|---|
| 15.525 | 08-03 03:57 | n/a | 10.50 | tickpair |
| 15.505 | 08-11 02:34 | 0.49 | 7.56 | statepair2 |
| 15.359 | 08-03 04:42 | n/a | 10.28 | clockpair |
| 15.340 | 08-03 17:19 | 0.77 | 9.03 | slowstate-ratio |
| 7.491 | 08-11 02:51 | 8.53 | 4.32 | m1-probe-load |
| 7.453 | 08-03 18:45 | 8.56 | 3.45 | tenant-100 |
| 7.124 | 08-11 03:10 | 8.88 | 5.30 | m1-abs-1200 |
| **4.811** | **08-12 11:00** | 8.92 | 1.83 | resident20 |
| **4.763** | **08-12 10:47** | **0.55** | 1.84 | waitms100 |
| **4.675** | **08-12 10:58** | 9.88 | 1.71 | resident1 |
| 4.335 | 08-11 02:10 | 10.24 | 2.97 | statepair-conc |
| **4.042** | **08-12 11:10** | 10.11 | 1.58 | residq1 |
| **3.635** | **08-12 10:34** | 3.24 | 1.42 | obs100 |
| **3.608** | **08-12 11:12** | 10.73 | 1.42 | residq20 |
| **3.550** | **08-12 11:39** | 2.83 | 2.19 | resids20 |
| **3.335** | **08-12 11:38** | **1.84** | 2.11 | resids1 |
| 2.396 | 08-03 07:49 | 13.60 | 2.17 | loadgen |

**Today's eight are bolded. Every one is at or below 4.81.**

## The residual does not explain it

Today's holds span `rest` from **0.55 to 10.73** and their job cores span **3.34 to 4.81** — a 20× range in the machine's other load producing a 1.4× range in the job. On 3 and 11 August the same workload reached 15.3–15.5 twice at `rest` 0.49 and 0.77.

So a quiet box today gives 3.3–4.8 cores where a quiet box nine days ago gave 15.3–15.5. **Today's ceiling is not set by the tenant.**

## Nor do the parameters

Every hold in the table runs `cpu-spin --units 100000000 --duty 0.27`, and `--resident` was tested directly across three regimes today — 1 against 20 at `rest` ~9, ~10 and ~2 — agreeing within 3%, 12% and 6%. One hold used `--wait-ms 46` instead of `--duty` and landed at 4.76, inside today's band.

## Both states are flat plateaus, not ramps

Reading the per-sample series rather than the medians:

| hold | job cores, sample by sample |
|---|---|
| tickpair, 08-03 | 0.0, 0.1, **15.5, 15.6, 15.5, 15.4, 15.6, 15.5, 15.4, 15.5, 15.5, 15.6, 15.4, 15.6** |
| slowstate-ratio, 08-03 | 0.0, 0.2, **15.4, 15.2, 15.4, 15.6, 15.4, 15.4, 15.4, 15.1, 15.0, 15.5, 15.6, 15.5** |
| resids1, 08-12 | 0.0, 0.2, **3.3, 3.3, 3.1, 3.2, 3.6, 3.4, 3.3, 2.4, 3.6, 3.4, 3.7** |

**Each reaches its level by the second sample and holds it.** So this is not a ramp that ran out of time, not a decay under sustained load, and not contention accumulating — the box has **two stable operating points at a hundred sessions, 4.7× apart**, and a hold settles into whichever one is in force within about four seconds.

That rules out the explanations a median cannot distinguish: a short hold catching a rise, a long hold catching a fall, or a mixture of the two states within one hold. Every sample in each hold belongs to one plateau.

## The sessions run less often, not slower

Units per core-second — work done per unit of CPU actually obtained, which is the sessions' efficiency rather than their share:

| hold | job cores | units/core-second |
|---|---|---|
| tickpair, 08-03 | 15.53 | 67.6 |
| slowstate-ratio, 08-03 | 15.34 | 58.9 |
| statepair2, 08-11 | 15.51 | 48.8 |
| m1-abs-1200, 08-11 | 7.12 | 74.4 |
| **resids1, 08-12** | 3.34 | **63.2** |
| **resids20, 08-12** | 3.55 | 61.6 |
| obs100, 08-12 | 3.64 | 38.9 |
| waitms100, 08-12 | 4.76 | 38.6 |

**Efficiency is the same in both states.** Today's `resids` pair sits at 62–63 units per core-second, inside the 49–74 that the fast holds span. A session that gets a core does the same work with it whichever state the box is in.

**So the sessions are running less often, not slower**, and the whole 4.7× gap is occupancy. That eliminates in one table every explanation that would cost work per cycle: cache pressure, memory bandwidth, frequency scaling, and thermal throttling would each show as *fewer units per core-second*, and none does.

What remains is a scheduling or wake question — the same hundred sessions obtain a fifth of the machine's time while eleven of sixteen cores sit idle.

## The mechanism: they oversleep, and by how much is the state

Decomposing a unit into the CPU it costs and the wall clock it occupies. A unit is one line; `cycle = 1 / units-per-session-per-second`; `compute = cores-each × cycle`.

| hold | compute/unit | cycle/unit | implied pause |
|---|---|---|---|
| tickpair, 08-03 | 14.8 ms | 95 ms | ~80 ms |
| slowstate-ratio, 08-03 | 17.0 ms | 111 ms | ~94 ms |
| statepair2, 08-11 | 20.5 ms | 132 ms | ~112 ms |
| m1-abs-1200, 08-11 | 13.4 ms | 189 ms | ~175 ms |
| **resids1, 08-12** | **15.8 ms** | **474 ms** | **~458 ms** |
| **resids20, 08-12** | **16.2 ms** | **458 ms** | **~442 ms** |

**Compute per unit is constant at 13–20 ms across every state and every day.** What varies by five times is the *pause*.

And the pause is always longer than requested. `--duty 0.27` asks for `compute × (1 − duty)/duty` — about **40 ms** after a 15 ms compute. The sessions get **80 ms** in the fast state and **458 ms** today: **they oversleep by 2× then and 11× now.**

`--wait-ms` confirms it by a different route: a fixed **46 ms** request, a measured cycle of 543 ms against a 26 ms compute, so **517 ms actual — the same 11×**. A mechanism that inflates both an adaptive pause and a fixed one is in the sleep and wake path, not in the arithmetic that computes the pause.

**This is the oversleep confound the repository records as cleared.** It was cleared from readings at one session, where a lone sleeper wakes on time — and it is the dominant term with a hundred of them. The clearing was correct about what it measured and never reached the regime where the effect lives.

## Oversleep scales with the number of sleepers, and the state multiplies it

The same decomposition across today's session-count series:

| sessions | compute/unit | pause got | pause asked | oversleep |
|---|---|---|---|---|
| 1 | 26.6 ms | 80.4 ms | 71.7 ms | **1.12×** |
| 10 | 27.3 ms | 126.2 ms | 73.8 ms | **1.71×** |
| 30 | 21.6 ms | 235.1 ms | 58.3 ms | **4.03×** |
| 100 | 15.8 ms | 458.6 ms | 42.7 ms | **10.73×** |

**Per session that is an achieved duty of 0.155 in the fast state and 0.033 in the slow one, against 0.27 requested** — the figure the sessions actually run at, which is what a hundred of them multiply into 15.5 or 3.3 cores.

**A lone session oversleeps by 12%; a hundred by nearly eleven times.** That is the whole self-limiting curve without invoking contention — the sessions ask for a 43 ms pause and get 459 ms, so they run a fifth as often on a box with eleven idle cores.

**And it locates the state.** The same hundred sessions overslept **2× on 3 August and 10.7× today**, so the count sets the shape and the state sets the multiplier. Two factors, both measured here, neither named before.

It also explains why a single-session check certified the confound as cleared: at 1.12× there is nothing to see, and the effect that dominates a hundred sessions is invisible at one.

## The daemon is not in the path, and four more candidates are dead

A hundred `cpu-spin --duty 0.27 --resident 1` started directly, output to `NUL`, **no daemon, no `sessionbench`, no job object**:

| | cores each |
|---|---|
| under `coggyd`, today | 0.0334 – 0.0355 |
| **bare processes, today** | **0.0391** |

**The same collapse.** So this is the workload's behaviour on this machine, and nothing in the instrument or the daemon participates.

Four candidates tested and eliminated in the same sitting:

- **Timer resolution.** The effective floor here is **15.64 ms** — 100 × `Sleep(1 ms)` takes 1564 ms, and `Sleep(10 ms)` takes 15.3. But a 15 ms compute at `--duty 0.27` requests a **40 ms** pause, already clear of that floor, so granularity cannot produce either 80 ms or 458 ms. And raising it does not propagate: a helper holding `timeBeginPeriod(1)` left another process still measuring 15.67 ms, so the request is per-process on this build.
- **A job CPU cap.** `sessionbench`'s job object is joined for membership and sets no limits — and the bare-process run has no job at all.
- **The daemon's drain.** Ruled out by the bare run above; the sessions never write to a pipe anyone reads.
- **`sessionbench`'s sampling.** Same reason.

What remains is Windows' scheduling of a hundred short-cycle sleepers, on a box that is 70% idle while it happens. That is where the next measurement has to look, and none of tonight's instruments reaches it.

## General wake latency is not it either

An independent process sleeping 40 ms, measured alone and again while a hundred `cpu-spin` run:

| | Sleep(40 ms) actual | oversleep |
|---|---|---|
| alone | 47.5 ms | 1.19× |
| with 100 sleepers running | 54.6 ms | 1.37× |

**1.15× worse — against the sessions' own 10.7×.** Windows wakes a sleeper essentially on time even with a hundred short-cycle processes running, so the inflation is not the OS failing to schedule wakeups. It is specific to the workload's own loop.

That is the fifth candidate eliminated in this sitting and the last one reachable from outside the workload. **The remaining suspect is `cpu-spin`'s own measurement of `computed`**, which sets the pause: the pause is `computed × (1 − duty)/duty`, so anything that inflates the *measured* compute inflates the pause proportionally — and scheduling delay between waking and running would land inside that measurement rather than beside it.

**That is a hypothesis this record cannot test.** Confirming it needs the workload to record its requested pause against its achieved one, which `cpu-spin` does not do, and adding it is a change to the thing being measured.

## Measured: the sleep is accurate, and the workload asks for the wrong pause

`cpu-spin` gained `--report-timing` (default off), which records the pause it asked for against the pause it got:

| | solo | one session among a hundred |
|---|---|---|
| `computed_ms` | 28–32 | **90–131** |
| `asked_ms` | 76–86 | 244–354 |
| `slept_ms` | 77–87 | 257–365 |
| **oversleep** | **1.01×** | **1.03–1.05×** |

**The sleep is accurate in both regimes.** The workload sleeps within 5% of what it asks for even with a hundred processes running. What changes is what it *asks for*: `computed` goes from ~30 ms to ~130 ms.

`computed` is `working.elapsed()` — **wall time across the spin** — so a session preempted mid-spin measures its own compute as four times longer, and the pause is `computed × (1 − duty)/duty`, so it then sleeps four times longer as a penalty for having been descheduled.

**That is a positive feedback with a stable fixed point.** Contention inflates measured compute → the pause lengthens → demand falls → contention falls. The system settles where those balance, which is why the occupancy is a flat plateau from the second sample and why a hundred sessions sit at 3.3 cores on a box that is 70% idle.

**Nothing is wrong with Windows.** The duty controller measures wall time and counts scheduling delay as work.

## The fixed-pause arm exposes an anomaly the feedback loop does not explain

The same instrument on `--wait-ms 46`, one session among a hundred:

```
timing units 50 computed_ms 491.720 asked_ms 46.000 slept_ms 87.927 oversleep 1.91
```

**`computed` is 491 ms of wall time for roughly 26 ms of actual CPU** — the session is not running for 95% of its own spin.

The two arms reach the same ~0.04 cores each by opposite routes. `--duty` **backs off**: inflated compute lengthens its pause, demand falls, contention falls, and `computed` settles near 130 ms. `--wait-ms` **cannot** back off, so it keeps asking, and `computed` reaches 491 ms.

**But you cannot be descheduled for 95% of a CPU-bound spin on an idle box.** Contention needs competitors, and the machine reads 5.3 of 16 cores busy while this happens. The spin is pure arithmetic with no I/O and no allocation.

**So the leading candidate is now throttling rather than contention.** Windows applies power throttling — EcoQoS — to background, windowless processes, running them at reduced speed on efficiency cores. That would produce exactly this signature: long wall time, little CPU counted against the process, and a machine that looks idle because the work is being done slowly rather than not at all. Every session in every hold here is started windowless by a background harness.

**This record does not test it.** `PROCESS_POWER_THROTTLING_EXECUTION_SPEED` can be queried and set per process, and neither has been done.

## A hundred sessions that never sleep still cannot have the machine

`--duty 1.0` has no pause at all — no feedback loop, no inflated `computed`, no oversleep:

| | cores total | cores each |
|---|---|---|
| **100 × `--duty 1.0`, no sleep** | **6.21 of 16** | **0.0621** |
| 1 × `--duty 1.0` (archive) | 0.99 | 0.99 |

**Each of a hundred holds 6% of a core doing pure arithmetic, while one holds a full core.** No I/O, no allocation, no pause, and ten of sixteen cores unused.

**So the sleep mechanism explains the duty arm's backoff and is not the cause of the ceiling.** Something limits these processes to a fraction of the machine whether or not they sleep, and the earlier sections' feedback loop rides on top of it rather than producing it.

That is the throttling signature with a control attached: **one is unthrottled at 0.99 cores and a hundred are not.** Whatever applies it is sensitive to the number of processes, which is what makes it look like contention while the machine sits idle.

**What this still does not name**: the mechanism that limits them. `GetProcessInformation` for `ProcessPowerThrottling` returned false on both a child and the caller here, so the per-process throttling state remains unread, and EcoQoS is a hypothesis rather than a finding.

## What it costs, retroactively

**Every measurement taken today was taken in this state.** That includes:

- The session-count curve — 92%, 66%, 31%, 13% of requested duty at 1, 10, 30, 100 — whose collapse may be this state rather than a property of session count.
- The claim that a hundred sessions do not saturate this box, withdrawn earlier today on archive evidence and now ambiguous again: they do not saturate it **today**, and they did on 3 August.
- The five rising-limb attempts, whose baselines and voids were all read against a machine in this condition.

It does **not** invalidate the within-today comparisons, which are the ones taken back to back: the `--resident` pairs, the observer attributions, and the gate-rule replay all compare arms measured minutes apart in the same state.

## What this does not establish

- **No cause, but the code is now excluded rather than merely doubted.** `git log --since="2026-08-11" -- coggyd/ workloads/` returns **nothing**: the side that *produces* the work is byte-identical to the code behind the 15.5-core holds. Every change since is in `sessionbench`, which observes. So a regression in the daemon or the workload is ruled out, and what remains is the machine or the observer's own path.
- **Not the documented slow state, necessarily.** That one is worth 2.2× and lasts about ninety minutes; this is 3–5× and has held for at least seventy minutes across eight holds. **And that state was characterised on a solo rate**, where this one leaves per-core efficiency untouched — so they may not be the same phenomenon at all.
- **Not the plug.** Every hold today is on mains, recorded in each artifact.
- **Thermal reads the same.** `doctor` reported 44.1 °C throughout today, as it did on earlier days — which is the reason that sensor is already recorded here as unable to name this box's states.
- **Seventeen holds, three days.** The 08-03 and 08-11 groups each contain both high and low readings, so the day alone does not predict the value either.

## What to do with it

1. **Do not compare today's hundred-session figures with the archive's.** They are not the same machine.
2. **The daemon and the workload are already eliminated by `git log`** — one command, no rebuild, and it should have been the first check rather than the fourth.
3. **The next hundred-session hold on a future day is the test.** If it returns to 15, this was a state that passed. If it stays at 4, the remaining suspects are the machine and `sessionbench`'s own observing path — which can be separated by running one hold from the 08-11 binary, since the daemon it drives is unchanged.
4. **A reboot is the cheapest probe** and has not been tried.

## Provenance

| | |
|---|---|
| Source | every `hold.json` under `bench-out/` with `sessions >= 50`, seventeen holds |
| Fields | `occupancy.median_cores`, `occupancy.rest_cores_median`, `units_per_session_per_sec`, directory unix stamp |
| Machine | 16 logical / 31 GiB / Windows 11, **mains** for every hold in the table |
