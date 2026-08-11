# The rising limb is reachable, and the tenanted arm is what loses it

**A 45-minute waiting run produced no pair and two facts worth more than one. A rising-limb baseline **is** reachable on this box — 1.03 cores held, below the 1.36–1.46 transition, with an idle observer. And what destroyed it was not the baseline but the *tenanted* arm: the browser returned mid-injection at **+12.20 cores**, so every gate improvement under consideration would have missed.**

## A hypothesis died, and the prediction was written first

Two consecutive runs had shown the quiet gate and the hold's own `rest` column disagreeing by almost exactly one core, which suggested a calibration mismatch between two instruments measuring the same quantity — a thing that has happened here before, where a machine column once sat on a sixteenth of the scale of the column it was subtracted from.

It was recorded as a hypothesis with a prediction attached: *if systematic, the next baseline lands near 1.9; if not, near 0.9.*

| gate reading (machine-wide) | baseline hold's `rest` | gap |
|---|---|---|
| 0.41 | 1.47 | 1.06 |
| 0.94 | 1.94 | 1.00 |
| **0.89** | **1.03** | **0.14** |

**Not systematic.** The first two gaps were real load arriving during those holds — the browser returning — and not an instrument offset. Writing the prediction down beforehand is what makes this a refutation rather than a re-reading: at 1.03 the natural move, absent a commitment, is to fold the reading in as noise around a systematic offset. Two agreeing points and a plausible mechanism is the density at which this investigation has gone wrong before.

## What the run actually did

`inject-tenant.ps1`, real quiet gate, **no `-AnyBaseline`**, 45-minute give-up so it would sit through a browser teardown rather than race one. Four attempts:

| void | kind | baseline `rest` | tenanted `rest` | delta |
|---|---|---|---|---|
| 1 | baseline above transition | 1.94 | — | — |
| 2 | baseline above transition | 1.78 | — | — |
| **3** | **tenancy moved** | **1.03** | **13.23** | **+12.20** |
| 4 | baseline above transition | 5.00 | — | — |

Attempt 3 is the cleanest rising-limb baseline this investigation has taken. The injection that followed was measured against a machine carrying twelve more cores than the baseline did, and the guard refused it — correctly, since that pair would otherwise have read as a spectacular injection effect.

## Every gate fix would have missed

Three instrument changes were under consideration, all aimed at the gate: a windowed mean rather than two consecutive point readings, a lower threshold, a longer give-up.

**None of them saves attempt 3.** The gate cleared at 0.89, the baseline passed at 1.03, and the failure happened afterwards, downstream of everything the gate can observe.

The gate's own behaviour is still worth fixing and is worth stating precisely, because it is not what it looks like. Across the run, **every** poll read below the 1.0 bar — 0.94, 0.88, 0.49, 0.78, 0.89, 0.90, 0.58, 0.83, 0.93, 0.62 — and most failed anyway, because the rule asks for two *consecutive* samples. This is not a loud machine being excluded; it is a machine at ~0.8 whose sampler occasionally reports above threshold. The census half of the gate contributed nothing at all: `0.00` on every poll for forty-five minutes, an `AND` term that is a constant `true`.

## The binding resource is window remaining, and nothing measures it

A pair needs roughly 90–120 seconds of continuous quiet **after** the gate fires: two polls, two 30-second holds, spawn and teardown. The window itself was measured at about four and a half minutes. But attempt 3 shows the browser can return within a minute of a passing baseline.

So window *length* is not the constraint — window length *remaining at the moment the gate clears* is, and a gate firing on two backward-looking polls says nothing about how much quiet lies ahead. That quantity cannot be predicted, which leaves failing fast as the only available lever: a spoiled tenanted arm is detectable by its second 5-second sample and currently costs a full hold, a teardown, and a share of a window that occurs about once an hour.

## What made this legible

Every figure above comes from one `inject.json`, written on the run's failure path by [a change committed two hours earlier](2026-08-12-042000-longer-holds-do-not-cut-the-spread-and-they-halve-the-acceptance.md). Before it, a run that produced no pair left nothing but transcript prose — and the watcher on this run dropped two of the four voids outright, emitting only the latest matching line per polling interval.

The stream was lossy and it did not matter. Watching a run and recording a run are different jobs, and only one of them has to be complete.

## What this does not establish

- **No pair, so nothing about the step.** The rising-limb sign contradiction is untouched: it now has two failed attempts against it rather than an answer.
- **One good baseline out of four attempts** says the rising limb is reachable, not how often.
- **The browser's return was not attributed in flight.** It is inferred from the tenanted arm's `rest` of 13.23 against a baseline of 1.03, which matches the Playwright GPU process's known size but was not sampled per-process during the hold.
- **`5.00` on attempt 4** is far above the other baselines and unexplained; it may be the same return still in progress.

## Provenance

| | |
|---|---|
| Run | `inject-tenant.ps1 -Tenants 6 -ExpectedPerTenant 0.19 -Duration 30 -MaxVoids 4 -GiveUpMinutes 45 -PollSeconds 10`, detached, no output redirect, 04:49:32–05:34 |
| Read from | `bench-out/inject-20260812-044932.json`, outcome `GaveUpOnVoids`, 4 voids, `pair: null` |
| Gate polls | ten, all below the 1.0 machine-wide bar, three achieving a consecutive pair |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, 0 survivors after teardown |
