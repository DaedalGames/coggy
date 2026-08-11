# A solo baseline cannot see the plug

**Two quiet holds of the same workload, ninety minutes apart, read 9.574 and 9.322 units/s — agreeing to 2.6%. One was on mains and the other on battery, a state worth 7.8× on this box.** The solo baseline is the fingerprint you would reach for to check that two runs saw the same machine, and it is blind to the largest single variable the machine has.

## How this was found

By nearly publishing the opposite. A twenty-four hold sweep was compared against the morning's [injection experiments](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md) and produced an arresting headline: at matched tenancy, 2.06 cores of injected `cpu-spin` gave 19.155 units/s where 2.80 cores of browser gave 8.490 — the same box, the same core count, half the throughput, so the neighbour's *identity* rather than its size. The licence for calling it the same box was that the two quiet baselines agreed to 2.6%.

`doctor` was then run as the last step of the gate and said **ON BATTERY**. Every hold in the sweep records `on_battery: true`; both injection holds record `false`.

**The comparison is void** — [a claim across a power state is not a claim](../../CLAUDE.md#treat-the-machine-as-a-variable) — and so is the sweep's other headline, that a 2× duty gap failed to reproduce: the 71.50 reading is a mains hold and the three 37.x readings that would have refuted it are battery.

## Why the check that was made did not catch it

| | cores held | rate | job | rate/job | power |
|---|---|---|---|---|---|
| injection baseline, 14:15 | 0.75 | 9.574 | 0.26 | 36.4 | **mains** |
| sweep, 16:24 | 0.76 | 9.322 | 0.26 | 35.7 | **battery** |

**2.6% apart, across a boundary worth 7.8×.**

The 7.8× was measured [at a hundred sessions at duty 0.27](2026-08-02-195840-the-same-command-on-battery.md), where the box is saturated and the power plan's ceiling binds. A lone session at duty 0.27 asks for a quarter of one core out of sixteen. **Nothing about that demand requires the machine to leave its lowest power state**, so the plug makes no difference to it — and the quantity most often used to say "these two runs saw the same machine" has no signal on the variable that matters most.

This does not weaken the fingerprint for what it was built for. Two ramps' solo rungs disagreeing still means the afternoon moved. It means agreement is not the converse: **a solo baseline agreeing proves the box is not in the *slow state*, and says nothing about the plug.**

## What the sweep does establish, entirely within one power state

All twenty-four holds of the 15:50–16:31 sitting are on battery, so comparisons inside it are sound. `rate/job` is the column; `job` is the achieved duty, which falls under load as the session is descheduled.

| band (cores held elsewhere) | duty 0.27 | duty 1.0 |
|---|---|---|
| 0.0–1.5 | **34.41** (n=3) | **37.37** (n=3) |
| 1.5–3.5 | 34.60 (n=2) | 34.68 (n=1) |
| 3.5–8.0 | — | — |
| 8.0+ | 32.62 (n=4, 6 censored) | 41.02 (n=11) |

- **Duty is worth about 9% per core-second on battery, not 100%.** The three duty-1.0 holds at 0.84, 1.10 and 1.19 cores read 37.03, 37.29 and 37.79 — agreeing to 2% with each other.
- **The duty-0.27 arm is flat across tenancy**: 35.73, 35.87, 31.61, 35.58, 33.62, 28.36, 33.54, 32.16, 36.44, from 0.76 to 12.55 cores. On battery, in this sitting, there is no tenancy step.

  > **Appended 2026-08-11 17:34: that flatness is a fact about battery, and the mains set was already on disk.** Eighteen `inject-before` baselines from the afternoon, never read as a set, give **+60.9%** across the same tenancy range where this sitting gives **−5.2%** — [the step is a mains phenomenon](2026-08-11-173433-the-tenancy-step-is-on-mains-and-absent-on-battery.md). So the sentence below, *whether that flatness contradicts the mains injections is exactly what cannot be said from here*, is answered: it does not contradict them, and the reason is the plug. What could not be said from here could be said from the artifacts next to it.

Whether that flatness contradicts the mains injections is exactly what cannot be said from here.

## What this does not establish

- **The injections are untouched.** They were controlled, guarded, and repeated with two binaries, both arms on mains within one minute of each other. Nothing here reaches them.
- **The battery flatness is one sitting**, and it cannot be set against a mains sitting until one exists at the same tenancies.
- **The 3.5–8.0 core band is empty in both arms.** This box's tenant is bimodal — near-idle or 10+ cores — so the middle of the curve went unsampled again.
- **The 8+ band is not a comparison.** Six duty-0.27 holds fell below the `job = 0.15` floor where `rate/job` stops being readable, against none for duty 1.0, because a session demanding a full core keeps a larger share than one that sleeps and loses its slot. The survivors of a censored arm are its least-starved holds. The floor and the bands were set before the data was read.
- **When the plug was pulled is not recorded.** The last mains hold is 14:42:55 and the first battery hold is 15:40:33, so it happened somewhere in that hour, and no artifact narrows it.

## The cheap fix this argues for

Every artifact already carries the power state, and `doctor` prints it. What is missing is that **nothing refuses a comparison that crosses it** — `compare` refuses a pair whose solo rungs disagree, which is precisely the check just shown to be blind here. A power-state mismatch is a single boolean sitting in both files.

## Provenance

| | |
|---|---|
| Sweep | `wait-for-quiet.ps1 -Duties 0.27,1.0 -MaxHolds 40 -FireBelow 20 -CountBelow 20`, 15:50–16:31, stopped by PID at 30 holds, **battery** |
| Injections | `inject-tenant.ps1`, 13:32 and 14:15, **mains** |
| Artifacts | `bench-out/*quiet-solo-*-r20-d*/`, `bench-out/*inject-*/`, transcript `bench-out/quiet-20260811-155011.log` |
| Read from | `hold.json` — `units_per_session_per_sec`, `occupancy.median_cores`, `occupancy.rest_cores_median`, `host.on_battery` |
| Analysis | [`scripts/duty-bands.ps1`](../../sessionbench/scripts/duty-bands.ps1), written before the data was read, so the bands, the sitting split and the `job = 0.15` floor could not be chosen for their answer — it did not check the power column, which is the defect this record is about. Ported from the throwaway it began as and checked against it: same 6 held-out holds, same 29 in 4 sittings, same band means |
| Machine | 16 logical / 31 GiB / Windows 11, 0 survivors after teardown |
