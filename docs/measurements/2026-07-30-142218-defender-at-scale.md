# Defender at scale, and a figure withdrawn · 2026-07-30 14:22:18

Two identical ladders of a write-heavy workload, one with the sessions' writes hidden from real-time scanning. The result settles the exclusion axis and **withdraws a `[measured]` claim that had reached [PLAN](../PLAN.md#why-this-exists) and three other records.**

## The two ladders

`file-write --files 2000 --size 128 --interval 200`, about 640 KiB/s per session, sixty-second holds.

| Sessions | Writing | Watched | Excluded |
|---|---|---|---|
| 1 | 37 MiB/min | 4.94 units/s · 0.2 cores | 4.94 units/s · 0.1 cores |
| 10 | 375 MiB/min | 4.94 units/s · 0.3 cores | 4.94 units/s · 0.3 cores |
| 25 | 937 MiB/min | 4.94 units/s · 0.5 cores | 4.93 units/s · 0.4 cores |
| 50 | **1,875 MiB/min** | 4.93 units/s · **0.9 cores** | 4.94 units/s · **0.7 cores** |

Neither reached a redline. Every rung held, and per-session work rate is flat to three significant figures across a fiftyfold range, watched or not.

## The withdrawal

An earlier record put Defender at **1.6 CPU-seconds per MiB written**, from two workloads that agreed to within a few percent.

At that rate, fifty sessions writing 1,875 MiB a minute would need **51 cores**. The machine has sixteen, and the measured total — sessions included — is **0.9**.

The upper bound alone refutes it, without having to separate 0.9 from 0.7. Even crediting Defender with every core either ladder used, the ceiling is **0.03 CPU-seconds per MiB**; the difference between the ladders puts it nearer **0.006**. The original figure is high by somewhere between fifty and two hundred and fifty times.

**Why two workloads agreed on a wrong number.** Defender is one machine-wide process. Both runs attributed all of its CPU to the session under test, so both measured the same thing: whatever else the machine was doing. Agreement between two measurements of the background is not corroboration, and reading it as corroboration is what promoted the figure to `[measured]`.

## What this changes

- **Defender is not the dominant term.** The claim that it wanted 72% of the machine's cores at a hundred sessions is withdrawn.
- **M3's exclusion work is not the largest lever.** [ROADMAP calls it a five-minute implementation with the largest felt impact](../../ROADMAP.md#m3--resource-governor); on this evidence the exclusion buys about 0.2 cores out of sixteen at fifty write-heavy sessions. It may still be worth five minutes. It is not worth reordering a milestone for.
- **The exclusion axis has its answer**, and it is small. Running the axis at one session could never have found this: at that scale the only visible term is Defender's CPU rate, which is exactly the quantity the background swamps.

## What survives

The instrument, and the discipline that caught this. The figure was withdrawn by the same benchmark that produced it, using a design — two ladders compared by an integer — chosen precisely because the first design's output could not be trusted. Everything else in the earlier record stands: RSS and work rate are deltas and medians over a window, not means of per-sample rates, and none of them was affected.

**This is the failure mode PLAN exists to prevent, caught inside PLAN.** A number measured badly, agreed with itself, promoted to a fact, and used to rank a milestone's work — found only because something eventually ran at a scale where the arithmetic had to hold.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| sessionbench | 0.0.0 at commit `ce5c3b1`, clean tree, release build |
| Ladders | 1, 10, 25, 50 sessions · 60 s holds, first 30 s unmeasured · resolution 2 |
| Defender | real-time protection on; exclusion held over the sessions' scratch root for ladder B only, verified back to zero afterwards |

## 2026-08-02: the citation above pointed at the wrong file

The bullet quoting *a five-minute implementation with the largest felt impact* cited PLAN's anti-patterns. **PLAN has never held that phrase** — it was ROADMAP's M3 line, from the scaffold commit onward, and the pointer is corrected above. The right document with a plausible-sounding section is the shape this repository keeps producing; a citation carrying no figure is checked by no test.

*Largest felt impact* has since left ROADMAP too, in the two commits that acted on this record. The quotation stands as what was written when this was measured, which is what a log is for.
