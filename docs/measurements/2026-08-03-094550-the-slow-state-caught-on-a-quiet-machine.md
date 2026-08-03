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
| Inputs | `bench-out/*-q1-daemon` through `q6` — **for the six holds above only.** This record grew thirteen sections past this point and now rests on more than forty one-session holds; the later inputs are named in the sections that use them. |
| Machine | on mains; a watcher had passed two consecutive 30 s windows with no sample above 30%, judged on the worst sample rather than the mean |
| Commit | `bee3133` — the state at the six holds. Later sections were written against later commits, each named in its own text where it matters. |

**This block sits mid-document because the document outgrew it.** It is left where it was written rather than moved, because a log's shape is part of what it records — but a reader arriving at it should know that everything below is a different, larger measurement than the one it describes.

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

## A one-to-three-core neighbour is worth 3.2%, in this state

The thirteen holds all sat in what today's other record calls the quiet band, but *quiet* there spans 1.07 to 2.86 cores held elsewhere. Sorted by that column, the rate follows it:

| rest under 1.5 cores | rest 1.5 and over |
|---|---|
| n = 8, mean **9.176** | n = 5, mean **8.883** |

**−3.2%**, correlation **−0.761** across the range.

Three objections, and two die on the same artifacts:

- **Time and tenancy might be entangled**, with later probes happening to be busier. They are not: time against rest is **+0.109**, and time against rate only −0.207. What predicts the rate is the neighbour, not the clock. **Withdrawn two probes later; see the end of this record.**
- **One point at 2.86 cores might be carrying it.** Dropped, the correlation is still **−0.675** over the remaining twelve.
- **It was measured in the slow state**, and this record spent a section arguing that a spread measured in one state does not carry to another. That objection stands and is not answerable from here.

So: a neighbour of one to three cores moves a one-session hold, downward, by about three percent per core-and-a-half — **on a slowed machine**. Whether the rested machine has the same slope is what [the twelve-hold repeat](2026-08-03-081500-a-neighbour-costs-the-solo-baseline-twenty-seven-percent.md) still wants, and it now has a number to test against rather than an open question.

**Why it matters at that size.** The gate misses its work-rate condition by 3–4%. `slowdown = solo ÷ concurrent`, so a neighbour suppressing the baseline by 3.2% moves the verdict by about as much as the verdict misses by, in the direction that flatters it. The exclusion earlier today cleared the gate's baselines of a *large* tenant and explicitly could not clear a small one; this is the first measurement of what a small one is worth.

## Two more probes withdrew the causal half of that, and kept the useful half

The section above dismissed one objection with a number: time and tenancy are not entangled, **+0.109** over thirteen points. Two probes later — rest 3.58 and 3.81 cores, the highest of the series and the latest — the same correlation is **+0.668** over fifteen.

| | n=13 | n=15 |
|---|---|---|
| rest vs rate | −0.761 | **−0.859** |
| time vs rest | +0.109 | **+0.668** |
| time vs rate | −0.207 | **−0.604** |

**So the confound that was tested and cleared came back when the sample grew.** The clearing was true of those thirteen points and did not survive two more; a neighbour that arrives late is indistinguishable, in this series, from a machine that slows late. **The attribution to the neighbour is withdrawn.** What stands is the correlation and the fact that something moved together with tenancy.

**What survives is the part that matters, and it is arithmetic rather than a fit.** Least squares over the fifteen gives −0.396 units/s per core held. At the least-tenanted point in the whole series, 1.07 cores, the fit reads **9.24** against a rested band of **18.9**. So even extrapolating the neighbour term to zero leaves the machine at half speed: **a neighbour explains none of the halving.** The state is not tenancy, whatever the slope inside it turns out to be.

That was the question the series was launched to answer, and it is answered in the direction that makes the slow state a real thing rather than a mislabelled crowd.

**Read the previous section knowing this one exists.** It is left standing because it is what the thirteen points said, and because a dismissal that expires after two more readings is worth seeing whole.

## Seventeen points, 214 minutes, and two series that behave differently

Two more probes at forty-minute spacing, both still slow. The correlations recomputed at each sample size:

| n | rest vs rate | time vs rest | time vs rate |
|---|---|---|---|
| 13 | −0.761 | +0.109 | −0.207 |
| 15 | −0.859 | **+0.668** | −0.604 |
| 17 | **−0.875** | **+0.433** | −0.318 |

