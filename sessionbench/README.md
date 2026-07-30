# sessionbench

> The concurrent session scaling benchmark for Windows-native terminals.

**This document owns the benchmark: its metric, axes, comparison set, integrity rules, and output format.** [README](../README.md#documentation) maps the rest.

Every existing terminal benchmark measures one session. Throughput, latency, task success — one axis, one session. Nothing measures the scaling curve against *N* concurrent sessions. That gap is the whole justification, and it equally means everything else already exists, so we build none of it.

## redline

**redline = the largest concurrent session count N that satisfies all four conditions at once:**

1. Per-session work rate within **2×** of the same workload run alone
2. Total RSS within **70%** of physical memory
3. **Zero** dropped session output
4. A finished session is replaced within **60 seconds**

Condition 1 is the one that matters, and it is the reason the other three exist. Asking for 100 concurrent sessions and getting 100 processes that each run at a third of their solo speed is not 100-way concurrency — it is 33-way concurrency with worse latency. The count you requested and the count you got are different numbers, and redline is the second one.

The conditions target **residency, not process creation.** Agent sessions here live for hours, so the machine spends essentially all of its time holding sessions rather than starting them. Spawn cost enters only through condition 4, which asks whether steady-state churn keeps up — not whether a cold start is fast. See [what this deliberately does not measure](#what-this-deliberately-does-not-measure).

Find it by **climbing to a bracket, then narrowing it.** Step N through 1, 10, 25, 50, 75, 100, 150, 200, holding each rung long enough to settle. The first rung that breaks leaves an interval whose two ends have both been measured, and the redline is found by halving that interval alone — which assumes nothing about behaviour outside it, unlike bisecting the range from the start. The condition that broke is recorded with the count.

The climb on its own is not enough, and this is measured rather than argued: a ceiling that sits at 25 reads as 10 from the coarse rungs, because 10 is simply the last one tried before 25 failed.

**A redline limited by work rate, RSS, or replacement lag is a budget drawn across a slope,** and moves if the budget moves. Only dropped output is an edge. On one machine the per-session work rate ran 1.96× solo at 25 sessions and 2.03× at 26, so the whole answer turned on where the 2× line was drawn — which is worth knowing about a number before quoting it.

**redline is a pair, not a scalar:** `84 / RSS`, never bare `84`. A redline without its limiting cause cannot be reproduced, and the cause is what says where to optimize next.

Different machines yielding different values is correct behavior, not noise. Every report carries its hardware: core count, RAM, disk, and Defender state.

## The six axes

Every axis is plotted against concurrent session count, producing six curves.

**Four that hold for any long-lived session workload:**

1. Per-session work rate (p50 / p99), against the same workload run alone
2. Total RSS
3. Total process count
4. Output bytes absorbed, and bytes dropped

**Two that Windows benchmarks never measure, and the reason this exists:**

5. **conhost process count.** The direct evidence for or against [Decision 1](../docs/PLAN.md#four-core-decisions). What it buys is resident memory, not spawn speed: 100 conhosts cost their RSS for the entire life of every session.
6. **Defender exclusion delta.** The same workload run twice, before and after exclusions, measured across a whole session rather than at startup. Generation writes files continuously, so scanning cost accrues the whole time and a large delta reorders the optimization queue.

## What this deliberately does not measure

**Cold-start spawn time.** The plan's own arithmetic rules it out as a ceiling: 1000 games a day at 1.5 hours each is 1500 session-hours, while 1000 spawns at even a full second each is 1000 seconds — **0.02% of the workload.** Three retries and five seconds a spawn still leaves it under 0.3%. Starting 100 sessions at once is real but happens once per daemon restart, so it belongs in a startup log rather than a ceiling metric. Condition 4 covers the part that recurs.

**Input latency.** Sessions here are headless agent CLIs on pipes with no human waiting on a keystroke. Keystroke-to-screen latency becomes meaningful when [M4](../ROADMAP.md#m4--audit-surface) puts a human in front of a focused session, and the axis gets added then rather than measured now against nobody.

## Running it

Requires Rust 1.88 or newer on Windows.

```
cargo run -p sessionbench -- doctor
```

`doctor` reports the machine and which axes are actually available. Run it before trusting any result: the Defender axis needs elevation, and a run without it silently measures five axes out of six while still printing a redline. A redline missing an axis is not a smaller result — it is a wrong one.

```
cargo run -p sessionbench -- observe --duration 300 -- <command>
```

`observe` runs one session to completion and records what holding it costs — RSS over time, process and conhost counts, output volume, and Defender's CPU split between startup and steady state. It writes `samples.jsonl` and `run.json` under `bench-out/`, then closes with a linear projection to 100 sessions. That projection is a floor rather than a forecast, and it is useful for exactly that reason: a floor that already breaks a condition settles the question without running the ramp.

Add `--pty` to give the session a pseudoconsole instead of pipes. Running one workload both ways is the direct measurement behind [Decision 1](../docs/PLAN.md#four-core-decisions), since the difference between the two is one conhost per session, resident for as long as the session lives.

```
cargo run -p sessionbench -- exclusion-delta -- <command>
```

`exclusion-delta` runs one workload watched and then excluded, several times over, and reports what the exclusion bought. **It changes this machine's real-time protection for the length of each excluded half**, over a directory the benchmark created for that run, and removes the exclusion afterwards whether or not the run succeeded. The removal is verified rather than assumed, and a failure to remove is printed where it cannot be missed.

Halves run as adjacent pairs because a single comparison cannot separate the exclusion from whatever else the machine was doing, and every half is preceded by an idle baseline for the same reason. When the spread across pairs is wider than what separates them, the run says inconclusive rather than averaging noise into a confident number. Fresh directories throughout, since reusing one would credit the exclusion with the scanning cache's work — [rule 4](#keeping-it-honest).

**At one session it will tell you nothing**, which is [measured rather than warned about](../docs/measurements/2026-07-30-exclusion-delta.md). The exclusion axis lives on the ramp: `ramp --exclude-scratch` holds one exclusion over the sessions' writes for a whole ladder, and two ladders compared by redline is the form that answers.

```
cargo run -p sessionbench -- ramp --hold 90 -- <command>
```

`ramp` climbs the ladder and produces the redline. Each rung holds its session count for the whole window — **replacing any session that finishes**, which is both what keeps the count honest and what makes the replacement condition measurable at all. A rung that let finished sessions stay finished would report the count it asked for while measuring a decaying one, and the machine gets easier as that happens, so the number would climb exactly when it should fall.

`--max-sessions` caps the climb. The full ladder reaches 200 and will take the machine with it for the duration.

Rungs are judged on all four conditions, so a run that cannot evaluate one does not print a smaller redline — it prints none.

## What we measure against

A benchmark that only measures `coggyd` is marketing. At minimum, these run on identical hardware.

| Target | Role | Reachable |
|---|---|---|
| A pseudoconsole per session | The as-is baseline — what a terminal gives a session today | [measured](../docs/measurements/2026-07-30-conhost-and-defender.md) |
| Pipes, no pseudoconsole | The floor, and what the daemon intends to default to | [measured](../docs/measurements/2026-07-30-first-redlines.md) |
| pwsh 7 against cmd against the workload alone | Control that isolates shell startup cost | this instrument |
| coggyd | Ours. **Added only after M1** | M1 |
| Windows Terminal, WezTerm, Alacritty, wmux | What an emulator costs to host N sessions | a different instrument · M4 |

**The last row is a different question, and finding that out was worth the row.** This instrument spawns the process and holds the reading end of its output; that is what lets it count units and notice a gap in them. A terminal emulator owns its own pseudoconsole and draws to a window, so there is no reading end to hold and no work rate to count — and `wt.exe new-tab` returns to us immediately while the session it opened belongs to a process that was already running. Attribution fails for the same reason: the job object is joined by *this* process before it spawns anything, and membership is inherited downward, so a program that started before us was never going to be in it.

Which is not a defect in either. The question this instrument answers is what a session costs to exist — and a session costs the same whether WezTerm or Windows Terminal is drawing it. What an emulator costs to *host* a hundred of them is a real question and a separate one: it measures one process's rendering rather than a hundred processes' residency, it needs a way to drive a UI into opening sessions, and it belongs with the axis that has a screen to compare against.

## Keeping it honest

Tuning `coggyd` against these results makes the gate grade a bucket it drew itself. Instrument and subject coming from the same hands is the classic failure here.

1. **Freeze the as-is baseline at M0.** Measure it before `coggyd` exists and never remeasure. If hardware changes, every target gets remeasured together.
2. **Workloads know nothing about `coggyd`.** Payloads are pure stdout generators and real agent CLI sessions. A workload that takes a COGGY-specific path is banned.
3. **Distrust one axis improving while five stay flat.** redline is a conjunction of four conditions, so optimizing a single axis cannot raise it. If it rose anyway, the cost moved somewhere we are not looking.
4. **Give every run a directory Defender has not seen.** Real-time scanning costs far less the second time a file is written, so re-running a workload over the same paths measures the cache rather than the workload. Two otherwise identical runs differed threefold on this axis before the rule existed, and the second one looked like the improvement.
5. **Watch the instrument's own cost, which every run records.** A scaling benchmark's one undetectable failure is the observer becoming the bottleneck, and it does not announce itself — it arrives looking like the machine collapsing. Twenty-five sessions that saturate every core starve the sampler for fifteen seconds a tick, while twenty-five that yield cost it fifty-six milliseconds at the same process count and the same resident memory. A rung the sampler could not keep up with is reported as inconclusive and stops the ladder, because a rung that could not be read has not failed.

**Those three rules are what buy credibility, not the repo boundary** — so this stays inside the COGGY repo. The evidence agrees: `alacritty/vtebench` split out and has not moved since January 2025 while Alacritty ships continuously, whereas Ghostty keeps its benchmarks in `src/benchmark/` and is the healthiest of the three. Splitting early buys a second CI and a sync problem.

Split it out when the pressure arrives from outside, ordered by how far outside it starts:

1. A reviewer discounts a result because the benchmark ships with what it measures — the rules stopped persuading.
2. Contributors open PRs touching only `sessionbench`.
3. A project other than COGGY cites a redline.

Use `git subtree split` then, which preserves history. **Never symlinks:** on Windows they need Developer Mode or admin, `core.symlinks` defaults to false, and the setting is not retroactive — anyone who already cloned gets text files containing paths.

## Report format

Machine-readable and human-readable are separate artifacts.

Every run writes both, into its own directory under `bench-out/`:

- **`ramp.json`** / **`run.json`** — the raw record. Canonical, not a derivative.
- **`ramp.md`** / **`run.md`** — for humans: the headline on the first line, the curves below it, machine and provenance last.
- Headline format — count, the condition that stopped it, workload, mode, machine, Defender: `redline: 10 sessions (WorkRate) · stdout-storm · pipe · 16C/31GiB · Defender on`

Two names rather than one, because the two commands answer different questions and a reader holding a file should be able to tell which. **The markdown is generated, never written by hand** — the first two records here were typed up from terminal output, and a figure retyped is a figure that can be retyped wrong.

Memory is reported in **GiB, and as the figure the operating system calls usable** rather than the number on the box. Condition 2 is a fraction of it, so the 7% between GiB and GB is enough to move a verdict, and a machine sold as 32GB has about 31 GiB to give.

**A redline quoted without its target, session count, and hardware is not quotable.** The moment a bare number circulates, the benchmark has become marketing.

Results worth keeping are written up under `docs/measurements/`, one file per session, each carrying the provenance block of the run behind it. The raw artifacts are not committed — a report that records its commit and its hardware can be reproduced, and a repository full of `samples.jsonl` cannot be read.

**Provenance block.** Neither the toolchain nor the measurement crates are pinned; both track latest, because going stale costs more than drift does. That trade is only safe if every run records what produced it, so `sessionbench.json` carries the rustc version, the `sessionbench` commit, and the resolved version of every crate touching a measurement. Frozen baselines stay comparable because they are *labeled*, not because the inputs were held still, and a baseline whose provenance differs from the current run is flagged rather than compared.

## What we take from prior art

| Prior art | What it measures | What we take | Why we don't overlap |
|---|---|---|---|
| **alacritty/vtebench** | PTY read speed only; the authors state outright it covers neither framerate nor latency | **The workload format: a directory plus an executable, with stdout as payload.** Copied as-is, along with the gnuplot plotting convention | X11/Linux assumptions, single session |
| **cmuratori/termbench** | Terminal output bandwidth, with **Windows as a first-class target** and Linux unverified | **The single-session bandwidth baseline, cited rather than remeasured.** 0.5–2.0 GB/s for a reasonable terminal is already published | Bandwidth of one session |
| **contour/termbench-pro** | Same bandwidth, but splits a `tb` CLI from a library so backends can be tested without the render pipeline | **The lib/bin split**, which this crate follows | Single session |
| **hyperfine** | CLI execution-time statistics | **Warmup, outlier rejection, and confidence intervals.** We write no statistics code | No scaling axis |
| **typometer / Dan Luu's latency work** | Input latency | **Held for M4**, when a human first watches a focused session. The camera-versus-timestamp rationale is the part worth taking | Single session, mostly X11; measures a human-facing axis this benchmark does not yet have |
| **Terminal-Bench** (Stanford + Laude) | Agent success rate on terminal tasks: 89 hand-built, human-verified tasks demanding end-to-end workflows | **The task directory format** — instruction, metadata, environment, solution, tests — as a model for workload definition, plus shipping a reference scaffold alongside the benchmark | Its axis is accuracy. It measures model capability, not infrastructure. **Complementary, not competing** |

In one line: Terminal-Bench measures *how well an agent performs*; sessionbench measures *how many agents you can run at once*. Both should run on the same machine.
