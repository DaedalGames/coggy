# The build lock reaches cooking, and this machine ran out of room · 2026-07-31 04:28:02

A cooking session holds 1.93 GiB steady and peaks at 5.02, so ten of them come to 19 GiB on the medians and 50 on the peaks. Whether those peaks land together decides whether a governor counts sessions or schedules them. Ramping concurrent cooks was meant to settle it. Two things stopped it, and both are worth more than the number would have been.

## Cooking takes the build lock on the way in

The first line of every cook's log, from a workload that calls `UnrealEditor-Cmd.exe` directly and never mentions `Build.bat`:

```
Build.bat is already running, waiting for existing script to terminate...
```

**The editor checks its target is current before cooking**, and that check goes through the same per-installation lock [a build takes](2026-07-31-034150-unreal-builds-serialise.md). Ten sessions queue on it, none reaches a cook inside a four-minute rung, and the ramp reads the retry loop as a workload running at 13.76 units a second.

This does not overturn [the earlier finding that two cooks run side by side](2026-07-31-040348-cooking-is-the-governed-state.md) — it locates it. **The lock is at startup, not throughout.** Two sessions alternate through it and then cook concurrently for minutes; ten never get past it.

`-nocompile -skipbuild` keeps the cook out of that path, which is what the workload now passes.

## And the machine had 8.7 GiB free

Of 31.4 GiB, 22.7 were held by everything else on this desktop. Ten concurrent cooks need 19 GiB of steady residency before their peaks are considered, so **this measurement could not have succeeded whatever Unreal did**.

That is a condition on the reading rather than a fact about the engine, and it is the first time [the background figure `doctor` now reports](../../sessionbench/README.md#running-it) has ruled a measurement out in advance rather than explained one afterwards.

**A census an hour later says what holds it, and it is not the engine or the instrument.** Of 21.1 GiB spoken for, browsers and chat clients hold about seven — Chrome 3.0, VS Code 1.8, rust-analyzer 0.9, Slack 0.6, Discord 0.5, a Steam helper 0.5. Every process's working set together comes to roughly 12 GiB, and the remainder is kernel and pool that no process shows.

**The free figure is not hiding a reclaimable cache, which was worth checking rather than assuming.** `FreePhysicalMemory` reads 10.30 GiB and `\Memory\Available MBytes` reads 10.28 — they agree to 0.2%, so the free list already carries the standby cache and there is no ninth gigabyte to recover. `sessionbench` reads the right one of the two either way, through `sysinfo`'s `available_memory`.

So the shortfall is user applications rather than anything this project put there, and closing it is the desktop owner's call rather than the instrument's. The requirement has also grown: [a session is the engine plus the agent driving it](2026-07-31-054657-the-driven-duty.md), which puts ten of them at **23.9 GiB** rather than 19.

## What is settled and what is not

| | |
|---|---|
| Cooks contend at startup | **Measured** — the lock message, from ten sessions that never cooked |
| Two cooks run concurrently past that point | [Measured](2026-07-31-040348-cooking-is-the-governed-state.md) — two editors for a full test |
| Whether peaks align | [Settled twenty minutes later](2026-07-31-043156-cook-peaks-scatter.md), by four sessions rather than ten — they scatter |
| Ten cooks specifically | **Unmeasured**, and the shared shader cache is the reason it might differ from four. Blocked on 23.9 GiB against 10.3 available, of which about seven sit in browsers and chat clients |

The ten-session ramp turned out not to be what the question needed. Four sessions answered it on the same machine minutes afterwards, which is [the lesson this record was written one step too early to know](../../CLAUDE.md): shrink the measurement before blaming the machine.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable, 8.7 free · Windows 11 (26200) |
| Engine | Unreal Engine 5.8, launcher install |
| Workload | `TP_BlankBP` cooked for Windows, one private project per session |
| sessionbench | 0.0.0 at commit `2ccd280`, release build |
| Ramp | 240 s holds, rungs 1 and 10, stopped during the second |

Teardown left nothing behind: no `cmd`, `UnrealEditor-Cmd`, `UnrealBuildTool`, `cl`, `link` or `ShaderCompileWorker` survived the kill, which is the check [the working rules ask for](../../CLAUDE.md) after a ramp is stopped rather than finished.
