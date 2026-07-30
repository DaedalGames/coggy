# An Unreal session, and the condition that actually binds · 2026-07-31 02:22:00

Unreal Engine 5.8 is installed on this machine, with Visual Studio 18 and its MSVC toolchain beside it. Every earlier record was written assuming no engine was reachable, so `d` was only ever read from workloads standing in for one. This is the reading from the thing itself.

## The measurement

`Templates\TP_Blank` copied to scratch and built as its editor target, back to back, each pass preceded by clearing the project's intermediate output so the compiler works rather than reports a cache hit.

| | |
|---|---|
| **Cores demanded** | **1.24 session + 0.20 Defender** |
| **Peak RSS** | **3.40 GiB** |
| Steady RSS | 1.69 GiB, median of the last quarter |
| Processes | 11 at peak, 9 typical |
| conhost | 4 at peak |
| Builds | 4 in 300 s — about 68 s each |

## Memory binds first, which reverses what M0 concluded

| Condition | Where it breaks |
|---|---|
| Work rate, `2ηC/d` = 24.6 / 1.24 | 19.9 sessions |
| **RSS, 21.97 GiB ÷ 1.69** | **13.0 sessions** |
| RSS if build peaks align, ÷ 3.40 | 6.5 sessions |

M0 measured memory against synthetic sessions holding 20 MiB and concluded it had orders of headroom. **An Unreal session holds eighty-five times that**, and the condition M0 retired as slack is the one that trips first.

At a hundred sessions the projection is 168.94 GiB against a 21.97 GiB budget — not a margin to manage but a factor of eight.

## `d` came in below the Rust proxy, and the reason matters

1.24 cores against [cargo's 2.63](2026-07-31-015246-what-a-session-costs.md). The proxy demanded *more*, which is the opposite of what a proxy for an engine was expected to do.

`TP_Blank` has two source files. Most of its 68 seconds is UnrealBuildTool starting, the header tool running, dependencies being scanned and the module being linked — work that is largely serial. `cargo build --release` over this workspace compiles several crates at once and fills more cores.

**So parallelism is a property of the project, not of the engine**, and a blank template is the least parallel thing Unreal can be asked to do. A generated game with real code and assets would compile wider and shorter, moving `d` up and the work-rate ceiling down toward the memory one.

## What a blank template does not include

Everything that makes a game. No assets to import, no shaders to compile, no content to cook, no editor running, no play session. Cooking in particular is where a generation session would spend most of its wall clock, and it is heavier than a compile on both axes measured here.

**This is a floor.** The smallest project Unreal can build already breaks the memory condition at thirteen sessions.

## Two things the instrument does not handle well here

**Steady RSS is a median, and this load has phases.** The samples ran 1.17, 3.04, 3.17, 3.12, 3.28 and 1.27 GiB — a threefold swing as compilers spawn and drain. A median of 1.69 describes neither end. The redline turns on whether a hundred sessions' peaks align, and nothing in the four conditions asks that question.

**conhost appears where it was not expected.** Four per session, four hundred at a hundred, from a workload started on pipes. Unreal's batch files spawn consoles of their own, so [the count that Decision 1 rests on](2026-07-30-120002-first-redlines.md#what-dropping-conhost-is-worth-at-any-session-weight) is not only a property of how COGGY spawns a session.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| Engine | Unreal Engine 5.8, launcher install, 30 GiB · Visual Studio 18 with MSVC |
| Project | `Templates\TP_Blank`, editor target, Win64 Development |
| sessionbench | 0.0.0 at commit `0c804db`, release build |
| Hold | 300 s, first 30 s unmeasured · sampled every 1 s |
| Defender | real-time protection on, no exclusions |

Background load ran 2.29 to 3.96 cores on this machine across the day's runs. The job object keeps it out of the attribution but not out of the contention.
