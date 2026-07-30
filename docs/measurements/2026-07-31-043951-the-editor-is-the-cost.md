# The editor is the cost, not the content · 2026-07-31 04:39:51

Every engine figure so far comes from the smallest project Unreal can make. Each record says so and calls its number a floor, but PLAN carried its engine ceiling without that qualifier — which is [the synthetic-workload mistake](../../CLAUDE.md) one level up. Twenty megabytes of synthetic session gave *memory is cheap*; a template with no content gave *eleven sessions*.

## Ten times the assets, one and a half percent

| | `TP_BlankBP` | `TP_ThirdPersonBP` |
|---|---|---|
| Assets | 8 files | 82 files |
| **Steady RSS** | **1.93 GiB** | **1.96 GiB** |
| Peak RSS | 5.02 GiB | 3.01 GiB |
| Cores | 1.64 | 1.40 |
| conhost | 19 | **0** |

**The cost is the editor loading itself.** A tenfold difference in asset count moves the steady figure by 1.5%, which is at or below what a median over eight samples resolves.

So the ceiling — [twelve sessions, once four readings had been taken](2026-07-31-045604-an-error-bar-for-the-engine.md) — is a property of running the editor, and it holds for any project small enough that the editor still dominates. What that threshold is, this does not say.

## The conhosts belonged to the build check

Nineteen against zero, and the only difference is `-nocompile -skipbuild` on the second run. **Those conhosts came from the target check on the way into a cook, not from cooking.**

That matters because [an earlier record read them as a property of cooking](2026-07-31-040348-cooking-is-the-governed-state.md) and connected them to [Decision 1](../PLAN.md#four-core-decisions), which turns on how many conhosts a hundred sessions carry. They are avoidable, and avoiding them is a flag.

## What two runs cannot resolve

The 30 MiB between the two templates is smaller than the noise on either. **Dividing it by 74 files to get a cost per asset would be inventing a number**, and multiplying that by the thousands of assets in a real game would be inventing a conclusion.

What can be said: at this scale the editor dominates, and nothing here bounds where that stops being true. A project large enough for content to matter is untested, and remains the largest open assumption behind the engine figures.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| Engine | Unreal Engine 5.8, launcher install |
| Workloads | `TP_BlankBP` and `TP_ThirdPersonBP` cooked for Windows, `-nocompile -skipbuild`, cooked output cleared between passes |
| sessionbench | 0.0.0 at commit `04cf347`, release build |
| Hold | 240 s, first 30 s unmeasured · sampled every 5 s |
| Defender | real-time protection on, no exclusions |

The work rate from both runs is quoted nowhere: the editor cycles in and out faster than a cook completes, so the ordinal counts iterations rather than cooks. RSS and process figures come from samples where the editor was live, and both runs mix loaded and unloaded samples the same way, which is what makes them comparable to each other.
