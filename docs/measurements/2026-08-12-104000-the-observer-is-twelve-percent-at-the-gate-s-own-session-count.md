# The observer is twelve percent at the gate's own session count

**A hundred sessions held for two minutes on the quietest box of the night put the agent at 0.419 cores — 12.9% of the residual and 11.5% of the job. The estimate this run was launched to confirm was "a few percent", and it is wrong by roughly four times. The same run also shows a hundred `cpu-spin --duty 0.27` sessions holding 3.6 cores rather than saturating sixteen.**

## Why it was measured rather than assumed

[The observer's cost](2026-08-12-092000-the-clearance-notification-wakes-the-observer-into-the-hold-it-announced.md) was established at one session, where it dominates: the job holds ~0.25 cores and the agent 1.6, so two thirds of the residual is the operator. That is the worst case by construction, and the natural inference was that at a hundred sessions the same 1.6 becomes a few percent of a saturated machine.

The inference had two moving parts and only one was checked.

## The reading

100 sessions, 120 s, `cpu-spin --duty 0.27 --resident 1`, exit-only watcher so nothing woke the agent mid-hold. `doctor` beforehand: **2.50 of 16 cores held**, the quietest reading of the night, tenant absent.

| | cores |
|---|---|
| job median | **3.635** |
| `rest_cores_median` | 3.239 |
| **`observer_cores_median`** | **0.419** |
| machine alone | 2.820 |

**Observer as a share of the residual: 12.9%. Of the job: 11.5%.**

## The estimate failed on the denominator, not the numerator

The agent's own cost behaved as expected — **0.419 cores against 1.58–1.99 while working**, confirming that an exit-only watcher keeps it near idle across a two-minute hold.

What failed is the assumption that a hundred sessions saturate the box. They hold **3.6 cores**, not sixteen, so every share computed against them is four times what a saturated denominator would give. Per-session that is 0.036 cores against the 0.25 a solo session holds — these sessions are self-limiting under contention rather than competing for a full machine.

**So the absolute is what transfers and the percentage transfers nowhere**, which is the same lesson a daemon's footprint already taught: 363 KiB a session travels, and "2% of what it holds" does not.

## The tenant returned mid-hold and the job did not notice

| t | machine | job |
|---|---|---|
| 30–70 s | 4.6–5.3 | 3.4–3.7 |
| **75 s** | 7.2 | 0.5 |
| 85–121 s | **13.8–14.1** | 3.9–4.2 |

The browser took roughly nine cores at 75 seconds and the job's own occupancy was unchanged afterwards. On a box with eleven cores spare, a hundred self-limiting sessions and a ten-core tenant do not contend — which is consistent with the sessions never having asked for the machine in the first place.

## What this does not establish

- **It does not touch the M1 figures.** Those holds ran a different workload for an hour; this is two minutes of one duty setting. What it removes is the *assumption* that the observer is negligible at a hundred sessions.
- **The 12.9% is against an unsaturated job.** A gate hold whose sessions genuinely take the machine would show a smaller share and the same 0.42 cores.
- **One hold, one duty, two minutes.** The job's plateau at 3.6 cores is stable across fifteen samples, but a longer hold or a heavier duty was not tried.
- **Nothing here says the sessions are behaving wrongly.** `--duty` stretches its pause to match a slower unit, so self-limiting under contention is what it is documented to do; whether a hundred of them *should* reach sixteen cores is a separate question this run does not ask.

## What it changes

1. **Read `observer_cores_median` before quoting any residual**, at every session count rather than only at one. The field exists on every hold now and costs nothing to check.
2. **The estimate that motivated this run is withdrawn.** "A few percent at gate scale" was an inference from a saturation that does not occur here.
3. **A hundred `cpu-spin --duty 0.27` sessions are not a saturating load on this box**, which is worth knowing before any future run assumes they are.

## Provenance

| | |
|---|---|
| Hold | `sessionbench hold --label obs100 --sessions 100 --interval 5 --duration 120 -- cpu-spin --duty 0.27 --resident 1`, detached, exit-only watcher |
| Read from | `bench-out/1786498462-obs100-daemon/hold.json` and its `concurrent-samples.jsonl`, 24 samples |
| Before | `doctor`: 2.50 of 16 cores held, 44.1 °C, **mains** |
| After | 100 of 100 sessions still running, 0 survivors after teardown |
