# The gate breaks at its own duty, and η is not a constant

[Gate M1's work-rate condition is decided by the workload's duty](../../ROADMAP.md#m1--headless-daemon) before the daemon does anything. Run at **the duty the gate is meaningful at — 0.27, [a real driven agent turn](2026-07-31-054657-the-driven-duty.md)** — a hundred sessions come back at **2.30× solo, against a condition of 2×.**

It is not close and it is not the daemon's doing. A hundred sessions wanting 0.27 of a core apiece demand 27 from sixteen; no supervisor can return that ratio. What the run adds to the arithmetic is how much worse than arithmetic it is.

## The measurement

| | |
|---|---|
| solo, six holds | 21.484 units/s/session |
| concurrent, 100 sessions | 9.336 |
| **slowdown** | **2.301** |
| Total RSS | 2.393 GiB of 3.73 — held |
| Fewest running | 100 |
| `evicted = read − 100 × 2,000` | 922,262 against 922,262 — exact |

**`η` falls out at 0.733.** The relation puts the core-limited floor at `N·d/C = 100 × 0.27 ÷ 16 = 1.6875`, and 1.6875 ÷ 2.301 is what sessions cost each other.

## η degrades as instantaneous concurrency rises

Three measurements today, same machine, same evening, same hundred sessions, differing only in duty:

| duty | slowdown | floor `N·d/C` | η |
|---|---|---|---|
| 0.172 (`--wait-ms 67`) | 1.257 | 1.075 | 0.855 |
| 0.172 (`--duty`) | 1.354 | 1.075 | 0.794 |
| **0.27 (`--duty`)** | **2.301** | 1.688 | **0.733** |

Duty is the fraction of the time a session spends computing, so `N·d` is how many are awake at any instant — 17 at 0.172 and 27 at 0.27. **The more of them are awake together, the more each costs the others.** That is what `η` was always described as — [memory contention between sessions rather than scheduling](2026-07-30-154348-duty-is-derivable.md) — and a memory system does get worse with more simultaneous claimants.

The consequence for the relation is direct: `redline = 2ηC/d` treats `η` as a constant, and a constant fitted at one duty reads high at a higher one. **Predicting across duties is therefore optimistic**, which is the unsafe direction for an admission ceiling.

The two duty-0.172 points differ by 7.7% and sit either side of 0.8; they were taken forty minutes apart with solo fingerprints 13% apart, so [that pair is not comparable](2026-08-01-210316-the-wait-mechanism-cancels.md) and only their neighbourhood is evidence. The 0.27 point stands on its own six baselines.

## What refused the run first, and why the refusal was wrong

The run came back `not_taken`: *the before baseline's own holds spread 8.5%, past the 5% it would judge with.* That guard was added the same afternoon and this was its first real firing.

It was wrong, and its own numbers say so:

```
before  21.825, 20.030, 21.643      range 8.48%,  mean 21.166 ± 2.9%
after   21.795, 21.845, 21.769      range 0.35%,  mean 21.803 ± 0.1%
probes  22.121, 21.918
```

**One hold of nine came back 8% low and the other eight sat inside 1.4%.** A single slow placement is exactly what the repeats exist to expose, and exposing it is not a reason to discard the run.

The defect is a category error: **the guard compared a range against an allowance written for a difference of means.** A range of three samples is roughly `1.7σ`; the mean's standard error is `σ/√3`. Comparing the first against a 5% allowance demands every hold agree with every other hold — a condition nobody wrote down and no baseline meets. The two sides' means agree to 3.0%, and the before side's mean is pinned to 2.9%, so the judgement was supported all along.

The guard now compares the **standard error of each side's mean** against the allowance, and the range is kept as a diagnostic beside it. [The run's own nine numbers are the regression test](../../sessionbench/src/daemon.rs).

**The slowdown here is arithmetic over recorded rates, not a second run.** Every hold's rate is in `bracket.json`; only the verdict over them changed. Re-measuring would have cost thirty-four minutes to re-derive numbers the artifact already holds.

## What this run cannot say

- **It does not fail the daemon.** [The gate measures whether `coggyd` disappears under load](../../ROADMAP.md#m1--headless-daemon), and it does: RSS holds with 36% of the budget spare, a hundred sessions run to the last report, and the eviction counters are exact. What breaks is a ratio the machine's core count decides.
- **One machine, one count, twenty minutes.** Whether `η` keeps falling past 27 awake sessions is unmeasured, and three points across two duties is a trend rather than a curve.
- **The two duty-0.172 points are not a controlled pair with this one.** They bracket 0.8 and this sits at 0.733; that is a direction, and the spacing is not calibrated.
- **The 20-minute duration is unchanged from [the earlier run's reason](2026-08-01-202112-gate-m1-at-twenty-minutes.md)** — the same load stopped this machine dead at forty-one minutes — so nothing here speaks to drift over an hour.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 1.92 of 16 logical before the run |
| Harness | `sessionbench hold --with-solo --solo-repeats 3` at commit `219bd95`, release build, driven detached by `scripts/m1-hour.ps1` |
| Daemon | `coggyd` 0.0.0, release build |
| Workload | `cpu-spin --units 100000000 --duty 0.27 --resident 20` |
| Shape | 3 × 120 s solo · 100 sessions for 1200 s · 3 × 120 s solo, sampled every 5 s |
| Verdict | recomputed from the recorded rates after the guard was corrected; the rates themselves are as measured |

## 2026-08-02: what the run says about the core ceiling, and about the machine the gate needs

Two things fall out of 2.301 without another run.

**The core redline at this duty is at least 87 sessions.** A redline is where the
slowdown reaches 2, and past saturation each session's share is `C/N`, so the
slowdown is linear in `N`. A hundred sessions returning 2.301 puts the crossing
at `100 × 2 ÷ 2.301 = 86.9` — **measured at the duty in question rather than
projected through a fitted constant.** The figure carried until now, 93, came
from `2ηC/d` with `η` fitted across ramps at other duties.

It is a floor rather than a replacement. Going from 100 sessions to 87 at the
same duty takes the awake count from 27 to 23.5, and this run's other finding is
that `η` improves as fewer are awake together — so the true crossing sits
somewhere in **87 to 93**, the lower end measured and the upper end projected.
Both are far above [the nine sessions memory allows a real generation
session](2026-07-31-150258-g0-frozen.md), so neither moves G0.

**And the gate is achievable, on a machine about 15% wider than this one.** The
condition needs `N·d ÷ (η·C) ≤ 2`, so `C ≥ 100 × 0.27 ÷ (2 × 0.733) = 18.4`
logical processors. Sixteen is what this box has. Gate M1 is therefore not
mis-specified — a nineteen-core machine clears it at the same duty — and the
figure is a floor for the same reason as above: `η` on a wider machine is
unmeasured, and more cores can mean more claimants on the same memory system.
