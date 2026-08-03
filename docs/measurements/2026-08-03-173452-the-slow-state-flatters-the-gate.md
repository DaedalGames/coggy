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

