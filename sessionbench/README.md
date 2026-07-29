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

Find it by **monotonic ramp, not binary search.** Step N through 1, 10, 25, 50, 75, 100, 150, 200, holding each step until RSS plateaus before advancing. The redline is the last N before any one condition first breaks, and the condition that broke is recorded with it.

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

## What we measure against

A benchmark that only measures `coggyd` is marketing. At minimum, these run on identical hardware.

| Target | Role |
|---|---|
| Windows Terminal + pwsh 7 | The as-is baseline — the thing we claim to beat |
| Windows Terminal + cmd | Control that isolates shell startup cost |
| conhost directly, no terminal UI | Floor that isolates the UI layer's cost |
| WezTerm (Windows) | Rust prior art; the honest comparison |
| Alacritty (Windows) | Upper bound for a minimal implementation |
| wmux | Electron control; the cost of that architecture choice, in numbers |
| coggyd | Ours. **Added only after M1** |

## Keeping it honest

Tuning `coggyd` against these results makes the gate grade a bucket it drew itself. Instrument and subject coming from the same hands is the classic failure here.

1. **Freeze the as-is baseline at M0.** Measure it before `coggyd` exists and never remeasure. If hardware changes, every target gets remeasured together.
2. **Workloads know nothing about `coggyd`.** Payloads are pure stdout generators and real agent CLI sessions. A workload that takes a COGGY-specific path is banned.
3. **Distrust one axis improving while five stay flat.** redline is a conjunction of four conditions, so optimizing a single axis cannot raise it. If it rose anyway, the cost moved somewhere we are not looking.
4. **Give every run a directory Defender has not seen.** Real-time scanning costs far less the second time a file is written, so re-running a workload over the same paths measures the cache rather than the workload. Two otherwise identical runs differed threefold on this axis before the rule existed, and the second one looked like the improvement.

**Those three rules are what buy credibility, not the repo boundary** — so this stays inside the COGGY repo. The evidence agrees: `alacritty/vtebench` split out and has not moved since January 2025 while Alacritty ships continuously, whereas Ghostty keeps its benchmarks in `src/benchmark/` and is the healthiest of the three. Splitting early buys a second CI and a sync problem.

Split it out when the pressure arrives from outside, ordered by how far outside it starts:

1. A reviewer discounts a result because the benchmark ships with what it measures — the rules stopped persuading.
2. Contributors open PRs touching only `sessionbench`.
3. A project other than COGGY cites a redline.

Use `git subtree split` then, which preserves history. **Never symlinks:** on Windows they need Developer Mode or admin, `core.symlinks` defaults to false, and the setting is not retroactive — anyone who already cloned gets text files containing paths.

## Report format

Machine-readable and human-readable are separate artifacts.

- **`sessionbench.json`** — the raw record: six axes per target per session count. Canonical, not a derivative.
- **`sessionbench.md`** — for humans: the redline on the first line, six curves below it, hardware table last.
- Headline format: `redline: 84 sessions (RSS) · Windows Terminal + pwsh 7 · 16C/64GiB · Defender on`

Memory is reported in **GiB, and as the figure the operating system calls usable** rather than the number on the box. Condition 2 is a fraction of it, so the 7% between GiB and GB is enough to move a verdict, and a machine sold as 32GB has about 31 GiB to give.

**A redline quoted without its target, session count, and hardware is not quotable.** The moment a bare number circulates, the benchmark has become marketing.

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
