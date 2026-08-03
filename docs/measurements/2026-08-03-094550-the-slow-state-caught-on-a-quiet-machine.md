# The slow state, caught on a machine proven quiet at the same instant

Six one-session holds inside a window a watcher had certified quiet. Every one returned about **9.1 units/s** — 48% of this box's rested rate — while its own samples put **1.07 to 1.92 cores** in the hands of everything else. Nothing was competing. The machine was simply half itself.

## The six holds

`hold --sessions 1 --interval 5 --duration 120 -- cpu-spin --units 100000000 --duty 0.27 --resident 20`

| | rate | cores held elsewhere |
|---|---|---|
| q1 | 8.994 | 1.92 |
| q2 | 8.954 | 1.39 |
| q3 | 9.253 | 1.07 |
| q4 | 9.275 | 1.56 |
| q5 | **9.412** | 1.07 |
| q6 | 8.980 | 1.25 |

**Mean 9.144, spread 5.0%.**

## Three bands, and the middle one is the trap

Every one-session hold measured today, sorted by what explains it:

| | n | mean | range |
|---|---|---|---|
| quiet and rested | 2 | 18.928 | 17.561–20.295 |
| **crowded** | 7 | **13.792** | 13.002–15.709 |
| quiet and slow | 6 | **9.144** | 8.954–9.412 |

**A crowded rested box is faster than a quiet slow one.** So a rate alone cannot name the state — 13.8 is neither of the two things a reader would guess, and any of these three bands can be reached from either cause. What separates them is the second column, which every hold has recorded only since this morning.

## What this is the first of

The slow state has been identified in this project by one thing: a solo hold's own rate, about 9.4 against about 21.5. That test cannot distinguish a machine that has slowed from a machine that is busy, and until today nothing recorded the difference. **This is the first observation of the state with a neighbour ruled out at the same instant** — the same window a watcher had passed on two consecutive thirty-second samples with no reading above 30%.

**And the state is steady where the machine is not.** These six spread 5.0%; the two rested holds today spread 14.4% and the seven crowded ones 19.6%. Half speed, and quieter about it than the fast machine is.

The 9.4 this project has quoted for the slow state turns out to be the **top** of the band rather than its centre.

## What put the box here

Unmeasured, and the candidates are not exotic: this machine spent the afternoon under a hundred saturating sessions of our own, a third-party tenant at 99%, and repeated full test runs. The state is bracketed at somewhere between twenty and forty-one minutes of saturation to induce, and about ninety minutes to pass. Today cleared that bar several times over. **Narrowed at the end of this record**: the timeline puts this box rested nineteen minutes after our own saturating hold, so that load is excluded for this instance.

## What it costs the gate

`slowdown = solo ÷ concurrent`. A bracket taking its baselines in a window like this gets **six baselines agreeing to 5%** — comfortably inside the allowance — around a numerator that is 48% of the machine's rested value. The run would report a slowdown against a box that is not the box the gate is about.

That is the same failure [three baselines under a tenant produced](2026-08-03-073401-three-baselines-agreed-to-under-three-percent-and-were-all-wrong.md), reached by the opposite road: there the machine was busy and the baselines agreed, here it is quiet and they agree. **Agreement is a statement about two numbers.** The rest column separates the causes; nothing separates them from the rate.

## What this run was launched for, and did not answer

It was the opportunistic first half of [the twelve-hold repeat](2026-08-03-081500-a-neighbour-costs-the-solo-baseline-twenty-seven-percent.md) — whether the 4.54% behind `SOLO_AGREEMENT_PERCENT` is this machine's property or one afternoon's, and what a two-or-three-core neighbour costs inside the quiet band. **Neither is measurable here.** A slow box is a different machine, so its spread is its own and its quiet band is not the rested one. The questions keep waiting for a window that is quiet *and* rested, which today has produced twice, both times for a single hold.

## Provenance

| | |
|---|---|
| Inputs | `bench-out/*-q1-daemon` through `q6` |
| Machine | on mains; a watcher had passed two consecutive 30 s windows with no sample above 30%, judged on the worst sample rather than the mean |
| Commit | `bee3133` |

## The reading was fixed before the run, and named this outcome least likely

Three branches were written down first: four or more quiet holds tight together gives the fingerprint; rates moving with the rest figure means a tenant; **rates moving while the rest stays quiet means suspect the box** — recorded at the time as "the most interesting and the least likely". It arrived, and the note is what makes it a prediction rather than a preference.

