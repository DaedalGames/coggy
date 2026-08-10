# Same CPU, less work, when the box goes quiet

**A lone session took the same share of a core and produced 36% fewer units once the neighbour left.** Its own occupancy did not move — 23.99% of one core with a tenant present, 26.42% without — while its rate fell from 14.872 to 10.294 units/s. Work per core-second dropped 43%, which contention cannot do in that direction.

## What was run

The twelfth alternating set of `wait-for-quiet.ps1`, fired 2026-08-11 08:28:05 after six consecutive polls below one core. It was expected to void, and the prediction was recorded before it fired: a five-minute set cannot survive this box's 49–70 second quiet gaps. The first hold came back with 12.30 cores held elsewhere, which is a void. The tenant then left in the middle of the set and stayed away, so the four holds became a 2×2 — two durations against two tenancy states, inside six minutes.

One `coggyd` session per hold, `target/release/coggyd.exe`, job-object membership, 5-second sampling. Mains power.

| hold | units/s | job CPU | machine CPU | rest cores | units per job-CPU% |
|---|---|---|---|---|---|
| 30 s, crowded | 14.872 | 23.99% | 1253.17% | 12.29 | 0.6200 |
| 120 s, crowded | 14.401 | 25.55% | 1248.28% | 12.23 | 0.5637 |
| 30 s, quiet | 11.178 | 25.76% | 112.03% | 0.86 | 0.4339 |
| 120 s, quiet | 10.294 | 26.42% | 124.96% | 0.99 | 0.3896 |

Matched by duration, the crowded hold is **1.3305×** the quiet one at 30 s and **1.3990×** at 120 s. Pooled, 14.637 against 10.736 — **+36.3%**.

The classification was fixed before the last hold was seen: rest cores below 2.0 is quiet, above 8.0 is crowded, anything between is a mixture spanning the tenant's return and is excluded. Both quiet holds landed at 0.86 and 0.99, so nothing needed excluding and the rule cost nothing. It is recorded because it would have mattered had the last hold landed at 5.

## Why this is not contention

> The clock reading this section reaches for was checked against the whole archive an hour later and does not survive as a mechanism. Read the appended section at the end before quoting anything below on that point; the elimination of contention stands, the clock as its replacement does not.

The obvious reading of "a lone session runs faster with a neighbour present" is that something is wrong with the measurement. The occupancy column is what rules that out. Across all four holds the session held 0.24 to 0.26 cores — flat to within 8% — and the two quiet holds took *more* CPU than the crowded ones while producing 30% less. The session was never starved, never descheduled, never waiting.

So the quantity that moved is how much work a core-second buys, and sharing a machine cannot raise it. A busier box has worse cache residency and less memory bandwidth per session, both of which make a core-second worth less. Every contention mechanism points the wrong way by roughly a factor of two against this result.

What raises the value of a core-second on a machine that just got busier is the machine clocking up. A box at 12.5 of 16 cores is in a boost state; a box at 1.1 cores is not. A lone session on a quiet machine runs at the low clock, and the same session beside a 12-core neighbour rides the boost the neighbour paid for.

**The clock was recorded, and it moves.** Every hold carries `host.processor_performance`, added to test a thermal claim and never read for this one:

| hold | units/s | clock | thermal |
|---|---|---|---|
| 30 s, crowded | 14.872 | 192.1% | 63.1 °C |
| 120 s, crowded | 14.401 | 183.5% | 63.1 °C |
| 30 s, quiet | 11.178 | 105.0% | 63.1 °C |
| 120 s, quiet | 10.294 | 149.4% | 63.1 °C |

**187.8% crowded against 127.2% quiet — a ratio of 1.476, against the rate ratio of 1.363.** Same direction, comparable magnitude, from an instrument that shares no input with the work-rate calculation. The thermal zone reads 63.1 °C in all four while the rate moves 44%, which is the fourth independent occasion on which it has predicted nothing here.

**It confirms the direction and cannot quantify it.** `processor_performance` is a point sample taken once at hold start, not a mean over the window. If the clock were the whole story, work-per-core-second divided by clock would be constant; it gives 0.00323, 0.00307, 0.00413 and 0.00261 — a 58% spread, with the 120-second quiet hold discordant, reading 149.4 at its start while returning the lowest rate of the four. A single reading at t=0 of a two-minute hold is precisely the instrument that produces that. **The counter belongs in `concurrent-samples.jsonl` beside `cpu_percent`, not in the host block.**

**A fifth reading, taken from `doctor` minutes after the set and independently of it, puts the bottom of the curve lower still**: with the box at 1.30 of 16 cores it reports the cores clocking at **94% of nominal**. That is below both quiet holds, which is what it should be — those holds each had a session running in them and so were never at rest. The ordering it gives is 94% idle, 105–149% under one light session, 183–192% beside a 12-core neighbour: a factor of two across this box's ordinary load range, and an explanation for why the two quiet holds disagree with each other rather than sitting together.

It also says the gate's exposure is wider than this set measured. A solo baseline on a properly quiet box sits nearer 94% than 127%, while a hundred-session hold saturates well past 12.3 cores and so past 192%. **The ratio M1 divides spans more of this curve than any two holds here have covered together.**

There is one consistency to note rather than claim. [CLAUDE.md records that `processor_performance` was tested against the slow state and failed to track it](../../CLAUDE.md). Here it tracks tenancy strongly. Those are compatible only if the slow state and the P-state are different phenomena — which this record argues below on separate grounds, so the older null result supports the split rather than contradicting this reading.

