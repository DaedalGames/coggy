# Workloads

**This document owns what a workload is and what it must promise.** [README](../README.md#documentation) maps the rest.

A workload is a directory holding one executable. Its stdout is the payload, and **each line it writes is one unit of work.** That is the whole contract.

The shape is [vtebench's](../sessionbench/README.md#what-we-take-from-prior-art), taken because it is the least a benchmark can demand of a payload and still be extended by someone who does not want to read the benchmark. The one change is that the executable is built by cargo rather than checked in as a script, so the repository stays a single toolchain.

## The contract

1. **One line of stdout is one unit of work, and the line opens with that unit's ordinal.** A unit means whatever the workload says it means — a file written, a frame rendered, a request served — and it only ever gets compared against itself. The ordinal counts from one and never repeats, which is what makes a *missing* line visible: a gap below the highest ordinal seen is dropped output, and dropped output is one of the four conditions. Everything after the ordinal is free, including a payload for workloads that exist to stress bandwidth.
2. **Runs identically alone and alongside ninety-nine others.** No coordination, no shared paths, no ports.
3. **Knows nothing about COGGY.** A workload that takes a COGGY-specific path stops being evidence, which is [the second integrity rule](../sessionbench/README.md#keeping-it-honest).
4. **Exits zero when it finishes, and finishes on its own.** `--duration` exists for the workloads that do not, and a run stopped that way is labelled in its report.
5. **Writes only under a directory it created for this run.** Real-time scanning charges far less the second time a file is written, so a workload that reuses paths measures Defender's cache instead of Defender.

## Why a unit is a line

Work rate is the condition the metric leans on: a hundred sessions each running at a third of their solo speed is thirty-three-way concurrency wearing a larger number. Measuring it needs the workload to say when it has done something, and the cheapest signal that every language already has is a line on stdout.

The comparison is always **the same workload, solo against concurrent**, so a unit costing more in one workload than another does not matter. Comparing unit rates *between* workloads is meaningless and no report does it.

## Available

| Workload | A unit is | Shaped for |
|---|---|---|
| [cpu-spin](cpu-spin/) | one chain of mixing rounds | Finding the core ceiling, and isolating it from anything the disk is doing |
| [file-write](file-write/) | one file written | Sessions that hold memory and write continuously, which is what generation does to a disk |

**The pair is the point.** When per-session work rate falls, the cores went either to sessions competing with each other or to Defender scanning what they wrote, and a workload that writes files always mixes the two. `cpu-spin` touches no disk, so running the same ramp against both makes the difference between them the scanning term — measured rather than inferred.

`file-write` defaults to holding 80 MiB resident and writing sixty 64 KiB files at 900 ms intervals, which is roughly the footprint and write rate of an agent CLI session. Its steady memory reproduces to 0.01 MiB across runs, which is what let [the first measurement](../docs/measurements/2026-07-30-conhost-and-defender.md) resolve a difference of 8.6 MiB.

`cpu-spin` holds the same memory and never sleeps, so it reaches the core ceiling far sooner than anything realistic would. That is deliberate: it is the harsher of the two on purpose, and the redline it produces is a floor for the machine rather than a forecast for the workload.

## Adding one

Create `workloads/<name>/` as a cargo crate, add it to the workspace members, and add a row above. Keep the defaults meaningful on their own: a workload whose numbers only make sense with six flags set will be run wrong.
