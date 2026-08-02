# The whole-machine CPU column was on a sixteenth of the job's scale

The column added so that a dipping hold could say *what took the machine* stored `global_cpu_usage()`, which averages the CPUs into 0–100, beside a job figure summed from per-process `cpu_usage`, where 100 is one core and a hundred sessions reach 1600. Its own doc comment said the two subtract. **They differ by the width of the machine.**

## As-is, in the artifacts that already had the column

Every hold recorded between the column landing and this fix, comparing the two columns in each sample:

| run | sessions | machine (median) | job (median) | samples where job exceeds machine |
|---|---|---|---|---|
| `1785700857-worsttick-daemon` | 8 | 61.5 | 796.1 | **9 of 10** |
| `1785700981-worsttick2-daemon` | 8 | 59.4 | 795.3 | **9 of 10** |
| `1785701337-tickmove-daemon` | 8 | 72.9 | 791.9 | **8 of 10** |
| `1785701632-slowdown-field-daemon` | 4 | 36.4 | 397.0 | **5 of 6** |

**31 of 36 samples describe a machine narrower than the work running on it.** Nothing complained, because nothing had subtracted the two columns yet — the whole point of the column is a difference nobody had taken.

## To-be

Same instrument, one multiplication, eight sessions at `--duty 0.50` on a box already 67% busy:

| run | sessions | machine (median) | job (median) | samples where job exceeds machine |
|---|---|---|---|---|
| `1785705102-scalecheck-daemon` | 8 | **1470.0** | 285.4 | **0 of 18** |

The remainder — what the instrument does not attribute — comes to 9.02 cores at its lowest, 12.07 median, 15.10 highest, against `doctor` reporting 10.68 cores of background before the run and 10.97 after.

**That agreement is not a second route.** `doctor` and the sampler both call `global_cpu_usage()`; one divides by 100 and multiplies by the core count, the other multiplies by the core count. It checks the arithmetic and not the reading, which is exactly the collapse this project warns about. What makes the fix a measurement rather than an assertion is the as-is table: a job cannot exceed the machine it runs on, and it did, in 86% of the samples that existed.

## What this cannot say

- **Whether `global_cpu_usage()` itself is right on this box.** Both figures here descend from it. An independent check needs a counter outside `sysinfo` — `\Processor Information(_Total)\% Processor Time` — and none was taken. **Taken later the same day; see the end of this record.**
- **Anything the four affected runs concluded.** They were tick-cost and slowdown-field comparisons; none subtracted these columns, which is why the defect survived them and why none of their results moves.
- **Whether the sixteen the column now multiplies by is the right sixteen.** It is `sys.cpus().len()`, the same route `machine.rs` takes, chosen so the two cannot disagree — not so they can confirm each other.

## What guards it now

Two unit tests on the extracted conversion, both shown failing with the multiplication removed and passing with it restored: one asserts a saturated machine reads 1600 rather than 100, the other replays this record's own 796-against-61.5 and requires the converted reading to clear the job.

## A borrowed column heading, inside the record about a borrowed scale

The session counts above first read 9, 9, 9 and 5. Those are `peak_processes` — the sessions plus the daemon — pulled from the artifacts by a script whose output column was then labelled `sessions`. `hold.json` says 8, 8, 8 and 4.

No figure in the finding moves, which is what makes it worth keeping: the numbers were right and the heading came from the field beside them. **A record correcting a mislabelled quantity carried one for its first ten minutes.**

## Provenance

| | |
|---|---|
| Inputs | `bench-out/1785700857-worsttick-daemon`, `1785700981-worsttick2-daemon`, `1785701337-tickmove-daemon`, `1785701632-slowdown-field-daemon`, `1785705102-scalecheck-daemon` |
| Machine | on mains, 67% background before and after the new hold — loud, and irrelevant to a conservation check between two columns of one sample |
| Commit | `87a61c1` |

## How it was found, which is the part worth keeping

Not by running anything. The column's doc comment claimed a scale; two lines already in this repository decided it. `machine.rs` turns the same call into cores with `/ 100.0 * logical_cores`, and `sampler.rs` turns the job figure into cores with `/ 100.0` alone — so one of them is a sixteenth of the other, and the comment asserting they subtract cannot be true.

The run came afterwards, and it was for the as-is column rather than for the answer. **Reading the crate's own documentation was the slower route and did not settle it**: `global_cpu_usage` is described as "the addition of all the CPUs", which reads like a sum, and following it into the Windows implementation reaches a stored field. The repository's own two uses of the same call were unambiguous and were four lines of `grep`.

## 2026-08-03, minutes later: the counter outside `sysinfo`, and what was holding the box

The open item above was that both figures descend from `global_cpu_usage()`, so nothing here checks the reading itself. It got checked incidentally, by a route that shares no code with it:

| | |
|---|---|
| `doctor`, via `sysinfo::global_cpu_usage` | **67%** |
| `\Processor Information(_Total)\% Processor Time`, three one-second samples | **67.0%** |

So the call is a 0–100 whole-machine average, which is what the fix assumed and what `machine.rs` had assumed for longer. **The scale question is closed by a counter that is not `sysinfo`.**

The same look answered something the earlier record could not. `\Process(*)\% Processor Time` names the load:

```
chrome-headless-shell     765.6% of one core   = 7.66 cores
chrome-headless-shell      37.3%
claude                     35.0%
epicgameslauncher          14.0%
```

**A headless browser was holding 7.7 cores** while this box read 67% busy. That is the *shape* [the 08-01 dips had](2026-08-03-024222-the-footprint-never-mattered.md) — something outside the job taking several cores and 1.65 GiB for a few minutes at a time, then giving both back. It is not evidence that this process caused those dips; nothing recorded what ran that afternoon, and this reading is two days later. What it does establish is that this machine runs third parties at that scale, so the inference did not need an exotic explanation.

**And the first-choice instrument was the wrong one.** `Get-Process | Sort-Object CPU` was reached for first and put `EpicGamesLauncher` on top with 18,263 — which is cumulative CPU seconds over the process's whole life, not what it is using. It is the adjacent instrument again: the column that sorts is not the column that answers.
