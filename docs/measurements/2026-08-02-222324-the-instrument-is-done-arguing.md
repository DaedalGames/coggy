# Both gate refusals were the first hold, and the instrument has nothing left to fix

Gate M1's bracket refused itself twice, and a day of work went into deciding whether the fault was the baseline. **It was neither the baseline's noise nor the machine leaving — both times it was the run's opening hold**, which is already fixed.

## The two refusals, read side by side

| | before side | after side | background |
|---|---|---|---|
| [Twenty-minute gate run](2026-08-01-202112-gate-m1-at-twenty-minutes.md) | **3.98%** | 0.32% | — |
| [The duty-0.27 run](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md) | **8.5%** | 0.35% | **1.92 of 16 cores** |

**Only the before side spread, in both.** A machine that moved during a run disturbs the after side too, and the after sides came back at 0.32% and 0.35%. The second run's background was 1.92 cores — the quietest reading of the whole day.

So neither refusal is [the mechanism that made one afternoon's solo set spread 25%](2026-08-02-221853-the-noisy-baseline-was-one-noisy-afternoon.md), where the background collapsed from 28% to 9% and both sides would have felt it. These are the first hold of a run being cold, which is the observation that put an uncounted warm-up in front of every hold and took an A·B·A control's drift from 6.29% to 0.37%.

## Which closes the question the day was really asking

Four attempts were made to get out from under the solo baseline:

- **`--solo-repeats 3`** — kept. At the 4–8% a hold actually spreads, three repeats put the standard error at 2.3–4.0%, inside the 5% allowance.
- **The warm-up hold** — kept, and it is what fixed both refusals above.
- **[Total throughput instead of a slowdown](2026-08-02-121359-eta-follows-the-awake-count.md)** — kept for the questions it answers. A hundred-session reference reproduces to 0.94% against a solo's 4–8%, so a comparison between two configurations is quieter through it. It gives ratios only, which is enough for *are these two the same* and not for *what is this one*.
- **[A redline from the knee](2026-08-02-204124-a-redline-without-a-solo-rung.md)** — closed. [There is no knee](2026-08-02-220514-there-is-no-knee.md); the rise it needed is not a line.

**Two of the four were solving a problem that turned out not to exist at the size it was quoted at.** They were worth building anyway: the warm-up is a real fix for a real effect, and the knee attempt is what found that sessions cost each other from the very first ones, which changes what M3 can offer.

## What is left is not the instrument

M1's gate is measured in full. RSS holds with a third of the budget spare, dropped output is zero, and work rate returns 2.301 against a 2 — on a machine about 15% narrower than the condition wants, which [stops dead at forty-one minutes](2026-08-01-202112-gate-m1-at-twenty-minutes.md) of the load the gate asks it to sustain for sixty.

**Six instrument fixes landed in one day** — the counted window, streamed samples, the failed-read counter, repeated baselines, the warm-up hold, and a standard error where a range had been. None of them moves either failing number, and after today there is no seventh worth reaching for. That is the finding: **the remaining work on gate M1 is a decision about hardware, and the instrument is done arguing with it.**

## What this cannot say

- **Two refusals is two.** The pattern — before side loud, after side quiet — is consistent across both and both are the same bracket on the same machine within two hours.
- **It does not prove the warm-up is why.** The warm-up was added after these runs; that it took a later control's drift from 6.29% to 0.37% is the evidence, and these two were never re-run with it.
- **Nothing here is a new measurement.** Two recorded brackets read together.

## Provenance

Arithmetic over brackets already in `docs/measurements/`, at commit `987d61e`. No run was made.
