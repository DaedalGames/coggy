# The step is at one and a half cores

**A lone session's rate jumps 33% when something else on the box crosses about 1.4 cores, and its own CPU does not move.** Seventeen 30-second holds inside forty minutes, tenancy spanning only 0.99 to 2.44 cores: `r(rate, rest) = +0.847`, with the job holding 0.259 to 0.275 cores throughout — a 6% spread while its rate varied 71%.

## The series

Every hold is one `coggyd` session running `cpu-spin --units 100000000 --duty 0.27 --resident 20`, fired by [`wait-for-quiet.ps1`](../../sessionbench/scripts/wait-for-quiet.ps1) whenever it certified the box quiet. Sorted by the cores held outside the job:

| rest cores | rate | clock | job cores |
|---|---|---|---|
| 0.99 | 11.039 | 118.4% | 0.26 |
| 1.01 | 10.409 | 151.2% | 0.27 |
| 1.26 | 10.079 | 127.8% | 0.26 |
| 1.36 | 11.768 | 138.1% | 0.27 |
| **1.46** | **15.592** | 131.0% | 0.26 |
| 1.48 | 14.464 | 155.1% | 0.27 |
| 1.89 | 14.873 | 107.7% | 0.26 |
| 1.98 | 17.055 | 106.3% | 0.27 |
| 1.99 | 15.086 | 139.0% | 0.26 |
| 2.00 | 17.090 | 111.8% | 0.26 |
| 2.03 | 17.218 | 115.0% | 0.27 |
| 2.04 | 16.660 | 117.4% | 0.26 |
| 2.05 | 17.222 | 113.8% | 0.27 |
| 2.07 | 15.605 | 106.8% | 0.27 |
| 2.28 | 15.915 | 118.9% | 0.27 |
| 2.29 | 16.985 | 115.1% | 0.26 |
| 2.44 | 15.611 | 160.9% | 0.27 |

Below 1.5 cores the mean is **12.225** across six holds; at 1.5 and above it is **16.302** across eleven. **+33.3%**, and the four lowest-tenancy holds are the four slowest.

**The transition sits between 1.36 and 1.46 cores.** That is the number [the archive could only bound between about 1.3 and 2.5](2026-08-11-083823-same-cpu-less-work-when-the-box-goes-quiet.md), because the browser this box hosts offers roughly zero cores or eleven and nothing in between. These holds got the middle by accident — the waiter certifies "no process over half a core", which admits a machine carrying one to two cores of small processes.

## The size agrees with the archive and the method does not

The archive gave +34% by banding 83 holds across nine days into "under 2 cores" and "2 and above". This gives +33.3% from one run of forty minutes with tenancy varying 1.45 cores. Same box, same workload, wholly different sampling — and the second does not inherit the first's weaknesses, since [a comparison across windows varies time as well as the thing you meant](2026-08-03-173452-the-slow-state-flatters-the-gate.md) and this one does not span windows.

It also explains why the effect looked like a step with flat shelves. Both shelves are far from the transition: at 2.0 to 2.44 cores the rate is already saturated at ~16.5, and [between 2.5 and 13.2 cores it does not move outside its own scatter](2026-08-11-083823-same-cpu-less-work-when-the-box-goes-quiet.md).

## The clock is refuted, backwards

`r(rate, clock) = −0.429`. The P-state reading predicts a positive relation — a busier box clocks up, so a core-second buys more — and the fastest holds here carry the **lowest** clock: 17.222 at 113.8%, 17.090 at 111.8%, 17.055 at 106.3%, against 10.409 at 151.2% and 10.079 at 127.8%.

That is stronger than the archive's refutation, which showed the clock climbing 21% while the rate moved 1.1% and falling while the rate rose. Here the two move consistently in opposite directions across seventeen holds in one run.

`processor_performance` remains a point sample at hold start, so this cannot quantify anything. As a sign test it is unambiguous.

## What the job did not do

The job's own occupancy ran **0.259 to 0.275 cores** — 6% — while its rate ran 10.079 to 17.222, a spread of 71%. It was never starved, never descheduled, never given more. Whatever changed, changed how much work a core-second buys, and it changed by 71% across 1.4 cores of load belonging to someone else on a sixteen-core machine.

## Core parking is out, measured rather than argued

`\Processor Information(*)\Parking Status` exists on this box and reads per core. **At idle it reports 12 of 16 cores parked** — 0 through 3 awake, 4 through 15 asleep. That alone is worth knowing for anything measured here at low load.

Driving the box with `cpu-spin` sessions and reading the counter at each level:

| load | cores busy | cores parked |
|---|---|---|
| idle | 2.30 | **12** |
| +2 sessions | 2.44 | **12** |
| +6 sessions | 4.27 | **12** |
| +12 sessions | 5.42 | 9 |
| after | 2.41 | **12** |

**The parked count does not move across the step.** Six sessions add about 1.6 cores, straddling the 1.36–1.46 transition, and parking is unchanged at 12. Unparking begins somewhere between 4.27 and 5.42 busy cores — roughly three times higher than where the rate jumps.

So core parking is eliminated, and with it the last candidate this record named. The clock was refuted backwards at `r(rate, clock) = −0.429`; core placement across this box's [2.1× tiers](2026-07-31-145412-the-cores-are-not-interchangeable.md) was refuted by [the low shelf showing no bimodality](2026-08-11-083823-same-cpu-less-work-when-the-box-goes-quiet.md); parking is refuted here. **Three mechanisms out, and the effect is as solid as it has ever been.**

**One caveat that could rescue it.** The injected load was `cpu-spin` at duty 0.27, which is bursty — 27% on, 73% idle. Parking responds to *sustained* utilisation, so a steady 1.6 cores might unpark where a bursty 1.6 does not. Testing that needs a full-duty workload at a controlled core count, which is a different run. The tenancy in the seventeen holds was also not ours and its duty is unknown.

## What this does not establish

- **One run, one box, one workload, seventeen holds.** Four of them came from test invocations of the waiter with a deliberately opened fire bar, which affects how they were *triggered* and not what their rest column recorded.
- **The mechanism is still unnamed.** Core parking remains the candidate — 1.4 cores of external load plus the job's own 0.26 is about 1.7 cores, which could plausibly cross an unparking threshold on a sixteen-core box — and nothing here records parked cores. This record names a location, not a cause.
- **The boundary has four points below it and thirteen above.** 1.36 → 1.46 is sharp in this data and thinly sampled at exactly the place it matters.
- **`rest_cores_median` is a median over a 30-second hold**, so a hold whose tenancy moved is summarised by its middle.
- **The waiter's certification is not the rest column.** It sums only processes above half a core; `rest_cores` counts everything. That difference is why these holds carry 1 to 2.4 cores while being called quiet, and it is what made this measurement possible.

## Provenance

| | |
|---|---|
| Run | `wait-for-quiet.ps1`, 2026-08-11 09:52–10:34, stopped early once the fire bar blocked on a steady ~1.7-core process |
| Holds | 17 × `hold --sessions 1 --interval 5 --duration 30`, labels `quiet-solo-*` |
| Read from | `hold.json` `occupancy.rest_cores_median`, `units_per_session_per_sec`, `host.processor_performance` |
| Machine | 16 logical / 31 GiB / Windows 11, mains |
