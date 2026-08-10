# The slow state flatters the gate

A bracketed hundred-session hold, run to ask whether the slow state distorts gate M1's work-rate ratio. **It does, and in the direction that would have produced a false pass.**

The bracket refused itself, so no slowdown is published. What follows is built on the concurrent hold, which stands on its own, and on the range of the solo holds rather than their mean.

## The run

| | |
|---|---|
| Harness | `sessionbench` 0.0.0 at `b78428a6980f`, release, clean tree |
| Shape | 30 s uncounted warm-up, 3 solo x 60 s, 100 sessions x 300 s, 3 solo x 60 s |
| Workload | `cpu-spin --units 100000000 --duty 0.27 --resident 20` |
| Power | mains, 100%, `SAMSUNG MODE` |
| Machine | 16 logical, 16 physical, 33.7 GB |

## The bracket refused, and not because of a neighbour

| side | 1 | 2 | 3 | mean | spread | std err | rest cores |
|---|---:|---:|---:|---:|---:|---:|---:|
| before | 12.280 | 14.124 | 15.271 | 13.891 | 21.5% | +/-6.3% | 1.84 |
| after | 11.330 | 9.801 | 14.129 | 11.754 | 36.8% | +/-10.8% | 1.30 |

The sides sit **16.7% apart** against a 5% allowance. **Both rest figures are low**, so this is the machine wandering rather than a tenant holding cores -- the distinction the rest column was added to make, doing its job on the first bracket that needed it.

The before triple climbs **monotonically, +24% across three consecutive holds**, immediately after the hundred-session warm-up. That is a candidate mechanism rather than a finding: the warm-up exists to remove cold-start bias from the baseline, and it is itself a saturating burst. One triple is not enough to say so, and the after triple is not monotone.

## What the concurrent hold says, which the refusal does not touch

| condition | figure | verdict |
|---|---:|---|
| peak RSS | 2.55 GB against a 4 GB budget | held |
| dropped output | 0 failed reads, 0 truncated | held |
| sessions | 100 held, fewest running 100 | held |
| occupancy | 15.34 cores median, 0.77 held outside the job | -- |

**Two of gate M1's four conditions pass at a hundred sessions on this box**, in the slow state, on a quiet machine. The refusal is specific to the ratio, and a ratio is the only thing it invalidates.

## The state moves the solo far more than it moves the hundred

Against [the 20 MiB run that owns the 2.0654 of record](2026-08-03-024222-the-footprint-never-mattered.md) -- same duty, same footprint, so the comparison is licensed:

| | rested | this run, slow | change |
|---|---:|---:|---:|
| solo, units/s/session | 21.484 | 13.891 | **-35.3%** |
| concurrent, units/s/session | 10.402 | 9.028 | **-13.2%** |
| slowdown | 2.0654 | refused | -- |

**A lone session loses about a third; a hundred sessions lose an eighth.** The direction survives the choice of comparison run: against [the reference run's 907.1 units/s total](2026-08-02-194155-eta-is-flat-where-it-was-said-to-fall.md), which is 9.071 a session, today's concurrent is **0.5%** away while the solo is still 35% down.

So the slow state is not a slower machine in the sense the name suggests. It suppresses a lightly-loaded measurement much more than a saturated one.

## Which means the gate's verdict is decided by the state

The slowdown is solo divided by concurrent, so a state that cuts the numerator harder than the denominator makes the ratio **smaller**. Bounding it with this run's own extremes rather than its refused mean:

| solo used | slowdown against 9.028 |
|---|---:|
| lowest hold seen, 9.801 | 1.086 |
| highest hold seen, 15.271 | 1.691 |

**Every solo hold in the bracket gives a ratio under the gate's 2**, where the same box rested returns 2.0654 and fails by 3.3%. That bound is robust to the refusal: it does not need the sides to agree, because it takes the worst and best individually.

**So running the gate's hour in the slow state would pass the work-rate condition for the wrong reason.** The slow state was the only window this box has offered in 7.62 hours, and it is the one window whose result would not mean what it says.

## What this does not establish

- **No mechanism.** Turbo availability is the obvious candidate -- one session can boost, a hundred cannot -- and `processor_performance` does not support it here: it read 175.6, 111.8 and 95.4 across the three before-solos, moving opposite to their rates. That counter has [already failed to identify this state once](2026-08-03-004512-a-saturating-burst-halves-the-box-for-an-hour.md).
- **One bracket.** The 16.7% refusal is a single trial against an estimate that half of hour-apart pairs would exceed the allowance. It is consistent with that and it is n=1.
- **The concurrent hold reports 72067 units evicted of 272067**, where every solo hold reports zero. `dropped_output` still reads held, so no gate condition is affected, but nothing here explains what eviction at a quarter of the units means and no other record mentions it.

## There are two slow states, and a solo hold cannot tell them apart

The section above says the slow state flatters the ratio. [A record from the same day says a slow box gives 3.958 against a rested 2.0654 -- 72% worse](2026-08-03-003443-the-footprint-result-was-the-machine.md). Both cannot be a fact about one state, so the two runs were set side by side:

| | this run | the 3.958 run |
|---|---:|---:|
| solo, units/s/session | 13.891 *(holds spanned 9.801-15.271)* | 9.752 |
| concurrent, units/s/session | 9.028 | 2.4636 |
| **concurrent, units/s total** | **902.8** | **246.4** |
| slowdown | 1.54, refused | 3.958, refused |

**This run's hundred sessions produced 902.8 units/s against [the reference run's 907.1](2026-08-02-194155-eta-is-flat-where-it-was-said-to-fall.md) -- 0.5% apart.** Under load this machine was entirely normal, and only its lone session was depressed. The other run's hundred sessions produced a quarter of that.

