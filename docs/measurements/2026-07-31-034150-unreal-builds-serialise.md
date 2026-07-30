# Concurrent Unreal builds serialise, and six defects on the way to finding out · 2026-07-31 03:41:50

A ramp of ten Unreal sessions reads as one. Not because the machine ran out of anything — **a single engine installation compiles one thing at a time**, so adding sessions adds processes and buys no build throughput.

## The reading

Ten sessions each building their own copy of `TP_Blank`:

| | 1 session | 10 sessions |
|---|---|---|
| Total RSS | 1.89 GiB | **1.89 GiB** |
| Processes | 11 | 18 |
| `cl.exe` alive at once | 1 | **1** |

Ten sessions, one compiler. The extra seven processes are nine loops spinning on a build that never starts.

## Two locks, one bypassable

`Build.bat` says so itself, in a log that had to be written outside the scratch directory to survive teardown:

```
Build.bat is already running, waiting for existing script to terminate...
```

Calling `UnrealBuildTool.exe` directly skips that, and two build tools then run side by side for the full two minutes of a test — **but `cl.exe` still never exceeds one**, and the second session's log stays empty while it waits.

| Lock | Where | Bypassable |
|---|---|---|
| Script guard | `Build.bat` | Yes, by invoking `UnrealBuildTool.exe` |
| Action execution | UnrealBuildTool, per engine installation | No |

So `-waitmutex` was never the cause. Dropping it changed nothing that mattered, and an earlier reading of two concurrent `cl.exe` was one build's own parallelism rather than two sessions overlapping.

## What this means, and where the answer already is

**A machine holding N generation sessions can build one of them at a time.** Memory and cores are not what stops the second; the engine is. Every capacity figure derived from a building session is therefore about a queue, not a crowd.

Epic ships the answer, and it is visible in the first build log this project ever took: `Total time in XGE executor` and a trace written to `Trace.uba`. **Unreal Build Accelerator** distributes build actions across machines, and `UbaCoordinatorHorde` ships in the engine's own binaries. Whatever COGGY does about concurrent builds, [it consumes that rather than routing around it](../../CLAUDE.md).

The alternative — one engine installation per session — costs 30 GiB each and is not an alternative.

## Six defects, and which half were ours

Getting to a three-line reading took six corrections. Four were in the workload and two were in the instrument, and the instrument's two are the ones worth keeping.

| | Defect | Symptom | Read as |
|---|---|---|---|
| 1 | Sessions shared one project directory | 10 sessions ≈ 1 | "the engine blocks parallelism" |
| 2 | `-waitmutex` passed | one `cl.exe` | "UnrealBuildTool serialises" |
| 3 | `.bat` written with bare LF | 3 builds a second | "the build is very fast" |
| 4 | Build output sent to `nul` | 1 to 3 invisible | — |
| 5 | **Scratch path was relative** | every build failed in 60 ms | "the workload is fast" |
| 6 | **Ramp gave every session one scratch** | 10 sessions ≈ 1 | "the engine blocks parallelism" |

Note that 1, 2 and 6 all produced the same reading, and only one of them was true.

**Defect 4 is the one that made the rest expensive.** A silenced build that fails in sixty milliseconds is indistinguishable from one that succeeds, and three ramps were spent before the diagnostic channel was reopened.

**Defects 5 and 6 were the instrument breaking its own [workload contract](../../workloads/README.md#the-contract).** Rule 2 forbids shared paths; the ramp handed every session in a rung the same directory. The contract also implies a usable scratch path, and the value came from a relative `--out` default. Neither showed up in three synthetic workloads, because all three open files inside their own process and name them uniquely — **a contract only exercised by workloads that cannot violate it has not been exercised.**

## What is still unmeasured

Whether sessions that are *not* building concurrently — waiting on a model, editing files, running tests — stack the way the synthetic workloads did. That is the common case for a generation session, and this record says nothing about it. The build is the peak, and the peak turns out to be a queue of one.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| Engine | Unreal Engine 5.8, launcher install · Visual Studio 18 with MSVC 14.51 |
| Project | `Templates\TP_Blank`, editor target, Win64 Development, one copy per session |
| sessionbench | 0.0.0 at commit `6cb1e62`, release build |
| Holds | 240 s per rung, first 30 s unmeasured |
| Defender | real-time protection on, no exclusions |