**One of these converges and the other wanders.** Rest against rate tightens monotonically as points arrive. The time confound went from nothing to strong to half of strong, moved each way by two readings — probe 10 came back at the lowest tenancy of the whole series, 1.06 cores, and the highest rate, 9.467.

So the withdrawal two sections up was right for a reason slightly different from the one given there. It is not that the confound is real and the slope is not; it is that **a correlation over fifteen points is not a quantity you can act on**, and the one that moved 0.56 on two readings is exactly the one that was used to clear an objection.

What survives is unchanged and does not depend on any of this: at 1.06 cores held — the quietest reading in 214 minutes — the machine still runs 9.467 against a rested band of 18.9. **A neighbour explains none of the halving.**

**And the floor is now 214 minutes**, with the state present at the first reading and the last. Seventeen holds, rate 7.862 to 9.467, mean 8.950, and nothing anywhere has yet watched this state end. The *about ninety minutes* this project quoted is not a duration; it is the gap between one observation and the next one that happened to be normal.

## Closing the series: nineteen points, 275 minutes, and the confound that was never there

Two final probes fifty minutes apart, both still slow. The whole series:

| n | rest vs rate | time vs rest | time vs rate |
|---|---|---|---|
| 13 | −0.761 | +0.109 | −0.207 |
| 15 | −0.859 | +0.668 | −0.604 |
| 17 | −0.875 | +0.433 | −0.318 |
| **19** | **−0.875** | **+0.170** | **−0.010** |

**Time explains nothing and tenancy explains what there is to explain.** The confound peaked at +0.668 on two readings and decayed to +0.170; time against rate ends at −0.010, which is no relationship at all. Rest against rate sat at −0.875 from seventeen points on and did not move.

So the withdrawal three sections up was right and its stated reason was wrong. It said the confound is real and unseparable. It is not real — **it was two points**. What was right is the other half of that paragraph: a correlation over fifteen points is not a quantity to act on, and the thing used to clear an objection was the thing least able to bear the weight. Four snapshots of the same number, taken as it grew, span 0.56.

**The neighbour slope survives as a correlation, still measured only in this state**, and the arithmetic that never depended on it is unchanged: at 1.03 cores held the machine runs 9.608 against a rested band of 18.9.

**Final floor: 275 minutes**, nineteen holds, rate 7.862 to 9.608, mean 8.997, and the state present at the first reading and at the last. Nothing has ever watched it end. Tracking stops here because five hours of two-minute probes have said all a probe can say; what would settle the duration is a machine that is left alone and asked once an hour, which is not this session.

## A twentieth reading, and the decision to stop asking

Forty-five minutes after the series closed, one more 120-second hold: **8.644 units/s at 2.60 cores held.** Still the slow state, now **at least 340 minutes** from the first reading.

Twenty questions, twenty identical answers, no trend and no end. **Continuing is hope rather than measurement** — the state is flat, nothing in the series predicts its end, and each probe spends two minutes of the machine it is watching. What would settle the duration is a box left alone and asked once an hour, which is not a session that also wants to run holds.

So the window this was waiting for does not open today, and that is a result: **a run whose verdict is a ratio can be blocked for the better part of a day on this hardware**, by a condition that reports nothing and that `doctor` cannot see. The check costs one hold, the answer arrives in two minutes, and on 2026-08-03 it was the same answer twenty times.

## An excursion, and the flatness that was a marker stops being one

The section above stopped asking after twenty identical readings. One more was taken anyway — two minutes against a five-hour wait is a positive expectation whatever the odds — and it came back **outside every band this record had drawn**.

| | rate | cores held elsewhere |
|---|---|---|
| window-check-2 | **15.216** | 2.01 |
| transit 1 | 12.636 | 1.39 |
| transit 2 | 9.299 | 0.87 |
| transit 3 | 9.874 | 0.93 |

Four holds inside about ten minutes, the machine quiet throughout and getting quieter. **15.216 is 58% above the slow band's ceiling of 9.608**, and by the fourth hold it is back inside it.

**What this costs the earlier reading is the flatness.** Twenty holds over 340 minutes sat between 8.606 and 9.435 and did not trend; that steadiness was written up here as the state's own property — *steadier than the fast machine is*. It moved 39% in ten minutes. So flatness described 340 minutes of it and is not a marker of the state.

