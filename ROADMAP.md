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

**10 of 31**, and the bottleneck is not code — M1's work-rate condition is 3–4% short at every session weight, the sizing that follows is half a core, and the hour has never been attempted at the duty the gate is stated in.

**The third obstacle is the measuring window itself, and it is newly measured rather than newly true.** A run whose verdict is a ratio needs a machine that is both quiet and rested, and those are different things: this box spends hours [quiet and running at half speed](docs/measurements/2026-08-03-094550-the-slow-state-caught-on-a-quiet-machine.md), and hours quiet-looking while a third party takes eleven to thirteen cores. One session at the gate's own workload runs about 18.9 units/s rested, 13.8 under a tenant and 9.1 in the slow state — **so a crowded rested box outruns a quiet slow one and no single reading names the state.** Every hold now prints the cores held outside the job beside its rate, which is what makes the pair readable; `doctor` answers only the first half and answered it wrongly in both directions on the day this was found.

| | done | |
|---|---|---|
| [M0 · attribution](#current-priority-m0--attribution) | **5 / 6** | gate G0 frozen |
| [M1 · daemon](#m1--headless-daemon) — what it builds | **3 / 6** | three deferred to M2 on purpose |
| M1 — its gate | **2 / 4** | work rate 3–4% short at every weight; the hour untried at this duty |
| [M2 · harness contract](#m2--harness-contract) | 0 / 5 | blocked on M1 |
| [M3 · governor](#m3--resource-governor) | 0 / 5 | |
| [M4 · audit surface](#m4--audit-surface) | 0 / 5 | |

[M5](#m5--engine-surfaces-exploratory) is not in the denominator and has no row: it enumerates nothing yet, because its gate gets rewritten after M4 measures. Everything below M1 is a plan rather than a count of work done.

**M0 — 5 of 6.** One session measured before the ramp; `sessionbench` built with its metric, six axes and comparison set; the duty relation derived and checked against a rung predicted in advance; G0 frozen at nine sessions with memory named as the cause; the attribution rule fired and Decision 1 re-grounded. Open: the comparison set has no daemon ramp, and its pseudoconsole row is recorded as unfilled.

**M1 builds five things and a CLI, and has three.** Pipe-first spawning, a ring-buffer scrollback bounded in both lines and bytes, and a job object per session all exist. Session status events exist as a `Status` a caller polls rather than a stream. **The socket API and the `coggy` CLI are deliberately absent** — [M2 derives the API backward from the calls a harness makes](#m2--harness-contract), so choosing verbs now invents what that milestone exists to discover, and the gate asks for none of it.

**Before any of it, ask whether the box is fit to measure on — a bracket's own baseline spread answers it and needs no remembered figure.** Healthy runs here spread **0.42%, 0.39% and 0.52%** across six solo holds; on 2026-08-10 the same binary and workload spread 5 to 37% for about **ten hours**, on an idle machine, against a previous longest instance of 340 minutes. A hold now prints the reading beside the number. **The work-rate condition is a ratio of two solo means and is unmeasurable while that state holds**, which is a fact about this hardware rather than about the daemon, and the check is what says when it ends. [The evening that mistook it for the instrument's noise floor](docs/measurements/2026-08-03-173452-the-slow-state-flatters-the-gate.md) is worth reading before deriving anything from a spread.

**M1's gate is measured in full: two conditions pass, one fails, one is untried.** Dropped output is zero and RSS holds. Work rate returns **2.0654** where the gate asks for 2, and the hour ends at twenty minutes because the machine stopped dead at forty-one — [on a rising load, which is not the one the gate is stated in](docs/measurements/2026-08-01-202112-gate-m1-at-twenty-minutes.md): that run held a fixed wall-clock wait, so its duty climbed from 0.172 toward 0.271 as the box slowed. **At a fixed 0.27 nothing has been held past twenty minutes**, so the third condition is untried rather than tried and failed. The 2.301 quoted until now was one hold that [lost the machine to something else for 18% of its samples](docs/measurements/2026-08-03-024222-the-footprint-never-mattered.md); on the intervals it held, the same run gives 2.0654, and a later hold of the same shape returned a rate 0.9% from that. **Neither failure is the daemon**: a hundred sessions wanting twenty-seven cores from sixteen is arithmetic, and the hard stop is thermal or power. **And RSS holding is a level rather than a slope**, which is what the hour would otherwise be needed to establish: [a twenty-minute hold counting 99.7% of its window peaks at 2.372 GiB against the 2.36 that two and five minutes give](docs/measurements/2026-08-03-173452-the-slow-state-flatters-the-gate.md) — 0.4% across ten times the exposure, so nothing accumulates per unit time and the memory condition carries to sixty minutes by argument. It is also the one condition a crowded box can still answer, since RSS is absolute where work rate is a ratio. **And the level has a mechanism rather than an extrapolation**: both holds retain exactly 200,000 lines — the scrollback cap at 2,000 a session times a hundred — while the work behind them differs by 134%, so once the buffer fills, output evicts rather than allocates and memory tracks the cap instead of the clock. What would break it is a higher line cap, more sessions, or longer lines, none of which is the hour. **And the margin is far thinner at the weights the budget permits than a 20 MiB run implies**: 33 and 36 MiB a session peak at 3.651 and 3.944 GiB against 20 MiB's 2.37, so 36 MiB leaves 1.4% of headroom rather than 40%. Dropped output holds across a five-fold span of line rates, zero failed reads from 217 to 1,055 units/s, so neither condition rests on the starved machine today's runs were taken on. **And a solo hold is worth about 0.42%** when this box is fit to measure on, which is the spread behind the gate's own baselines and leaves the 5% allowance twelve times the margin it needs. The 6% this said, pooled over thirteen holds on 2026-08-10, was [a sick machine read as the instrument's noise floor](docs/measurements/2026-08-03-173452-the-slow-state-flatters-the-gate.md) — and a wide spread names *unfit to measure on*, not the slow state, since one bracket ran at 9.4 units/s and spread 0.28%.

**The session's weight is not the lever it looked like.** [Read per interval rather than as a mean](docs/measurements/2026-08-03-024222-the-footprint-never-mattered.md), the 20 MiB hold lost the machine in 18% of its samples — real, since its output fell with them — and that is most of what looked like a footprint effect. Compared only where all three runs held the machine, their slowdowns are 2.065, 2.057 and 2.089: **the same within 1.5%, and every one 3–4% above the condition of 2.** So the session weight is not a lever, the memory budget never becomes the constraint, and whether *4GB* is decimal or binary buys nothing.

**The larger term is the machine's own state.** The same hundred sessions at 20 MiB return **2.0654** on a rested box and **3.958** on one that has recently been saturated — [72%](docs/measurements/2026-08-03-003443-the-footprint-result-was-the-machine.md), from [a slow state that arrives on its own and lasts about an hour](docs/measurements/2026-08-03-004512-a-saturating-burst-halves-the-box-for-an-hour.md), whose cause is not established — a deliberate saturating burst did not induce it. M1's verdict depends on that more than on anything the daemon or the workload does. **And the state is two states, which a solo hold cannot tell apart.** [A hundred sessions held in a quiet slow window produced 902.8 units/s against the reference 907.1 -- 0.5% apart -- while a lone session ran a third under](docs/measurements/2026-08-03-173452-the-slow-state-flatters-the-gate.md), so the ratio read 1.54 and the gate would have *passed*. The 3.958 run's hundred sessions produced 246.4. Yet their solo holds agree to half a percent, 9.752 against 9.801. So a depressed solo rate names neither the state nor the direction it moves the gate, and what separates them -- the concurrent hold's own total throughput -- is recorded in every run and was not being read. **Read across all nine hundred-session holds on disk it is bimodal**: six between 903 and 1055 units/s, three between 217 and 288, and nothing in the 3.1x gap. Two of the three low ones are where 3.958 and 3.261 come from, so those are slowdowns of a machine at a quarter throughput rather than of this one. Three of the six agree to 0.5% across separate runs, tighter than any solo agreement recorded here. **All nine were quiet, and the gap has since been filled twice** — at 344.9 by a tenanted hold, and at 756.1 by the quietest hold yet taken, 0.49 cores held elsewhere. So the bimodality described ten readings rather than the machine, and what survives is that a total far below ~900 means something is wrong while the rest column says whether it is a neighbour.

**So the bottleneck is hardware and one unattempted run, not code, and the fork it looked like is closed.** The gate is not restated to fit this box: it stands, this box is about 3% short of it, and [what to do on a machine that can answer it is written down](sessionbench/README.md#running-gate-m1-on-a-machine-that-can-pass-it) — including which figures to re-measure there and which need no re-deciding anywhere. **The sizing is half a core**: `C ≥ N·d/(2η)` gives **16.52 logical processors** against sixteen at the run of record, and 16.46 to 16.71 across the three weights — so it barely depends on the session weight. The 17.3 and 18.4 quoted earlier came from a run whose mean carried its own interruptions, and nineteen was 18.4 rounded up. **The 16.4 that stood here for a day was `η` rounded to 0.82 before the division**, which is 0.06 of a core low and below every weight's own figure; it also mixed the 33 MiB run's `η` into a sentence quoting the 20 MiB run's slowdown. The hour is the other half and is untouched — the box still stops dead at forty-one minutes, on the rising load rather than the flat one the gate states. Writing more daemon moves neither, and **any figure quoted for this gate has to name the session weight and the machine state it was taken in**.

**Nor does writing more instrument, and that is measured rather than assumed.** A day of fixes — a counted window, streamed samples, a failed-read counter, repeated baselines, an uncounted warm-up, a standard error where a range had been — moved neither failing number, and [the bracket's two refusals both turned out to be the run's opening hold](docs/measurements/2026-08-02-222324-the-instrument-is-done-arguing.md) rather than a noisy baseline or a moving machine. Two attempts to escape the solo baseline ended one useful and one [closed by there being no knee](docs/measurements/2026-08-02-220514-there-is-no-knee.md).

**What has landed since is about evidence rather than accuracy, and none of it can move a gate figure** — checked mechanically: no changed line touches how a rate, an RSS peak or a verdict is computed. Every verdict now carries the number it was computed from, holds and rungs report the occupancy a run actually held rather than a mean that hid an interruption, every sample carries the whole machine's CPU, and a hold records its worst tick as a ramp always did. The rule that told you to pre-screen a ratio on `doctor` readings is gone: four of them spread 26.7% and preceded the quietest bracket this instrument has produced.

## Current Priority: M0 · Attribution

**Closed on 2026-07-31.** [Gate G0 is frozen](docs/measurements/2026-07-31-150258-g0-frozen.md) and the work in flight is [M1](#m1--headless-daemon).

The heading above still says *current* because thirteen references point at its anchor and seven of them are [measurement records](docs/measurements/), which are logs rather than living documents. Renaming it would edit seven files that say what was true when they were written. The section stays; the claims in it are dated.

M0 built `sessionbench` and nothing else. It establishes the as-is **redline** and names what limits it. The metric, the six axes, the comparison set, and the rules that keep the benchmark honest live in [sessionbench/README.md](sessionbench/README.md).

**Before the ramp, run one session to completion.** A single real generation session, logged for RSS over time, Defender's cost at startup versus during file writes, and output volume. One afternoon, one machine, no ramp harness. Multiply by 100 on paper and compare against the redline conditions: if the shape contradicts them, fix the conditions before building the instrument that applies them. Arithmetic already retired spawn time as a ceiling; this is the same move applied to what replaced it.

Done on 2026-07-30 as far as a machine without the harness can take it, and [the conditions survived](docs/measurements/2026-07-30-101141-conhost-and-defender.md#do-the-redline-conditions-survive-this) — memory has room, cores do not, and the condition that would trip is the one the metric already leans on. The ramp was then built and run, and it [brackets this machine between 25 and over 100 sessions](docs/measurements/2026-07-30-120002-first-redlines.md) depending only on how much CPU one session wants.

**That bracket has since collapsed to a formula.** A session's redline moves inversely with how much of its time it spends computing, and [the relation turned out to be derivable rather than fitted](docs/measurements/2026-07-30-154348-duty-is-derivable.md): `redline = 2ηC/d`, where `d` is that fraction, `C` is the machine's logical processors, and `η` is what sessions cost each other through the memory system. Two ramps at different duties agree on `η` to within 0.4%, and a rung predicted from it in advance landed.

So G0 no longer waits on a harness for a whole measurement. It waits for **two scalars and one rung** — a real session's duty, an `η` from any rung held past saturation, and a ladder to check the answer rather than to find it. **`C` being in the formula is the part that travels**, since the configured machine will not have sixteen logical processors and the flat `25/d` this replaces would have been wrong there in proportion.

**Gate G0 — frozen 2026-07-31 at [nine sessions, limited by RSS](docs/measurements/2026-07-31-150258-g0-frozen.md).** A generation session is an agent CLI holding 0.52 GiB for its whole life plus an engine holding 1.87 GiB while it cooks, and `21.97 ÷ 2.39` is where this machine stops. Cores would not bind until [≈97 on a rested box, or 51 on one recently saturated](docs/measurements/2026-08-03-024222-the-footprint-never-mattered.md) — an order above nine either way, which is why the freeze names memory. The pseudoconsole row of the comparison set is open and recorded as such, because the decision it would have informed was settled by arithmetic before a ramp could reach it.

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

**Gate M1:** 100 sessions held for an hour at total RSS under 4GB, per-session work rate within 2× of solo, and no dropped output.

**What that sentence leaves open, and where each was already decided.** A gate nobody can adjudicate is not a gate, and four of these were settled in `sessionbench` and written down nowhere else — found by reading the code that judges them rather than the sentence that states them. **4GB is decimal**: `--rss-budget-gb` multiplies by `1e9`, so the budget is 3.725 GiB, which is the 3.73 every hold has reported against. **RSS is the peak**, not a steady value or a mean, and it covers **the daemon and everything it holds** rather than the sessions alone. **Solo is bracketed** — one hold before and one after, because [two solo triples ten minutes apart differ by 8.5% where each spreads under 3.5%](docs/measurements/2026-08-01-163935-what-the-harness-says-about-itself.md), so drift between runs is three times the noise inside one.

**What is still open is the workload**, and it is the term that decides the outcome: duty sets how many sessions are awake at once and footprint sets `η`, so *a hundred sessions* names a count and not a load. This gate is run at the measured driven duty of 0.27 for that reason, and the footprint is bounded by the RSS condition rather than chosen — [at 33 MiB a session, where a hundred peak at 3.632 GiB of 3.725](docs/measurements/2026-08-02-225348-the-two-failing-conditions-are-not-independent.md).

**A third axis was looked for and does not exist**, which is worth writing down so nobody looks again: what a session's lines cost the daemon used to be set by their length, and the scrollback's byte budget bounds a hundred sessions to about 43 MB whatever they write. Duty and footprint are the two the gate leaves open, not three. Cold start under 30 seconds is a nice-to-have recorded in the startup log, not a gate — [it is 0.02% of the workload](docs/PLAN.md#residency-not-spawning).

**The work-rate condition is decided by the workload's duty before the daemon does anything**, which the sentence above does not say and a run costs an hour to discover. A session wanting `d` cores means a hundred want `100d`; past what the machine has, each gets `C/100` instead and the slowdown is `100d/C`. On sixteen logical processors, within 2× needs `d ≤ 0.32`. So a hundred CPU-bound sessions cannot pass on this machine and no daemon could make them — while [the measured duty of a real driven agent turn, 0.27](docs/measurements/2026-07-31-054657-the-driven-duty.md), predicts 1.69× and passes with little to spare. The gate is meaningful at the duty a session actually has and arithmetic at any other, so a run that does not state its duty has not stated its result.

**1.69× is the floor rather than the estimate**, since it assumes sessions cost each other nothing. **Measured at that duty, the condition breaks: [a hundred sessions come back at 2.0654×](docs/measurements/2026-08-03-024222-the-footprint-never-mattered.md)**, 3.3% past the 2 the gate asks for, while RSS holds and every session runs to the last report. The first hold of it [published 2.301](docs/measurements/2026-08-01-213912-the-gate-breaks-at-its-own-duty.md), 10.2% high because it lost the machine to something else for part of its window. So **gate M1 is not passable on this machine at the duty it is meaningful at**, and no daemon can change that — a hundred sessions wanting 27 cores from sixteen is arithmetic before it is engineering.

**The gate is not mis-specified, it is above this machine — by about 3%.** The condition is `slowdown ≤ 2` and this box returns 2.0654, so it needs to be `2.0654 ÷ 2` wider: **16.52 logical processors against its sixteen**, half a core. That is the whole of the sizing, and the `N·d ÷ (η·C)` form reduces to it once `η` is the value this machine's own slowdown gave. So 16.4 is an estimate carrying this machine's `η`, not a requirement derived from the gate — on a wider box the test is one hold and a reading rather than a calculation. [The steps, for whoever runs it on a machine wide enough](sessionbench/README.md#running-gate-m1-on-a-machine-that-can-pass-it) — including which figures to re-measure there rather than carry from here. The corrected slowdown puts the core ceiling at **≈97**, and the older 87 and 93 are the same formula with an `η` taken before this run's lost occupancy was separated out. **The ceilings are the slowdown written another way**, not a measurement checking a projection: substitute `η = N·d ÷ (C·s)` into `2ηC/d` and the cores, the duty and `η` itself all cancel, leaving **`2N/s`**. So *≈97 against a hundred sessions* and *2.0654 against 2* are one statement in two units — the same 3.3%. What licenses reading a ceiling at any count other than the one that was run is not the relation but [`η` being flat above saturation](docs/measurements/2026-08-02-194155-eta-is-flat-where-it-was-said-to-fall.md), which was measured separately and is what the extrapolation actually rests on. The session's weight barely moves it — [2.0654, 2.0569 and 2.0887 at 20, 33 and 36 MiB](docs/measurements/2026-08-03-024222-the-footprint-never-mattered.md), the same within 1.5% — while the machine's own state does, the same hold giving 51 once the box has slowed. **A ceiling quoted without its workload and its machine state is not quoted.** Both sit far above [the nine a real generation session allows](docs/measurements/2026-07-31-150258-g0-frozen.md), so neither moves G0, and [the value itself rests on a single slowdown](docs/measurements/2026-08-01-213912-the-gate-breaks-at-its-own-duty.md) until the knee is run.

**And what the contention term follows is now measured: the product `N·d`, the number of sessions awake at any instant.** A hundred sessions at duty 0.27 and a hundred and thirty-five at 0.20 leave twenty-seven awake either way, and [their total throughputs agree to within 1.5% while the session count differs by 35% and the duty by 26%](docs/measurements/2026-08-02-121359-eta-follows-the-awake-count.md). What that rules out is a dependence on either term alone: the two holds differ 35% in session count and 26% in duty, and 1.5% is not somewhere a 35% effect hides. **That is the quantity [M3's governor](#m3--resource-governor) admits against** — a hundred quiet sessions and twenty-seven busy ones are the same load, and one number is enough to decide.

**And within that regime it is flat, so the constant survives** — which narrows the paragraph above rather than contradicting it, since a constant is a function of `N·d` that happens not to vary. Read directly rather than through a solo baseline, [`η` moves 1.10% against 0.69% of drift while the awake count rises 67%](docs/measurements/2026-08-02-194155-eta-is-flat-where-it-was-said-to-fall.md). An earlier reading had it falling — 0.733 against about 0.8 — and both halves of that comparison were weaker: each came through a solo hold, the noisiest number this instrument makes, and the higher one sat at `N·d ÷ C = 1.07`, a tenth above saturation where the relation's own assumption stops holding. **So `2ηC/d` is usable with a fixed `η` above saturation**, and what needs care is the boundary — which is exactly where a governor decides whether to admit one more.

One more thing follows. **Stating the duty is not enough: a run has to measure the duty it actually held.** A fixed wait calibrated on a warm machine gave 0.172 where 0.271 was asked for, because it turns a 79% difference in compute speed into 13.6% of work rate — which is why the gate's script uses `--duty`, [the flag that delivers a stated duty on any machine](docs/measurements/2026-08-01-210316-the-wait-mechanism-cancels.md).

**This gate measures the daemon, not capacity, and G0 is why that has to be said.** 4GB across 100 sessions is 40 MiB each; [a real generation session holds 2.39 GiB](docs/measurements/2026-07-31-150258-g0-frozen.md), which is sixty times more and would put a hundred of them at 239GB. The figure is reachable because the sessions here are the daemon's own bookkeeping plus a shell on pipes, which [measures 8.97 MiB](docs/measurements/2026-07-31-035111-between-builds.md) — so it asks whether `coggyd` disappears under load, and answers nothing about how many generation sessions fit. **That number is nine**, it is set by memory the daemon does not own, and admission against it belongs to [M3](#m3--resource-governor). Reading one as the other is [the conflation that cost M0 a day](docs/measurements/2026-07-31-050322-the-agent-side-of-a-session.md).

**Measurement rule:** take these three numbers with **the same harness** used at M0. Build a second bench and the M0 baseline becomes incomparable.

That rule already bit once. [A hundred sessions were held for an hour on 2026-08-01](docs/measurements/2026-08-01-103225-an-hour-of-a-hundred-sessions.md) with `Get-Process` and a PowerShell sampler, which is the second bench — the figures were sound and could not clear the gate. `sessionbench hold` now takes the same reading through the same job, sampler and artifact shape.

**All three can be asked, and two of them only after the question was put differently.** Work rate needed a second run rather than a different daemon, and [`hold --with-solo` is that run](sessionbench/README.md#running-it) — a solo baseline on either side of the concurrent hold, refusing to produce a ratio when the two disagree by more than the machine makes while being itself.

**Dropped output was called unreachable for a year of afternoons and was not.** The reasoning was that the condition is found by watching ordinals in a session's own stream, and under `coggyd` that stream ends in the daemon's scrollback, so nothing outside holds what the workload emitted. True, and the wrong question: **a pipe blocks rather than dropping**, so between a session's `write` and the scrollback there is nothing that can lose a line, and the only loss available is the daemon's own reader giving up. It [used to give up on the same branch as a clean end-of-file](coggyd/src/lib.rs), which is what made the one observable failure unobservable. The daemon counts it now and reports `failed_reads`; zero answers the condition. The words stay distinct — `out_of_reach` still means no route exists, and **replacement still has none**, since nothing in the daemon restarts a session that exited.

**Replacement is not one of these three**, though it is easy to count as one: it is [the redline's fourth condition](sessionbench/README.md#redline) and the daemon has no counterpart for it either. A ramp under the daemon therefore loses two of four where a hold loses one of three, and the coincidence of small numbers is what makes the two lists blur.

**So all three of those are measured, and the gate does not pass.** RSS holds, dropped output is zero, and work rate returns **2.0654** against a 2. The gate's fourth condition is the hour, and it is untried at this duty rather than failed — the run that stopped the box at forty-one minutes held a fixed wait whose duty climbed as the machine slowed, while every hold at a fixed 0.27 has finished clean at twenty. **Neither the failure nor the gap is the daemon**, so M1 is not waiting on code.

**The session's weight is not a lever, and three readings of that took a night.** `η` looked like it rose steeply with the footprint, then like it fell, and neither survived: read per interval on a rested box, [a hundred sessions slow down 2.0654, 2.0569 and 2.0887 at 20, 33 and 36 MiB](docs/measurements/2026-08-03-024222-the-footprint-never-mattered.md) — the same within 1.5%, with the footprint worth about 2%. RSS held at every one of them and dropped output was zero. **So work rate fails by 3–4% at every weight this machine can hold**, the memory budget never becomes the binding condition, and whether *4GB* is decimal or binary buys nothing.

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

**A ceiling may be the wrong shape for it, and that is measured rather than suspected.** Admitting *up to* a number assumes the cost of one more session changes somewhere. Sweeping eight to twenty-eight sessions [found no such place](docs/measurements/2026-08-02-220514-there-is-no-knee.md): per-session throughput falls 34% across that range while demand stays under three quarters of the free cores, and the marginal return of four more sessions drops from 37.9 units/s to 3.1. **The decline starts at the first sessions and never has a corner**, so what a governor can offer is not a count but a price — at twenty-four sessions the next four buy 3.1 units/s and cost every incumbent 13% of its rate. Whether that is worth paying is a question the gate does not currently ask.

**And whatever shape it takes, it is per session weight — and by less than the machine's own state.** On a rested box, [a hundred sessions slow down 2.0654, 2.0569 and 2.0887 at 20, 33 and 36 MiB](docs/measurements/2026-08-03-024222-the-footprint-never-mattered.md) — the same within 1.5%, so the ceiling is ≈97 at every weight and the session's footprint is worth about 2%. The same pairing on a slowed box reads 51 and 61, so **the state is worth more than the weight** and a ceiling quoted without both is not quoted. A governor reading one ceiling and applying it to whatever arrives is reading a number for a workload it may not be admitting, on a machine it may not be admitting it on. What sets the direction is unmeasured — the footprint does not change what a unit costs alone, so it is contention of some kind and a slowdown does not say which.

**And the ceiling a governor would admit against is two quantities multiplied.** `η` divides by the machine's core count, and [the sessions in these runs held 14.11, 15.33 and 15.27 of sixteen](docs/measurements/2026-08-03-023657-eta-was-two-quantities-multiplied.md) — so `η` carries an occupancy term that belongs to whatever else the machine was doing, on top of an efficiency term that belongs to the sessions. Divided by the cores actually held, what looked like an 11.4% footprint effect falls to 2.5% — and read per interval it falls again to about 2%, because the occupancy difference was one run losing the machine rather than a standing property of the weight. **A governor admitting against `η` is admitting against a number that moves when something unrelated wakes up**, which is the argument for pricing on measured occupancy rather than on a fitted constant, whatever moved it.

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
