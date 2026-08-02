# On undisturbed intervals every footprint gives the same slowdown, and the gate is 3% short at all of them

Tonight built a footprint story: `η` rises 11.4% from 20 to 33 MiB, plateaus by 36, and the gate misses by 3.3% at its best weight. **Most of that 11.4% was one afternoon losing its machine.**

No run was made. This is the three rested holds' own samples, read per interval instead of as a mean.

## The 20 MiB run stopped being measured 18% of the time

Occupancy across each hold's concurrent window, spin-up dropped:

| | median | mean | sd | min | samples under 14 cores |
|---|---|---|---|---|---|
| 20 MiB | 15.37 | **14.31** | **2.64** | **0.58** | **40 of 224 (18%)** |
| 33 MiB | 15.48 | 15.46 | 0.13 | 14.99 | 0 |
| 36 MiB | 15.39 | 15.39 | 0.13 | 15.07 | 0 |

**The medians agree to 0.7%.** The 20 MiB mean is dragged down by dips the other two do not have, one of them to 0.58 cores.

**The dips are real, not a sampler artifact.** Output falls with them in the same interval: at a sampled 9.83 cores the run produced 690 units/s, at 15.43 cores 1040 — 34% less output for 36% fewer cores.

## Compared on intervals where all three held the machine

Restricting to intervals at 15 or more cores puts the three runs at the same occupancy and asks only how well they used it:

| | intervals | cores | units/s | units per core-second |
|---|---|---|---|---|
| 20 MiB | 152 | 15.43 | 1040.2 | **67.41** |
| 33 MiB | 225 | 15.47 | 1061.1 | **68.61** |
| 36 MiB | 226 | 15.39 | 1045.5 | **67.91** |

```
20 -> 33   +1.8%
33 -> 36   -1.0%
```

**The footprint is worth about two percent, not eleven.**

## Which puts the gate at the same distance everywhere

Each run's undisturbed output against its own six solo holds:

| | solo mean | published slowdown | undisturbed | |
|---|---|---|---|---|
| 20 MiB | 21.484 | 2.3010 | **2.0654** | −10.2% |
| 33 MiB | 21.826 | 2.0655 | **2.0569** | −0.4% |
| 36 MiB | 21.837 | 2.0799 | **2.0887** | +0.4% |

**They span 1.5% and every one sits 3–4% above the condition of 2.**

So the figure of record, 2.301 at 20 MiB, carries 10.2% of that afternoon's disturbances. The two runs taken tonight are barely touched, because nothing interrupted them.

## What this retracts

- **That the footprint lever bought 11.4% and then ran out.** It buys about 2% and the plateau is real but small.
- **That 33 MiB is the best weight this machine has.** All three are the same within 1.5%.
- **That `η` is 0.733 at 20 MiB.** On undisturbed intervals it is 0.817, the same as the other two.

What survives, and is now stronger for resting on three runs rather than one: **a hundred sessions at duty 0.27 slow down about 2.06 on this box, whatever they hold, and the gate asks for 2.** The sizing that follows is `C ≥ N·d/(2η)` at `η ≈ 0.82`, which is **16.4 logical processors against sixteen** — under half a core, at every session weight.

## What this cannot say

- **What took the machine during those dips.** The job's own CPU is all these samples attribute; `16 − job` is an inference. The run is from 2026-08-01 and nothing recorded what else was running.
- **Whether the other two runs have smaller versions of the same thing.** Their minima are 14.99 and 15.07 against medians of 15.48 and 15.39, so if they do it is under 3%.
- **Whether a 15-core threshold is the right cut.** It was chosen because two runs never go below it; a different cut would move the 1.8% somewhat.
- **Anything about the hour, or the slow machine state.** All three are twenty-minute holds on a rested box.

## Provenance

| | |
|---|---|
| Inputs | `bench-out/1785586144-m1-daemon`, `1785687266-r33-rested-daemon`, `1785689778-r36-rested-daemon` |
| Machine | not used |
| Commit | `3f43e68` |

## Same night: what the dips look like, and three things they are not

