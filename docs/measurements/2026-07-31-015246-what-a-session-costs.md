# What an agent session actually costs · 2026-07-31 01:52:46

Every redline this project has produced is `2ηC/d`, and `d` — the cores a session demands — had never been measured against anything but a synthetic workload built to demand exactly one. This is the first reading from the kind of session COGGY is for.

## The measurement

A session doing the tool work an agent turn does in this repository: touch a source file, `cargo build --release`, repeat. Held for 120 seconds so the sampler's thirty-second spin-up still leaves a steady state, and counted through the job object, so the machine's own background never enters the figure.

| | |
|---|---|
| **Cores demanded** | **2.63 session + 0.22 Defender** |
| Builds completed | 20 in 120 s — 6.0 s each |
| Steady RSS | 323.57 MiB, peaking at 406.79 |
| Processes | 4 |

Six seconds an incremental build matches [the 5.4 s recorded when the developer loop was tuned](2026-07-31-001314-instrument-corrections.md), which is the cross-check that this is the normal path rather than a cold one.

## `d` is not bounded by one, and that changes the arithmetic

The synthetic workload is single-threaded, so its `d` ran from 0.25 to 1.0 and the relation was only ever exercised there. A real session runs `cargo build` with sixteen cores available and takes 2.63 of them.

**At `d` above 1 the redline falls below the constant.** `24.6/d` gives **9 sessions** for a session that builds without pause, against 24 for one that spins on a single core.

Projecting this session to a hundred, which is what [PLAN's arithmetic asks for](../PLAN.md#why-this-exists):

| | Needed at 100 sessions | Available | |
|---|---|---|---|
| Cores | **286** | 16 | breaks |
| RSS | **31.60 GiB** | 21.97 GiB budget | breaks |

Both conditions, not one.

## What a real turn does to it

A session does not build continuously. It waits on a model, builds, waits again, and `d` is the demand averaged over the whole turn:

    d_session = 2.63 × build / (build + wait)

The first term is measured. The second is a property of the model and the task, and this machine cannot supply it:

| Wait between 6 s builds | `d` | Redline |
|---|---|---|
| 10 s | 0.99 | 25 |
| 30 s | 0.44 | 56 |
| 60 s | 0.24 | 103 |
| 120 s | 0.13 | 195 |

**So the answer swings by an order of magnitude on a number nobody has measured**, and it is the ratio that matters rather than either duration alone.

## Why this points down rather than up for game generation

The table above is built on a six-second build. **A game engine's build is minutes.** At 120 s of build against 30 s of wait, the build is four fifths of the turn: `d ≈ 2.1`, and the redline is **about 12**.

That is the shape of the sessions COGGY exists to hold, and it is the shape this machine cannot supply — which is what [M5](../../ROADMAP.md#m5--engine-adapters-exploratory) waits on, now for a reason with a number attached rather than a category.

**The three residency costs M0 measured were all cheap, and this is the one that is not.** Memory, Defender and output each turned out to have orders of headroom; cores do not, and a build-heavy session is where that bites.

## What this does not settle

- **`cargo` is not an engine.** A Rust build and a Unity build differ in parallelism, in how much they touch the disk, and in how much of the wall clock is linking. The 2.63 transfers as a shape, not as a number.
- **The machine was not quiet.** Background load ran 2.29 to 3.96 cores across these runs, a seventh to a quarter of the box. The job object keeps it out of the attribution but not out of the contention, so 2.63 is what the session got rather than what it would take on an idle machine. `doctor` now reports this before a run rather than after.
- **One session, one repository.** No ramp, no repeat, and none of [the error bar the redline carries](2026-07-30-164912-redline-reproducibility.md).

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| sessionbench | 0.0.0 at commit `f227213`, release build |
| Session | `cargo build --release` in a loop, each pass preceded by touching `sessionbench/src/lib.rs` |
| Hold | 120 s, first 30 s unmeasured · sampled every 1 s |
| Defender | real-time protection on, no exclusions |
