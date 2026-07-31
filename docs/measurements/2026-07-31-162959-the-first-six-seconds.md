# A session's first six seconds run 46% fast · 2026-07-31 16:29:59

`observe` controls its memory figure — it reports the first measured quarter against the last, so an RSS number that is really two different sessions averaged says so. It had no such control on the work rate, which is the axis [G0 leans on hardest](2026-07-31-150258-g0-frozen.md): `d` in `2ηC/d` is read off a run's cores, and a machine that moved mid-observation would hand that field a blend in silence.

Adding the control took twenty minutes. Its first real reading was a false alarm, and chasing that down is the measurement.

## The alarm

A single `cpu-spin` session, 45 seconds, nothing else asked of the machine:

```
work drift   42.37 units/s early, -18.8% by the end
work rate    37.26 units/s
```

**The mean hides it completely.** A session that slowed by a fifth reports as one number.

## Two explanations, both falsified

**Core migration.** [This machine's sixteen cores span 2.1× in three tiers](2026-07-31-145412-the-cores-are-not-interchangeable.md), so a session that drifts from a performance core to a low-power one loses roughly what was seen. Pinning the process to core 5 — 190% of nominal under full load — left it unchanged:

| | Early | Drift |
|---|---|---|
| Unpinned | 42.37 units/s | −18.8% |
| **Pinned to core 5** | 44.48 units/s | **−19.1%** |

**The core clocking down.** Loading that one core and reading its own delivered performance every five seconds gives 142, 136, 129, 144, 146, 138, 134, 130, 142% of nominal — **no trend**, with the package flat at 59.1 °C. It is a laptop on a vendor power plan, so sustained-load throttling was the obvious candidate, and it is not what happened.

## The samples say it was over before either test started

Rate computed straight from the run's own `samples.jsonl`, five seconds at a time:

| Window | Rate | CPU |
|---|---|---|
| **1–6 s** | **50.31 units/s** | 97.9% |
| 6–11 s | 34.45 | 98.1% |
| 11–16 s | 34.03 | 94.6% |
| 16–42 s | 34.0 – 35.0 | 87–99% |

**Flat for forty seconds, and 46% faster for the first six.** CPU sits at 98% throughout, so the session held a full core the whole time — it was simply a faster core for six seconds. That is single-core boost, and the clock counter above missed it because its first sample landed at five seconds, after the window had closed.

## So the defect was in the control, not the machine

`observe` quarters the whole run for RSS, which is correct — residency has no such transient. The work-rate control inherited that convention when it should have taken the one the cores figure already uses: **everything past `STARTUP_WINDOW`**. Quartering across the boost reports drift that never happened.

Corrected, the same run shape reads **−0.6%**.

**The thirty-second startup window turns out to be generous rather than arbitrary.** The transient here lasts about six seconds, so the cut has five times the margin it needs — on this machine, for this workload.

## What it costs elsewhere

- **Nothing in the ramps.** A rung's spin-up is a third of its hold, floored at the same thirty seconds, so every ramp already measured past the boost. The numbers frozen in G0 are untouched by this.
- **The short `observe` runs used as pre-flights today read high** — 36.83, 47.29 and 57.24 units/s over runs of one to eight seconds. None was quoted as a result, and none could have been: they are boost rates.
- **A workload measured for less than the startup window measures the boost**, whatever else it thinks it is doing.

## What this rests on

- **One workload, compute-bound, 8 MiB resident.** `cpu-spin` at this size is pure ALU, which is where a clock difference shows most. A memory-bound session would see less of it, the same way [the 80 MiB workload hid the core tiers](2026-07-31-145412-the-cores-are-not-interchangeable.md).
- **The clock counter never caught the boost directly**, only its absence afterwards. The evidence for it is the rate curve plus a flat CPU percentage, which is two routes rather than a reading of the clock itself.
- **One machine, one power plan.** How long a boost lasts is a firmware decision.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Power | AC, SAMSUNG MODE plan, 59.1 °C throughout |
| Workload | `cpu-spin --units 999999 --resident 8`, duty 1.0 |
| Runs | 45 s unpinned, 45 s pinned to core 5, 45 s of per-core counter sampling, 75 s to verify the fix |
| sessionbench | 0.0.0 at commit `106cff2`, debug build — the drift is a ratio, so the build's own cost cancels |
