# The noisy baseline was one noisy afternoon

Six documents call a solo hold *the noisiest number this instrument produces*, each pointing at a set that spread 25% across six holds. Two designs were built to avoid it and a third was costed at nine repeats a side. **Put the day's four solo sets side by side and three of them are quiet.**

## Every solo set measured today

| run | spread | normalised, sorted |
|---|---|---|
| gate hold, duty 0.27 | 8.4% | 0.932, 1.007, 1.013, 1.014, 1.016, 1.017 |
| **awake control, duty 0.20** | **28.5%** | 0.858, 0.887, 0.976, 1.014, 1.122, 1.144 |
| `η` pair, `--wait-ms 67` | 6.2% | 0.957, 0.996, 0.996, 1.016, 1.016, 1.019 |
| salvaged 120 s, duty 0.27 | 3.9% | 0.978, 1.004, 1.018 |

Three sets land inside 3.9–8.4%, which as a standard error over three holds is 2–4% — comfortably under the 5% a bracket judges with. **One set is four times worse than the others**, and it is the one every citation traces back to.

## What was different about it

Its record's own provenance says: **background 28% before the run and 9% after.** No other run moved like that — the others sat at 13%→21%, at 1.92 cores, or steady.

A solo hold is one session. Whatever the background does, it does to that one session at full strength. A hundred-session reference takes the same disturbance and divides it a hundred ways, which is why references in tonight's sweep held to 0.94% while the background sat at 33%.

**So a large part of what was called placement noise was background noise**, and it was measured on the afternoon the background happened to be collapsing.

## What that undoes

**A σ of 12.6% per hold, and everything costed from it.** An hour before this, the bracket's reported standard error at three holds — 7.3% — was turned into a per-hold σ and used to conclude that nine solo repeats a side would be needed to clear the allowance, and 181 to match a reference's reproducibility. That 7.3% is this one run's. Read from the other three, σ is 4–7% and three repeats already sit near the allowance.

**And it was one measurement feeding several conclusions**, which is [the shape that has now bitten four times today](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md): the same figure arrives in three sentences and reads as three facts. Four sets existed all day; putting them in one table cost a single calculation.

## What it does not undo

- **The solo baseline is still the loudest term in a slowdown**, at 4–8% against a reference's 0.94%. Preferring total throughput where the question allows it remains right for that reason.
- **The gate run's refusals were real.** Its own before-side held an 8% outlier and its means were 3.0% apart; that is a genuine reading of a genuine baseline, not a symptom of this.
- **`--solo-repeats 3` stays.** At σ 4–7% it puts the standard error at 2.3–4.0%, inside the allowance, which is what it was raised to do.

## What to do instead of repeating more

The prescription that follows is not more holds but **not measuring while the background moves**. A bracket already takes solos on both sides, so the spread between its two sides is the signal — and this run's 28.5% was that signal, read as a property of the instrument rather than as the machine leaving. `doctor` reports the background, and [a ratio wants a steady machine rather than a quiet one](../../CLAUDE.md).

## What this cannot say

- **Four sets, one machine, one day.** Whether a quiet background reliably gives 4–8% is three observations, and one of the three is at a different duty and wait mechanism from the others.
- **The background is correlated with, not shown to cause, the spread.** The run whose background collapsed is also the only run at duty 0.20 with proportional waits and no warm-up hold. Nothing here separates those.
- **No new measurement was made.** This is four recorded sets read together.

## Provenance

Arithmetic over solo figures already in `docs/measurements/`, at commit `2f7bb75`. No run was made.
