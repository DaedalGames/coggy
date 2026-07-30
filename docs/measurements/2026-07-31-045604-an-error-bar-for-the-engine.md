# An error bar for the engine figure · 2026-07-31 04:56:04

Every engine number so far came from one run. The synthetic redlines had to clear repeats, a fitted slope and a drift control before they could be quoted, and PLAN was carrying *eleven sessions* to a lower standard than that. Closing the gap by ramping eleven cooking sessions needs 21 GiB of a machine that has had under 11 free all evening — so this repeats the per-session figure instead, which is where the noise was already known not to live.

## Three repeats

| | Steady RSS | Peak RSS | Cores |
|---|---|---|---|
| 1 | 1.83 GiB | 2.41 GiB | 1.41 |
| 2 | 1.89 GiB | 2.43 GiB | 1.16 |
| 3 | 1.81 GiB | 2.42 GiB | 1.31 |
| **Spread** | **4.3%** | **0.8%** | **19%** |

A fifth reading, taken once `observe` had a control of its own, came in at 1.89 GiB and **held**: 1.87 early against 1.89 by the end, +0.8%. With the first `TP_BlankBP` run at 1.93, five readings give **1.87 GiB ± 6.4%**.

The binding condition survives that. `21.97 ÷ 1.87` is **11.7 sessions**, and the spread puts it between **11.4 and 12.1** — so twelve, where a single reading had said eleven.

**That drift figure is what makes the median a ceiling rather than an average of two states.** A cook cycles the editor in and out, so the worry was that a steady figure describes neither end; both quarters carrying the same mix is what answers it, and [four concurrent sessions summing to 1.94 apiece](2026-07-31-043156-cook-peaks-scatter.md) confirms it from outside — one session's average over time and four sessions' total at an instant agree.

Cores reproduce at 19%, which would have been useless had cores been what bound the machine. They are not, and that is luck rather than design.

## Which is a rung rather than a search, and that is why this works

[The seven-run reproducibility study](2026-07-30-164912-redline-reproducibility.md) found the ladder's *search* spreading identical runs across 12.5% while the rungs themselves held within 2%. A per-session figure is a rung. Repeating it buys the same error bar a ramp would, without the memory a ramp needs — [the smallest design that can still be wrong in the way that matters](../../CLAUDE.md).

What a fitted slope would have bought is not available here and not needed: [total memory is the per-session figure times the count by construction](../../ROADMAP.md#current-priority-m0--attribution), so repeats and a drift control are the whole of an RSS redline's rigour, and both are now in hand.

## The 5.02 GiB peak was the build, not the cook

Peak RSS reads 2.41 to 2.43 here against [5.02 in the first cook measurement](2026-07-31-040348-cooking-is-the-governed-state.md). The only difference is `-nocompile -skipbuild`, which is also [what took the nineteen conhosts to zero](2026-07-31-043951-the-editor-is-the-cost.md).

**So the cook's own peak is 2.42 GiB, and the 5.02 included the target check's compile.** That matters for more than tidiness: peak against steady is 1.3× rather than 2.6×, which makes [scattered peaks](2026-07-31-043156-cook-peaks-scatter.md) even less able to bunch into a ceiling.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| Engine | Unreal Engine 5.8, launcher install |
| Workload | `TP_BlankBP` cooked for Windows, `-nocompile -skipbuild`, cooked output cleared between passes |
| sessionbench | 0.0.0 at commit `54eade9`, release build |
| Runs | three of 150 s back to back, plus a fourth once `observe` carried its own RSS control · first 30 s of each unmeasured |
| Defender | real-time protection on, no exclusions |

Back to back on a machine whose background was not controlled, and with no drift check between them — the three agreeing to 4.3% is the argument that neither mattered here.
