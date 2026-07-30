# The build lock reaches cooking, and this machine ran out of room · 2026-07-31 04:28:02

A cooking session holds 1.93 GiB steady and peaks at 5.02, so ten of them come to 19 GiB on the medians and 50 on the peaks. Whether those peaks land together is what puts the engine redline at eleven or at six, and it decides whether a governor counts sessions or schedules them. Ramping concurrent cooks was meant to settle it. Two things stopped it, and both are worth more than the number would have been.

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

## What is settled and what is not

| | |
|---|---|
| Cooks contend at startup | **Measured** — the lock message, from ten sessions that never cooked |
| Two cooks run concurrently past that point | [Measured](2026-07-31-040348-cooking-is-the-governed-state.md) — two editors for a full test |
| Ten cooks, peaks aligned or scattered | **Unmeasured**, and needs about 25 GiB free rather than 8.7 |
| Whether the redline is eleven or six | **Unmeasured**, and it is the difference between a governor that counts and one that schedules |

The workload is fixed and the ramp is one command. What it needs is a machine with its memory free — the same condition [G0 has been waiting on](../../ROADMAP.md#current-priority-m0--attribution), arriving from a different direction.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable, 8.7 free · Windows 11 (26200) |
| Engine | Unreal Engine 5.8, launcher install |
| Workload | `TP_BlankBP` cooked for Windows, one private project per session |
| sessionbench | 0.0.0 at commit `2ccd280`, release build |
| Ramp | 240 s holds, rungs 1 and 10, stopped during the second |

Teardown left nothing behind: no `cmd`, `UnrealEditor-Cmd`, `UnrealBuildTool`, `cl`, `link` or `ShaderCompileWorker` survived the kill, which is the check [the working rules ask for](../../CLAUDE.md) after a ramp is stopped rather than finished.