## The timeline says our own saturation did not cause it

Minutes are from the start of a deliberate hundred-session hold, which ran for 150 seconds.

| | run | rate | rest |
|---|---|---|---|
| +0.0 | 100 sessions, saturating | 2.165 | 13.60 |
| **+19.1** | 1 session | **20.295** | **1.24** |
| +21.6 | 1 session | 15.709 | 13.40 |
| +24.3 | 1 session | 13.418 | 10.47 |
| +99.7 | 1 session | 8.994 | 1.92 |
| … | four more | 8.95–9.41 | 1.07–1.56 |
| +113.0 | 1 session | 8.980 | 1.25 |
| **+128.3** | 1 session | **8.606** | **2.86** |

**Nineteen minutes after the saturating hold this box was rested**, at the top of the rested band with nothing else running. So the 150-second load did not induce the state — a third negative beside the three-minute and twenty-minute bursts that also failed.

**What ran between +24 and +99 was not ours.** A third-party tenant held eleven to thirteen cores across that window, in bursts of minutes, alongside repeated compiles. The state was present by +99.7 and has held since.

That is worth stating carefully. It does not show the tenant caused it; nothing was controlled and compiles ran too. What it does show is that **the inducing load need not be a hundred sessions of our own**, which is how every earlier attempt was framed, and that a machine can arrive in this state while nobody is running a benchmark on it.

**Duration gets a floor from inside a single observation for the first time.** Slow at +99.7 and still slow at +128.3 is **at least 28.6 minutes**, ongoing. The *about ninety minutes* this project quotes came from one earlier observation; this one is not finished and cannot yet confirm or refute it.

**And 8.606 sits below the six-hold band.** One point, 3.9% under the band's minimum, taken fifteen minutes after it. Whether the state deepens or that is a hold's own noise needs more than one reading — the band's own spread is 5.0%, so it does not clear it.

## Twelve points, 88.4 minutes, and the end was never seen

Six back-to-back holds, then a probe every twelve minutes until the watch expired. Every one quiet — 1.07 to 2.86 cores held by anything else — and every one in the slow band.

| minutes | rate | rest | | minutes | rate | rest |
|---|---|---|---|---|---|---|
| 0.0 | 8.994 | 1.92 | | 28.6 | 8.606 | 2.86 |
| 2.7 | 8.954 | 1.39 | | 37.6 | 9.435 | 1.08 |
| 5.4 | 9.253 | 1.07 | | 50.3 | 8.868 | 1.19 |
| 8.0 | 9.275 | 1.56 | | 63.0 | 8.817 | 1.79 |
| 10.7 | 9.412 | 1.07 | | 75.7 | 9.346 | 1.11 |
| 13.3 | 8.980 | 1.25 | | 88.4 | 9.164 | 1.49 |

**Mean 9.092, spread 9.1% over 88.4 minutes.**

**This is a floor, not a duration.** The state was present at the first reading and at the last; nothing here saw it end. *About ninety minutes* has been quoted from one earlier instance where the next measurement happened to be normal — a gap, not a transition. **Neither observation has watched this state stop.**

What the floor is worth is a decision rather than a description: a gate hour that fails and slows the box cannot be retried for at least an hour and a half, and that is now measured rather than assumed.

**Flat across an hour and a half.** The series does not trend — 9.4 at 37 minutes, 8.8 at 63, 9.2 at 88. Whatever this is, it neither deepens nor lifts while it holds, which is what "two steady levels rather than a decay" has meant since the state was first described, now with twelve points instead of two.

### The spread grew and then stopped, and it does not travel

Six holds over 13 minutes spread 5.0%; twelve over 88 spread 9.1%. So the width belongs to the window, not to the machine — the same shape as [a solo rung reproducing to 0.37% inside one ladder against 8.5% between two triples ten minutes apart](2026-08-01-163935-what-the-harness-says-about-itself.md). It stopped growing after about an hour, which is either a bounded wander or this workload's own floor at that length.

**And the answer does not carry to a rested box.** Reading 9.1% as "the hold's noise floor" and applying it to the rested band was written down here and withdrawn a few minutes later: the rested figure of record is **4.54% over six minutes**, less than half of it, from twelve holds on a machine that was not in this state. A noise floor measured in one machine state is a fact about that state. It is the third instance of a rule this repository already carries in two forms — a conclusion drawn from a stand-in workload, then from a stand-in window, and now from a stand-in *state*.

