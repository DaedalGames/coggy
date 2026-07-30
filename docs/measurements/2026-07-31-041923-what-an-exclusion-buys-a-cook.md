# What a Defender exclusion buys a cook · 2026-07-31 04:19:23

The engine redline sits near eleven sessions, set by a cook holding 1.93 GiB. A cook also writes thousands of small files in bursts, which is real-time scanning's worst case — so that ceiling might have been describing an unconfigured machine rather than Unreal. This is the check, and it is the first time [the sixth axis](../../sessionbench/README.md#the-six-axes) has been pointed at a workload heavy enough to answer with something other than *inconclusive*.

## The reading

Two pairs, each an idle baseline, a watched cook, another idle baseline and an excluded cook.

| Pair | Exclusion saved | Baselines moved | Work rate, watched against excluded |
|---|---|---|---|
| 1 | +2.44 s/min | 0.73 s/min | 12.22 against 11.77 units/s |
| 2 | +1.68 s/min | 1.50 s/min | 9.85 against 10.07 units/s |

**Verdict: 2.06 s of Defender CPU per minute, which is 0.034 cores.** Both pairs agree in sign and the saving exceeds the drift between their baselines in both, so the effect is real. It is also small.

## It does not move the ceiling, and the reason is structural

The redline for a cooking session is **memory**: 1.93 GiB steady against a 21.97 GiB budget is about eleven sessions. **An exclusion returns CPU, not memory.** Saving 0.034 cores per session leaves that arithmetic untouched, so the eleven stands as a fact about Unreal rather than about Windows Defender.

Work rate says the same thing from the other side. The two halves came out 12.22 against 11.77 in one pair and 9.85 against 10.07 in the other — **the sign flips**, so cooking is not measurably faster with its output hidden from the scanner.

## Which makes M0's Defender conclusion stronger than it was

Defender was [first recorded at a cost two orders of magnitude too high, then withdrawn](2026-07-30-142218-defender-at-scale.md), and [an exclusion measured against one lightweight session said nothing at all](2026-07-30-140251-exclusion-delta.md). The reason given there was the workload: `file-write` emits sixty 64 KiB files at 900 ms intervals, a trickle.

A cook is the opposite — thousands of small files written in bursts, which is the pattern scanning charges most for. **The exclusion still buys 0.034 cores a session.** A conclusion that survives its own worst case is worth more than one that was never tested against it.

At a hundred sessions the saving would be 3.4 cores, which is worth having and is not what stops the machine.

## What the run also showed about the machine

Defender used **6.96 and 9.73 seconds a minute with no session running at all** — 0.12 to 0.16 cores of an otherwise idle desktop, and that with an exclusion already in place. Without the idle baselines this axis takes on either side of each half, that figure would have landed inside the delta and been read as the exclusion's doing.

## What this does not settle

- **One cook at a time.** Ten concurrent cooks write ten times the files, and scanning may or may not scale linearly with that.
- **A Blueprint template.** A generated game cooks more content, so both the scanning load and the memory floor move up together.
- **Exclusions were held over the scratch root only.** The engine's own intermediate output lives elsewhere and stayed watched.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| Engine | Unreal Engine 5.8, launcher install |
| Workload | `TP_BlankBP` cooked for Windows, cooked output cleared between passes |
| sessionbench | 0.0.0 at commit `d350956`, release build |
| Halves | 150 s each · 2 pairs, alternating · sampled every 2 s |
| Exclusion | added over the run's scratch root and removed afterwards, verified |
