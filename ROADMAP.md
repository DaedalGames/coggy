# ROADMAP

- [What This Is](#what-this-is)
- [Where This Stands](#where-this-stands)
- [Current Priority: M0 · Attribution](#current-priority-m0--attribution)
- [Sequencing Direction](#sequencing-direction)
- [M1 · Headless Daemon](#m1--headless-daemon)
- [M2 · Harness Contract](#m2--harness-contract)
- [M3 · Resource Governor](#m3--resource-governor)
- [M4 · Audit Surface](#m4--audit-surface)
- [M5 · Engine Surfaces (Exploratory)](#m5--engine-surfaces-exploratory)
- [Non-Goals](#non-goals)
- [Decision Principle](#decision-principle)

**This document owns sequence: milestones, their gates, and what must be true to advance.** [README](README.md#documentation) maps the rest.

## What This Is

COGGY does not build games. It decides **how many game-building harnesses can run at once.**

So the unit here is not a feature but a gate. Every milestone ends in a number. No number, no advancing. And when a number contradicts the plan, we change the plan rather than the number — [PLAN.md](docs/PLAN.md) gets rewritten, not the gate.

There are no dates. Duration estimates exist, and an estimate is not a promise.

## Where This Stands

**10 of 31**, and the bottleneck is not code.

| | done | |
|---|---|---|
| [M0 · attribution](#current-priority-m0--attribution) | **5 / 6** | gate G0 frozen |
| [M1 · daemon](#m1--headless-daemon) — what it builds | **3 / 6** | three deferred to M2 on purpose |
| M1 — its gate | **2 / 4** | both failures are the machine |
| [M2 · harness contract](#m2--harness-contract) | 0 / 5 | blocked on M1 |
| [M3 · governor](#m3--resource-governor) | 0 / 5 | |
| [M4 · audit surface](#m4--audit-surface) | 0 / 5 | |

**M0 — 5 of 6.** One session measured before the ramp; `sessionbench` built with its metric, six axes and comparison set; the duty relation derived and checked against a rung predicted in advance; G0 frozen at nine sessions with memory named as the cause; the attribution rule fired and Decision 1 re-grounded. Open: the comparison set has no daemon ramp, and its pseudoconsole row is recorded as unfilled.

**M1 builds five things and a CLI, and has three.** Pipe-first spawning, a ring-buffer scrollback bounded in both lines and bytes, and a job object per session all exist. Session status events exist as a `Status` a caller polls rather than a stream. **The socket API and the `coggy` CLI are deliberately absent** — [M2 derives the API backward from the calls a harness makes](#m2--harness-contract), so choosing verbs now invents what that milestone exists to discover, and the gate asks for none of it.

**M1's gate is measured in full and two conditions fail.** A hundred sessions hold under 4 GB with a third to spare, and dropped output is zero. Work rate returns 2.301 where the gate asks for 2, and the hour ends at twenty minutes because the machine stops dead at forty-one. **Neither failure is the daemon**: a hundred sessions wanting twenty-seven cores from sixteen is arithmetic, and the hard stop is thermal or power.

**So the bottleneck is a decision, not an implementation.** Either the gate says what it asks of the hardware it runs on, or it runs on hardware that can answer it — about nineteen logical processors, and one that survives an hour of saturation. Writing more daemon does not move either number.

## Current Priority: M0 · Attribution

**Closed on 2026-07-31.** [Gate G0 is frozen](docs/measurements/2026-07-31-150258-g0-frozen.md) and the work in flight is [M1](#m1--headless-daemon).

The heading above still says *current* because thirteen references point at its anchor and seven of them are [measurement records](docs/measurements/), which are logs rather than living documents. Renaming it would edit seven files that say what was true when they were written. The section stays; the claims in it are dated.

M0 built `sessionbench` and nothing else. It establishes the as-is **redline** and names what limits it. The metric, the six axes, the comparison set, and the rules that keep the benchmark honest live in [sessionbench/README.md](sessionbench/README.md).

**Before the ramp, run one session to completion.** A single real generation session, logged for RSS over time, Defender's cost at startup versus during file writes, and output volume. One afternoon, one machine, no ramp harness. Multiply by 100 on paper and compare against the redline conditions: if the shape contradicts them, fix the conditions before building the instrument that applies them. Arithmetic already retired spawn time as a ceiling; this is the same move applied to what replaced it.

Done on 2026-07-30 as far as a machine without the harness can take it, and [the conditions survived](docs/measurements/2026-07-30-101141-conhost-and-defender.md#do-the-redline-conditions-survive-this) — memory has room, cores do not, and the condition that would trip is the one the metric already leans on. The ramp was then built and run, and it [brackets this machine between 25 and over 100 sessions](docs/measurements/2026-07-30-120002-first-redlines.md) depending only on how much CPU one session wants.

**That bracket has since collapsed to a formula.** A session's redline moves inversely with how much of its time it spends computing, and [the relation turned out to be derivable rather than fitted](docs/measurements/2026-07-30-154348-duty-is-derivable.md): `redline = 2ηC/d`, where `d` is that fraction, `C` is the machine's logical processors, and `η` is what sessions cost each other through the memory system. Two ramps at different duties agree on `η` to within 0.4%, and a rung predicted from it in advance landed.

So G0 no longer waits on a harness for a whole measurement. It waits for **two scalars and one rung** — a real session's duty, an `η` from any rung held past saturation, and a ladder to check the answer rather than to find it. **`C` being in the formula is the part that travels**, since the configured machine will not have sixteen logical processors and the flat `25/d` this replaces would have been wrong there in proportion.

**Gate G0 — frozen 2026-07-31 at [nine sessions, limited by RSS](docs/measurements/2026-07-31-150258-g0-frozen.md).** A generation session is an agent CLI holding 0.52 GiB for its whole life plus an engine holding 1.87 GiB while it cooks, and `21.97 ÷ 2.39` is where this machine stops. Cores would not bind until 93. The pseudoconsole row of the comparison set is open and recorded as such, because the decision it would have informed was settled by arithmetic before a ramp could reach it.

**Gate G0:** an as-is redline, frozen, with its limiting cause named. [The steps, for whoever runs it on a machine with the engines attached](sessionbench/README.md#running-gate-g0-on-a-configured-machine) — including why the agent CLI is what you measure once, not what you ramp.

- **Pass:** a redline pair such as `84 / RSS` for every reachable target in [the comparison set](sessionbench/README.md#what-we-measure-against), each reproducible from its recorded hardware, each carrying a drift check inside a couple of percent, and each rigorous in the way its own limiting condition allows.
- **Fail:** any run producing a bare number, one aggregate figure plus a guess at the cause, or a count taken from one reading of whichever quantity binds.

**What rigour means depends on which condition trips**, and reading it as one thing sent this gate looking for something that does not exist. A work-rate redline is a threshold crossing on a noisy curve, so it needs [the fitted slope](docs/measurements/2026-07-30-164912-redline-reproducibility.md) — seven identical runs of one configuration spanned 12.5% without it, and the ladder's search rather than the measuring was why. **An RSS redline has no slope to fit.** Total memory is the per-session figure times the count by construction, so what a fit would buy is bought instead by repeating that figure: [five readings of a cooking session give 1.87 GiB and 6.4%](docs/measurements/2026-07-31-045604-an-error-bar-for-the-engine.md), and the ceiling follows by division.

Both still need the drift control, because a machine that changed under the ladder invalidates either one. A G0 frozen from a single reading of whichever quantity binds would hand M1 a target wrong by more than most optimizations are worth, and freezing is the one thing this gate does.

**No pass/fail threshold attaches to the redline value itself.** The four conditions define what counts as *sustained*, not a bar COGGY must clear. Whatever as-is produces is the right answer: M0 locates the ceiling rather than beating it, and M1 carries the first target COGGY has to hit.

**Attribution rule — fired 2026-07-30.** [Decision 1](docs/PLAN.md#four-core-decisions) claimed that dropping conhost was worth half the project. If RSS turned out not to be the limiting condition, or if it was but conhost was not a material share of it, that claim was false and the decision had to be rewritten before M1 began. This bound, and it has been honoured: RSS is nowhere near limiting at a hundred sessions, [conhost buys no meaningful session count at any weight](docs/measurements/2026-07-30-120002-first-redlines.md#what-dropping-conhost-is-worth-at-any-session-weight), and the decision now stands on process count instead. Pipes remain the default for a smaller and better-supported reason.

The rule did not need G0 to fire. It was written expecting the as-is redline to settle it, and the arithmetic settled it first — which is the outcome a falsifiable claim is supposed to allow.

### Reading before M0

Digest these before writing code.

1. **cmuratori/refterm** FAQ and demo videos — the measurement methodology for why Windows Terminal is slow
2. **Microsoft Docs: ConPTY / CreatePseudoConsole** — the canonical account of what can and cannot be bypassed
3. **Windows working set and commit accounting** — what RSS actually counts when 200 processes sit resident for hours

## Sequencing Direction

These rules set the order.

1. **Measurement precedes structure.** No architecture box becomes a crate before M0 closes.
2. **Headless precedes screen.** The harness drives batches through the CLI. A UI is only needed for human audit, so it lands at M4.
3. **Off-the-shelf precedes hand-built.** Every milestone names what it consumes before what it writes.

**Dependency rule:** each milestone starts only on the **output** of the one before it. No starting the next in parallel while a gate is open. M2 is the sole exception, for the reason given in its section.

## M1 · Headless Daemon

2–3 weeks. Requires G0.

Build the `coggyd` daemon and the `coggy` CLI: pipe-first spawning, ring-buffer scrollback, session status events, socket API, and **a job object per session**.

**G0 moved that last item forward from [M3](#m3--resource-governor), where it sits as a quota mechanism.** A daemon that owns sessions has to be able to end one, and killing the process it spawned does not do that: fifty wrapped sessions left [exactly fifty stragglers and a teardown 361× slower](docs/measurements/2026-07-31-141334-the-shell-costs-teardown.md), because the shell dies and its child does not. The same asymmetry appears from the other side, where [a pseudoconsole belongs to whoever created it rather than to the session it serves](docs/measurements/2026-07-30-101141-conhost-and-defender.md). Job membership is inherited downward, so terminating the job takes the tree. **Reclaiming a slot is what a session supervisor is for, and without this the daemon cannot.**

**Per session is the load-bearing word, and `sessionbench` is the counterexample rather than the model.** It creates one job, calls `assign_current_process`, and every session it spawns inherits into it — which gives it attribution, since membership answers *who belongs to this measurement*. It cannot give it termination, because terminating that job would take the benchmark with it. So its teardown kills each root and reaps whatever survives, and [the 361× it reports](docs/measurements/2026-07-31-141334-the-shell-costs-teardown.md) is a consequence of that shape rather than a fact about Windows. **One job holding everything answers a different question from one job per session**, and the daemon needs the second.

**What it consumes:**

- **wezterm/portable-pty** — ConPTY wrapping is already done. We do not write it.
- **alacritty_terminal** or **termwiz** — VT parser and grid model. Both are designed to be used as libraries. **Not needed to clear this gate**, which asks for held sessions, RSS, work rate and dropped output: a pipe session's scrollback is the ring buffer, and parsing those bytes into a grid only matters once something renders them. Pick the library here and reach for it when a consumer appears — `capture-pane` at [M2](#m2--harness-contract) if the harness calls it, otherwise [M4](#m4--audit-surface). A PTY session needs it sooner, and [PTY is opt-in](docs/PLAN.md#four-core-decisions).
- **Zellij** — Rust session daemon plus thin client. The prototype for the `coggyd` / `coggy-ui` boundary.
- **Ghostty / libghostty** — boundary design for extracting a core as a library. That cmux wrapped it into a product is evidence the design is right.
- **cmux socket API** — naming conventions across 40+ subcommands. [PLAN's fixed contracts](docs/PLAN.md#fixed-contracts) bind us to this vocabulary. **Take the shape, not the surface.** [M2 derives the API backward from the calls the harness actually makes](#m2--harness-contract), and [Decision 4](docs/PLAN.md#four-core-decisions) is that the two are designed together — so choosing verbs here invents the thing that milestone exists to discover. What M1 owes is a transport and a way to reach the pool behind it; the vocabulary arrives with a caller. The gate agrees: it asks for sessions held, RSS, work rate and dropped output, and a socket appears in none of them.

**Gate M1:** 100 sessions held for an hour at total RSS under 4GB, per-session work rate within 2× of solo, and no dropped output. Cold start under 30 seconds is a nice-to-have recorded in the startup log, not a gate — [it is 0.02% of the workload](docs/PLAN.md#residency-not-spawning).

**The work-rate condition is decided by the workload's duty before the daemon does anything**, which the sentence above does not say and a run costs an hour to discover. A session wanting `d` cores means a hundred want `100d`; past what the machine has, each gets `C/100` instead and the slowdown is `100d/C`. On sixteen logical processors, within 2× needs `d ≤ 0.32`. So a hundred CPU-bound sessions cannot pass on this machine and no daemon could make them — while [the measured duty of a real driven agent turn, 0.27](docs/measurements/2026-07-31-054657-the-driven-duty.md), predicts 1.69× and passes with little to spare. The gate is meaningful at the duty a session actually has and arithmetic at any other, so a run that does not state its duty has not stated its result.

**1.69× is the floor rather than the estimate**, since it assumes sessions cost each other nothing. **Measured at that duty, the condition breaks: [a hundred sessions come back at 2.301×](docs/measurements/2026-08-01-213912-the-gate-breaks-at-its-own-duty.md)**, 15% past the 2 the gate asks for, while RSS holds with a third of the budget spare and every session runs to the last report. So **gate M1 is not passable on this machine at the duty it is meaningful at**, and no daemon can change that — a hundred sessions wanting 27 cores from sixteen is arithmetic before it is engineering.

**The gate is not mis-specified, it is above this machine.** The condition needs `N·d ÷ (η·C) ≤ 2`, so at a hundred sessions and the measured duty it wants **about nineteen logical processors** where this box has sixteen — a floor, since `η` on a wider machine is unmeasured and more cores can mean more claimants on one memory system. And the same run measures the core ceiling directly rather than projecting it: a hundred sessions at 2.301 puts the crossing at **87**, against the 93 that `2ηC/d` gave. Both sit far above [the nine a real generation session allows](docs/measurements/2026-07-31-150258-g0-frozen.md), so neither moves G0.

**And what the contention term follows is now measured: the product `N·d`, the number of sessions awake at any instant.** A hundred sessions at duty 0.27 and a hundred and thirty-five at 0.20 leave twenty-seven awake either way, and [their total throughputs agree to within 1.5% while the session count differs by 35% and the duty by 26%](docs/measurements/2026-08-02-121359-eta-follows-the-awake-count.md). **That is the quantity [M3's governor](#m3--resource-governor) admits against** — a hundred quiet sessions and twenty-seven busy ones are the same load, and one number is enough to decide.

Two more things follow. The contention term is **not the constant `2ηC/d` treats it as** — 0.733 at duty 0.27 against about 0.8 at 0.172, falling as more sessions are awake at once, so a value fitted at one duty predicts high at a higher one and that is the unsafe direction for an admission ceiling. And **stating the duty is not enough: a run has to measure the duty it actually held.** A fixed wait calibrated on a warm machine gave 0.172 where 0.271 was asked for, because it turns a 79% difference in compute speed into 13.6% of work rate — which is why the gate's script uses `--duty`, [the flag that delivers a stated duty on any machine](docs/measurements/2026-08-01-210316-the-wait-mechanism-cancels.md).

**This gate measures the daemon, not capacity, and G0 is why that has to be said.** 4GB across 100 sessions is 40 MiB each; [a real generation session holds 2.39 GiB](docs/measurements/2026-07-31-150258-g0-frozen.md), which is sixty times more and would put a hundred of them at 239GB. The figure is reachable because the sessions here are the daemon's own bookkeeping plus a shell on pipes, which [measures 8.97 MiB](docs/measurements/2026-07-31-035111-between-builds.md) — so it asks whether `coggyd` disappears under load, and answers nothing about how many generation sessions fit. **That number is nine**, it is set by memory the daemon does not own, and admission against it belongs to [M3](#m3--resource-governor). Reading one as the other is [the conflation that cost M0 a day](docs/measurements/2026-07-31-050322-the-agent-side-of-a-session.md).

**Measurement rule:** take these three numbers with **the same harness** used at M0. Build a second bench and the M0 baseline becomes incomparable.

That rule already bit once. [A hundred sessions were held for an hour on 2026-08-01](docs/measurements/2026-08-01-103225-an-hour-of-a-hundred-sessions.md) with `Get-Process` and a PowerShell sampler, which is the second bench — the figures were sound and could not clear the gate. `sessionbench hold` now takes the same reading through the same job, sampler and artifact shape.

**All three can be asked, and two of them only after the question was put differently.** Work rate needed a second run rather than a different daemon, and [`hold --with-solo` is that run](sessionbench/README.md#running-it) — a solo baseline on either side of the concurrent hold, refusing to produce a ratio when the two disagree by more than the machine makes while being itself.

**Dropped output was called unreachable for a year of afternoons and was not.** The reasoning was that the condition is found by watching ordinals in a session's own stream, and under `coggyd` that stream ends in the daemon's scrollback, so nothing outside holds what the workload emitted. True, and the wrong question: **a pipe blocks rather than dropping**, so between a session's `write` and the scrollback there is nothing that can lose a line, and the only loss available is the daemon's own reader giving up. It [used to give up on the same branch as a clean end-of-file](coggyd/src/lib.rs), which is what made the one observable failure unobservable. The daemon counts it now and reports `failed_reads`; zero answers the condition. The words stay distinct — `out_of_reach` still means no route exists, and **replacement still has none**, since nothing in the daemon restarts a session that exited.

**Replacement is not one of these three**, though it is easy to count as one: it is [the redline's fourth condition](sessionbench/README.md#redline) and the daemon has no counterpart for it either. A ramp under the daemon therefore loses two of four where a hold loses one of three, and the coincidence of small numbers is what makes the two lists blur.

**So all three conditions are measured, and the gate does not pass.** RSS holds with a third of the budget spare and dropped output is zero; work rate returns 2.301 against a 2 that needs about nineteen cores, and the hour is unreachable because the box stops dead at forty-one minutes of this load. **Both failures are the machine rather than the daemon**, so M1 is not waiting on code — it is waiting on a decision about what a gate should ask of the hardware it runs on.

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

- **Windows Job Objects** — `JOBOBJECT_CPU_RATE_CONTROL_INFORMATION` for per-session CPU and memory ceilings. The job itself lands in [M1](#m1--headless-daemon), where it is what lets the daemon end a session at all; this milestone adds the quotas to a thing already there.
- **Defender exclusion automation** — a five-minute implementation, and M0 has now measured the term: [0.034 cores a session against a cooking workload](docs/measurements/2026-07-31-041923-what-an-exclusion-buys-a-cook.md), real and small. It returns CPU, and [the ceiling it would have to move is made of memory](docs/measurements/2026-07-31-045604-an-error-bar-for-the-engine.md), so it buys comfort rather than sessions. Still worth five minutes; no longer worth sequencing around.

**Gate M3:** the governor admits sessions up to the measured ceiling, refuses past it, and the machine does not swap while they run — with per-session work rate staying within 2× of solo.

**This replaces "100 concurrent session builds", which G0 made impossible twice over.** [Builds serialise per engine installation](docs/measurements/2026-07-31-034150-unreal-builds-serialise.md) through a lock that cannot be bypassed, so a hundred at once would need a hundred installations; and [memory binds at nine](docs/measurements/2026-07-31-150258-g0-frozen.md) long before that. A gate no machine can pass grades nothing. **The governor's job is admission against a number it measures, not survival of a number someone chose** — which is also why the ceiling belongs in the daemon as a reading rather than a constant.

## M4 · Audit Surface

3 weeks. **The first screen appears here.**

`coggy-ui`: render 1–4 focused sessions and draw the other 96 as status lines. Alert ring, magnetic docking window with HWND tracking.

**What it consumes:**

- **orca** (stablyai, YC-backed) — session status UI, alerts, account switching, usage tracking. 27k+ stars since its March 2026 launch. Both a UX reference and a speed benchmark
- **Zed GPUI** or **wgpu** directly — native 60fps rendering
- **typometer / Chad Austin, "Terminal Latency on Windows"** — how to measure keystroke-to-screen latency, which only becomes meaningful once a human is watching a focused session

**Gate M4:** a human reads the state of a 100-session batch from the UI alone within 5 minutes.

## M5 · Engine Surfaces (Exploratory)

Duration undecided. The gate gets rewritten after M4's measurements.

**Engines reach COGGY twice, and only one of those is this milestone.**

As **workloads**, they are what a session turns out to cost, and that measurement belongs to M0. [An Unreal build holds 1.69 GiB where a synthetic session holds twenty megabytes](docs/measurements/2026-07-31-022200-an-unreal-session.md), which reversed which condition binds, and [one engine installation compiles one thing at a time](docs/measurements/2026-07-31-034150-unreal-builds-serialise.md), which caps concurrent building at one however much machine is left. Each engine contributes numbers to the governor's admission rule and nothing else. Measuring all four in sequence buys less than measuring the two extremes: Godot's editor and a Blender render span two orders of magnitude, and Unreal and Unity sit between them. **Both middle cases are installed on the development machine** — Unreal 5.8 and Unity 6000.4.7f1 — so a second engine is measurable now rather than at M5, and the extremes are the two that are not.

As **MCP servers**, they are what the harness drives, and COGGY consumes rather than writes. Attach the four official servers from [the semantic layer reference](docs/PLAN.md#semantic-layer-reference-surveyed-2026-07) — Unreal 5.8, Unity 6, Blender, Godot AI — plus a PIE capture pane. Godot is where we have the most room to contribute.

**There is no per-engine adaptation to sequence.** The daemon spawns a process, drains its output and governs its resources; nothing in it knows an engine exists. A milestone that adapted to Unreal and then to Unity would be building [engine control, which the vendors already own](docs/PLAN.md#anti-patterns-these-kill-the-project), and that is a permanent non-goal. Serialised builds are the same story: **Unreal Build Accelerator** and `UbaCoordinatorHorde` ship inside the engine, so distributing build actions is something to consume when M3 needs it.

## Non-Goals

The seven ways to lose live in [PLAN's anti-patterns](docs/PLAN.md#anti-patterns-these-kill-the-project), with the reasoning attached to each. One list rather than three, because a rule restated in three documents is a rule that drifts in two of them.

They are ordered by the milestone at which each temptation first appears, which makes most of them look like a schedule. Only two actually expire: **UI**, released at M2, and **productizing**, after six months of internal use. The other five are permanent, and the two worth naming twice are engine control, which the vendors already own, and generation logic, which belongs to the harness.

## Decision Principle

One question decides whether to start any piece of work.

**Does this move the number on the gate that is currently open?**

If it does not, it belongs to a later milestone, and it does not happen now.
