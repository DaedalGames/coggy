# Workloads

**This document owns what a workload is and what it must promise.** [README](../README.md#documentation) maps the rest.

A workload is a directory holding one executable. Its stdout is the payload, and **each line it writes is one unit of work.** That is the whole contract.

The shape is [vtebench's](../sessionbench/README.md#what-we-take-from-prior-art), taken because it is the least a benchmark can demand of a payload and still be extended by someone who does not want to read the benchmark. The one change is that the executable is built by cargo rather than checked in as a script, so the repository stays a single toolchain.

## The contract

1. **One line of stdout is one unit of work, and the line opens with that unit's ordinal.** A unit means whatever the workload says it means — a file written, a frame rendered, a request served — and it only ever gets compared against itself. The ordinal counts from one and never repeats, which is what makes a *missing* line visible: a gap below the highest ordinal seen is dropped output, and dropped output is one of the four conditions. Everything after the ordinal is free, including a payload for workloads that exist to stress bandwidth.
2. **Runs identically alone and alongside ninety-nine others.** No coordination, no shared paths, no ports.
3. **Knows nothing about COGGY.** A workload that takes a COGGY-specific path stops being evidence, which is [the second integrity rule](../sessionbench/README.md#keeping-it-honest).
4. **Exits zero when it finishes, and finishes on its own.** `--duration` exists for the workloads that do not, and a run stopped that way is labelled in its report.
5. **Writes only under a directory it created inside `SESSIONBENCH_SCRATCH`,** falling back to the temporary directory when the variable is unset. Two rules in one: a fresh directory every run, because real-time scanning charges far less the second time a file is written and a workload reusing paths measures Defender's cache instead of Defender; and a directory the benchmark named, because **the benchmark is the only thing that can remove it.** Every ramp rung ends by killing its sessions, and a killed process runs no cleanup — one working ramp left 299 directories behind before this rule existed.

## Why a unit is a line

Work rate is the condition the metric leans on: a hundred sessions each running at a third of their solo speed is thirty-three-way concurrency wearing a larger number. Measuring it needs the workload to say when it has done something, and the cheapest signal that every language already has is a line on stdout.

The comparison is always **the same workload, solo against concurrent**, so a unit costing more in one workload than another does not matter. Comparing unit rates *between* workloads is meaningless and no report does it.

## Available

| Workload | A unit is | Shaped for |
|---|---|---|
| [cpu-spin](cpu-spin/) | one chain of mixing rounds | Finding the core ceiling, and isolating it from anything the disk is doing |
| [file-write](file-write/) | one file written | Sessions that hold memory and write continuously, which is what generation does to a disk |
| [stdout-storm](stdout-storm/) | one large line written | The output path, and the condition that tolerates zero dropped bytes |

**Each exists to isolate one thing from another.** When per-session work rate falls, the cores went either to sessions competing with each other or to Defender scanning what they wrote, and a workload that writes files always mixes the two. `cpu-spin` touches no disk, so running the same ramp against it and against `file-write` makes the difference between them the scanning term — measured rather than inferred.

`file-write` defaults to holding 80 MiB resident and writing sixty 64 KiB files at 900 ms intervals, which is roughly the footprint and write rate of an agent CLI session. Its steady memory reproduces to 0.01 MiB across runs, which is what let [the first measurement](../docs/measurements/2026-07-30-101141-conhost-and-defender.md) resolve a difference of 8.6 MiB.

`cpu-spin` holds the same memory and, left alone, never waits — so it reaches the core ceiling far sooner than anything realistic would. That is deliberate: flat out is the harshest case, and the redline it produces there is a floor for the machine rather than a forecast for the workload.

**How it waits is the knob that turned that floor into a formula.** `--duty` sets the share of wall time spent computing, deriving each pause from the unit before it so the ratio survives a loaded machine. `--wait-ms` pauses for a fixed span instead, which is the shape of a session waiting on a model — its duty climbs as its compute slows. The two are mutually exclusive, and pairing them at equal solo duty is what [tests whether the mechanism matters](../docs/measurements/2026-07-30-154348-duty-is-derivable.md).

`stdout-storm` is the one whose payload *is* its output, which is what vtebench's format was taken for. It exists because the condition that tolerates zero dropped bytes had never seen bytes worth dropping: the other two emit about twenty per unit, so every run reported zero drops without the path ever being tested.

## Adding one

Create `workloads/<name>/` as a cargo crate, add it to the workspace members, and add a row above. Keep the defaults meaningful on their own: a workload whose numbers only make sense with six flags set will be run wrong.
