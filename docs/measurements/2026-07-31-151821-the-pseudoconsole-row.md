# The pseudoconsole row, filled but not comparable · 2026-07-31 15:18:21

[The G0 freeze](2026-07-31-150258-g0-frozen.md) left one row of the comparison set open: a pseudoconsole per session had been measured [before the ramp had a drift control](2026-07-30-120002-first-redlines.md), so it did not meet the standard the other rows do. This fills it, and finds that filling it is not the same as closing it.

## The row

| | |
|---|---|
| **Redline** | **14 sessions / WorkRate** |
| Fitted crossing | 14.1 through 5 saturated rungs, slope 0.1423 |
| Drift check | 10 sessions ran 24.71 units/s early and 24.59 after, **+0.5%** |
| Solo rate | 35.89 units/s |

**Every rung held exactly two processes per session and one conhost per session** — 1/2/1, 10/20/10, 25/50/25, 17/34/17, 13/26/13, 15/30/15. The count Decision 1 now rests on reproduces perfectly across the ladder.

## It does not compare to the pipes redline of 27

The [pipes ramp](2026-07-31-141334-the-shell-costs-teardown.md) gave 27 with an identical workload, hold, ladder cap and resolution. The only intended difference was the transport. The unintended one is larger:

| | Solo rate | Cores reaching sessions at rung 10 | Background at start |
|---|---|---|---|
| Pipes, 06:01 | 74.11 units/s | 9.9 | 20% |
| Pseudoconsole, 15:05 | 35.89 units/s | 6.9 | 24%, ending at 32% |

**A single session ran at half the rate nine hours later**, and ten sessions received seven cores where they had received ten. Both ramps are internally sound — each holds its per-core throughput flat across its own rungs and each passes its own drift check — and between them the machine is not the same machine.

So 27 and 14 are both real and their difference is not conhost. **A drift control catches a machine that changes under a ladder; nothing here catches a machine that changed between two of them**, and that is the gap this record actually found.

## Which leaves the row filled and the comparison open

The gate asks for a redline pair per reachable target, each with a drift check inside a couple of percent. **This row now has that**, so it no longer sits below the standard the others meet.

What it does not have is the thing a comparison set exists for. Taking the pty and pipes redlines against each other needs both run back to back on one machine state, and this machine has sat between 20% and 44% background all day — [it is a laptop on a vendor power plan](2026-07-31-145412-the-cores-are-not-interchangeable.md), and the load is someone else's work rather than something to schedule around.

**That comparison is not worth waiting for.** [Decision 1 was already rewritten under the attribution rule](../PLAN.md#four-core-decisions): a conhost is 8.55 MiB against a 2.39 GiB session, or 0.35%, and dropping a hundred of them returns 0.84 GiB against a budget the sessions break for other reasons. What survives is the process count, and this ramp reproduced that at 2.00 per session on every rung. A paired redline would refine a number nothing depends on.

## What it opens

**Ramps are compared across time and nothing checks that they can be.** Every cross-ramp claim in this repository — pipes against pseudoconsoles, `cmd` against `pwsh`, one duty against another — assumes the two ran on the same machine. Within a ramp that assumption is tested; between ramps it is not. The shell-control trio was sound because it ran back to back inside twenty minutes, which was luck of scheduling rather than a control.

The cheap fix is the one the drift check already uses: **a ramp's solo rung is a machine fingerprint**, and two ramps whose solo rates differ by more than the metric's own spread cannot be set against each other. Nothing computes that today, and every comparison here was read by eye.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Workload | `cpu-spin --units 1000000`, 80 MiB resident, duty 1.0 |
| Ramp | 60 s holds, ladder capped at 60, resolution 2, `--pty` |
| Background | 24% at the start, 32% by the end — above the 10% `doctor` calls quiet |
| sessionbench | 0.0.0 at commit `eb747dd`, release build |
| Defender | real-time protection on, no exclusions |
