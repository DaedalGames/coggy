# Gate M1's two failures pull opposite ways, and nothing measured satisfies both

M1 has been written up as failing on work rate with RSS comfortable — [2.301× against a 2, and 2.393 GiB of a 4 GB budget](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md) — which reads as one problem and one condition to spare. **They are the same problem seen from two sides.** The spare RSS is what makes the work rate fail, and spending it is the only thing that could fix the work rate.

No run was made. This is arithmetic over figures already recorded, and its purpose is to say which run is worth making.

## The two conditions, written against the same variable

The redline relation gives the work-rate condition directly. A hold of `N` sessions passes when `N ≤ 2ηC/d`, so at the gate's hundred:

```
eta needed  =  N·d / (2C)  =  100 × 0.27 / (2 × 16)  =  0.844
```

`d = 0.27` is not a knob. It is [what a driven session actually does](2026-07-31-054657-the-driven-duty.md) — a live sampler over 601 s read 0.273 — so lowering it to pass would be measuring a different workload.

The RSS condition bounds the same session from the other end. A hundred sessions of 20 MiB came back at 2.393 GiB rather than the 1.95 GiB their footprints add to, an overhead factor of **1.225**, and holding that constant:

```
footprint ceiling  =  4 GiB / (100 × 1.225)  =  33.4 MiB per session
                      (31.1 MiB if the gate's "4GB" is decimal)
```

## And `η` moves with the footprint

Today's split of the four recorded `η` values [by workload rather than by run](2026-08-02-202535-the-core-ceiling-is-four-numbers-for-two-workloads.md) is what makes these two conditions meet:

| workload | `η` | 100 sessions need | verdict |
|---|---|---|---|
| 20 MiB resident | 0.734, 0.781 | ≥ 0.844 | **fails work rate** |
| 80 MiB resident | 0.842, 0.930 | ≥ 0.844 | passes work rate — **and 7.81 GiB of RSS, 1.95× the budget** |

**Every `η` that clears 0.844 was measured on a workload that cannot fit the RSS budget, and every workload that fits was measured below 0.844.** That is stated from the four measurements alone, with nothing interpolated.

The mechanism is the one already recorded: a session that is already waiting on memory has less left to lose to the sessions beside it. So the same property that makes a heavy session efficient under contention is what makes it too heavy to hold a hundred of.

## What this changes

**"The machine is about 15% too narrow" was the right conclusion for the wrong reason.** It came from the work-rate number alone, as though RSS were spare capacity. It is not spare: it is the budget the work rate would have to spend, and the spend is 1.95× over.

**The needed core count follows from the same line.** Passing at `d = 0.27` with a footprint that fits needs `C ≥ N·d/(2η)`, which at the best `η` measured on a fitting workload — 0.781 — is **17.3 cores**, and at the worst is 18.4. The nineteen quoted elsewhere is 18.4 rounded up to a whole core, and 18.4 is the `η` this hold produced about itself — so the two routes to a core count differ by 6.5%, which is the same disagreement the two 20 MiB `η` values already carry. `C × slowdown / 2` and `N·d/(2η)` are one expression, not two: substituting `η = N·d/(C · slowdown)` turns either into the other.

## The one thing nobody has measured

**The middle.** `η` is measured at 20 MiB and at 80 MiB and nowhere between, and the RSS ceiling sits at 33.4 MiB — inside that gap. If `η` rises steeply from 20 to 33 MiB the gate passes on this machine; if it rises gently, it does not.

Straight-line between the two upper values puts `η(33 MiB)` near 0.794, which fails. That is an interpolation across two points and is worth exactly what two points are worth. **The run that settles it is one hold**: a hundred sessions at `--resident 33 --duty 0.27`, twenty minutes, on mains. It is the only remaining path to passing M1 on this hardware, and it was invisible while `η` was being read as one number.

## What this cannot say

- **`η` from two footprints is a line through two points.** Monotone is plausible from the mechanism and unmeasured as a shape. The 33 MiB estimate is the weakest thing here.
- **The 1.225 RSS overhead is from one workload at one count.** Whether it holds at 33 MiB per session is assumed, and a heavier session may amortise it.
- **`η = 0.734` cannot be used to predict that hold's own slowdown**, because it was derived from it. The independent value is 0.781, fitted across ladders, and it is the one used above.
- **Nothing here re-measures anything.** Every figure is quoted from a record, and the only new content is the arithmetic that puts them in the same inequality.

## Provenance

| | |
|---|---|
| Inputs | [the gate hold at duty 0.27](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md), [the four `η`](2026-08-02-202535-the-core-ceiling-is-four-numbers-for-two-workloads.md), [the driven duty](2026-07-31-054657-the-driven-duty.md) |
| Machine | not used |
| Commit | `d4bc980` |
