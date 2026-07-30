# ROADMAP

- [What This Is](#what-this-is)
- [Current Priority: M0 · Attribution](#current-priority-m0--attribution)
- [Sequencing Direction](#sequencing-direction)
- [M1 · Headless Daemon](#m1--headless-daemon)
- [M2 · Harness Contract](#m2--harness-contract)
- [M3 · Resource Governor](#m3--resource-governor)
- [M4 · Audit Surface](#m4--audit-surface)
- [M5 · Engine Adapters (Exploratory)](#m5--engine-adapters-exploratory)
- [Non-Goals](#non-goals)
- [Decision Principle](#decision-principle)

**This document owns sequence: milestones, their gates, and what must be true to advance.** [README](README.md#documentation) maps the rest.

## What This Is

COGGY does not build games. It decides **how many game-building harnesses can run at once.**

So the unit here is not a feature but a gate. Every milestone ends in a number. No number, no advancing. And when a number contradicts the plan, we change the plan rather than the number — [PLAN.md](docs/PLAN.md) gets rewritten, not the gate.

There are no dates. Duration estimates exist, and an estimate is not a promise.

## Current Priority: M0 · Attribution

**This is the only work in flight.** No other crate exists.

We build `sessionbench` and nothing else. It establishes the as-is **redline** and names what limits it. The metric, the six axes, the comparison set, and the rules that keep the benchmark honest live in [sessionbench/README.md](sessionbench/README.md).

**Before the ramp, run one session to completion.** A single real generation session, logged for RSS over time, Defender's cost at startup versus during file writes, and output volume. One afternoon, one machine, no ramp harness. Multiply by 100 on paper and compare against the redline conditions: if the shape contradicts them, fix the conditions before building the instrument that applies them. Arithmetic already retired spawn time as a ceiling; this is the same move applied to what replaced it.

Done on 2026-07-30 as far as a machine without the harness can take it, and [the conditions survived](docs/measurements/2026-07-30-101141-conhost-and-defender.md#do-the-redline-conditions-survive-this) — memory has room, cores do not, and the condition that would trip is the one the metric already leans on. The ramp was then built and run, and it [brackets this machine between 25 and over 100 sessions](docs/measurements/2026-07-30-120002-first-redlines.md) depending only on how much CPU one session wants.

**That bracket has since collapsed to a formula.** A session's redline moves inversely with how much of its time it spends computing, and [the relation turned out to be derivable rather than fitted](docs/measurements/2026-07-30-154348-duty-is-derivable.md): `redline = 2ηC/d`, where `d` is that fraction, `C` is the machine's logical processors, and `η` is what sessions cost each other through the memory system. Two ramps at different duties agree on `η` to within 0.4%, and a rung predicted from it in advance landed.

So G0 no longer waits on a harness for a whole measurement. It waits for **two scalars and one rung** — a real session's duty, an `η` from any rung held past saturation, and a ladder to check the answer rather than to find it. **`C` being in the formula is the part that travels**, since the configured machine will not have sixteen logical processors and the flat `25/d` this replaces would have been wrong there in proportion.

**Gate G0:** an as-is redline, frozen, with its limiting cause named. [The steps, for whoever runs it on a machine with the engines attached](sessionbench/README.md#running-gate-g0-on-a-configured-machine) — including why the agent CLI is what you measure once, not what you ramp.

- **Pass:** a redline pair such as `84 / RSS` for every reachable target in [the comparison set](sessionbench/README.md#what-we-measure-against), each reproducible from its recorded hardware, each drawn from the fitted slope, and each carrying a drift check inside a couple of percent.
- **Fail:** any run producing a bare number, one aggregate figure plus a guess at the cause, or a count taken from a single bisection.

**That last clause was added after M0 measured its own metric.** The gate originally asked only for a pair, which was written before anyone knew what a redline costs to reproduce: seven identical runs of the same configuration returned counts spanning 12.5%, and [the ladder's search was the noisy part rather than the measuring](docs/measurements/2026-07-30-164912-redline-reproducibility.md). A G0 frozen from one bisection would hand M1 a target wrong by more than most optimizations are worth, and freezing is the one thing this gate does.

**No pass/fail threshold attaches to the redline value itself.** The four conditions define what counts as *sustained*, not a bar COGGY must clear. Whatever as-is produces is the right answer: M0 locates the ceiling rather than beating it, and M1 carries the first target COGGY has to hit.

**Attribution rule — fired 2026-07-30.** [Decision 1](docs/PLAN.md#four-core-decisions) claimed that dropping conhost was worth half the project. If RSS turned out not to be the limiting condition, or if it was but conhost was not a material share of it, that claim was false and the decision had to be rewritten before M1 began. This bound, and it has been honoured: RSS is nowhere near limiting at a hundred sessions, [conhost buys no meaningful session count at any weight](docs/measurements/2026-07-30-120002-first-redlines.md#what-dropping-conhost-is-worth-at-any-session-weight), and the decision now stands on process count instead. Pipes remain the default for a smaller and better-supported reason.

The rule did not need G0 to fire. It was written expecting the as-is redline to settle it, and the arithmetic settled it first — which is the outcome a falsifiable claim is supposed to allow.

### Reading before M0

Digest these before writing code.

1. **cmuratori/refterm** FAQ and demo videos — the measurement methodology for why Windows Terminal is slow
2. **Microsoft Docs: ConPTY / CreatePseudoConsole** — the canonical account of what can and cannot be bypassed
3. **Windows working set and commit accounting** — what RSS actually counts when 200 processes sit resident for hours

## Sequencing Direction

Three rules set the order.

1. **Measurement precedes structure.** No architecture box becomes a crate before M0 closes.
2. **Headless precedes screen.** The harness drives batches through the CLI. A UI is only needed for human audit, so it lands at M4.
3. **Off-the-shelf precedes hand-built.** Every milestone names what it consumes before what it writes.

**Dependency rule:** each milestone starts only on the **output** of the one before it. No starting the next in parallel while a gate is open. M2 is the sole exception, for the reason given in its section.

## M1 · Headless Daemon

2–3 weeks. Requires G0.

Build the `coggyd` daemon and the `coggy` CLI: pipe-first spawning, ring-buffer scrollback, session status events, socket API.

**What it consumes:**

- **wezterm/portable-pty** — ConPTY wrapping is already done. We do not write it.
- **alacritty_terminal** or **termwiz** — VT parser and grid model. Both are designed to be used as libraries.
- **Zellij** — Rust session daemon plus thin client. The prototype for the `coggyd` / `coggy-ui` boundary.
- **Ghostty / libghostty** — boundary design for extracting a core as a library. That cmux wrapped it into a product is evidence the design is right.
- **cmux socket API** — naming conventions across 40+ subcommands. [PLAN's fixed contracts](docs/PLAN.md#fixed-contracts) bind us to this vocabulary.

**Gate M1:** 100 sessions held for an hour at total RSS under 4GB, per-session work rate within 2× of solo, and no dropped output. Cold start under 30 seconds is a nice-to-have recorded in the startup log, not a gate — [it is 0.02% of the workload](docs/PLAN.md#residency-not-spawning).

**Measurement rule:** take these three numbers with **the same harness** used at M0. Build a second bench and the M0 baseline becomes incomparable.

## M2 · Harness Contract

2 weeks, running **in parallel** with the new harness build. This is the one exception to sequential ordering, because both sides have to design the contract together.

The new harness drives game-generation batches through the `coggy` CLI and socket API. No UI. The API surface is derived backward from the calls the harness actually makes. Session reattachment after daemon restart lands here too.

**Gate M2:** **100 games/day** completed unattended. Not 1000 — see [the credit ceiling](docs/PLAN.md#honest-constraint-unlocking-sessions-leaves-credits-locked).

**Remeasurement obligation:** at the same gate, measure per-game tokens and wall-clock against the new harness and update [Why this exists](docs/PLAN.md#why-this-exists) with the result. The old harness's figures are retired at this point.

**This is when it gets announced.** The repo is already public — see [the license section](docs/PLAN.md#license-gpl-30-or-later) — but nothing is posted anywhere until G-M2 passes, because announcing something that does not run earns issues rather than stars.

## M3 · Resource Governor

2 weeks.

Job Object quotas, core budget scheduler, build queueing, automated Defender exclusions.

**What it consumes:**

- **Windows Job Objects** — `CreateJobObject` plus `JOBOBJECT_CPU_RATE_CONTROL_INFORMATION` for per-session CPU and memory ceilings
- **Defender exclusion automation** — a five-minute implementation with the largest felt impact on game projects. M0 will already have measured how large this term actually is

**Gate M3:** the machine survives 100 concurrent session builds without swapping, and per-session work rate stays within 2× of solo while they run.

## M4 · Audit Surface

3 weeks. **The first screen appears here.**

`coggy-ui`: render 1–4 focused sessions and draw the other 96 as status lines. Alert ring, magnetic docking window with HWND tracking.

**What it consumes:**

- **orca** (stablyai, YC-backed) — session status UI, alerts, account switching, usage tracking. 27k+ stars since its March 2026 launch. Both a UX reference and a speed benchmark
- **Zed GPUI** or **wgpu** directly — native 60fps rendering
- **typometer / Chad Austin, "Terminal Latency on Windows"** — how to measure keystroke-to-screen latency, which only becomes meaningful once a human is watching a focused session

**Gate M4:** a human reads the state of a 100-session batch from the UI alone within 5 minutes.

## M5 · Engine Adapters (Exploratory)

Duration undecided. The gate gets rewritten after M4's measurements.

Attach the four official MCP servers from [the semantic layer reference](docs/PLAN.md#semantic-layer-reference-surveyed-2026-07) — Unreal 5.8, Unity 6, Blender, Godot AI — plus a PIE capture pane.

**Nothing is built new.** All four engines have official or quasi-official MCP, and COGGY only consumes. Godot is where we have the most room to contribute.

**Adapters wait on sequence rather than on hardware.** Unreal 5.8 is installed on the development machine with Visual Studio and MSVC beside it, which is enough to build and to measure but not a reason to start here — an adapter written before M1 has a daemon to attach to would be rewritten. What that install did buy is M0's last input: [a real engine session, measured](docs/measurements/2026-07-31-022200-an-unreal-session.md), which turned out to break a different condition than every synthetic one before it.

## Non-Goals

The seven ways to lose live in [PLAN's anti-patterns](docs/PLAN.md#anti-patterns-these-kill-the-project), with the reasoning attached to each. One list rather than three, because a rule restated in three documents is a rule that drifts in two of them.

They are ordered by the milestone at which each temptation first appears, which makes most of them look like a schedule. Only two actually expire: **UI**, released at M2, and **productizing**, after six months of internal use. The other five are permanent, and the two worth naming twice are engine control, which the vendors already own, and generation logic, which belongs to the harness.

## Decision Principle

One question decides whether to start any piece of work.

**Does this move the number on the gate that is currently open?**

If it does not, it belongs to a later milestone, and it does not happen now.
