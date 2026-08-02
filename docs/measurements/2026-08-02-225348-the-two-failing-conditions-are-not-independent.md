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

The mechanism offered is that a session already waiting on memory has less left to lose to the sessions beside it, so the same property that makes a heavy session efficient under contention is what makes it too heavy to hold a hundred of. **The hold this record asked for reversed it** — [`η` at 33 MiB came back at 0.518, below both anchors](2026-08-03-000430-the-footprint-lever-runs-backwards.md) — so everything below rests on a direction that the measurement did not support. The inequality still holds; the escape route it described does not exist.

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

## 2026-08-02, same evening: the overhead is additive, and it was measured

The weakest assumption above was that a session's 1.225 RSS overhead is a *factor*. It is not. Two 25-second solo observations, twenty minutes after this was written:

| `--resident` | steady RSS | overhead |
|---|---|---|
| 20 MiB | 24.06 MiB | **+4.06** |
| 33 MiB | 37.06 MiB | **+4.06** |

Identical to two decimals, where the multiplicative model predicted 39.7 MiB at 33. The overhead is a constant per process — an executable, a stack, a runtime — and nothing about it scales with what the process then allocates.

**A second route agrees.** A hundred sessions at `--resident 20` came back at 24.50 MiB each and a solo one at 24.06; the 0.44 MiB between them is close to [the daemon's own 254 KiB a session held](2026-08-01-163935-what-the-harness-says-about-itself.md) plus the harness around it, and 24.06 + 0.25 = 24.31 against 24.50 is 0.8%.

So the ceiling is `budget/100 − 4.06 − 0.44`:

```
4 GiB budget      ->  resident <= 36.5 MiB
4 GB decimal      ->  resident <= 33.6 MiB
```

**The run parameter does not change and its reason does.** `--resident 33` was picked from the multiplicative ceiling of 33.4; it survives because it is under the stricter of the two readings of *"under 4GB"* — 3.662 GiB, 98.3% of the decimal budget and 91.6% of the binary one. 34 would clear the binary reading and exceed the decimal one, which is not a margin worth arguing about mid-run.

**And this widens the `η` question rather than settling it.** The footprint that fits is larger than this record first said, so if the gate's budget is the binary 4 GiB there is room up to 36.5 MiB — still far below the 80 MiB where `η` was measured above 0.844, and still inside the gap nobody has run.

*Neither of these observations is a hold.* One session at 25 seconds says what one process costs; it says nothing about `η`, which needs a hundred.

## 2026-08-02, later still: the RSS half is measured at a hundred, and it fits

The additive model predicted 3.662 GiB for a hundred sessions at `--resident 33`. **Measured: 3,899,691,008 bytes — 3.632 GiB, 0.9% under the prediction and 97.49% of the budget.** Verdict `Held`, with 100 MB spare.

| | |
|---|---|
| Sessions | 100, `fewest alive` 100 |
| Window | 181 173 ms counted of 182 558 ms held |
| Peak RSS | **3.632 GiB of 3.725** |
| Per session | 37.190 MiB against a solo 37.06 |
| `failed_reads` | **0** — dropped output `Held` |
| Work rate | `NotTaken`, and deliberately |

**The work rate was not taken, because the machine could not carry it.** Four consecutive `doctor` readings before the run gave 4.80, 5.30, 6.40 and 6.58 cores of background — a **30.8% spread**, against the 6.3% this question is chasing between `η = 0.794` and `η = 0.844`. A swinging background hands each hold a different machine, so no bracket was requested and the instrument reported `NotTaken` rather than a number. The rate it does print, 5.604 units/s/session, is a fact about this afternoon and divides into nothing.

**So the kill-check passes and the gate question survives.** Had a hundred sessions at 33 MiB broken the RSS budget, `η` would not have mattered — the footprint that could clear the work rate would not exist. It does exist, with 2.5% of the budget to spare.

**One number does not reconcile.** Per-session RSS above solo is +0.130 MiB here and +0.44 MiB in the resident-20 gate hold, both peaks of a hundred sessions. The daemon's own cost is [254 KiB a session](2026-08-01-163935-what-the-harness-says-about-itself.md), which sits between them and explains neither. Two peaks taken minutes apart on a machine whose background moved 30% is the obvious suspect and is not evidence. It is recorded because it is the kind of small disagreement that gets rounded away and then quoted.

## 2026-08-02, last: the number that would not reconcile was the daemon, and it decides the hour

Per-session RSS above solo came out at +0.130 MiB in the 33 MiB hold and +0.447 in the 20 MiB one. Both peaks were sampled at **101 processes**, so the denominator is not the difference. Separating the daemon from the sessions — `total − 100 × solo` — closes it:

| hold | held | output | total | `100 × solo` | **daemon** |
|---|---|---|---|---|---|
| `--resident 20` | 1204 s | 23.58 MB | 2450.7 MiB | 2406.0 | **44.7 MiB** |
| `--resident 33` | 183 s | 2.02 MB | 3719.1 MiB | 3706.0 | **13.1 MiB** |

**The daemon's term is the scrollback, and it grows with lines read rather than with what the sessions hold.** The [hour-long hold measured the same thing directly](2026-08-01-103225-an-hour-of-a-hundred-sessions.md) — 11.8 MiB at the start of filling, rising through it, and a per-line cost of 130–134 B by two routes. Neither of these two holds is a third route to that constant; they agree with it in sign and size and are two points.

**And 44.7 MiB is a plateau, not a slope.** At 19.9 bytes a line the 20-minute hold emitted about 11 800 lines a session against a scrollback capped at 2000, so it had been at capacity for most of the run. That is the figure at the cap, measured rather than extrapolated.

**Which answers the question the three-minute kill-check could not.** The gate asks for an hour, and the check ran for three minutes with the daemon at a fifth of its eventual size — a pass that could have been bought entirely by the short window:

```
100 × 37.06  +  44.7  =  3750.7 MiB  =  3.663 GiB
budget                =  3.725 GiB
spare                 =  67 MB  (1.7%)
```

It fits, and **the daemon's term is bounded by construction rather than by this workload's line length.** The scrollback carries [a byte budget beside its line count, holding a hundred sessions to about 43 MB whatever they write](2026-08-01-103225-an-hour-of-a-hundred-sessions.md). The 44.7 MiB measured above is that budget plus the daemon's own baseline, which is why it plateaued.

**1.7% is not much margin for a run that takes an hour.** It is above the 0.9% the additive model already reproduced and below anything else quoted here, so the hour at `--resident 33` should be treated as expected-to-fit rather than known-to-fit — and the reason to run it is `η`, which the RSS side does not answer either way.

### Correction, minutes later: 624.9 MiB was never the daemon

The paragraph above first read *the same daemon holding `ping` at 47 bytes a line reached 625 MiB*, and built a third workload axis on it — line length, with a 14× multiplier on the daemon.

**624.9 MiB is that record's "Peak *total* RSS"**, the whole job: a hundred `ping` sessions and the daemon together. The daemon's own cost there is stated two tables down as **363 KiB a session held**, which is 36.3 MiB at a hundred — near the 44.7 MiB measured here, not twenty-five times under it. The arithmetic is what caught it: 2000 lines × 100 sessions × 130 B is 26 MB and cannot reach 625.

**And the axis does not exist in this daemon.** The same record's later note says the scrollback took a byte budget beside the line count, which bounds a hundred sessions to about 43 MB *whatever they write*. Line length moved the old buffer and cannot move this one, so there is no third term to add to duty and footprint — the design removed it before the measurement went looking.

The conclusion is unchanged and better grounded: the hour at `--resident 33` fits because the daemon is capped by construction, not because `cpu-spin` happens to write short lines.