So there are two conditions wearing one name:

- **Solo-slow.** A lone session runs about a third under, a hundred sessions run normally. The slowdown is solo over concurrent, so it reads **low** -- 1.54 here -- and the gate passes for the wrong reason.
- **Machine-slow.** Everything is down, aggregate throughput to a quarter. The slowdown reads **high** -- 3.958 -- and the gate fails much harder than it should.

**And the fingerprint does not separate them.** The other run's solo was 9.752; this run's own second after-hold was **9.801**, half a percent away. Two holds that agree to half a percent, taken in states whose effects on gate M1 point in opposite directions and differ by a factor of 2.6 in the ratio they produce.

That is a correction to a rule this repository relies on. [The measurement index says what identifies the state is the workload's own rate](README.md), and the bracket's own reading procedure is built on comparing a side's rate against a rested figure. **A solo hold says that something is slow. It does not say what**, and the two answers move the gate in opposite directions.

**What separates them is already recorded and was not being read**: the concurrent hold's own total throughput. 902.8 and 246.4 are not close to anything, where 13.891 and 9.752 sit in the same band. A run that takes both figures can tell which state it is in; a solo triple cannot, however many holds it averages.

**What this does not establish.** Two runs, one of each state. Nothing here says how often either occurs, whether they share a cause, or whether a box can be in both at once -- and this run's solos spanning 9.801 to 15.271 inside eight minutes means a single solo triple may straddle more than one condition. The two-state reading is what two runs support; a third could make it three.

## Nine hundred-session holds, two clusters, nothing between them

The section above rests on two runs. Every hundred-session hold on disk was then read the same way -- `units_per_session_per_sec` times `sessions`, from `hold.json` and from each `bracket.json`'s concurrent phase:

| run | per session | **total units/s** |
|---|---:|---:|
| `r33-rested` | 10.548 | 1054.8 |
| `r36-rested` | 10.499 | 1049.9 |
| `tickpair` | 10.498 | 1049.8 |
| `clockpair-load` | 10.283 | 1028.3 |
| `m1` | 9.336 | 933.6 |
| `slowstate-ratio` (this run) | 9.028 | 902.8 |
| `eta-at-33` | 2.875 | 287.5 |
| `r20-slow-regime` | 2.464 | 246.4 |
| `loadgen` | 2.165 | 216.5 |

**Six between 903 and 1055, three between 217 and 288, and nothing in the 3.1x gap between.** Nine runs taken across days for unrelated purposes, and not one lands in the middle. So machine-slow is a state with a boundary rather than a bad afternoon, and it is legible in any hundred-session hold without a baseline, a bracket or a comparison.

**Three of nine were taken in it**, and two of those three are where [3.958 and 3.261](2026-08-03-003443-the-footprint-result-was-the-machine.md) come from -- the figures quoted for what the state costs the gate. They are that, and they are also slowdowns measured on a machine running at a quarter of its throughput, which is not the machine gate M1 is asked about.

**And the concurrent hold is the quiet instrument.** `r33-rested`, `r36-rested` and `tickpair` agree to **0.5%** across three separate runs -- tighter than any solo agreement recorded here, where a single bracket's own triple spread 21.5% and 36.8% on the same afternoon. A hundred sessions average over the placement noise that dominates one. The project has been naming the machine's state with the noisier of the two numbers it already collects.

**What this does not establish.** Nine runs is a distribution, not a mechanism, and the labels are workloads run for other reasons -- `loadgen` is not this record's workload, so its 216.5 is in the low cluster without being comparable to the others in the same way. The clean gap is what nine points support; where the boundary sits, and whether a run can cross it mid-hold as [one already did](2026-08-03-003443-the-footprint-result-was-the-machine.md), they do not.

## The gap is a fact about quiet boxes, and a tenant lands in it

[Nine hundred-session holds, two clusters, nothing between them](#nine-hundred-session-holds-two-clusters-nothing-between-them) reads the total throughput as naming the machine's state. A tenth hold, taken deliberately while a third-party tenant held most of the box, lands **inside the gap**:

| | the quiet reference | this run |
|---|---:|---:|
| cores held by the job | 15.34 | **7.45** |
| cores held outside it | 0.77 | **8.56** |
| **total units/s** | 902.8 | **344.9** |
| peak RSS | — | 2.36 GiB of 3.73, held |
| dropped output | — | 0 failed reads, held |

**Every one of the nine was taken on a quiet box.** So the clusters and the 3.1x gap are a fact about *that* population, and `total` alone no longer separates the states: 344.9 with 8.56 cores held elsewhere is a crowd, where 246.4 on a quiet machine is a box running at a quarter for its own reasons.

That is the same defect as the solo rate, in the figure built this morning to replace it. **What the pair does say, and neither number says alone**: throughput reports how much the box is producing, the rest column reports whether anything else is taking it, and the state needs both — read at both load levels, since a solo hold cannot see contention and a concurrent hold cannot see turbo.

**A tenant also costs more than its core share** — [withdrawn an hour later by a second tenanted hold that came back proportional](#a-second-tenant-and-the-core-share-claim-does-not-reproduce), and left here because a record is a log. The job held 7.45 cores against 15.34 quiet — **48.6%** — and produced 344.9 against 902.8, **38.2%**. Ten points of throughput went somewhere the core count does not explain, which is the direction [a neighbour costing a solo baseline 27% without starving it](2026-08-03-081500-a-neighbour-costs-the-solo-baseline-twenty-seven-percent.md) already pointed, now at a hundred sessions instead of one.

**Two of gate M1's conditions held anyway**, which is worth recording because it is the first time they have been measured against real competition: RSS at 2.36 GiB of a 3.73 GiB budget and zero failed reads, with all hundred sessions alive at the end.

**What this does not establish.** One tenanted hold against nine quiet ones, and the tenant's own size moved during the run — `doctor` read 66% and 85% eight minutes apart. The median rest of 8.56 smears that, exactly as a median does to a burst. What the run supports is that a tenanted total falls in the gap, not where in the gap it falls.

## A second tenant, and the core-share claim does not reproduce

The section above says a tenant costs more than its core share, from one run. A second tenanted hold an hour later says it does not.

| | job cores, mean | share of the quiet 15.34 | total units/s | share of the quiet 902.8 |
|---|---:|---:|---:|---:|
| tenant A | 7.46 | 48.6% | 344.9 | 38.2% |
| tenant B | 5.02 | **32.7%** | 297.3 | **32.9%** |

**Tenant B is proportional to a fifth of a percent.** Whatever cost tenant A ten points of throughput beyond its core share, it is not a general property of having a neighbour — and the claim was made from a single run, which is the tell this repository already names. The caveat *one tenanted hold against nine quiet ones* was written into the same commit and did not travel with the sentence that needed it.

What both runs do support is the ordering: 8.56 cores held gave 344.9 and 10.24 gave 297.3, and both sit inside the gap the quiet holds left empty.

**The pair was run as a go/no-go probe and that is what it delivered.** `doctor` read 19% immediately before launch; the solo hold two minutes later saw **4.12** cores held elsewhere, and the hundred-session hold four minutes after that saw **10.24**. The tenant ramped from about three cores to about ten inside six minutes. Read alone the solo's 11.952 units/s is an unremarkable slow-band reading and the window looks open; its rest column is what says a neighbour was arriving. First deliberate use of the pair, and it returned a refusal with a reason rather than a number.

It is not a bracket and does not pretend to be: the two holds are four minutes apart, and this box has changed state inside a five-minute hold before. Six minutes buys a decision about whether to spend an hour. It does not buy a slowdown anyone should quote.

**RSS and dropped output held again** — 2.36 GiB of 3.73 and zero failed reads, with all hundred sessions alive — which is now twice under real competition.

## A quiet hold landed in the gap, so the gap is not a boundary

[Nine hundred-session holds, two clusters](#nine-hundred-session-holds-two-clusters-nothing-between-them) reads the empty 3.1x span as evidence of two states, and [a tenanted hold landing in it](#the-gap-is-a-fact-about-quiet-boxes-and-a-tenant-lands-in-it) was explained by the tenant. **A tenth quiet hold lands there too, and it is the quietest reading this instrument has produced.**

| | |
|---|---:|
| solo | 10.739 units/s, 0.93 cores held elsewhere |
| hundred, total | **756.1** units/s, **0.49** cores held elsewhere |
| job occupancy | 15.51 cores median |
| implied slowdown | **1.42** |

0.49 cores held while a hundred sessions run is lower than any of the nine, so nothing was competing. The quiet population now reads **217, 246, 288, 756, 903, 933, 1028, 1050, 1050, 1055** — and the span that had nothing in it has something in it.

**So the bimodality was a fact about ten readings, not about the machine.** Two clusters and a clean gap is what nine points happened to show; the tenth says the box takes intermediate values, and nothing here can now say whether that is a third state, a continuum, or a boundary that moves. What survives is weaker and still useful: a hundred-session total far below ~900 means something is wrong, and the rest column says whether the something is a neighbour.

**The decision the pair exists to make was still correct.** A slowdown of 1.42 flatters gate M1 harder than the 1.54 that first raised this, so the run would have passed a condition asking for 2 while the box was visibly not rested. The classifier calls this solo-slow and says not to spend the hour, which is right — but *solo-slow* is a two-state label on something that just demonstrated a middle, and the label should be read as a verdict rather than a description.

**Still no rested window.** Roughly ten hours of holds and the count of rested ones has not moved off two, on the quietest background yet measured. RSS held at 2.36 GiB of 3.73 and dropped output was zero for the third time today.

## The neighbour has a name, and it holds twelve cores flat

The tenanted holds above were taken against something unidentified. Named with `Get-Counter '\Process(*)\% Processor Time'`, sampled six times across a minute:

```
02:44:08  chrome-headless-shell=1,197%
02:44:21  chrome-headless-shell=1,063%   aihost=94%   claude=37%
02:44:34  chrome-headless-shell=1,184%
02:44:47  chrome-headless-shell=1,213%
02:45:00  chrome-headless-shell=1,104%   claude=62%   claude=48%
02:45:14  chrome-headless-shell=1,207%
```

**Eleven to twelve of sixteen cores, and steady** — the minimum across a minute is 1,063%, so nothing here averages out over a two-minute hold the way a bursting neighbour would. [The same process was recorded at 760% earlier](2026-08-03-081500-a-neighbour-costs-the-solo-baseline-twenty-seven-percent.md), so it has grown by half.

**It reframes what gate M1 is waiting for.** The hour was blocked on finding a rested window and treated as a matter of patience. Background went 66, 98, 19, 8 and 83 percent inside about ninety minutes, and the quiet stretches did not last the six minutes a state probe costs — one probe read 0.49 cores held during its concurrent hold and 13.34 fifteen minutes later. A gate needing sixty consecutive minutes is not waiting for a rare state; it is waiting for a process to stop, and waiting does not make a window longer.

**And it explains readings that were attributed to the machine.** The box did not change state between 8% and 83%; a browser started. Any figure here taken while this was running is a figure about a four-core machine, which is why the rest column travels with every one of them.

## The window closed in under five minutes, and the probe caught it

The gate's twenty-minute run was launched the moment `chrome-headless-shell` read **0.0 cores** and `doctor` read 7% — the trigger this record argued for. It refused, and what it refused on is worth more than a pass would have been.

| phase | figure | cores held elsewhere |
|---|---:|---:|
| two solo probes | 9.547 and 11.341, **17.2% apart** | **0.86** |
| load probe | **431.9** units/s across a hundred | **8.53** |

**The tenant returned between the solo probes and the load probe**, so the window that looked open at launch was gone within about five minutes. A run needing sixty consecutive minutes is not waiting for a rare machine state; it is waiting for a process whose absences are shorter than the measurement.

**The state classifier's first real use was correct.** It read the load probe's 8.53 cores and printed *a tenant is present*, from a figure the script discarded until this morning — and 431.9 is a third tenanted total in the span the quiet clusters left empty, alongside 344.9 and 297.3.

**The two things must not be conflated.** The solo probes were **quiet**, 0.86 cores held, and still disagreed by 17.2%. That gap is not the neighbour. It is either the machine wandering or the probes being **30 seconds** where [the 4.8% median adjacent gap the 5% threshold descends from](2026-08-03-094550-the-slow-state-caught-on-a-quiet-machine.md) was measured on **120-second** holds. A quarter of the duration samples a quarter of the placement noise, so a shorter probe is systematically wider, and a threshold borrowed across that difference would refuse runs on its own sampling rather than on the machine.

**That has a precedent in this script's own header**: an earlier precondition asking for four `doctor` readings within 10% refused its sibling nineteen times in one day without ever running. Same disease, different gauge. Unresolved here, and measurable — it needs the adjacent-pair spread of 30-second solo holds, which nothing has taken.

**The refusal cost five minutes rather than forty**, which is the precondition earning its place even while its threshold is in question.

## The thirty-second probe is not the problem, and the threshold should not be loosened

[The refused run](#the-window-closed-in-under-five-minutes-and-the-probe-caught-it) raised the possibility that `m1-hour`'s precondition compares two **30-second** probes against a 5% threshold descended from **120-second** holds, and so refuses on its own sampling. Eight back-to-back 30-second holds, the exact shape the script runs, say otherwise.

| # | rate | cores held elsewhere |
|---|---:|---:|
| 1 | 15.896 | 12.33 |
| 2 | 16.382 | 12.34 |
| 3 | 16.247 | 12.20 |
| 4 | 14.386 | 7.67 |
| 5 | 11.530 | **1.56** |
| 6 | 16.844 | 2.97 |
| 7 | 14.046 | 13.53 |
| 8 | 15.434 | 12.93 |

The tenant came and went during the series, which is what makes it readable. Splitting the seven adjacent gaps by whether the rest column moved by as much as one core:

| tenancy held still | tenancy moved |
|---|---|
| **3.1%, 0.8%, 9.9%** | 12.9%, 24.8%, 46.1%, 19.9% |

**Every gap above 12% coincides with tenancy moving at least 1.4 cores**, and holds 1-3 — constant at 12.33, 12.34 and 12.20 cores held — agree to 3.1% and 0.8%. An instrument too short to measure anything cannot produce 0.8%. **So the hypothesis is refuted and the fix it implied would have been the wrong one**: loosening the threshold would have let a moving machine through, which is the one thing the precondition exists to stop. The 17.2% that refused the gate run was the machine changing between two half-minutes, not the probe being brief.

**It is not a clean pass for the threshold either.** One of the three stable pairs still reached 9.9%, so the precondition will sometimes refuse a machine whose *tenancy* is constant, and three stable pairs is a thin basis for saying how often. What the series does settle is the direction: the dominant term is the neighbour arriving and leaving, not the probe length.

**One reading is unexplained and left standing.** Hold 5 is the quietest of the eight at 1.56 cores held and the **slowest**, 11.530, while holds under twelve cores of tenancy returned 15.9 to 16.8. That is backwards, it is a single hold, and nothing here accounts for it.

## Twenty minutes says the memory cost is a level, not a slope

Every hold in this record is two to five minutes, so nothing here says whether a hundred sessions cost *more memory the longer they are held* — which is the question gate M1's hour actually poses, and the one an hour would answer at three times this exposure.

Run without a bracket, deliberately: **RSS and dropped output are absolute conditions, so a neighbour cannot corrupt them.** It only slows the work. That makes them measurable in a window too short and too crowded for any ratio, which is what this box offers.

| | |
|---|---:|
| window | **1,203,178 ms counted of 1,207,165 held — 99.7%**, 239 samples |
| sessions | 100, fewest alive **100**, peak processes 101 |
| **peak RSS** | **2.372 GiB** of a 3.73 GiB budget — **Held** |
| dropped output | **0 failed reads**, 0 truncated — **Held** |
| scrollback | 438,077 evicted, which is policy rather than loss |
| throughput | 530.3 units/s total, **8.88 cores held outside the job** |

**Peak RSS at twenty minutes is 2.372 GiB against 2.36 at two and at five — 0.4% apart.** Ten times the exposure adds four thousandths, so the footprint is a level the sessions reach and hold rather than something that accumulates. Nothing leaks per unit time at this session count, and the same argument carries the RSS condition to an hour without running one.

**The throughput is discarded and the run is still good.** The tenant held 8.88 cores against the job's 7.12 median, so 530.3 is a figure about half a machine and belongs beside 344.9, 431.9 and 297.3 rather than beside 902.8. The occupancy line also catches it moving — a median of 7.12 against a mean of 8.90 only separates that far when the window is lopsided.

**What this does not claim.** Not the duration condition: twenty minutes is not sixty, and [this box hard-stopped at forty-one minutes of exactly this load](2026-08-03-004512-a-saturating-burst-halves-the-box-for-an-hour.md), so the cap is a safety limit rather than a shortage of patience. Not the work rate, which needs a bracket and a quiet box. What it adds is that two of the four conditions now hold at a third of the gate's exposure, with the window counted at 99.7% rather than assumed.

## Why it is a level: the scrollback cap, not the clock

[The section above](#twenty-minutes-says-the-memory-cost-is-a-level-not-a-slope) reads 2.372 GiB at twenty minutes against 2.36 at two as evidence that memory does not accumulate. The obvious attack is that the twenty-minute hold ran on a half-machine and therefore did far less work than a quiet twenty minutes would — so if RSS tracks *work* rather than *time*, the result is about a slow box. The artifacts answer it, and the answer is a mechanism rather than a reassurance.

| run | units | evicted | **retained** | peak RSS |
|---|---:|---:|---:|---:|
| five minutes | 272,067 | 72,067 | **200,000** | 2.375 GiB |
| twenty minutes | 638,077 | 438,077 | **200,000** | 2.372 GiB |

**Both retain exactly 200,000 lines** — [`DEFAULT_SCROLLBACK_LINES`](../../coggyd/src/lib.rs) at 2,000 a session, times a hundred sessions — while the work behind them differs by **134%**. Once the buffer fills, further output evicts rather than allocates, so memory stops tracking work and starts tracking the cap.

So the level is not an observation that happened to hold across two exposures; it is what a bounded buffer does after it fills. **The two peaks agree to 0.13%** where the work differs 2.3×, which is a stronger statement than the 0.4% quoted above, and the attack is what produced it.

**It also says what would break it**, which an empirical reading could not: raising the line cap, raising the session count, or a workload whose lines are longer — since [`DEFAULT_SCROLLBACK_BYTES`](../../coggyd/src/lib.rs) bounds the content where the line count bounds only the per-line overhead. None of those is the hour, so the memory condition carries to sixty minutes for a reason rather than by extrapolation.

## A thirty-second hold is not a short two-minute hold

The eight probes above were taken to test the precondition, and they answer a second question they were not launched for. Their tenancy spans **1.56 to 13.53 cores held inside five minutes**, which is [the time-controlled condition a neighbour's cost has never had](2026-08-03-081500-a-neighbour-costs-the-solo-baseline-twenty-seven-percent.md) — every earlier comparison varied tenancy by waiting, so it varied time too.

Regressed, the neighbour costs nothing detectable: **+0.151 units/s per core held, r² = 0.17**, and the sign is positive where the earlier reading implies negative. Four holds between 12.20 and 13.53 cores held span 14.046 to 16.382 — **17% at near-identical tenancy** — so tenancy explains almost none of the variation. **The 27% that a neighbour was said to cost may have been time rather than the neighbour**, and this is the first set that could tell them apart.

**It cannot settle it either, because the instrument changed.** These are 30-second holds averaging **15.10 units/s**, where today's 120-second solo holds run about **11.2** — 35% apart, same box, same workload, same afternoon. A short hold reads a *different level*, not a noisier version of the same one, so the two sets are not comparable and neither is a clean answer about neighbours.

**That difference is a live defect in the gate script.** `m1-hour` takes 30-second probes and prints a NOTE placing their mean against bands measured at 120 seconds — 18.9 rested, 13.8 tenanted, 9.0 slow. A probe systematically 35% high against a 120-second band will name the wrong band, and the branch that fires decides what an operator is told about spending the hour. The precondition itself is unaffected, since [it compares two probes of equal length with each other](#the-thirty-second-probe-is-not-the-problem-and-the-threshold-should-not-be-loosened) and that comparison was measured sound; what is wrong is comparing a probe to a band.

**What this does not establish**: why a shorter hold reads higher. Start-up cost amortised over a quarter of the window, scheduling luck that has not averaged out, or the workload's own early behaviour are all candidates and none is tested. Eight holds, one afternoon, and the direction is what is solid rather than the 35%.

## The 35% carries the same confound it was used to expose

The section above doubts a neighbour's 27% because it varied tenancy by waiting, and then states a 35% difference between 30-second and 120-second holds **which was arrived at the same way**. Every solo hold on disk, by duration:

| duration | n | mean | min | max |
|---|---:|---:|---:|---:|
| 20 s | 1 | 21.40 | 21.40 | 21.40 |
| 30 s | 10 | **14.17** | 9.55 | 16.84 |
| 120 s | 11 | 13.32 | 10.10 | 20.30 |
| 121 s | 41 | **10.23** | 7.86 | 15.71 |

**All ten 30-second holds were taken inside one five-minute window**; the fifty-two at two minutes span the whole day, including the stretches where this box ran at half speed. So the comparison varies duration and time together, which is the objection the previous section raises against the 27%, made again in the act of raising it.

Held to the same standard: the nearest-in-time 120-second holds, from the same hour, read **10.7 to 12.0** against the 30-second mean of 14.17 — so the direction survives and the size drops to roughly a quarter, on a handful of points. The 20-second reading of 21.40 is from a different day on a rested box and is not evidence of a trend, however neatly it extends one.

**What stands is the direction and the consequence, not the figure.** A short hold appears to read higher, and that is enough to say a 30-second probe should not be placed against a 120-second band — which is the [script change](../../sessionbench/scripts/m1-hour.ps1) the finding produced, and which needs only the sign. What would settle the size is a set alternating 30-second and 120-second holds inside one window, which nothing has run.

## A hold's own first thirty seconds are the slow part, which kills the obvious explanation

[The section above](#the-35-carries-the-same-confound-it-was-used-to-expose) leaves the duration effect as a direction with a doubtful size, and offers start-up amortised over a short window as the leading explanation. That one is testable with no new run: every 120-second hold samples `work_units` every five seconds, so a hold's own first thirty seconds can be compared with its remainder.

Across **52 one-session holds**, the first thirty seconds run a median **19.3% slower** than the rest of the same hold.

**That is the wrong sign.** If short windows were intrinsically fast, a hold's own opening should read high; it reads low. So nothing inside a hold explains why standalone 30-second holds averaged above 120-second ones, and the start-up explanation is refuted rather than merely unproven.

**Which leaves the confound as the likeliest reading of the whole thing.** All ten standalone 30-second holds came from a single window; the 120-second holds span a day that included stretches at half speed. A difference that survives no mechanism and disappears under the nearest-in-time comparison is most simply the sitting, not the duration.

**The per-hold spread is enormous** — +55.7% to −44.7% across the 52 — so the median is the finding and no single hold is evidence of anything. That spread is itself consistent with a wandering box rather than with a repeatable warm-up curve.

**The script change stands and its justification narrows.** Labelling a 30-second probe's placement against a 120-second band as a hint is conservative and costs nothing if the effect is not real; what cannot be said is that the band is wrong *because a short hold reads high*. The controlled test is four alternating holds inside one window, which is five and a half minutes and has not been run.

## The arithmetic predicts the opposite sign, which makes three

A hold's reported rate is `units read / counted seconds / sessions` — [the counted window, untrimmed](../../sessionbench/src/daemon.rs). `Occupancy` drops everything before a run reaches its own median, so the *cores* figure excludes spin-up; the *rate* does not. And the warm-up is a separate throwaway hold, so the counted hold spawns its own sessions and carries its own opening inside the window it is divided by.

Put together with the section above: the opening runs about 19% slow, and it fills a **quarter** of a 30-second window against a **sixteenth** of a 120-second one. **So the arithmetic predicts a short hold reads lower.** It reads higher.

That is three independent arguments against the duration effect being real — the within-hold measurement, the nearest-in-time comparison, and now the way the number is computed — and no argument for it except a comparison whose two arms were gathered in different sittings.

**It turns the pending test into a prediction rather than an exploration.** Four alternating holds inside one window should find no effect, or a small one favouring the *long* holds. A result the other way would mean something is going on that none of these three accounts for, and would be worth far more than confirming the confound.

## The controlled set fired on a tenant that was starting, not gone

The prediction test was armed as a waiter that polls for the neighbour to drop below one core and fires immediately, because [five windows were lost between observing quiet and issuing a command](../../sessionbench/scripts/wait-for-quiet.ps1). It fired at a reading of **0.00 cores held**. Its first hold recorded **12.20**.

| hold | rate | cores held elsewhere |
|---|---:|---:|
| 30 s, first | 16.505 | 12.20 |
| 120 s, first | 14.168 | **2.36** |
| 30 s, second | 15.376 | 12.27 |
| 120 s, second | 16.134 | 13.71 |

**The set is void**, by the rule written into the task before any number existed: any hold above about 2.5 cores held disqualifies it, and three of four are. What went wrong is not the tenant's timing but the waiter's: a single `Get-Counter` reading of 0.00 means a process that is *starting* as readily as one that is gone, which is [the point-sample-is-not-a-window rule](../../CLAUDE.md) broken inside the instrument built to obey it. The waiter now needs consecutive quiet readings before firing.

**Read opportunistically it corroborates the prediction without replacing it.** At matched tenancy — 12.20, 12.27 and 13.71 cores — the two 30-second holds average **15.94** against the 120-second hold's **16.134**, a difference of **1.2%** where the original comparison claimed 25 to 35%. That is a fourth thing pointing the same way as the within-hold measurement, the arithmetic, and the nearest-in-time comparison. It is not the controlled test and does not close the question.

**The backwards pattern shows a third time.** The quietest hold here, at 2.36 cores held, is the slowest at 14.168, where holds under twelve cores of tenancy return 15.4 to 16.5. The same inversion appeared in [the octet's quietest hold](#the-thirty-second-probe-is-not-the-problem-and-the-threshold-should-not-be-loosened) at 1.56 cores and 11.530. Three sightings, no mechanism, and it is now the most repeatable unexplained thing in this record.

## Sixty seconds of verified quiet bought ninety seconds of actual quiet

The waiter was fixed to require six consecutive polls below one core before firing, and [proved on purpose to refuse when nothing can meet the threshold](../../sessionbench/scripts/wait-for-quiet.ps1). It then rejected three windows — quiet broken at 10.32, 8.38 and 10.50 cores after two or three polls — before one held a full minute.

| hold | rate | cores held elsewhere |
|---|---:|---:|
| 30 s | 15.815 | **2.92** |
| 120 s | 15.130 | 13.52 |
| 30 s | 15.441 | 12.27 |
| 120 s | 13.245 | 7.72 |

**Void again, and the rule stays at 2.5 cores.** The first hold's 2.92 raised the possibility that the threshold sits below this box's resting floor; the second hold's 13.52 answered it — 2.92 was a neighbour on the way up. Loosening a threshold after seeing the number is what [#58 proved wrong about the precondition](#the-thirty-second-probe-is-not-the-problem-and-the-threshold-should-not-be-loosened).

**What both attempts establish together is about the machine, not the question.** Sixty seconds of *verified* quiet did not survive one 30-second hold plus one 120-second hold. Across roughly eight hours, five manual attempts and two instrumented ones, **no five-minute quiet window has occurred** — so gate M1's hour is not merely rare here, and the shortfall is three orders of magnitude rather than a matter of waiting for the afternoon.

**Three things fall out of the void set anyway.** Pooling both sets where tenancy matches, at 12.20 to 13.71 cores, the 30-second holds average **15.774** against the 120-second holds' **15.632** — **0.9%** across five holds, where the comparison that opened this claimed 25 to 35%. That is a fifth account agreeing with the within-hold measurement, the arithmetic, the nearest-in-time comparison and the first void set. Second, the lone session returns 13.245 to 15.815 while the neighbour swings from 2.92 to 13.52 cores, which is the tenancy null again.

**And the third is a retraction.** The quietest hold here is the **fastest**, 15.815 at 2.92 cores, where [the pattern recorded twice before](#the-controlled-set-fired-on-a-tenant-that-was-starting-not-gone) had the quietest hold slowest. Three sightings became a task; the fourth chance came back the other way, so what looked like the most repeatable unexplained thing in this record is more likely three draws from a wide distribution.

## A thirty-second hold is twice as noisy as a two-minute one, which ends the question

A third alternating set, fired after the waiter verified sixty seconds below one core with no rejections. The neighbour was back above twelve cores before the first hold finished — **under ninety seconds from a verified-quiet start** — and then held remarkably still.

| | rate | cores held elsewhere |
|---|---:|---:|
| 30 s | 16.693 | 12.56 |
| 120 s | 15.821 | 12.92 |
| 30 s | 14.337 | 12.94 |
| 120 s | 14.716 | 12.24 |

**Void by the rule, and the most informative of the three** — every hold sits between 12.24 and 12.94 cores held, a range of 5.7%, so the neighbour is as close to constant as this box has offered.

| | mean | spread |
|---|---:|---:|
| 30 s arm | **15.515** | **16.4%** |
| 120 s arm | **15.269** | **7.5%** |

**The duration difference is 1.6%**, agreeing with the 0.9% from the pooled earlier sets and with the four other accounts, against the 25 to 35% that opened the question. **And the short arm's own scatter is ten times that difference.**

**The new figure is the repeatability**: 16.4% against 7.5%, roughly halved for four times the duration, which is what averaging does. A 30-second hold is about twice as noisy as a 120-second one, so any effect it seemed to show is a fifth the size of its own noise.

**That reconciles the earlier octet rather than contradicting it.** There, three 30-second holds at constant tenancy agreed to 3.1%; here two agree to 16.4%. Both are real — the instrument's repeatability is itself variable — which is exactly why [a precondition that refuses on disagreement is the right design](#the-thirty-second-probe-is-not-the-problem-and-the-threshold-should-not-be-loosened) and why widening its threshold would have been wrong. It also explains the gate run's 17.2% probes on a quiet box without needing the machine to have moved.

**So the question is not resolvable here, and no longer worth resolving.** Two holds an arm cannot separate a 1.6% difference when one arm's repeatability is 16.4%; it would take roughly a dozen holds an arm, which is thirty-plus minutes of sustained quiet on a box that has not produced five. The reason to stop is the instrument's noise rather than the tenant.

## The two held conditions were checked against a starved machine, and survive it

Every hundred-session run today gave the job **4.3 to 7.5 cores** because a browser held the rest, so both conditions this record calls held were measured on a machine producing a third of its work. A session starved to a third emits a third of the lines, which is exactly the pressure `failed_reads` exists to detect — so *zero failed reads* on a half-speed box says little about a full-speed one.

**The answer was already on disk, across every hundred-session run ever taken here:**

| total line rate | runs | failed reads |
|---|---|---|
| 1,028–1,055 units/s | `r33-rested`, `r36-rested`, `tickpair`, `clockpair-load` | **0** |
| 903–934 | `m1`, `slowstate-ratio` | 0 |
| 217–530 | seven tenanted or slow runs | 0 |

Zero across a **five-fold span of line rates**, including runs at twice today's, which is a much stronger statement than zero at one rate. The objection dies and the condition is genuinely held.

**RSS survives for the reason already recorded**: retention caps at 200,000 lines and both a five-minute and a twenty-minute hold *reached* the cap, evicting 72,067 and 438,077 lines. A starved run reaches it later and still reaches it, so the peak is unchanged.

**But the same table says the memory margin is far thinner than today's runs suggest.** `r33-rested` and `r36-rested` peak at **3.651 and 3.944 GiB** where today's runs sit at 2.36 to 2.39, because they hold 33 and 36 MiB a session against 20. Against a 4 GB budget that is **1.4% of headroom at 36 MiB**, not the 40% a 20 MiB run implies. The condition holds at every weight measured and it holds *narrowly* at the weights [the budget actually permits](2026-08-03-024222-the-footprint-never-mattered.md) — which is worth carrying beside the word "held" wherever it appears.

## Four void sets pooled: the duration effect is null and a hold is worth about 6%

A fourth alternating set fired after a window that followed three *descending* rejections — 8.14, 2.20 then 1.51 cores — rather than after none. That was written up beforehand as the set with the best odds of surviving. Its first hold read **12.12 cores**, so the descent bought nothing and the two cases are indistinguishable: **four sets, four voids**, three of them fired after verified sustained quiet and all spoiled inside a single 30-second hold.

The wreckage is the result. Sixteen holds, thirteen of them between 12.12 and 13.71 cores held — the closest to a controlled tenancy condition this box has offered, entirely by accident.

| | n | mean | sd | se |
|---|---:|---:|---:|---:|
| 30 s | 7 | **15.927** | 0.914 (5.7%) | 2.2% |
| 120 s | 6 | **15.270** | 1.020 (6.7%) | 2.7% |

**The difference is +4.3% at 1.2 standard errors** — what noise produces. The duration effect is null, which is where six earlier accounts pointed, and this is the first version of the claim with enough holds to say so rather than infer it.

**Two of this record's own figures fall.** [*A 30-second hold is twice as noisy as a two-minute one*](#a-thirty-second-hold-is-twice-as-noisy-as-a-two-minute-one-which-ends-the-question) was one set — 16.4% against 7.5% — and the next set reversed it at 3.9% against 19.7%. Pooled, the arms scatter **5.7% and 6.7%**, indistinguishable, so hold length changes neither the level nor the repeatability. And the *0.9%* quoted from an earlier pooling was under-powered; the honest figure is 4.3% and not significant.

**What survives is the number every two-hold comparison here has been betting on: a solo hold is worth about 6%.** That is the per-hold standard deviation at matched tenancy, from thirteen holds. It validates [three repeats a side](../../sessionbench/src/daemon.rs) — 6% per hold gives a standard error near 3.5%, inside the 5% allowance — and it explains why three-in-a-row patterns kept appearing and dissolving all night. It also means a *single* pair of probes disagreeing by 17% is unremarkable, which is what the gate run saw.

