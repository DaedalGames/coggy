# What the harness says about itself · 2026-08-01 16:39:35

The instrument grew a way to drive `coggyd` today, and the figures that came out of building it were sitting in task notes rather than anywhere a next clone can read. **A number used to justify a design decision is a claim**, and three of these were.

None of this is the gate. [The gate's hour](2026-08-01-103225-an-hour-of-a-hundred-sessions.md) waits on a machine steadier than this one has been all day.

## The tool runs at the count the gate asks for

`sessionbench hold --sessions 100 --duration 30 -- ping -n 600 127.0.0.1`:

| | |
|---|---|
| Fewest sessions alive at any sample | **100** |
| Peak total RSS | **581.6 MiB** — 15% of the 4 GB the gate allows |
| Survivors after a graceful stop | 0 |

**The arithmetic checks the denominator from the other side.** 3,234 lines over 100 sessions in 30 seconds is 1.04 a session a second, and `ping` emits one a second. The property that makes it useless for measuring work rate — [it is a clock](../../sessionbench/README.md#running-it) — is what makes it right for asking whether the count and the window are what they claim.

## Two solo holds ten minutes apart differ by more than a concurrent run does

Three twenty-second solo holds of `cpu-spin`, then three more ten minutes later:

| | units/s/session | spread within |
|---|---|---|
| first triple | 585, 590, 606 | 3.5% |
| second triple | 644, 653, 635 | 2.8% |
| **between the means** | 593.7 → 644.0 | **+8.5%** |

**Drift between runs is three times the noise inside one.** That is the whole argument for a work-rate baseline that brackets its concurrent hold rather than sitting before it: a solo pass taken ten minutes earlier carries 8.5% into a ratio, where the effect being measured is a slowdown of maybe 20%.

It is also why [`compare`'s five-percent allowance](2026-07-31-171719-what-a-baseline-is-worth.md) transfers to holds. Five sits above what one sitting produces and well below what ten minutes does.

## A rate is steadier than a count, and only slightly

Across one triple, the raw counts spread **2.80%** and the per-second rates **2.44%**.

Units run to end-of-file and the elapsed figure is taken after the stop, so both span the same window including teardown — and teardown time varies. Dividing cancels it. **The improvement is real and small**, which says most of what is left is the machine rather than the window, and that a longer hold buys more than a better estimator would.

## What this does not claim

- **`ping` is not a session and `cpu-spin` is not a generation session.** The first holds 5.8 MiB and does nothing; the second saturates a core and holds 20 MiB. [A real one holds 2.39 GiB](2026-07-31-150258-g0-frozen.md).
- **Nothing here is a gate figure.** The hold above ran thirty seconds against the gate's hour, and its RSS is a hundred `ping`s rather than a hundred sessions.
- **Every reading was taken at 30–40% background**, deliberately. RSS and process counts barely notice it; the solo triples are ratios against each other, which is a weaker condition than quiet but not the same as no condition — the 3.5% and 2.8% spreads carry that.
- **The 8.5% is one pair of triples.** It says the gap between runs can exceed the noise within one, which is enough to decide the bracketing question, and it does not say what that gap usually is.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 30–40% throughout, spread across consecutive readings 25–44% |
| Harness | `sessionbench hold` at commits `0626e0d` through `35a88b0` |
| Daemon | `coggyd` 0.0.0, release build |
| Workloads | `ping -n 600 127.0.0.1` for the count check; `cpu-spin --units 10000 --duty 1.0 --resident 20` for the rates |
