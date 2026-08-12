# The neighbour helps one session and robs a hundred

> **2026-08-12 — this now has a mechanism: the neighbour keeps cores UNPARKED.** Windows parks cores on this box when nothing sustained runs. [A 2x2 gives 5.09-5.37 machine cores with `chrome-headless-shell` absent against 14.47-14.60 with it present](2026-08-12-143500-ten-of-sixteen-cores-are-parked-under-a-hundred-sessions.md), and three holds minutes apart read 14.53, 5.15 and 14.60 as it came and went. So the neighbour raises what the box delivers while taking a share of it — which is why one session gains and a hundred, already limited by their own duty, only lose the share. The correlation recorded below is that effect seen through throughput; the core count is the thing itself.

**The same tenant that makes a lone session 34% faster costs a hundred sessions three quarters of their throughput.** Gate M1's work-rate condition divides the first by the second, so tenancy pushes both halves of the ratio the same way, and the verdict moves by roughly a factor of four with nothing in the run's own report saying which case it was.

## The two arms, measured separately

**One session.** [Banded by the cores held elsewhere](2026-08-11-083823-same-cpu-less-work-when-the-box-goes-quiet.md), a solo hold steps about **+34%** upward once more than ~2 cores are held by anything else, flat on both sides. Reproduced inside four separate sittings at +44.8%, +40.4%, +20.3% and +31.1%, so it is not a comparison across windows.

**A hundred sessions.** The seven hundred-session mains holds that carry a rest column run the other way, and hard:

| when | per-session | rest cores | total |
|---|---|---|---|
| 08-03 17:19 | 9.0276 | 0.77 | 903 |
| 08-11 02:34 | 7.5611 | 0.49 | 756 |
| 08-11 03:10 | 5.3033 | 8.88 | 530 |
| 08-11 02:51 | 4.3188 | 8.53 | 432 |
| 08-03 18:45 | 3.4492 | 8.56 | 345 |
| 08-11 02:10 | 2.9727 | 10.24 | 297 |
| 08-03 07:49 | 2.1651 | 13.60 | 217 |

**r = −0.950**, slope **−0.4889 units/s/session per core held**. Fitted, the line predicts 829 units/s total at 0.5 cores held and 267 at 12 — a span of **3.1×**.

This is ordinary contention and it makes sense in a way the solo arm does not. A hundred sessions already want every core, so a neighbour taking twelve is pure loss. One session wants a quarter of a core, so it never competes with the neighbour at all, and whatever the neighbour does to the machine's state is the only effect left.

## What it does to the gate

`slowdown = solo ÷ concurrent-per-session`, asked to come in under 2. Tenancy raises the numerator and lowers the denominator, so it compounds. Using each arm's pooled figures to show the size:

| | solo | conc/session | slowdown | |
|---|---|---|---|---|
| both arms quiet | 11.327 | 9.0276 | **1.255** | pass |
| solo tenanted, concurrent quiet | 14.741 | 9.0276 | **1.633** | pass |
| solo quiet, concurrent tenanted | 11.327 | 2.9727 | **3.810** | fail |
| both arms tenanted | 14.741 | 2.9727 | **4.959** | fail |

**The four cells span 3.95×, against a threshold of 2, and the browser decides which one you get.**

**That table is an illustration, not a measurement.** Its cells pool arms from different sittings, which is the confound this repository has been caught by twice. What is measured is the two arms separately: the solo step within sittings, and the concurrent slope across seven holds. The table only multiplies them out.

## It resolves a pair the instrument called unresolvable