The dips are not scattered. Dropping spin-up, they fall in two episodes:

```
447 - 574 s    11 samples
915 - 1092 s   29 samples
```

Six to seven minutes of clean running between them, and the second runs to within 110 s of the window's end. Two bursts of two and three minutes, not a steady load and not a periodic one.

**Three candidates are ruled out by the samples themselves.**

| | dips (<14 cores) | clean (≥15) |
|---|---|---|
| job | 9.83 cores | 15.43 |
| **Defender** | **0.04 cores** | **0.03** |
| **processes in the job** | **101.0** | **101.0** |
| **free memory** | **10.02 GiB** | **11.67 GiB** |

- **Not Defender.** Flat across both, and small.
- **Not sessions exiting.** Exactly 101 processes throughout — the daemon and its hundred.
- **Not the sessions themselves growing.** Their RSS is bounded by the workload.

**What correlates is 1.65 GiB of free memory going missing** at the same time. Something outside the job held it and used cores for two episodes, then gave both back.

**This is as far as these artifacts go.** The sampler attributed only what was inside the job, so the thing that took the machine cannot be named from here — which is [why every sample now carries the whole machine's CPU](../../sessionbench/src/sampler.rs). The next run that dips will say how much of the machine was busy, and the difference will be a measurement instead of an inference.

### And the other two runs do not have a smaller version of it

The 15-core cut above was chosen because two runs never go below it, which cannot answer whether they carry the same thing at a smaller size. Measured instead against **each run's own median**, counting anything more than 2% under it:

| | median | samples under | mean occupancy lost |
|---|---|---|---|
| 20 MiB | 15.37 | 73 of 225 (32%) | **1.096 cores** |
| 33 MiB | 15.49 | 8 of 228 (4%) | **0.014 cores** |
| 36 MiB | 15.39 | 3 of 228 (1%) | **0.004 cores** |

**Seventy to two hundred and fifty times apart.** The two runs taken tonight lose 0.1% and 0.03% of their occupancy to this; the 20 MiB run loses 7.1% of its median. Its five episodes at this threshold are 60–100 s, 376–391, 427–574, 915–1117 and 1162–1167.

The 1.096 cores lost also reconciles the two ways of reading that run: its median is 15.37 and its mean 14.31, a gap of 1.06.

**So the 2% footprint effect is not contamination at a smaller scale.** Whatever it is, it is not this.

### The shipped metric, which counts every sample under the median

`sessionbench` now reports this per hold and per rung, and it takes no threshold — every sample below the run's own median counts, so ordinary tick noise contributes and the floor is not zero:

| | median | mean | lost |
|---|---|---|---|
| 20 MiB | 15.39 | 14.28 | **1.173** |
| 33 MiB | 15.48 | 15.46 | **0.061** |
| 36 MiB | 15.40 | 15.40 | **0.052** |

As a share of each median that is **7.6% against 0.3–0.4%**, a twenty-fold separation where the 2% variant above gives seventy. The constant is what buys the extra, and a constant taken from this machine would have to be right on every other one.

Spin-up is dropped by the same self-calibrating rule — everything before the run first reaches its own median — which is what stops a short hold reporting its own startup as an interruption.

### A later run confirms the number this record inferred

The re-reading above says the 20 MiB hold's published **9.3363 units/s/session carries that afternoon's interruptions**, and that undisturbed it was doing **10.402** at **67.41 units per core-second**. Those are inferences from intervals inside a run that had already happened.

A fresh hold at the same shape — a hundred sessions, `--resident 20 --duty 0.27`, 90 s counted, rested box — ran afterwards for an unrelated reason:

| | inferred | measured |
|---|---|---|
| units/s/session | 10.402 | **10.4982** — +0.9% |
| units per core-second | 67.41 | **67.73** — +0.5% |
| occupancy lost | — | 0.045 cores, undisturbed |

**+12.4% against the figure of record**, and within a percent of what this record said was underneath it. Nothing was tuned to make that happen: the run's purpose was to match a tick-cost comparison, and its rate was read afterwards.

So the disturbance correction is not only internally consistent — it predicted a later measurement.
