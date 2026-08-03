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

Unmeasured, and the candidates are not exotic: this machine spent the afternoon under a hundred saturating sessions of our own, a third-party tenant at 99%, and repeated full test runs. The state is bracketed at somewhere between twenty and forty-one minutes of saturation to induce, and about ninety minutes to pass. Today cleared that bar several times over.

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
