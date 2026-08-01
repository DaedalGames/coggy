# The baseline is the noisy term

A bracketed hold [refused itself when its two solo baselines sat 6.9% apart](2026-08-01-144212-the-run-moves-the-machine-it-is-measured-on.md), and the question that opened was whether the allowance it was judged against asks the right thing of a bracket. Three repeats of the same shape answer a different question first: **the post-load deficit does not reproduce, and the solo baseline scatters nearly as far on its own.**

## What was run

Fifteen holds back to back, three repeats of solo · eight concurrent · three trailing solos. Every hold twenty seconds but the load at twenty-four, one session except the load's eight, `cpu-spin --duty 1.0` throughout.

## The deficit is not there

Each repeat's trailing solos, as a percentage below that repeat's own pre-load solo:

| repeat | s1 | s2 | s3 |
|---|---|---|---|
| 1 | +1.49 | −1.56 | −0.43 |
| 2 | −0.09 | +1.38 | +1.80 |
| 3 | −3.73 | −2.50 | +0.11 |

| point | mean | low | high | |
|---|---|---|---|---|
| s1 | −0.77 | −3.73 | +1.49 | inconclusive |
| s2 | −0.89 | −2.50 | +1.38 | inconclusive |
| s3 | +0.49 | −0.43 | +1.80 | inconclusive |

Every point straddles zero and every range is wider than its own centre, which is [`exclusion-delta`'s rule](../../sessionbench/src/main.rs) for a comparison that has established neither a direction nor a size. **No shape to read, so the ramp-or-step question the earlier run left open has nothing to answer it with — and did not need to be asked yet.**

## The eight-session load reproduces seven times better than one session

The same table read down its other column, which is where the finding is:

| | repeat 1 | repeat 2 | repeat 3 | spread |
|---|---|---|---|---|
| load, 8 sessions | 34.054 | 34.034 | 34.059 | **0.07%** |
| solos, 1 session each | 44.23–45.60 | 43.58–44.42 | 43.78–45.46 | **4.54%** across all twelve |

Eight sessions averaged inside one hold land within 0.07% of each other across three separate runs. One session, twelve times, spans 4.54%. Independent noise would have given the eight-session figure `1.5%/√8 ≈ 0.53%`; it is seven times steadier than that, so what moves a solo hold is not something that averages away within a hold — it is fixed for the hold's whole length.

## Same share, less work

The samples say which of the two it is. Across all twelve solo holds:

| | mean | spread |
|---|---|---|
| CPU share | 78.2% | 1.95% |
| work rate | 44.5 units/s | 4.54% |

**The sessions were given the same share and did different amounts of work with it.** That rules out contention and scheduling — nothing else was running and nobody was starved — and points at what a CPU-second buys, which on this machine is [not a constant: its sixteen cores differ 2.1× under load](2026-07-31-145412-the-cores-are-not-interchangeable.md). A solo session is one process on one core for twenty seconds, so its rate is largely a property of where it landed. Eight sessions cover the tiers every time.

Cache and memory locality fit *same share, less work* as well as core speed does, and this run does not separate them. What it does establish is the axis: the variance is in what the session got to run on, not in how much it was allowed to run.

## Three more things the samples rule out

Read down the twelve solo holds again, sorted by rate:

- **Memory is not the variable.** Peak RSS is 28.70 MiB in all twelve, to the
  tenth of a mebibyte. The workload is deterministic and its footprint says so,
  which is what makes a 4.54% spread in *speed* the only thing that moved.
- **Neither is how much CPU each got at the end.** Last-sample share runs 96.5%
  to 99.2% and does not track the rate: the fastest hold sat at 98.5% and the
  slowest at 98.3%.
- **And the machine did not warm up over the run.** Per-repeat solo means go
  44.96, 44.04, 44.50 across roughly six minutes — down then up, so there is no
  trend for a thermal or a scheduler-settling story to sit on. Repeat means
  spread 2.07% where holds within a repeat spread about 3%, which is what
  per-hold placement noise looks like and not what drift looks like.

## What this says about the allowance

`SOLO_AGREEMENT_PERCENT` is 5%, and [the record that set it](2026-07-31-171719-what-a-baseline-is-worth.md) measured a ramp's solo rung reproducing to 0.37% over two and a half minutes, with gaps across ramps forming a 0.0–4.2% band. Five sits just above that band.

**A bracket's baselines are not that population.** A ramp's solo rung is repeated inside one ladder — the comment on the constant already says why that is the weaker statistic, that a fresh solo rung pays process startup and a cold cache again. A bracket's two baselines are exactly that: two fresh daemon launches with one session each. Measured here, that population spans 4.54% with no load between the readings at all.

So the allowance is not mis-shaped for averaging. **It is calibrated on a quieter population than the one it judges**, and sits close enough to the fresh-solo spread that an unlucky pair of core placements refuses a sound run. The 6.9% that started this is 1.5× the spread seen here, which is the same order rather than the same number — enough to say the refusal was not necessarily reading the load.

## What this run cannot say

- **It does not show the earlier 6.9% was noise.** Twelve holds here span 4.54%; a 6.9% gap is outside that, and this run has no way to reach back and re-measure an afternoon. What it removes is the assumption that a gap that size needs a load to explain it.
- **One dose, one duration.** Eight sessions for twenty-four seconds. A heavier or longer load may leave a deficit this one does not.
- **The mechanism is inferred from an axis, not isolated.** Core placement is the reading that fits every figure here and one measured elsewhere; nothing pinned a session to a core to check it.
- **[The teardown correction landed in the same afternoon](2026-08-01-144212-the-run-moves-the-machine-it-is-measured-on.md) and is 0.14%.** It is in these numbers and cannot be what moved them.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 40% before the run, 38% after |
| Harness | `sessionbench hold` at commit `9c15f42`, release build, driven by `scripts/bracket-calibration.ps1` |
| Daemon | `coggyd` 0.0.0, release build |
| Workload | `cpu-spin --units 10000 --duty 1.0 --resident 20` |
| Shape | three repeats of solo 20 s · 8 sessions 24 s · solo 20 s × 3, sampled every 4 s, back to back |
