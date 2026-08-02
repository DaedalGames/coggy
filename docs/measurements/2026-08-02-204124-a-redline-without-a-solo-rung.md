# A redline is twice the knee, and the knee needs no solo rung

Every redline in this repository divides by a solo hold. The ladder fits `slowdown = b·N` through the origin and solves `b·N = 2`, and a slowdown is a rung's per-session rate over the solo rung's. So the noisiest number this instrument produces — [25% across six holds on one afternoon](2026-08-02-114542-the-control-refused-its-own-run.md) — sits under the headline figure of the whole benchmark.

**The same model gives the same answer without it.**

## The derivation

Below saturation every session gets the cores it asks for, so a hold's total output is the session count times what one session does:

```
total = N·d / w                 rising, slope d/w
```

Above saturation the machine is fully claimed and the total stops depending on how many sessions divide it:

```
total = η·C / w                 flat, the plateau
```

The two meet at `N_sat = (η·C/w) ÷ (d/w) = η·C/d`. **Both `w` and `d` cancel**, so the knee is the plateau divided by the rising slope and neither term needs measuring on its own.

And the redline is twice it. The ladder's own line passes through `slowdown = 1` at `N = 1/b = η·C/d`, which is the knee, and through `slowdown = 2` at twice that. `redline = 2ηC/d` has always said this; read as a measurement rather than a formula it says **the redline is twice the session count at which total throughput stops rising.**

## It checks out on the figures already recorded

| | |
|---|---|
| Rising slope `d/w`, from `d = 0.27` and `w = 12.567 ms` | 21.48 units/s per session |
| Plateau, measured at a hundred sessions | 904.0 |
| Knee, plateau ÷ slope | **42.1** |
| Redline, twice the knee | **84.2** |
| The hold-based figure for the same workload | 87 |

3.2% apart, from two routes that share only the plateau. **This one is a genuine second route** rather than [the rearrangements that agreed to four digits](2026-08-01-213912-the-gate-breaks-at-its-own-duty.md): those all reduced to `N·d/(C·s)` and divided by the same solo, while this uses a slope measured below saturation where no solo appears.

`η` falls out of the plateau as well — `plateau·w/C` gives 0.710 against the 0.733 the slowdown route gave, the same 3% by the same argument.

## What it buys

- **The solo rung leaves the critical path.** It stays useful as a fingerprint for deciding whether two runs are comparable, which is what a fingerprint is for; it stops being a divisor under the headline number.
- **The ladder stops halfway.** A redline of 87 needs rungs up to 87; a knee at 42 needs rungs bracketing 42. Half the sessions, and the heavy end of a ladder is where a machine [stops dead at forty-one minutes](2026-08-01-202112-gate-m1-at-twenty-minutes.md).
- **Two measurements where there was one.** The fitted crossing and the knee use different parts of the curve, so [`compare`'s habit of reporting both a ladder's search and its fit](../../sessionbench/src/redline.rs) extends to a third figure that can disagree informatively.

## What this cannot say

- **Nothing was measured.** This is algebra over the model the ladder already assumes, checked against figures already recorded. Its claim is that a measurement path exists, not that it has been walked.
- **The knee may be rounded.** Contention starts before full saturation, so the corner is a curve and a two-line fit locates the transition's centre rather than its start. How rounded is unmeasured, and it is [the region a governor works in](../../ROADMAP.md#m3--resource-governor).
- **The rising slope needs sub-saturation rungs**, which a ladder searching for a redline currently skips past. Measuring the knee means deliberately spending rungs where the old ladder saw nothing worth recording.
- **3.2% between the routes is not yet a tolerance.** One comparison, one workload, one machine.

## Provenance

Algebra over `sessionbench/src/redline.rs` and figures from `docs/measurements/`, at commit `7044832`. No run was made.
