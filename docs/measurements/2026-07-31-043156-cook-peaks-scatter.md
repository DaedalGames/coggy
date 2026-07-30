# Concurrent cook peaks scatter, so the redline is eleven rather than six · 2026-07-31 04:31:56

A cooking session holds 1.93 GiB steady and peaks at 5.02. Whether concurrent sessions reach their peaks together is the difference between a governor that counts sessions and one that has to schedule them, and none of the four redline conditions asks it — alignment is a property of a crowd and cannot be read from one session.

## Four cooks, forty seconds

| Elapsed | Editors | Total RSS | Free RAM |
|---|---|---|---|
| 10 s | 4 | 5.32 GiB | 3.5 GiB |
| 20 s | 4 | **7.76 GiB** | 3.7 GiB |
| 30 s | 4 | 2.50 GiB | 8.4 GiB |
| 40 s | 4 | 6.95 GiB | 4.4 GiB |

**Four aligned peaks would be 20 GiB. The busiest sample is 7.76.**

Better than that, the busiest sample divides to **1.94 GiB a session** — the single-session *steady* figure of 1.93, not its peak. And the total swinging from 2.50 to 7.76 within twenty seconds is sessions sitting in different phases at every moment, which is what scattering looks like.

## Which sets the redline at eleven

Concurrent cooks sum to their steady figure rather than their peak, so the memory ceiling is `21.97 ÷ 1.93` — **about eleven sessions**, and the six that aligned peaks would have implied does not happen.

**The governor counts.** It does not need to stagger cooks to keep their peaks apart, which is a materially smaller thing to build than the alternative.

## The lock message survived the flags

`-nocompile -skipbuild` did not remove `Build.bat is already running` from the cook logs — all four still carry it. Four sessions got through anyway, staggering into the lock and then cooking together for the rest of the run.

So [the earlier reading](2026-07-31-042802-the-lock-reaches-cooking-too.md) is unchanged in substance and wrong in its remedy: the startup lock is real, the flags do not avoid it, and what decides whether sessions get past it is how many are queued against how long the rung holds. Ten inside four minutes did not; four inside forty seconds did.

## What this rests on

**Four sessions, four samples, forty seconds** — the run was stopped early. The signal is large against its noise, since aligned peaks would have shown 20 GiB and nothing came within a factor of two of that, but this is not a ramp and carries none of [the error bar a redline does](2026-07-30-164912-redline-reproducibility.md).

Two ways it could still be wrong. Ten sessions might synchronise where four do not, most plausibly through the shared shader cache. And a generated game cooks longer than a blank template, which lengthens every phase and could let sessions drift into step rather than out of it.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable, 8.7 free at the start · Windows 11 (26200) |
| Engine | Unreal Engine 5.8, launcher install |
| Workload | `TP_BlankBP` cooked for Windows, one private project per session, cooked output cleared between passes |
| Sampling | every 10 s over the process tree, outside sessionbench |
| Defender | real-time protection on, no exclusions |

Nothing survived the stop: no `cmd`, `UnrealEditor-Cmd`, `ShaderCompileWorker`, `UnrealBuildTool`, `cl` or `link`, and free memory returned to 10.8 GiB.
