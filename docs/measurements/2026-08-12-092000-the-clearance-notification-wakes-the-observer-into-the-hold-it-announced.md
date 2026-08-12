# The clearance notification wakes the observer into the hold it announced

**Three baselines taken in a nine-minute window with the ten-core tenant entirely absent came back at 2.29, 2.71 and 2.17 cores held, while the gate had read 0.81–0.97 machine-wide seconds earlier. The harness is not the difference: `sessionbench` and `coggyd` together cost 0.064 cores across a hold's first five seconds. The agent costs 1.58, and it is awake at exactly those moments because the gate's own clearance event is what wakes it.**

## What was suspected, and what it was

The gap between a gate reading and the baseline that follows it has appeared all night at 1.06, 1.00, 0.14, 1.34 and 1.90 cores. With the tenant present it could always be blamed on the browser returning, and [once it was](2026-08-12-064500-the-idle-floor-sits-on-top-of-the-transition-it-is-measured-across.md).

A nine-minute window with `chrome-headless-shell` absent removed that explanation, leaving a hypothesis: the abort predicate fires at **sample 1, five seconds in**, which is when the hold is still spawning — and `sessionbench` and `coggyd` sit *outside* the job object, so their cost lands in `rest` by construction. An [earlier attribution](2026-08-12-045500-the-anonymous-neighbour-is-a-software-rasteriser-and-the-observer-is-bigger-than-the-treatment.md) found both under 0.02 cores, but it sampled 8–28 seconds in, with startup already past.

Measuring the first five seconds directly refutes it.

| process | cores, first 5.1 s of a hold |
|---|---|
| **`claude` (two processes)** | **1.58** |
| `cpu-spin` — the measured session | 0.25 |
| `MsMpEng` | 0.17 |
| `WmiPrvSE` | 0.14 |
| **`sessionbench` + `coggyd`** | **0.064** |
| total attributed | 2.82 |

**The harness costs 0.064 cores at startup.** The hypothesis is dead.

## The arithmetic closes by an independent route

During run 57608 the gate polled a machine reading 0.81–0.97 with the tenant gone. Adding the observer's 1.58 to a floor near 0.85 gives **≈2.4** — against baselines of 2.29, 2.71 and 2.17.

That is not a fitted number. The floor comes from the gate's own polls, the 1.58 from a per-process diff taken separately, and the 2.1–2.7 from three holds. Three sources, one sum.

## The mechanism is the instrument announcing itself

The gate clears. The watcher emits `quiet: mean 0.95 over 3 polls`. **That notification wakes the agent**, which writes a reply — and the reply overlaps the baseline hold the clearance has just started.

So the observer is not merely present during measurements. It is *summoned* by the event that begins them, and it is largest precisely in the seconds that decide whether a baseline qualifies. Every clearance tonight was followed within seconds by agent activity worth more than the entire injected treatment.

The three baselines are the evidence: `2.29`, `2.71`, `2.17`, taken after clearances at `0.95`, `0.81` and `0.97`, in a window where nothing else was running.

## Why "stay idle" did not cover it

The standing rule says do nothing while a measurement runs, and it was followed in the ordinary sense — no builds, no commits, no edits during the run. What it did not anticipate is that **an agent driven by notifications has no idle state it controls**. Each clearance, each void, each heartbeat produces an event, and each event produces a turn.

A waiting run therefore generates its own interference on a schedule set by the thing being measured, and the busier the gate, the worse the contamination.

## What this does not establish

- **It does not restore the rising-limb design.** Subtracting 1.58 from 2.29 gives 0.71, below the 1.3 guard — but that is arithmetic on a figure measured at a different moment, not a baseline. The correct claim is that these three holds are *uninterpretable*, not that they would have passed.
- **It does not re-open the abort predicate.** [The predicate](2026-08-12-053500-the-rising-limb-is-reachable-and-the-tenanted-arm-is-what-loses-it.md) did exactly what it was told with the ceiling it was given; it was reading a real 2.29 cores. The reading was contaminated, not the check.
- **The 1.58 is one sample of a working agent**, taken while running the attribution command itself. It is a plausible figure for a turn in progress and not a constant.
- **Nothing here says how much of tonight's earlier data is affected.** Holds taken while no notification had recently fired may be clean; nothing distinguishes them without per-hold attribution, which no artifact carries.

## What it changes

1. **A hold that matters must not be observable by the thing that would react to it.** Either the watcher stops emitting during holds, or the run completes its attempts before reporting, or the agent's own processes are excluded from `rest` — the last being the only one that makes the artifact self-correcting.
2. **`rest_cores_median` should exclude the agent**, on the same reasoning that it excludes the session's job: it is not a neighbour the daemon competes with, it is the instrument's operator. That is a change to what the column means and needs stating wherever the column is documented.
3. **The floor-band finding needs re-examining**, since some of its eleven gated baselines were taken in the seconds after a clearance event.

## Provenance

| | |
|---|---|
| Startup attribution | `Get-Process` snapshot before launch, diff of `TotalProcessorTime` 5.1 s after, ranked |
| The hold | `sessionbench hold --label startupcost --sessions 1 --interval 5 --duration 30 -- cpu-spin --duty 0.27 --resident 20` |
| The three baselines | `bench-out/inject-20260812-090516.json`, outcome `NoQuietWindow`, 3 voids |
| Window | tenant watcher reported absent 09:04:47, returned 09:13:50, absent again 09:16:52 |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, 0 survivors after teardown |
