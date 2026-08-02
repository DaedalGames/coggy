# coggyd

> The session supervisor. It owns sessions and knows nothing about what they do.

**This document owns the daemon: what a session is, what owning one means, and where its boundary is.** [README](../README.md#documentation) maps the rest.

## What a session is

**One process this daemon spawns, and everything that process spawns in turn.**

Not a definition chosen for tidiness. [G0 measured that killing the process you spawned does not end a session](../docs/measurements/2026-07-31-141334-the-shell-costs-teardown.md): fifty sessions wrapped in a shell left exactly fifty stragglers, and the teardown ran 361× slower for it. A pseudoconsole shows the same asymmetry from the other side — [it belongs to whoever created it rather than to the session it serves](../docs/measurements/2026-07-30-101141-conhost-and-defender.md).

So the tree is the unit, and [the frozen ceiling divides by it](../docs/measurements/2026-07-31-150258-g0-frozen.md): an agent CLI at 0.52 GiB plus the engine it runs at 1.87 gives nine sessions on this machine. Read the two halves as separate sessions and the same measurements give 11.7.

## Owning one

**A job object per session, carrying `KILL_ON_JOB_CLOSE`.** Membership is inherited downward, so everything the session starts lands in the job without being told to, and releasing the handle takes the whole tree. Nothing here calls `TerminateJobObject`.

**Per session is the load-bearing word.** `sessionbench` creates one job and joins it with `assign_current_process`, which answers *who belongs to this measurement* and cannot answer *end this one* — terminating that job would take the benchmark with it. Its 361× teardown follows from that shape rather than from Windows.

Consequences the code carries:

- **A session is alive while its job is, not while its root is.** Asking the root is the natural implementation and the measured-wrong one: a supervisor reading it would call fifty occupied slots free and hand them out again.
- **`Session::spawn` kills on the error path.** `std::process::Child` does not kill on drop, so returning an assignment failure straight through would leave a running session with no owner — the exact straggler this type exists against, produced by the type itself.

## Output

Both streams are piped and drained to end-of-file, because an undrained session fills its pipe and blocks. That reads as a slow session rather than a stuck one, and it is [the failure condition 3 exists to catch](../sessionbench/README.md#redline).

**Two different things get called dropped output and the gate means one of them.** A line the daemon never read is a gate failure and shows up as a gap in [the workload contract's ordinals](../workloads/README.md#the-contract). A line the scrollback aged out is policy. `Scrollback` counts `read`, `evicted` and `truncated` separately so neither can hide behind the other.

**All three leave the process, and for a while none of them did.** The separation sat in the library and the binary printed only held and running, so [an hour-long hold could not be asked about a third of the gate](../docs/measurements/2026-08-01-103225-an-hour-of-a-hundred-sessions.md) — no length of run produces a number nothing reports. `read` is the term a benchmark subtracts from what its workload emitted, since the daemon knows what it read and can never know what it missed.

**The per-line cap is at the reader, not at the buffer.** `read_until` grows until it meets a newline, so a session emitting a gigabyte without one takes the daemon down before anything downstream can trim it. Capping what is kept is not a memory ceiling; refusing to hold it is. Bytes past the cap are consumed rather than left in the pipe, since leaving them would stall the session.

**And the scrollback holds a byte budget as well as a line count, because the count alone bounds nothing here.** Every grid terminal caps by lines — tmux, WezTerm, Alacritty — and is right to: their lines are as wide as the terminal, so counting lines counts bytes. Pipes have no width, and this inherited the convention without the property. Multiplied out, the line count and the per-line cap reached [13.1 GB across a hundred sessions against a gate written for four](../docs/measurements/2026-08-01-103225-an-hour-of-a-hundred-sessions.md). The byte budget is [Ghostty's shape](https://ghostty.org/docs/config/reference#scrollback-limit), and the evidence it is needed is [tmux's, at about 48 GB](https://github.com/tmux/tmux/issues/4859). The line count stays, with a smaller job: bounding the fixed per-line cost that a budget of bytes would let a flood of empty lines run up.

**Its drain is not shared with `sessionbench`, which drains pipes too.** The instrument and its subject keeping separate implementations is what stops the benchmark measuring its own reader.

## Identity

`COGGY_SESSION_ID` is injected into every session, following the `CMUX_WORKSPACE_ID` precedent [the contracts bind us to](../docs/PLAN.md#fixed-contracts).

**And `${session}` in a session's arguments expands to the same number**, because reading the variable is not always available to the thing being started. A caller wanting a hundred sessions to write in a hundred directories cannot ask the program to work it out — [a workload that takes a COGGY-specific path stops being evidence](../workloads/README.md#the-contract) — and the daemon must not know what that caller calls its scratch. Expanding a placeholder leaves the caller writing an ordinary path and the program receiving one, neither having learned about the other.

**A name that is not `session` fails the spawn rather than passing through.** Left as written it would hand every session the same path, silently, which is the failure the placeholder exists to prevent. Braces are required and there is no escape character, so a Windows path needs no special handling.

**It is a counter, never the pid.** Windows reuses process ids, and the instrument already carries a note about what that costs. A supervisor keyed on a pid would hand a dead session's slot to whatever inherited the number and keep counting. `id()` and `pid()` differ in type as well as name, which caught a call site asking the kernel about a session using the wrong one.

## Holding many

`Pool` keeps sessions by identity and answers two numbers that are easy to conflate. **Held is not running.** A finished session still occupies a slot and its memory until it is reaped, and a supervisor that reported one figure would admit against a seat it had not freed — on a machine [whose ceiling is nine](../docs/measurements/2026-07-31-150258-g0-frozen.md).

`reap()` frees only what has actually gone, and **leaves `Unknown` alone**: a job that could not be read says nothing about whether its session is running, and freeing a seat on that reading is how a supervisor sells one twice.

Admission against a ceiling is [M3](../ROADMAP.md#m3--resource-governor). This counts; it does not judge.

## The process

```
coggyd --sessions 3 -- ping -n 40 127.0.0.1
holding 3 session(s); stdin closes to stop
held 3 · running 3 · read 12 · bytes 564 · evicted 0 · truncated 0 · failed_reads 0
cleared
```

It holds its pool until stdin reaches end-of-file, then clears it. End-of-file rather than a signal because that needs no extra crate and no console — [a console-dependent wait was measured returning instantly without one](../docs/measurements/2026-07-31-035111-between-builds.md).

**The stop condition and the report clock are separate, and were not.** One loop read stdin and checked the interval after each read, which ties how often the daemon speaks to how much its caller types: [an hour-long hold whose holder wrote nothing produced no periodic line at all](../docs/measurements/2026-08-01-103225-an-hour-of-a-hundred-sessions.md). That looked like a choice the measurement made until something needed the line — a benchmark scraping `read` for a unit count would have seen a hundred sessions as silent.

**This exists so the benchmark has something to start**, not as an API. [The comparison set holds a row for `coggyd`](../sessionbench/README.md#what-we-measure-against) and could not fill it against a library. The verbs stay unwritten because [M2 derives them backward from the calls a harness makes](../ROADMAP.md#m2--harness-contract).

Printing held and running separately earned itself on the first run: a smoke test showed `held 3 · running 2`, because all three sessions had been given the same `waitfor` signal name. One number would have read as fine.

## Where the boundary is

**The daemon knows only whether a session is alive.** Retry, repair and verification verdicts belong to the harness and never enter here — [a fixed contract](../docs/PLAN.md#fixed-contracts), so `Status` has no room to grow one. It carries `Running`, `Exited`, and `Unknown`; the last is separate because reporting a session gone on the strength of a failed query is how a supervisor loses one.

Nothing in the daemon knows an engine exists. [Engine adapters are M5](../ROADMAP.md#m5--engine-surfaces-exploratory) and live in a bridge, not here.

## What is not built

The cmux-compatible CLI, the socket API, the resource governor and the audit surface are [target state](../ROADMAP.md#m1--headless-daemon). The binary above is a process that owns sessions, not that CLI. What exists is the part [G0 said mattered first](../docs/measurements/2026-07-31-150258-g0-frozen.md): a session's lifetime, its output, its identity, a pool that counts them, and a process that holds one.

**Known and open:** a session is assigned to its job after spawning rather than before, which leaves a window where it runs unowned. Closing it needs `CREATE_SUSPENDED` and a resume, which needs unsafe, and [the workspace forbids that](../CLAUDE.md). The window is microseconds wide and the session cannot have built a tree inside it.
