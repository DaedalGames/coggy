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

**So for work rate the budget is solved on the slope rather than searched for.** Bisection lets one reading of one rung decide which way to search next, and near the budget a rung landing a percent to either side sends it in opposite directions: [seven identical runs](../docs/measurements/2026-07-30-164912-redline-reproducibility.md) returned 30, 31, 31, 31, 33, 34 and 34 — a spread of 12.5% from rungs that each reproduced within 2%. Fitting `slowdown = b·N` through every saturated rung and solving `b·N = 2` uses all of them, so no single one can drag the answer, and the same seven then agree to 2.3%.

The line goes through the origin because that is what the relation says: `slowdown = N·d/(η·C)` is linear in `N` with no constant term, so **the fitted slope is `η`** and the crossing is `2ηC/d`. Letting the intercept float made the answer less reproducible, which is what a parameter absorbing noise does. The climb and the refinement still run — they are what put rungs on both sides of the budget for the line to be drawn through.

**`C` is the machine's core count, and the sessions do not get all of it**, so the fitted `η` carries whatever else was on the box. Three hundred-session holds put the job at 14.11, 15.33 and 15.27 of sixteen — [a spread that read as an 11% property of the workload until it was divided out](../docs/measurements/2026-08-03-023657-eta-was-two-quantities-multiplied.md). Within one ladder the convention cancels, since every rung divides by the same sixteen; **across ladders it does not**, which is one more reason two redlines from different afternoons are not subtractable. Each rung now records what it actually held.

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

**It also reports the machine's own headroom**, the least physical memory free at any point in the run. The RSS condition reads a working set, which is what a process holds in RAM now — and that figure *falls* once Windows starts paging, so it goes quiet during exactly the failure it exists to catch. Commit charge has the opposite fault: a runtime that reserves a heap and never touches it reads as pressure that never arrives. Headroom is neither, being a fact about the machine rather than about any process, and it was collected on every sample from the first run and reported nowhere until it was needed.

**Its RSS figure carries its own control.** The median of the first measured quarter is reported beside the median of the last, because a memory-limited redline is the budget divided by that figure and is a ceiling only if the session held the same amount throughout — a run whose two ends disagree is averaging two different sessions. It is the single-session counterpart to a ramp repeating a rung, and it is the whole of an RSS redline's rigour, since [there is no slope to fit when total memory is the per-session figure times the count](../docs/measurements/2026-07-31-045604-an-error-bar-for-the-engine.md).

`observe` runs one session to completion and records what holding it costs — RSS over time, process and conhost counts, output volume, and Defender's CPU split between startup and steady state. It writes `samples.jsonl` and `run.json` under `bench-out/`, then closes with a linear projection to 100 sessions. That projection is a floor rather than a forecast, and it is useful for exactly that reason: a floor that already breaks a condition settles the question without running the ramp.

```
cargo run -p sessionbench -- hold --sessions 100 --duration 3600 -- <command>
```