[`BracketedReport`'s fifth reading step](../../sessionbench/src/daemon.rs) records two runs whose solo holds agreed to half a percent — 9.752 and 9.801 — and whose hundred-session throughputs were **246.4 and 902.8 units/s**, giving slowdowns of **3.958 and 1.54**. One fails by 98% and the other passes a condition asking for 2. The instrument's advice was that no number of solo repeats separates them and only the concurrent throughput can say which machine you were on.

The advice was right and the reason was not a named machine state. **Those two slowdowns sit almost exactly on the two mixed cells above** — 3.810 and 1.633 — and 902.8 is the top of the tenancy line while 246.4 is near its bottom. The pair is what this relation produces when the browser is present for one run's concurrent hold and absent for the other's.

**The 246.4 hold predates the rest column, so it cannot be checked directly** — that attribution is inference from the line, not a reading. What is checkable is that the two clusters the record describes, "six runs between 903 and 1055 and three between 217 and 288 with nothing in the 3.1× gap", straddle exactly the range this line spans, and the gap has since been filled from both sides at 344.9 and 756.1, both of which sit on it.

## A competing explanation, raised the same hour, which fits better

**The solo arm may not be a benefit at all.** Two facts already recorded elsewhere point the other way:

- Nine one-session holds gave **18.9 units/s under four cores held** and **13.8 over nine** — quiet *faster* than crowded, the opposite sign to the step above.
- Of 45 holds whose tenancy is known, **33 were slow, 7 tenanted, 3 between and 2 rested.** The slow state is most of what this box does when nothing else is on it.

So a quiet box is **bimodal** — rarely rested at 18.9–20.3, usually slow at 9–11 — while a tenanted box sits reliably in the thirteens and fourteens. A low-rest sample is then dominated by slow-state holds and a high-rest sample contains none, which produces a step without tenancy causing anything.

**The shape of the two groups favours this reading over mine.** If tenancy added speed, the high shelf would be the low shelf translated upward and would keep its width. Instead it is *narrower*: σ 9.2% across 48 holds against 17.7% across 35, and it never reads below **11.952** where the low shelf reaches 9.246. Truncation, not translation.

**And the within-sitting control does not separate them.** Inside a sitting where the box is in the slow state, the low-rest arm is slow and the high-rest arm is not, which reads as a step on either account. The four sittings establish that the difference is not a comparison across windows; they do not establish direction of cause.

**What would separate them is deliberate co-tenancy**: inject a known load onto a box already measured as slow, and see whether the lone session recovers. If it does, the neighbour excludes the slow state and this is a finding about the slow state. If it does not, the step is what it looked like. Nothing on disk answers it, because every low-rest hold here got that way by the browser leaving rather than by anything being added.

**Until then the causal claim in this record's title is not established, and the concurrent arm is unaffected** — that one is ordinary contention at r = −0.950 and does not depend on which reading of the solo arm is right.

### The competing reading made a prediction, and the archive already held the test

If the neighbour is not adding speed but forcing the machine into a definite state, then it should pull a lone session toward that state **from either side** — up from the slow band, and *down* from a rested one. So on a rested box the step should **reverse**.

| sitting | quiet arm | tenanted arm | step |
|---|---|---|---|
| 08-03 08:08 | **20.30** (rested) | 13.42, 15.71 | **−28.2%** |
| 08-03 14:34 | 9.30, 9.87, 10.06, 10.07 … | 15.22 | +44.8% |
| 08-11 02:07 | 9.55, 10.74, 11.34, 11.53 | 11.95, 14.05, 14.39, 15.43 … | +40.4% |
| 08-11 05:08 | 9.25, 10.63, 10.74, 12.43 … | 12.41, 12.67, 12.75, 12.96 … | +20.3% |
| 08-11 08:00 | 10.29, 11.18 | 12.30, 13.86, 14.40, 14.48 … | +31.1% |

**The one sitting whose quiet arm was rested reverses, and its tenanted arm lands at 13.42 and 15.71 — the same place every other sitting's tenanted arm lands.** The shelf shapes agree: the tenanted group spans 11.952–17.101 at σ 9.2%, sitting *inside* the quiet group's 9.246–20.295 at σ 17.7%. A clamp toward ~14.7, not a shift upward.

**This reconciles the two-band figure rather than overruling it.** *18.9 quiet against 13.8 crowded* was measured when this box was rested, so the neighbour pulled it down; the +34% here is mostly slow-state holds, so the neighbour pulled them up. Both are correct readings of one phenomenon, and each looked like a contradiction only because each saw one side of the clamp.

**The rested case is n = 1**, and that one hold is the 20.295 already flagged as an outlier. A single observation confirming a prediction is still a single observation, and it is the whole of the evidence for the reversal. What would settle it is cheap and needs no injection: **a solo pair taken on a rested box, then again while a neighbour is present.** Rested windows are rare here — 2 of 45 holds — but they are recognisable at the time rather than only afterwards.

## It does not reach the gate's own figures, and that was checked rather than assumed

The compounding above is a hazard for any slowdown whose two arms sat in different tenancy states. **Gate M1's published figures are not among them**, and the check works despite `rest_cores_median` being younger than those artifacts.

[The gate run](2026-08-03-014841-the-gate-misses-by-three-percent.md) records solo baselines of **21.809, 21.787, 21.902** before and **21.708, 21.656, 21.858** after, spread 0.42%, from a box confirmed rested by a 21.555 hold beforehand. A tenanted solo reads about 14.7 under the clamp above; the rested band is 18.9–20.3 and the slow band 9–11. **A clamp that pins to 14.7 cannot produce 21.9**, so those six holds had no neighbour — the rate itself places the state, which is the general way to recover tenancy from any artifact predating the column. The denominator is filtered the same way: the interval-level re-reading compares only intervals where the job held **15+ of 16 cores**, leaving under one for anything else.

**So 2.065, 2.057 and 2.089 are ratios of two untenanted states, and M1's three-to-four-percent shortfall needs no correction from this record.** What the contamination does reach is the 3.958/1.54 pair, the 246.4/902.8 throughputs, and any casual hold used to judge whether a window is open.

One loose end: 21.7–21.9 sits *above* the rested band of 18.9–20.3 that nine holds established. Either that band's ceiling is higher than nine holds showed, or the gate ran in a better state than "rested" names — a fourth state, which would matter more, since M1's baseline would then be reproducible only in something nobody has learned to recognise.

**The archive cannot decide it.** Only four solo mains holds sit at 17 units/s or above — 17.10, 17.56, 20.30, 21.40 — and of those two predate the rest column and one is a 20-second hold, a duration this repository does not treat as interchangeable with 120. The gate's own 21.7–21.9 baselines were pruned from `bench-out` long ago. Four points, two of unknown tenancy, one of odd length, is not a test for bimodality.

### The clamp is a band, not a point

Twelve of the thirteen solo holds at 16 units/s and above are **tenanted** — rest 12.12 to 13.71, rates 16.134 to 17.101. So the tenanted group reaches 17.1, well above the 14.7 its mean sits at, and "clamps toward ~14.7" overstates what the data supports.

What is supported: tenanted holds occupy a **narrower band of roughly 12–17** (n=48, σ 9.2%) against quiet holds' 9.2–20.3 (n=35, σ 17.7%), and on the one rested occasion the tenanted arm sat below the quiet one. Tighter and higher on average, not pinned to a value. The reversal remains the load-bearing observation and remains n = 1.

## Most of a solo set's spread is band-mixing, not instrument noise

One workload on this box has produced spreads of 3.95%, 0.42% and 5–37% on three different days, which read as three noise levels of one instrument. Sorting every solo mains hold into sittings and labelling each by the band it landed in — quiet-slow under 2 cores held and below 18 units/s, rested under 2 cores and at or above 18, tenanted at 2 cores or more — separates them cleanly:

| sitting | n | mean | spread | bands present |
|---|---|---|---|---|
| 08-11 03:59 | 12 | 15.282 | **6.4%** | tenanted |
| 08-03 15:56 | 11 | 10.864 | **9.7%** | quiet-slow |
| 08-03 06:50 | 6 | 14.163 | 11.3% | tenanted + unknown |
| 08-11 08:00 | 8 | 13.240 | 12.4% | quiet-slow + tenanted |
| 08-11 05:08 | 24 | 13.824 | 14.3% | quiet-slow + tenanted |
| 08-03 14:34 | 10 | 10.979 | 15.0% | quiet-slow + tenanted |
| 08-11 09:20 | 5 | 11.817 | 16.8% | quiet-slow + tenanted |
| 08-03 08:08 | 3 | 16.474 | 17.4% | rested + tenanted |
| 08-11 02:07 | 12 | 13.695 | 17.8% | quiet-slow + tenanted |

**Every set that stayed in one band spread less than every set that straddled two — 6.4% and 9.7% against 11.3% through 17.8%, with no overlap.** Mixing 10.5 with 14.7 gives about 33% on its own, before any instrument noise. So the three daily figures are three amounts of mixing rather than three noise levels, and a spread quoted for holds that wandered between bands is measuring the tenant's schedule.

**It does not reach 0.42%, and the residue is the useful part.** Neither single-band set gets close: 6.4% and 9.7%. The 9.7% one is entirely quiet-slow, and the slow state is independently *slow and variable*; the 6.4% one is entirely tenanted but its `rest_cores` standard deviation is **3.86**, so tenancy ranged from about 2 to 13 cores inside the band. A band is a coarse bin, not a controlled condition. Spread here decomposes into at least three terms — band-mixture, tenancy variation within a band, and what is left when both are held still, which is [the gate's 0.42%](2026-08-03-014841-the-gate-misses-by-three-percent.md).

**So quote a spread only for holds that stayed in one band, and say which.** Two sets each is thin, and the separation is complete rather than marginal.

## Appended later the same day: the two arms are one curve, not two effects

This record's title sets "helps one session" against "robs a hundred" as though they were opposite findings that happened to coexist. [Holds taken across the whole tenancy range](2026-08-11-103621-the-step-is-at-one-and-a-half-cores.md) show they are two points on one relationship.

A lone session's rate **rises** across ~1.4 cores of external load, is flat through the middle, and **collapses** above ~4 — an inverted U. One session at 0.26 cores never competes with the neighbour for cores, so it lives on the rising limb; a hundred sessions want every core they can get, so they live on the falling one and never see the rise at all.

The footprint arms separate the limbs by mechanism as well as by position. The rise is **footprint-independent** — +17.5% at `--resident 20` against +17.8% at `--resident 1` — while the fall is not: resident 20 drops to 10.824 above 4 cores held where resident 1 holds 15.269, 41% apart. So the falling limb is **memory contention**, which is exactly why a hundred sessions each holding 20 MiB suffer so badly, and the rising limb is something else entirely.

**The `r = −0.950` above is unaffected**, being measured only on hundred-session holds, which never leave the falling limb. What changes is the framing: there was never a tension between the two arms to resolve.

## What this does not explain

**The solo slow state survives untouched.** A lone session reading 9.0 units/s against a rested 18.9, [with 1.03 cores held at the time](2026-08-03-094550-the-slow-state-caught-on-a-quiet-machine.md), is not tenancy — the neighbour was absent and the session was still at half speed. The step described here is worth 34%; that is worth 2×, and it remains unexplained. The two are separate findings that both happen to involve a quiet machine.

Other limits:

- **Seven concurrent holds.** r = −0.950 is strong for n = 7 and it is still n = 7, from two days.
- **The line is fitted across tenancy only.** These holds differ in other ways — footprint, duty, time of day — and nothing controls for those.
- **No deliberate variation.** Every point comes from the browser arriving or leaving on its own schedule. A controlled co-tenancy sweep would say whether the relation is causal and where it bends.
- **One machine.** All of it.

## Provenance

| | |
|---|---|
| Read from | 7 hundred-session and 83 one-session `hold.json` on mains, `bench-out/` |
| Fields | `units_per_session_per_sec`, `occupancy.rest_cores_median`, `sessions`, `host.on_battery` |
| Machine | 16 logical / 31 GiB / Windows 11, mains |
| Tenant | `chrome-headless-shell`, ~4 min present and ~50–70 s absent in a repeating cycle |
