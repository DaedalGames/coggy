# The sixteen cores are not interchangeable · 2026-07-31 14:54:12

[The duty relation](2026-07-30-154348-duty-is-derivable.md) is `redline = 2ηC/d`, where `C` is the machine's logical processors and `η` is what sessions cost each other through the memory system. That formula has one core count in it, which assumes every core is the same core.

On this machine they differ by **2.1×**, and nothing had looked.

## What sixteen loaded cores actually deliver

Sixteen sessions of `cpu-spin`, one per logical processor, with each core's delivered performance read against its nominal clock:

| Cores | Delivered | Count |
|---|---|---|
| 4–11 | **169–208%** | 8 |
| 12–15 | 125% | 4 |
| 0–3 | 95–99% | 4 |

Three tiers, measured under simultaneous load rather than inferred from a spec sheet. The fastest core returns **2.1× the slowest**.

The machine is an **Intel Core Ultra 7 356H** — a mobile part with performance, efficient and low-power cores. `doctor` reports it as *16 physical / 16 logical* and says nothing about the split, because nothing asked it to.

## The workload agrees, by a different route

Per-core throughput from the ramp that exposed this:

| Sessions | units/s per core |
|---|---|
| 1 | **68** |
| 3 | 34 |
| 5 | 34 |
| 10 | 37 |

**Flat from three sessions to ten, and double at one.** Contention degrades gradually; this is a step, which is what core placement looks like — the solo session lands on a performance core and later sessions do not.

The throughput gap is 2.4× against the frequency counter's 2.1×, and the difference is instructions per cycle, which the frequency counter cannot see. [Two routes agreeing](../../CLAUDE.md) is the check; here they agree on the finding and disagree by exactly the amount that identifies what else is going on.

## Why nothing saw it until now

Every earlier ramp held **80 MiB** resident and re-touched it once a second. That workload waits on memory often enough that a fast core cannot use its speed, so performance and efficiency cores return nearly the same rate — and the machine reads as homogeneous.

| Resident | Solo | Ten sessions | Reads as |
|---|---|---|---|
| 80 MiB | 74.11 units/s | 64.91 | flat, no degradation per core |
| 8 MiB | 82.04 units/s | 29.54 | a cliff between one session and three |

**Shrinking the workload to fit a busy machine is what removed the shield.** That was done to make a ramp affordable, and it changed the workload from memory-bound to compute-bound — which is [a conclusion about the stand-in](../../CLAUDE.md) arriving as a side effect rather than a result. Both readings are correct about what they measured.

## What it costs the relation

`C = 16` treats sixteen unequal cores as one number. Two consequences:

**Per-session throughput falls with session count before any contention exists.** The first session gets a performance core; the thirteenth gets a low-power one. The ramp sees that as slowdown and the fit attributes it to `η`, which is defined as memory contention between sessions. **Some share of `η = 0.77` is core placement, and no measurement here separates them.**

**`η` is workload-dependent in a way the records already hint at.** It was measured on the 80 MiB workload, where the tiers converge. A compute-bound session would see a different `η` on the same machine — not because sessions interfere differently, but because the cores they land on differ more.

## What this does not overturn

- **The memory ceilings stand.** [Nine sessions](2026-07-31-054657-the-driven-duty.md) follows from resident bytes divided into a budget, and no core is involved.
- **The redlines taken at 80 MiB describe what they measured**: memory-bound sessions on this machine. That is closer to a real generation session than a compute-bound one, since [an engine cook holds 1.87 GiB](2026-07-31-045604-an-error-bar-for-the-engine.md).
- **`C` being logical rather than physical still holds** — [that correction](2026-07-30-154348-duty-is-derivable.md) was about which count to use, and this is about the counts not being equal.

## What it opens

- **`doctor` reports a core count that hides three tiers, and this was tried and put back.** Windows keeps the split as an efficiency class per processor, reachable only through `GetSystemCpuSetInformation` — and the workspace sets `unsafe_code = "forbid"`, which no local allow can override. The pure-Rust alternative, [`cpu_info`](https://crates.io/crates/cpu_info), groups cores by observed clock frequency; those converge on an idle machine, which is exactly when `doctor` runs, so it would report one tier on this machine and be believed. Weakening a workspace-wide safety invariant to print a figure already recorded here is the worse trade, and a heuristic that fails silently in the common case is worse than both. [Intel's HybridDetect](https://github.com/GameTechDev/HybridDetect) is the reference if that balance ever changes.
- A redline quoted for this machine is a redline for **8 performance + 4 efficient + 4 low-power cores**, and does not transfer to sixteen equal ones. `C` in the relation wants an effective figure, and finding it is a measurement nobody has designed yet.
- The machine is a **laptop on a vendor power plan**, which no record has said. Ceilings taken here are ceilings for that.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 physical / 16 logical · 31 GiB · Windows 11 (26200) |
| Power | AC, battery 100%, SAMSUNG MODE plan, 59 °C — neither throttling nor on battery |
| Per-core reading | 16 concurrent `cpu-spin --resident 8`, `\Processor Information(*)\% Processor Performance` after 10 s |
| Throughput reading | `sessionbench ramp`, `cpu-spin --resident 8 --duty 1.0`, 60 s holds |
| Repository | at commit `5c5a21c` |

The ramp that produced the throughput column was stopped rather than completed, because its purpose became visible before its redline did.
