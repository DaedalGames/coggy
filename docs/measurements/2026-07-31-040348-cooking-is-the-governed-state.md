# Cooking is the state a governor has a job in · 2026-07-31 04:03:48

Builds serialise, so no amount of scheduling changes how many happen at once. Sessions at rest cost nine megabytes, so no amount of scheduling is needed. That left an open question about whether the governor this project is named for has anything to govern. **Cooking is the answer**: heavy, long-lived, and concurrent.

## One cook

`TP_BlankBP` cooked headlessly through `UnrealEditor-Cmd`.

| | |
|---|---|
| **Peak RSS** | **5.02 GiB** |
| Steady RSS | 1.93 GiB |
| **Peak processes** | **47** |
| Peak conhost | 19 |
| Cores | 1.64 session + 0.32 Defender |

The 47 processes are shader compile workers, which appear during the first cook and not afterwards once their results are cached. Nineteen conhosts arrive with them, from a session started on pipes.

## Two cooks, side by side

| Elapsed | `UnrealEditor-Cmd` | Combined RSS |
|---|---|---|
| 10 s | 2 | 3.42 GiB |
| 60 s | 2 | 3.62 GiB |
| 90 s | 2 | 4.53 GiB |
| 150 s | 2 | 3.62 GiB |

**Two editors, the whole time.** Cooking takes no per-installation lock, so unlike a build it scales with sessions until something runs out — and at 1.93 GiB steady, what runs out is memory at about **eleven sessions**.

## The three states, and which one the governor is for

| State | Cost | Concurrent | What a governor can do |
|---|---|---|---|
| [At rest](2026-07-31-035111-between-builds.md) | 8.97 MiB | yes | nothing worth doing |
| [Building](2026-07-31-034150-unreal-builds-serialise.md) | 3.27 GiB peak | **no** — the engine queues | nothing it can do |
| **Cooking** | **5.02 GiB peak, 1.93 steady** | **yes** | **admission control** |

A hundred cooking sessions want 193 GiB against a 22 GiB budget. That ceiling is neither transient nor imposed by the engine, which is what makes it the one worth building against — and it puts [the redline's four conditions](../../sessionbench/README.md#redline) on a state that a generation session genuinely occupies.

## What is not settled

- **Blueprint template, first cook.** A generated game cooks more content and compiles more shaders, so 5.02 GiB is a floor the same way the build figures were.
- **Repeat cooks are cached.** After the first, a cook with nothing changed returns quickly, and the workload counted those as units. The RSS and process figures come from the cooking passes; the work rate does not, and is not quoted here.
- **Whether cooks contend beyond memory.** Two ran without visible interference. Ten might contend on disk or on the shader cache, and this says nothing about that.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| Engine | Unreal Engine 5.8, launcher install |
| Project | `Templates\TP_BlankBP`, cooked for Windows, unattended |
| sessionbench | 0.0.0 at commit `31fc739`, release build |
| Hold | 240 s, first 30 s unmeasured · sampled every 5 s |
| Defender | real-time protection on, no exclusions |
