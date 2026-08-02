# Three minutes of a hundred sessions halve this box, and it stays halved

> **The cause in this title did not reproduce.** A deliberate burst of the same shape left the box in the fast state — 21.45 units/s before and 21.43 after — so what stands is two steady levels 2.2× apart and not the mechanism named for them. See the section at the foot of this record.

A solo session at `--resident 33 --duty 0.27` ran at **20.0 units/s** at 23:17. Every solo since has run at **9.4**, including the ones inside [the gate bracket twelve minutes later](2026-08-03-000430-the-footprint-lever-runs-backwards.md) and a fresh pair ninety minutes after that.

Between them: a three-minute hold of a hundred saturating sessions.

## The readings

| when | what | units/s |
|---|---|---|
| 23:17 | `observe`, one session, 25 s | **20.04** |
| 23:17 | `observe`, one session at `--resident 20`, 25 s | 19.92 |
| — | **a hundred sessions, three minutes** | — |
| ~23:30 | bracket solo ×3, 120 s each | 9.433, 9.408, 9.407 |
| ~23:50 | bracket solo ×3, 120 s each | 9.372, 9.332, 9.308 |
| +90 min | `observe`, one session, 60 s | **9.69** |
| +90 min | `hold --sessions 1` under `coggyd`, 60 s | 9.78 |

**Both observations are internally flat.** Per-sample rates before the burst run 17.8–21.8 across 24 samples; after, 8.8–9.4 across 11. There is no transient, no warm-up shoulder, and no trend inside either — two steady states 2.2× apart.

## What it is not

- **Not the daemon.** `observe` spawns the session itself and `hold --daemon` puts it under `coggyd`; taken minutes apart they agree to **0.9%** (9.69 against 9.78). The harness path costs nothing measurable here.
- **Not the footprint.** 20 MiB and 33 MiB solos before the burst gave 19.92 and 20.04 — **0.6% apart**. A session's held memory does not change what a unit costs when it runs alone.
- **Not the window.** 25 s and 60 s observations are each flat across their own samples.
- **Not the plug.** On mains at 100% throughout, `SAMSUNG MODE`.

## What it threatens

[Last night's footprint result](2026-08-03-000430-the-footprint-lever-runs-backwards.md) compared a 20 MiB hold from 08-01 against a 33 MiB hold taken *after* this burst. Its solo rungs were 21.484 and 9.377 — and that 2.29× was read as the footprint moving the baseline. **The footprint does not move the baseline; 0.6% says so.** The burst does.

So the two holds sat in different machine states, and `η` falling from 0.733 to 0.518 may be the state rather than the weight. A slowdown is a within-run ratio and cancels a uniform slowdown, but nothing here establishes that contention scales the same way in both states.

**The controlled test is available only while this state lasts**: a hundred sessions at `--resident 20`, bracketed, taken now. If its slowdown comes back near 3.2 the footprint result was the machine; near 2.3 and it was the weight.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical · 31 GiB · Windows 11 (26200) |
| Power | on mains, SAMSUNG MODE, 100%, throughout |
| Background | 24% at the last pair |
| Harness | `sessionbench` 0.0.0 at `921af83`, release |
| Artifacts | `bench-out/1785679068-rss-20-pipe`, `1785679095-rss-33-pipe`, `1785683930-pathA-observe-pipe`, `1785680927-eta-at-33-daemon` |

## 2026-08-03, forty minutes later: the burst does not reproduce it

This record's title asserts a cause. **A deliberate attempt to induce the state failed.**

A solo hold, then the same three-minute hold of a hundred sessions at `--resident 20`, then a second solo hold — with the ACPI thermal zone and `% Processor Performance` sampled through both solos:

| | before the burst | after |
|---|---|---|
| solo rate | 21.45 units/s | **21.43** |
| thermal zone | 39.1 °C | 39.1 |
| `% Processor Performance` | 173.1 | 173.3 |
| frequency | 1591 MHz | 1595 |

**Nothing moved.** The box stayed in the fast state through a burst matching the one this record blamed, so three minutes of a hundred sessions is not sufficient.

**What survives is the state, not the cause.** Two steady levels 2.2× apart, each flat across its own samples, are measured and not in question — 20.04 units/s before, 9.4 for the following ninety minutes, and 21.45 tonight. What this record cannot support is *why*, and the title says otherwise.

**And the counters could not be tested.** They were sampled to see whether either names the state; with both phases in the same state they agree to 0.1%, which says nothing either way. That question is still open and now needs the slow state to arrive on its own.

The candidates left are the ones timing alone cannot separate: cumulative load across the evening rather than one burst, a scheduled background task, or something outside the box entirely. **Timing put the burst in the gap and the burst is what got named** — the same shape as reading a knob's name instead of measuring its effect, one pass after that was written down.