`hold` is [gate M1's shape](../ROADMAP.md#m1--headless-daemon): a stated number of sessions under `coggyd`, held for a stated time. **Not a ladder, and not a mode of `observe`.** A ramp asks how many fit; `observe` measures one and multiplies; this puts the sessions there and reads them. Its report carries no projection for that reason — a projection field beside a direct measurement is a number with nothing behind it.

**A short `--duration` reads low, and the reason is not the one it looks like.** The daemon reports every ten seconds, which invites the thought that a shorter hold cannot be counted — it can, because a final report fires at end-of-file, and five seconds of two sessions comes back with a full count. What actually costs is that starting the daemon and its sessions lands inside the measured window while the work does not: **a five-second hold counted 6.36 seconds and a twenty-second one counted 20.45**, so the fixed cost is 27% of the short one and 2.2% of the longer. The report interval bounds something else again — the fewest-running figure is a minimum over those lines, so a dip between two of them is not seen. The gate's own run is [an hour](../docs/measurements/2026-08-01-103225-an-hour-of-a-hundred-sessions.md) and meets none of this; it is the smoke tests around it that do.

```
cargo run -p sessionbench -- hold --sessions 100 --with-solo -- <command>
```

**Each side is three holds by default, not one.** A solo baseline is one session on one core, and [which core it got is worth 4.54% while its CPU share moves 1.95%](../docs/measurements/2026-08-01-173927-the-baseline-is-the-noisy-term.md) — against an allowance of five. **That figure is `--duty`'s**; a fixed `--wait-ms` dilutes the same noise to well under a percent, and pays for it by letting the duty drift when the machine's speed changes. Three a side is cheap either way, and [the first bracketed gate run would have refused itself on one](../docs/measurements/2026-08-01-202112-gate-m1-at-twenty-minutes.md). That is fixed for a hold's whole length, so `--solo-duration` buys nothing against it and only separate launches sample it; `--solo-repeats` is the knob. Each side reports its own spread beside the gap, and **a side that scatters further than the allowance refuses the run**, which is [`compare`'s rule about a baseline lending a judgement finer than itself](#comparing-two-ramps) arriving one level down.

**`--with-solo` takes the baseline on both sides**, because the baseline is the thing that moves. Two triples of solo holds ten minutes apart had means 8.5% apart where within either the spread was 2.8%, so one pass taken beforehand is a baseline from a machine that may have left. The two bracket the run, their gap is the drift control, and the reported figure is **solo over concurrent** — the same direction as the ramp's own column, so the number a reader compares against 2 is the one printed.

**A gap past the allowance produces no ratio at all**, rather than one with a caveat attached. On its first real run this refused itself: eight CPU-saturating sessions for 24 seconds left the machine 6.9% slower 16 seconds later, and a ratio taken across that would have been the afternoon. The allowance is [borrowed from a check that compares two finished ramps](#comparing-two-ramps) and cannot average anything, where a bracket averages its two baselines and so already corrects drift that is linear — what its gap must be small enough for is curvature, not sameness. **That framing turns out to be the wrong worry.** Repeating the shape three times [found no post-load deficit to shape at all, and found the baseline itself spanning 4.54% between fresh single-session holds](../docs/measurements/2026-08-01-173927-the-baseline-is-the-noisy-term.md) — where the allowance was calibrated on a ramp's rung repeated inside one ladder, which reproduces far better. So it is not mis-shaped for averaging; it is set from a quieter population than the one a bracket hands it.

**One condition comes back unanswered whatever you do, and it says so in a different word from the one that was merely waiting.** Replacement is `out_of_reach`: nothing in the daemon restarts a session that exited, so there is no replacement to time and more running would not help.

**Dropped output was in that sentence and is not any more.** The argument for it was that every other target hands this process the reading end of each session's output — which is what finds a gap in the ordinals — and under the daemon that end belongs to the daemon. That is true and it is the wrong question: a pipe blocks rather than dropping, so the only way a line goes missing is the daemon's reader giving up, and [the daemon counts that now](../coggyd/README.md). `failed_reads 0` answers the condition; anything else is the number of sessions whose tail is gone.

Work rate is `not_taken` **without `--with-solo`, and judged with it** — which is what the two words exist to keep apart. It is a ratio against the same workload held alone, so it was waiting on a second run rather than on a different daemon, and the flag above is that run. One word for both would have read as three impossibilities and the tractable one would have stopped being looked for. Neither word is a pass.

```
cargo run -p sessionbench -- ramp --daemon target/release/coggyd.exe -- <command>
```

**`--daemon` puts each rung under `coggyd`** instead of spawning its sessions here, which is what fills [the comparison set's row for ours](#what-we-measure-against). It composes with `--pty` rather than replacing it — the daemon decides who owns the sessions, the mode decides how their output is wired — and a daemon ramp correctly reports `pipe`, because that is still what the sessions are on. Two of the four conditions come back unmeasured, for the same reasons `hold` cannot answer its own two — **but the denominators differ and the coincidence of "two" hides it.** [`redline` is four conditions](#redline) and includes replacement; [gate M1 is three](../ROADMAP.md#m1--headless-daemon) and does not. A ramp loses dropped output and replacement out of four; a hold loses dropped output out of three, and takes work rate with `--with-solo`.

**The runs that wait on something live in [`scripts/`](scripts/) rather than in a note**, because a measurement that waits on a window may wait across sessions and a window that opens is not a window to type in. `throughput-sweep.ps1` earned its place the hard way: the same shape was written inline into `bench-out/` four times in one day, and each run took its script with it. `m1-hour.ps1` is the gate's hold, its length a parameter that defaults to twenty minutes because [this machine stops dead at forty-one](../docs/measurements/2026-08-01-202112-gate-m1-at-twenty-minutes.md); `bracket-calibration.ps1` is the five short holds that decide whether the bracket's allowance is asking the right question. **Neither waits for a quiet machine, and getting there took a day.** Both once asked for four `doctor` readings within 10% of each other; the calibration script was refused nineteen times without ever running. Background steadiness is not what threatens either run. `bracket-calibration.ps1` now asks for nothing but room for its own load and repeats its sequence instead, taking the verdict rule from [`exclusion-delta`](#running-it): spread across repeats against the effect, inconclusive when the first is larger. `m1-hour.ps1` keeps a precondition because seventy-six minutes is worth protecting, but checks **the quantity that decides** — two fresh solo holds, refused if they sit further apart than the allowance the run itself will be judged by. A repeat measures whether the machine held still; a precondition on something adjacent only assumes it.

**That check answers whether the machine is steady and not which machine it is**, and the difference is worth 72%. This box runs a solo at ~21.5 units/s rested and ~9.7 in a slower state, and the two probes taken inside the slow one agreed with each other to 0.3% — the precondition would have passed them. So the script now prints a note when the *level* says post-saturation and refuses nothing on it: the run is valid, it is a figure for a different machine, and refusing would have cancelled the run that found this. `thermal-probe.ps1` is the attempt to induce that state on purpose, kept for its negative result — three minutes of a hundred sessions does not do it, the duration is a parameter, and the script says so at the top so nobody re-derives it.

**`throughput-sweep.ps1` refuses one thing and measures around the rest.** It sweeps `--duty`, `--resident` or the session count with a reference hold between every pair, reads total throughput rather than a slowdown — so no solo baseline enters, and the machine's core count and unit time cancel between adjacent holds — and reports the drift across its references, which is the floor an effect has to clear. A session sweep also locates the bend and prints the redline as twice it, [which is what the relation says when read as a measurement](../docs/measurements/2026-08-02-204124-a-redline-without-a-solo-rung.md), and says so when too few points sit below the plateau for the rising slope to mean anything. The one refusal is the plug: [on battery the same command returns 7.8× less](../docs/measurements/2026-08-02-195840-the-same-command-on-battery.md) while producing an artifact that looks entirely healthy, and that is the failure a reader cannot be asked to catch.

**Three checks of the coupling live in [`examples/`](examples/)**, and they are the only automated evidence for what the daemon path does under a machine rather than in isolation: `hold_daemon` holds sessions and asserts on the effects — alive while the pipe is held, RSS attributed through the armed job, nothing left behind; `daemon_dies` kills the daemon mid-hold and requires the run to refuse itself, which before that guard was fifty-nine minutes of empty samples and no complaint; `arm_thrice` arms the tree three times in one process, as `--with-solo` does, because a silent fallback to a parent walk would attribute a ratio's two halves by different methods. Each needs `coggyd`'s binary, which is why none is a test — **`cargo test` does not build a sibling crate's executable, and a test that skips when it is missing is a test nobody knows is not running.** The cost of that choice is that `cargo test` compiles them and runs none, so they belong to the checklist before a daemon measurement rather than to the gate:

```
cargo build -p coggyd
cargo run -p sessionbench --example hold_daemon
cargo run -p sessionbench --example daemon_dies
cargo run -p sessionbench --example arm_thrice
```

**And `ping` is a clock, not a workload.** It is the obvious thing to hold a hundred of while checking the plumbing, and it emits one line a second whatever the machine is doing — so its unit count is elapsed time wearing a work rate's name. Three solo holds of it returned 23 units each, which reads as a machine that reproduces perfectly and is really a machine that was never asked. [The same three with `cpu-spin`](../workloads/cpu-spin/) gave 585, 590 and 606. Use it to check that sessions start and stop; use something that competes for a core to measure anything.

`${session}` in the workload's arguments expands to each session's own id, so a hundred sessions can be handed a hundred paths from one command line — [the contract wants each its own directory](../workloads/README.md#the-contract), and neither the workload nor the daemon may learn the other's naming to arrange it.

Add `--pty` to give the session a pseudoconsole instead of pipes. Running one workload both ways is the direct measurement behind [Decision 1](../docs/PLAN.md#four-core-decisions), since the difference between the two is one conhost per session, resident for as long as the session lives.

```
cargo run -p sessionbench -- exclusion-delta -- <command>
```

`exclusion-delta` runs one workload watched and then excluded, several times over, and reports what the exclusion bought. **It changes this machine's real-time protection for the length of each excluded half**, over a directory the benchmark created for that run, and removes the exclusion afterwards whether or not the run succeeded. The removal is verified rather than assumed, and a failure to remove is printed where it cannot be missed.

Halves run as adjacent pairs because a single comparison cannot separate the exclusion from whatever else the machine was doing, and every half is preceded by an idle baseline for the same reason. When the spread across pairs is wider than what separates them, the run says inconclusive rather than averaging noise into a confident number. Fresh directories throughout, since reusing one would credit the exclusion with the scanning cache's work — [rule 4](#keeping-it-honest).

**At one session it will tell you nothing**, which is [measured rather than warned about](../docs/measurements/2026-07-30-140251-exclusion-delta.md). The exclusion axis lives on the ramp: `ramp --exclude-scratch` holds one exclusion over the sessions' writes for a whole ladder, and two ladders compared by redline is the form that answers.

```
cargo run -p sessionbench -- ramp --hold 90 -- <command>
```

**And it holds the solo rung once more after that.** Every rate a ramp reports is read against one session measured once, in a single window, never repeated — and [`compare`](#comparing-two-ramps) uses that same figure as the fingerprint deciding whether two ramps may be set against each other. It was load-bearing twice and measured neither time. The solo check is what a baseline reproduces to under the run's own conditions, which is the floor on any difference two ramps can claim: a ramp whose solo rung moved 8% cannot lend it to a 5% judgement.

**Every ramp ends by holding one rung a second time** — the lowest saturated one — and reports how the two readings compare. It is the only control the ramp has on itself: each figure it produces assumes the machine at the last rung is the machine that was at the first, and the fit makes that assumption load-bearing, since averaging over rungs removes noise but carries drift straight through. A machine that slowed as the ladder ran would steepen the slope and report a ceiling too low with nothing anywhere to show it. One extra hold; `--skip-drift-check` turns it off when a ramp is being run for its shape rather than its number.

`ramp` climbs the ladder and produces the redline. Each rung holds its session count for the whole window — **replacing any session that finishes**, which is both what keeps the count honest and what makes the replacement condition measurable at all. A rung that let finished sessions stay finished would report the count it asked for while measuring a decaying one, and the machine gets easier as that happens, so the number would climb exactly when it should fall.

`--max-sessions` caps the climb. The full ladder reaches 200 and will take the machine with it for the duration.

Rungs are judged on all four conditions, so a run that cannot evaluate one does not print a smaller redline — it prints none.

## Running gate G0 on a configured machine

[G0](../ROADMAP.md#current-priority-m0--attribution) wants an as-is redline for a real generation session, and this machine has never had one to point at. The steps below are what it takes on a machine that does.

**Do not ramp the agent CLI itself.** Eight rungs at sixty seconds with thirty-odd sessions each is thousands of session-minutes of real inference, and the bill is the least of it — sessions that fail on rate limits mid-rung produce a redline describing the API. Measure the real session once, then ramp a synthetic workload shaped to match it.

1. **`doctor --strict`.** A run missing an axis is not a smaller result, it is a wrong one, and the Defender axis needs elevation.
2. **`observe` one real session, to completion.** Its cores figure is `d`, the share of wall time it spends computing; its RSS and output volume say whether either is anywhere near a budget. One session, one afternoon, no ramp.
3. **`ramp` `cpu-spin --duty <d>`**, sized to that figure. [How the session waits does not matter](../docs/measurements/2026-07-30-154348-duty-is-derivable.md) — proportional and fixed pauses give the same curve — so the synthetic stands in honestly for a session waiting on a model.
4. **Read the drift line before anything else.** Past a few percent the machine changed under the ladder and the redline is reading low.
5. **Check the answer against `2ηC/d`**, with `C` the machine's logical processors rather than the physical count the headline carries. The fitted slope is `η` and the report prints it. The two disagreeing means one of the assumptions behind the relation does not hold on that machine, which is worth more than either number.
6. **Record the pair, never the bare count**, with the hardware it came from.

Step 3 is where a heavier session would change the answer: `η` belongs to the workload as much as the machine, and `cpu-spin --resident` is the knob for it if the real session's footprint is far from 20 MiB.

## Running gate M1 on a machine that can pass it

[M1](../ROADMAP.md#m1--headless-daemon) asks for a hundred sessions held an hour, under 4 GB, within 2× of solo work rate, with nothing dropped. **This machine answers all three and fails two**, and both failures are arithmetic rather than daemon: a hundred sessions wanting `100 × d` cores from sixteen cannot come within 2× at the duty a real session has, and the box stops dead at forty-one minutes of that load.

So the gate needs a wider machine, and what travels is the sizing rather than the numbers.

1. **Hold a hundred sessions and read the slowdown. That is the whole test.** The condition is `slowdown ≤ 2`, the run reports it, and no model stands between the two. Sizing a machine in advance means predicting that number instead of measuring it, and the prediction needs `η` from the machine you do not have yet.
2. **If you must size before you buy: this box needs about 15% more cores.** Its slowdown is 2.301, and `C_required = C × slowdown ÷ 2` — which is what `N·d ÷ 2η` reduces to once `η` is the `N·d ÷ (C·s)` this machine's own run gave. So "nineteen logical processors" is *sixteen times 2.301 over two*, and it holds only if the wider machine has the same `η`. It is a starting estimate wearing the clothes of a requirement.
3. **Check the plug.** [The same command returns 7.8× less on battery](../docs/measurements/2026-08-02-195840-the-same-command-on-battery.md) with every other recorded figure identical, so a laptop measured unplugged is a different machine. `doctor` says which state it is in.
4. **Run the hour only if the machine survives it.** Sustained full-core load stopped this one at forty-one minutes with no bugcheck. Hold twenty minutes first; if that completes, the hour is a question about the daemon rather than about the hardware.
5. **Read the drift control before the verdict.** `hold --with-solo` brackets the run and refuses a ratio when its two baselines disagree by more than the machine makes while being itself.
6. **State the duty the run actually held**, not the one it asked for. `--duty` delivers a stated duty on any machine; a fixed `--wait-ms` does not, and [calibrated warm and run cold it delivered 0.172 where 0.271 was asked for](../docs/measurements/2026-08-01-210316-the-wait-mechanism-cancels.md).

**What does not need re-deciding:** dropped output is answered by the daemon's own `failed_reads` on any machine, since a pipe blocks rather than dropping; and replacement stays out of reach until something restarts a session that exited.

## What we measure against

A benchmark that only measures `coggyd` is marketing. At minimum, these run on identical hardware.

| Target | Role | Reachable |
|---|---|---|
| A pseudoconsole per session | The as-is baseline — what a terminal gives a session today | [measured](../docs/measurements/2026-07-30-101141-conhost-and-defender.md) |
| Pipes, no pseudoconsole | The floor, and what the daemon intends to default to | [measured](../docs/measurements/2026-07-30-120002-first-redlines.md) |
| pwsh 7 against cmd against the workload alone | Control that isolates what a shell wrapper costs | [measured](../docs/measurements/2026-07-31-141334-the-shell-costs-teardown.md) — same redline three times, 361× the teardown |
| coggyd | Ours. **Added only after M1** | `ramp --daemon <path>` runs — ten rungs of `ping` gave 11 processes and 62.14 MiB at ten sessions. What the row still wants is a ladder on a quiet machine. **It measures two of the four conditions**, where `hold --daemon` reaches three — for the reason below |
| Windows Terminal, WezTerm, Alacritty, wmux | What an emulator costs to host N sessions | a different instrument · M4 |

**Our own row will be a partial, and saying so is the point.** Every other row spawns the sessions itself and holds the reading end of each one's output, which is what lets it count units and notice a gap in them. Under `coggyd` the daemon holds that end: a session's output reaches its scrollback and stops there. Work rate survives with a different numerator — the daemon reports how many lines it read — and RSS is untouched, since the job this process creates still contains the daemon and everything under it. **Dropped output does not survive *this* route.** Gaps are found by watching ordinals in a session's stream, nobody here holds what the workload emitted, and there is nothing to subtract from. **But the route is not the condition**, and `hold --daemon` reaches it another way: a pipe blocks rather than dropping, so the only way a line goes missing is the daemon's reader giving up, and [the daemon counts that](../coggyd/README.md#output). What a ramp is missing is the wiring, not the number — `dropped_units` still returns `None` here because a count of sessions whose tail is gone is not a count of units, and putting one in the other's field is how a quantity ends up in the wrong unit. Replacement does not survive by either route: nothing in the daemon restarts a session that exited.

So the pair that lands in this row will carry *conditions 3 and 4 not exercised* beside it. [The gate fails a run producing one aggregate figure plus a guess at the cause](../ROADMAP.md#current-priority-m0--attribution) — a labelled partial is not that, and an unlabelled one would be.

**These do not need retaking against an engine-shaped session, and the arithmetic says why.** What separates a pseudoconsole from pipes is one conhost per session, 8.7 MiB, and that is a property of the wiring rather than of the workload. Against a synthetic session holding 20 MiB it came to 3.9% of the budget; against [a real one holding 580 MiB of agent CLI](../docs/measurements/2026-07-31-050322-the-agent-side-of-a-session.md) and [1.87 GiB of engine](../docs/measurements/2026-07-31-045604-an-error-bar-for-the-engine.md) it is 0.35%. Retaking the comparison would move [Decision 1](../docs/PLAN.md#four-core-decisions) further in the direction it already went.

**The shell row came back saying the wrapper is not where the cost is either.** Bare, `cmd` and `pwsh` gave 27, 26 and 26 sessions — three fitted crossings inside 3.6%, because a shell sleeps while its child computes and the condition that binds is work rate. What it does cost is 9 MiB for `cmd` and 70 for `pwsh`, and **361× the teardown**: every wrapped session leaves exactly one straggler, since killing the process that was spawned does not kill the session. Shell *startup* cost, which this row was originally written to isolate, is still unmeasured — these sessions never exited.

**One thing about it does change.** An engine session does not carry one conhost — [a cook spawned nineteen and a build three](../docs/measurements/2026-07-31-043951-the-editor-is-the-cost.md), all from a session started on pipes, because the tools it runs open consoles of their own. So the count is a property of what a session runs and not only of how it was spawned, which is the part of Decision 1 that survives.

**The last row is a different question, and finding that out was worth the row.** This instrument spawns the process and holds the reading end of its output; that is what lets it count units and notice a gap in them. A terminal emulator owns its own pseudoconsole and draws to a window, so there is no reading end to hold and no work rate to count — and `wt.exe new-tab` returns to us immediately while the session it opened belongs to a process that was already running. Attribution fails for the same reason: the job object is joined by *this* process before it spawns anything, and membership is inherited downward, so a program that started before us was never going to be in it.

Which is not a defect in either. The question this instrument answers is what a session costs to exist — and a session costs the same whether WezTerm or Windows Terminal is drawing it. What an emulator costs to *host* a hundred of them is a real question and a separate one: it measures one process's rendering rather than a hundred processes' residency, it needs a way to drive a UI into opening sessions, and it belongs with the axis that has a screen to compare against.

## Comparing two ramps

A drift control tests one ladder against itself. Every comparison here spans two, and until `compare` nothing tested that the two saw the same machine.

```
sessionbench compare <left>/ramp.json <right>/ramp.json
```

**The solo rung is the fingerprint** — one session, no contention, the same work — and two ramps whose solo rungs disagree by more than 5% are measuring different afternoons. It exits non-zero when they do, so a script that pairs ramps fails rather than publishing a difference that is really the machine moving.

**Recalibrating that 5% takes three ramps and no new code.** Run the same ramp three times back to back on a quiet machine, then compare each pair:

```
sessionbench ramp --label calib-1 --hold 60 --max-sessions 60 -- <workload>   # and calib-2, calib-3
sessionbench compare bench-out/<calib-1>/ramp.json bench-out/<calib-2>/ramp.json
```

Each ramp's own solo check gives the **noise** — what a baseline reproduces to under one run's conditions — and the gaps between the three give **noise plus whatever the machine did in between**. The allowance belongs above the first and around the second, and until both are measured separately it is a tolerance rather than a figure.

It refuses [the pipes-against-pseudoconsole pair that prompted it](../docs/measurements/2026-07-31-151821-the-pseudoconsole-row.md) at 51.6%, and admits the shell-control trio at 3.1%. That 5% is calibrated on three points and is the weakest part of it.

## Keeping it honest

Tuning `coggyd` against these results makes the gate grade a bucket it drew itself. Instrument and subject coming from the same hands is the classic failure here.

1. **Freeze the as-is baseline at M0.** Measure it before `coggyd` exists and never remeasure. If hardware changes, every target gets remeasured together. [Frozen on 2026-07-31 at nine sessions, limited by RSS](../docs/measurements/2026-07-31-150258-g0-frozen.md).
2. **Workloads know nothing about `coggyd`.** Payloads are pure stdout generators and real agent CLI sessions. A workload that takes a COGGY-specific path is banned.
3. **Distrust one axis improving while five stay flat.** redline is a conjunction of four conditions, so optimizing a single axis cannot raise it. If it rose anyway, the cost moved somewhere we are not looking.
4. **Give every run a directory Defender has not seen.** Real-time scanning costs far less the second time a file is written, so re-running a workload over the same paths measures the cache rather than the workload. Two otherwise identical runs differed threefold on this axis before the rule existed, and the second one looked like the improvement.
5. **Watch the instrument's own cost, which every run records.** A scaling benchmark's one undetectable failure is the observer becoming the bottleneck, and it does not announce itself — it arrives looking like the machine collapsing. Twenty-five sessions that saturate every core starve the sampler for fifteen seconds a tick, while twenty-five that yield cost it fifty-six milliseconds at the same process count and the same resident memory. A rung the sampler could not keep up with is reported as inconclusive and stops the ladder, because a rung that could not be read has not failed.

6. **Distrust a rung that did not hold the machine.** The five above guard against the instrument influencing the subject; this one is about a third party taking the cores while a rung is being timed. A rung interrupted for part of its window reads as a rung that was slower, which a ladder calls saturation — so a redline can be set by something that was not the sessions. It is measured rather than feared: three twenty-minute holds of a hundred sessions each, and one of them [lost 1.173 cores of a 15.39 median where the others lost 0.061 and 0.052](../docs/measurements/2026-08-03-024222-the-footprint-never-mattered.md), across two multi-minute episodes that its mean absorbed and its report never mentioned. Every hold and every rung now carries `median`, `mean` and `lost` cores, and the rung line flags a loss past a tenth of a core. **Read the median; a mean is where an interruption hides.**

**Those rules are what buy credibility, not the repo boundary** — so this stays inside the COGGY repo. The evidence agrees: `alacritty/vtebench` split out and has not moved since January 2025 while Alacritty ships continuously, whereas Ghostty keeps its benchmarks in `src/benchmark/` and is the healthiest of the three. Splitting early buys a second CI and a sync problem.

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

Cores in the headline are **physical**, because the claim beside them is how many sessions fit and hyperthreads flatter that number without adding a core's worth of capacity. **The cores column in the rungs is logical**, since that is the unit a process's CPU usage arrives in — one busy hardware thread reads as 1.0. The two differ on any machine with hyperthreading, and [the duty relation](../docs/measurements/2026-07-30-154348-duty-is-derivable.md#the-derivation) takes the logical count, matching what it is compared against.

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