**And the tenancy relation runs the other way here.** Across those four the machine gets quieter — 2.01, 1.39, 0.87, 0.93 cores held — while the rate falls. The nineteen-point series gave rest against rate at −0.875; these four are positive. Four points establish nothing, which is the same reason the −0.875 was not treated as a cause, but they do rule out reading this excursion as a neighbour leaving.

**No mechanism is claimed.** What was measured is that a machine locked at half speed for five and a half hours reached 15.2 for at least one two-minute window with nothing else running, and returned. Whether that is the state ending and re-forming, a sampling artifact of something with a period longer than two minutes, or an excursion that means nothing, four holds cannot say.

**What it does settle is a procedure.** A single 120-second hold names a band, and that band is what several checks here now key on. This sequence shows a single hold landing 58% away from where the next one lands. **The check is cheap enough to repeat, so repeat it** — one reading opens a window, two agreeing ones justify spending fifteen minutes in it. **And *agreeing* means against the allowance this project already uses, not by eye**: the closing pair here, 9.299 and 9.874, look tight and sit **6.0% apart**, past the 5% `SOLO_AGREEMENT_PERCENT` a bracket refuses on. The verdict is unchanged — both are inside the slow band, so the window is shut either way — but the pair does not license a run on its own.

## Eight points after the excursion, and a settling rule that passes when it should not

The pair rule written above — *one reading opens a window, two agreeing ones justify a run* — was applied. Two holds, launched detached after a first attempt died at a tool timeout that its own arithmetic predicted:

| | rate | rest |
|---|---|---|
| pair 1 | 11.022 | 1.39 |
| pair 2 | 10.074 | 1.19 |

Both land under 11.5 and so read as the same band, which the rule calls settled. **It is not.** Taken with the four before them the sequence is **15.216 · 12.636 · 9.299 · 9.874 · 11.022 · 10.074** across about half an hour, quiet throughout, crossing band boundaries three times. These two agree only in the sense that a wandering series will sometimes put two adjacent points on the same side of a line.

**And 9.0% apart is not agreement here.** Twenty holds spanning 340 minutes of the settled state covered 9.6% in total. A pair separated by nearly that much is not evidence of a level; it is one draw from a distribution as wide as the whole earlier series.

So the rule needs its second half: **same band is necessary and not sufficient, and the pair's gap has to be small against the band's own width** — which for this box's slow band is 9.6%, so 9.0% clears nothing. What passes is a pair inside a few percent, in the same band, with both rest figures low.

That is the fourth correction this record has made to a rule stated in it, and every one came from the next two readings rather than from an argument. **The window did not open in this session.** Six points after the excursion say the box is somewhere between its slow state and its rested one, moving, and no pair taken two minutes at a time has caught it still.

## Forty minutes of not settling, and a rule that earned its second half twice

The tightened rule — one band **and** within a few percent — was applied again fifteen minutes later. Two more holds, detached:

| | rate | rest |
|---|---|---|
| pair3 1 | 10.064 | 1.16 |
| pair3 2 | 10.615 | 1.22 |

**5.3% apart.** Same band, and refused. Under the rule as first written — same band is enough — this pair passes too, so the correction has now caught two pairs in two applications.

The full sequence since the excursion, quiet throughout at 0.87 to 2.01 cores held:

```
15.216 · 12.636 · 9.299 · 9.874 · 11.022 · 10.074 · 10.064 · 10.615
```

**Forty minutes, eight readings, and no pair inside 5%.** The last four sit between 10.0 and 11.0, which is above the settled band's ceiling of 9.435 — but that band spans 9.6% of itself, so a tenth above its top is inside the noise it already carries. **Whether the level moved is open**; what is measured is that the box has not held still long enough for a pair to agree.

**And that is the session's answer to its own question.** The window `#56` needs never opened. Twenty readings said the state was flat and unmoving; eight more, taken after one excursion, say it is neither. Both are the same box within six hours, and the only instrument that told them apart was two holds instead of one.

## The 5% the pair rule borrowed refuses half of all adjacent pairs

Two pairs were refused above and read as evidence that the box would not hold still. Twenty-six adjacent one-session pairs exist in today's artifacts — every consecutive pair less than twenty minutes apart — and their gaps are:

```
0.1  0.2  0.2  0.4  0.6  1.5  1.5  1.9  2.0  2.7  3.3  4.3  4.7  4.9
5.3  5.8  6.0  6.2  6.2  9.0  9.2  11.0  15.7  18.5  25.5  30.4
```

