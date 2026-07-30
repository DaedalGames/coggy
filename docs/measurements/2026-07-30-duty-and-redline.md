# Duty and redline · 2026-07-30

**Still not the M0 baseline** — see [the first redlines](2026-07-30-first-redlines.md). What this does is reduce the baseline to a single number that a real session can supply.

## The finding

A session's redline moves inversely with how much of its time it spends computing. Three points, and their product is flat:

| Duty | Solo rate | redline | `redline × duty` |
|---|---|---|---|
| 1.00 | 81.6 units/s | 25 | 25 |
| 0.50 | 41.2 units/s | 48 | 24 |
| 0.25 | 20.4 units/s | **100** | 25 |

**redline(duty) ≈ redline(1.0) ÷ duty**, holding to within 4% across a fourfold range.

The mechanism is in the runs rather than inferred. Cores plateau at **15.1 to 15.3 of 16 at every break**, whatever the duty — the machine has the same ceiling each time, and duty only decides how many sessions it takes to reach it. Memory is nowhere near involved: a hundred sessions at quarter duty held in 2.35 GiB.

Those core figures were taken with a fifteen-second hold, whose spin-up was too short to clear session startup, so they read somewhat high — see [the correction](2026-07-30-first-redlines.md#correction-the-cores-columns-above-read-high). The plateau itself survives it: at these session counts the machine is saturated by a wide margin, and the redline values come from work rate rather than cores.

## Why this matters more than the number

[Gate G0](../../ROADMAP.md#current-priority-m0--attribution) needs a redline for a real generation session, and no such session exists on this machine. That looked like a blocked gate. It is a **missing scalar**:

1. The machine's redline at full duty is measurable today, and is 25 here.
2. A real session's duty is one cheap measurement once the harness runs — `observe` already reports the cores a session occupies, which is duty against the solo figure.
3. Dividing gives the redline.

So the harness is needed for one number rather than for the whole gate.

## What would break the relation

Stated because it is worth checking rather than assuming when the harness arrives.

- **This workload waits by sleeping.** A generation session waits on a model and on I/O. Sleeping frees a core cleanly; a blocking read may not free it as cleanly, and that would bend the curve.
- **This workload writes nothing while it waits.** [Defender charges roughly 1.6 CPU-seconds per MiB written](2026-07-30-conhost-and-defender.md), and that term scales with sessions independently of their duty, so a write-heavy session has a second slope the relation does not carry.
- **Memory never entered.** A session much heavier than 24 MiB reaches the RSS condition on its own schedule, and the two ceilings would have to be taken together.

The relation is a way to read the machine, not a law about sessions. Check it against a real one before quoting it.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| sessionbench | 0.0.0 at commit `42b543b85739`, clean tree, release build |
| Workload | `cpu-spin --units N --resident 20 --duty D` |
| Holds | 15 s per rung, first third unmeasured |
| Resolution | 2 sessions at duty 0.5, 3 at duty 0.25 |
| Defender | real-time protection on, no exclusions |

The duty-1.0 point is from [the first redlines](2026-07-30-first-redlines.md#the-core-ceiling--cpu-spin), taken at an earlier commit with a 20 s hold and a resolution of 1.
