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

Two consequences the code carries:

- **A session is alive while its job is, not while its root is.** Asking the root is the natural implementation and the measured-wrong one: a supervisor reading it would call fifty occupied slots free and hand them out again.
- **`Session::spawn` kills on the error path.** `std::process::Child` does not kill on drop, so returning an assignment failure straight through would leave a running session with no owner — the exact straggler this type exists against, produced by the type itself.

## Output

Both streams are piped and drained to end-of-file, because an undrained session fills its pipe and blocks. That reads as a slow session rather than a stuck one, and it is [the failure condition 3 exists to catch](../sessionbench/README.md#redline).

**Two different things get called dropped output and the gate means one of them.** A line the daemon never read is a gate failure and shows up as a gap in [the workload contract's ordinals](../workloads/README.md#the-contract). A line the scrollback aged out is policy. `Scrollback` counts `read`, `evicted` and `truncated` separately so neither can hide behind the other.

**The per-line cap is at the reader, not at the buffer.** `read_until` grows until it meets a newline, so a session emitting a gigabyte without one takes the daemon down before anything downstream can trim it. Capping what is kept is not a memory ceiling; refusing to hold it is. Bytes past the cap are consumed rather than left in the pipe, since leaving them would stall the session.

**Its drain is not shared with `sessionbench`, which drains pipes too.** The instrument and its subject keeping separate implementations is what stops the benchmark measuring its own reader.

## Identity

`COGGY_SESSION_ID` is injected into every session, following the `CMUX_WORKSPACE_ID` precedent [the contracts bind us to](../docs/PLAN.md#fixed-contracts).

**It is a counter, never the pid.** Windows reuses process ids, and the instrument already carries a note about what that costs. A supervisor keyed on a pid would hand a dead session's slot to whatever inherited the number and keep counting. `id()` and `pid()` differ in type as well as name, which caught a call site asking the kernel about a session using the wrong one.

## Where the boundary is

**The daemon knows only whether a session is alive.** Retry, repair and verification verdicts belong to the harness and never enter here — [a fixed contract](../docs/PLAN.md#fixed-contracts), so `Status` has no room to grow one. It carries `Running`, `Exited`, and `Unknown`; the last is separate because reporting a session gone on the strength of a failed query is how a supervisor loses one.

Nothing in the daemon knows an engine exists. [Engine adapters are M5](../ROADMAP.md#m5--engine-surfaces-exploratory) and live in a bridge, not here.

## What is not built

The CLI, the socket API, the resource governor and the audit surface are [target state](../ROADMAP.md#m1--headless-daemon). What exists is the part [G0 said mattered first](../docs/measurements/2026-07-31-150258-g0-frozen.md): a session's lifetime, its output, and its identity.

**Known and open:** a session is assigned to its job after spawning rather than before, which leaves a window where it runs unowned. Closing it needs `CREATE_SUSPENDED` and a resume, which needs unsafe, and [the workspace forbids that](../CLAUDE.md). The window is microseconds wide and the session cannot have built a tree inside it.