**Median 4.8%.** So a 5% threshold passes 14 of 26 and refuses 12 — it cuts the distribution almost exactly in half.

**That weakens the reading those two refusals were given.** 5.3% sits a whisker above the median and is close to a coin toss; 9.0% is in the top third and says more. Two refusals are not evidence of a wandering machine when the threshold refuses half of everything.

**The distribution has two groups and 5% is not between them.** Fourteen gaps run 0.1 to 4.9, five run 5.3 to 6.2, and seven sit at 9.0 and above. The lower group reaches 0.1%, so this box does repeat tightly when it repeats. A cut around **6.5%** would take both lower groups and refuse the 9.0-and-up tail — 19 of 26 rather than 14 — and it is a figure from this machine on this workload, which is why it is written here and not compiled in.

**The 5% was borrowed rather than derived.** It is `SOLO_AGREEMENT_PERCENT`, calibrated for a bracket's two *sides* — each a mean of three holds — against ramp-to-ramp gaps of 0.0 to 4.2%. A single pair of raw holds is a noisier object than a difference of means, and the pair rule took the constant without asking whether it fit. **A threshold for means does not transfer to individuals**, which is the same shape as taking a spread measured in one machine state into another.

## It settled, at a level that is neither of the two known ones

Two more holds, and this time they agree: **10.599 and 10.389, 2.00% apart**, both quiet at 1.16 and 1.32 cores held. With the 10.615 before them that is three readings inside 2% across about fifteen minutes.

**The wandering ended.** The full run from the excursion: 15.216 · 12.636 · 9.299 · 9.874 · 11.022 · 10.074 · 10.064 · 10.615 · 10.599 · 10.389 — half an hour of no pair agreeing, then three that do.

**And the level it settled at is not one this record knew.** 10.5 is **15% above** the twenty-hold slow band's mean of 8.997 and **44% below** rested. The slow band spans 9.6% of itself, so 10.5 is outside it rather than in its tail. Whether that is a third level of the same phenomenon, a partial recovery, or the beginning of one that stalled, three readings cannot say — but the box this session hands on is measurably not the box it spent five hours in.

**The threshold change bought nothing here, which is the honest way to record it.** The pair rule's "few percent" was retuned from a borrowed 5% to about 6.5% on the measured distribution of adjacent pairs. This pair is 2.00% apart and passes either one. So the retune is derived from a measurement and has not yet reversed a verdict; what it is for is pairs like the 5.3% one refused an hour earlier, and none has arrived since.

## Thirty minutes later the level had moved, and the pair rule passed anyway

The three holds at 10.5 were left alone for half an hour and re-measured:

| | rate | rest |
|---|---|---|
| pair5 1 | 9.682 | 0.93 |
| pair5 2 | 10.099 | 0.95 |

**4.2% apart, same band, quiet — so the pair rule calls this settled**, and it also called the 10.5 triple settled thirty minutes earlier. The two "settled" readings are **8.7% apart from each other**.

So the rule's verdict means *stable across these ten minutes* and not *this is the level*. Two windows can each be internally tight and sit well apart, which is the same thing twenty holds over 340 minutes said before one reading hit 15.216 — **tightness is a property of the window you looked through.**

**What the box actually did**, across the whole session: 8.6–9.4 for 340 minutes, an excursion to 15.2, half an hour of wandering, a quarter hour at 10.5, and now 9.7–10.1 — sitting just above the original band's ceiling of 9.435. It may be returning to where it started. Two readings cannot say, which is the same sentence this record has written four times, and the reason it keeps being true is that two readings is what a two-minute check buys.

**The session ends here.** The window `#56` needs — quiet *and* rested — never opened: every one of the twenty-nine holds taken today after the state appeared came back under 15.3, and rested on this box is 18.9.

## Fourteen points say what no pair could: it did not go back

Every pair in the last two hours was read on its own and each said only *stable across these ten minutes*. Read as a series — sliding two-hold means, in order — the same readings say more:

```
8.92 → 13.93 → 10.97 → 9.59 → 10.45 → 10.55 → 10.07 → 10.34 → 10.61 → 10.49 → 9.89
```

**The excursion is two windows wide and everything after it sits between 9.6 and 10.6.** What was written up as half an hour of wandering was the tail of one jump; the eight windows since span **1.0 units/s**, about 10% of themselves, which is the same relative width as the twenty-hold band that preceded all of this.

