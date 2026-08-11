# The neighbour helps one session and robs a hundred

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
