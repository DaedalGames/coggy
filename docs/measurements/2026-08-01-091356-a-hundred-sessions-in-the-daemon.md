# A hundred sessions in the daemon · 2026-08-01 09:13:56

[Gate M1](../../ROADMAP.md#m1--headless-daemon) asks for a hundred sessions held for an hour at total RSS under 4GB. `coggyd` can now be started, so the memory half of that gate is answerable — RSS does not care that the machine was at 24% background, which the work-rate half would.

This is the first number the daemon has produced about itself.

## Holding them

```
coggyd --sessions 100 -- ping -n 600 127.0.0.1
```

| | |
|---|---|
| Sessions alive after 25 s | **100** |
| Daemon | **12.2 MiB** |
| Sessions, summed | 575.8 MiB |
| **Total against a 4 GB budget** | **588.0 MiB** |

**The daemon is 2% of what it holds.** [The gate's own note says it measures the daemon rather than capacity](../../ROADMAP.md#m1--headless-daemon), and 12.2 MiB against a hundred sessions is what that was asking about.

## Killing the daemon reclaimed all hundred trees

The probe ended with `Stop-Process -Force` rather than by closing stdin — the ungraceful path, where no destructor runs and nothing gets a chance to tidy up.

**Zero `ping` processes survived.**

That is the [job-per-session design](../../coggyd/README.md#owning-one) working where it matters most. `KILL_ON_JOB_CLOSE` is a property of the job, and the last handle to a job closes when the process holding it dies, however it dies. A supervisor that reclaimed sessions in its own shutdown path would have left a hundred behind here.

## The two figures disagree, and the reason is the interesting part

| | |
|---|---|
| Sessions, summed working set | 575.8 MiB |
| Machine free memory, moved | **174 MiB** |

**3.3× apart.** Working set counts a shared page once per process, and a hundred copies of one executable share nearly all of theirs — the image, the loaded DLLs, the read-only data. Summing them counts that memory a hundred times; the machine counts it once.

**This reaches the frozen ceiling.** [Nine sessions](2026-07-31-150258-g0-frozen.md) is a per-session figure multiplied by a count, so if sessions share pages the product overstates and the real ceiling is higher.

What keeps it from moving is that the same pair of routes was read during M0 and agreed. A ten-session `cpu-spin` rung moved the job object's RSS by **1.79 GiB** while machine free memory moved **1.69** — 6% apart, and [the rule that came out of it](../../CLAUDE.md) is why both were being watched.

**So the axis is not how identical the sessions are**, which was the first explanation and does not survive that rung: ten `cpu-spin` processes are as identical as a hundred `ping`s and showed no such gap. **It is how much of a session's footprint is shared image rather than private data.** `cpu-spin` touches 80 MiB of its own per session and its image is a rounding error beside that; `ping` holds about 5.8 MiB that is mostly the executable and its DLLs, which every copy shares.

That reading makes the ceiling firmer rather than weaker. [A generation session's 2.39 GiB](2026-07-31-150258-g0-frozen.md) is engine content and agent context — memory each session faulted in for itself, which no other session can share.

So the ceiling stands, and this run says what would move it: a workload whose sessions hold little beyond the image they were started from.

## What this does not claim

- **It is not the gate.** The gate wants an hour and this held for twenty-five seconds. Nothing here says the figure is stable over that.
- **`ping` is not a session.** It holds about 5.8 MiB and does nothing, where [a generation session holds 2.39 GiB](2026-07-31-150258-g0-frozen.md). This measures the daemon's own cost at a hundred sessions, not what a hundred real ones would take.
- **The work-rate half is untouched.** These sessions do no work, so nothing here speaks to per-session throughput, and the machine was too loud to ask.
- **One run.** No repeat, no drift control — `coggyd` has no instrument of its own and [the comparison set's row for it](../../sessionbench/README.md#what-we-measure-against) is still empty.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 24% at the start, 6.31 GiB free |
| Daemon | `coggyd` 0.0.0 at commit `555c083`, release build |
| Workload | `ping -n 600 127.0.0.1`, one process per session, no shell |
| Sampling | one reading at 25 s, `Get-Process` working sets and `Win32_OperatingSystem` free memory |