**So the box did not return.** The original band ceiling is 9.435 and these eight sit above it — closer to it than to the excursion, and not on it. Rested is 18.9, so this is not recovery either. Whatever the machine is doing, it moved once and settled somewhere else.

**This is the reading a pair cannot produce.** Four times in this record two holds were declared unable to say something, and each time that was true. Fourteen holds, taken two at a time over two hours for other reasons, say it between them — which is an argument for keeping cheap artifacts rather than for taking more of them: [nothing here was run to answer this question](../../CLAUDE.md).

## The sixteenth point broke the level the fifteen had established

One more pair, taken to extend the series rather than to ask anything: **9.802 then 12.638**, both quiet at 1.15 and 1.57 cores held. **25.3% apart** — in the top tail of the 26 adjacent pairs measured today, beside 25.5 and 30.4 — and the window mean of 11.22 is outside the 9.6–10.6 band the previous eight windows held for an hour.

**That is the third time today a level died on the reading after it was written up.** Twenty holds over 340 minutes, flat, then 15.216. Three holds at 10.5 inside 2%, then 9.7–10.1 half an hour later. Eight windows inside 1.0 units/s for an hour, then this.

So the most durable thing measured on this box today is not a level but its absence: **whatever window you look through, the next one is somewhere else.** Each of those three write-ups was correct about its own window and wrong the moment it was read as a property, and the pattern only became visible because the next reading was cheap enough to take.

**What the next session inherits** is therefore a rule rather than a number: the box was between 9.6 and 12.6 in the last two hours, it is not rested — 18.9 — and any figure taken from it describes the ten minutes it was taken in. The check before a run is the series, not a pair, and the series says this machine has not been still since its excursion.

## Forty-two holds over 9.3 hours, and one of them was rested

The session's last pair — 11.402 then 10.405, quiet at 1.26 and 1.18 — closes the series. Every one-session hold carrying the rest column, classified by its own two numbers:

| | n | share |
|---|---|---|
| slow | 33 | 78.6% |
| tenanted | 5 | 11.9% |
| between the bands | 3 | 7.1% |
| **rested** | **1** | **2.4%** |

Two more rested holds exist — 17.561 and 21.405 — from before the column shipped, so they cannot be classified by the same rule and are excluded rather than counted twice. Either way the shape holds: **across 9.3 hours this box offered a rested two-minute window about as often as it offered anything else three times over.**

**That is the number [gate M1's hour](../../ROADMAP.md#m1--headless-daemon) has to survive.** The run needs a rested machine for sixty consecutive minutes; the machine supplied one rested two-minute hold in a day of measuring. This was two observations at breakfast and is forty-two now, and it did not take a single run of its own — every hold was taken to answer something else and classified itself on the way past.

**What made the tally possible is the same thing that made it late.** The rest column arrived this morning, so two of the three rested holds this box produced sit outside the count. A classification is only as old as the field it classifies on.

## Three of those holds were recoverable, and the tally moves to 4.4%

The section above counted holds whose `hold.json` carried `rest_cores_median` and set two rested holds aside as unclassifiable. **One of them was not.** `fingerprint-2` predates the field but not the fix behind it: its samples carry `machine_cpu_percent` at the corrected scale — a machine median of 326.3, which is 3.26 cores — so its rest of 2.99 is real and it classifies as rested.

`clockpair-idle` genuinely cannot be recovered. Its machine median is **24.1**, and a box with a session running cannot have its whole machine under a quarter of one core, so that figure is on the pre-fix 0–100 scale and the 0.03 cores it implies is meaningless. **The discriminator is arithmetic rather than a timestamp**: on this hardware the corrected column cannot read below about 100 while a session runs.

Recomputed over every hold whose rest is either stored or recoverable:

| | n | share |
|---|---|---|
| slow | 33 | 73.3% |
| tenanted | 7 | 15.6% |
| between the bands | 3 | 6.7% |
| **rested** | **2** | **4.4%** |

**45 holds, and the conclusion is unchanged in shape while the number moved by 80%.** Rested was 1 of 42; it is 2 of 45. Gate M1 still needs sixty consecutive minutes of a state this box offered twice in a day — but *2.4%* was wrong, and it was wrong because a field's absence was read as a measurement's absence when the measurement was one directory away.