## What it costs the gate

Every solo baseline in this repository is taken on a deliberately quieted machine. That is what `wait-for-quiet.ps1` is for, and what eleven void sets were spent pursuing. It is also, if this reading is right, the state that puts the box at its lowest clock.

Every concurrent hold at a hundred sessions saturates the box into boost.

The work-rate condition of gate M1 is a ratio of those two — solo over per-session concurrent — so its numerator is systematically measured in the slow state and its denominator in the fast one. That understates the slowdown, and **understating the slowdown flatters a gate whose condition is "under 2×"**. The procedure built to remove the neighbour's contention would then be introducing a larger error than the contention it removes, in the direction that makes the gate easier to pass.

The size is not yet known. This set says 36% between 12.5 cores of neighbour and 1.1; a hundred-session hold saturates far past 12.5, so the relevant gap is not this one.

## What this does not establish

- **The clock was read at one instant per hold, not across it.** The direction is confirmed and the size is not: normalising by it leaves a 58% spread. Sampling `processor_performance` per tick, beside `cpu_percent`, is what would close this, and it is a small change to a sampler that already runs.
- **One set, one tenancy transition.** The 2×2 came from the neighbour happening to leave mid-run. Nothing was varied on purpose.
- **The tenant's departure is the only recorded change.** Anything else that shifted at 08:31 would look identical in this data.
- **This is not [the slow state](2026-08-03-173452-the-slow-state-flatters-the-gate.md).** That one lasts about ninety minutes, arrives after tens of minutes of saturation, and is flat on both sides of its transition. A P-state follows load within seconds. They may interact; they are not the same observation, and the older record's two-state finding is untouched by this.
- **One workload, one machine, one power state.** `coggyd` sessions at ~0.25 cores each. A session that saturates its core may sit at a different point on the frequency curve entirely.

## What it settles

[#64 asked whether an idle box is a worse measurement environment than a loaded one](2026-08-03-173452-the-slow-state-flatters-the-gate.md), pooling 28 crowded holds against 11 quiet ones taken hours apart, and found +17.4% at 3.5 standard errors. The standing objection was the one this repository has been caught by twice: a comparison across windows varies time as well as the thing you meant, and the two designs that varied tenancy *inside* five minutes found nothing at all.

This set varies tenancy inside ninety seconds, on the same machine, in the same run, with the same workload. It agrees with the pooled result and roughly doubles its size. That is the first evidence for #64 that does not have the confound.

**It does not restore the generalisation that record withdrew.** The withdrawal was that idle boxes are worse measurement environments *as a property* — 2026-08-02 was slow and tight, 9.4 units/s at 0.28%, which a low clock does not describe. This set is the same box on the same morning as the pooled holds it agrees with, so it makes the effect real *here* and says nothing about elsewhere. What it adds is a mechanism, and a mechanism is what would let the question be asked of another machine without repeating the whole night.

## Appended 2026-08-11, an hour later: the clock is a correlate, not the mechanism

The sections above were written from four holds. Every `hold.json` ever produced here carries `host.processor_performance`, so the relation was checkable across the whole archive at no cost, and it was — **83 one-session holds on mains, with both a clock reading and a rest column**, spanning clocks from 80.7% to 203.7%.

**The direction survives and generalises.** Rate against clock across all 83 is **r = +0.530**, so the morning's finding is not one set: busier box, higher clock, faster lone session, across days.

**The mechanism does not.** Two tests, both failed:

- **Normalising by clock makes the rate worse.** If the clock were what moved, dividing it out should collapse the spread. Rate alone has σ 17.8%; rate/clock has **σ 21.8%**. The clock over-predicts — across bands the rate rises 11.453 → 14.574, about 27%, while the clock rises from under 110% to over 170%, more than 60%.
- **Within a band of tenancy the clock stops predicting anything.** At rest under 2 cores, r = 0.052 across 35 holds; at 12–14 cores, r = 0.208 across 37 — and the bands carry real clock spread (σ 21.8% in the first), so these are tests with power rather than empty ones.

The partial correlations say it in one line. Controlling for tenancy, **r(rate, clock | rest) = +0.116**, down from +0.530. Controlling for clock, **r(rate, rest | clock) = +0.486**, barely down from +0.666. **The neighbour predicts the rate; the clock is a correlate of the neighbour.**

**What this does not do is refute the clock**, and the reason is the same instrument defect named above: `processor_performance` is one point sample at hold start, and noise in a predictor attenuates exactly these correlations. A genuinely causal clock, read badly, produces this result too. So the honest position is that the clock is **unproven** — both tests that could have supported it failed, neither can convict it, and the per-tick sampling this record asked for is now necessary rather than merely tidy.

**And the mechanism is open again.** The 2×2 stands: same CPU, 36.3% less work, with ordinary contention pointing the wrong way by roughly two. What raises the value of a core-second on a busier box is once more unexplained.

## Provenance

| | |
|---|---|
| Run | `wait-for-quiet.ps1` twelfth set, fired 08:28:05, complete 08:35 |
| Artifacts | `bench-out/1786404485-dur-30-1-daemon/`, `-1786404553-dur-120-1-`, `-1786404709-dur-30-2-`, `-1786404775-dur-120-2-` |
| Read from | `hold.json` and `concurrent-samples.jsonl`; the streamed console lines agreed but were not used |
| Machine | 16 logical / 31 GiB / Windows 11, mains |
| Tenant | `chrome-headless-shell`, present ~4 min and absent ~50–70 s in a repeating cycle |
