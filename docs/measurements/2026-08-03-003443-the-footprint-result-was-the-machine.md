# Compared inside one machine state, `η` rises with footprint after all — and the state is worth 72%

[Half an hour ago](2026-08-03-000430-the-footprint-lever-runs-backwards.md) a hundred sessions at 33 MiB slowed 3.261× against 2.301× at 20 MiB, and that was written up as `η` falling with a session's footprint. **The two holds were in different machine states.** Repeating the 20 MiB hold inside the state the 33 MiB one ran in reverses the sign.

## The same regime, both footprints

| | resident 20 | resident 33 |
|---|---|---|
| solo (slow state) | 9.752 | 9.377 |
| concurrent, 100 sessions | 2.4636 | 2.8753 |
| **slowdown** | **3.958** | **3.261** |
| **`η = N·d/(C · slowdown)`** | **0.426** | **0.518** |
| peak RSS | 2.371 GiB | 3.648 GiB |

**The heavier session costs its neighbours less, not more.** That is the direction [the original split proposed](2026-08-02-202535-the-core-ceiling-is-four-numbers-for-two-workloads.md) and the direction withdrawn last night on a comparison that spanned two states.

The 20 MiB baseline here is **one-sided** — the run's own bracket refused, for the reason below — so 3.958 is a figure this instrument declines to publish and it is used here only against a same-state counterpart. Its two before-solos agree to 0.51%.

## What the machine state is worth

The 20 MiB gate figure of record is **2.301**, taken with a solo rung of 21.484. The same hundred sessions, same duty, same footprint, in the slow state give **3.958** — **72% worse**, and the difference is a condition of the box rather than anything the daemon or the workload did.

**Gate M1's verdict therefore depends on machine state by more than it depends on anything measured so far.** 2.301 fails a condition of 2 by 15%; 3.958 fails it by 98%.

## The state recovers, and the instrument caught it recovering

The run's own solo holds, in order:

```
before:  9.776   9.727        <- slow
after:  12.095  21.516        <- climbing, then fully recovered
```

**21.516 against the 21.484 recorded on 08-01 is 0.15% apart** — two independent sightings of the same fast state, taken two days apart on the same box. So the slow state is transient, it lasted roughly an hour after [a three-minute saturating burst](2026-08-03-004512-a-saturating-burst-halves-the-box-for-an-hour.md), and it ended partway through a five-minute concurrent hold.

**The bracket refused the run and named it**: *solo holds 9.752 and 16.805 sit 53.1% apart against a 5% allowance — the machine moved under the run.* This is the first time that refusal has fired on a genuine mid-run machine move; [the two before it were the run's opening hold being cold](2026-08-02-222324-the-instrument-is-done-arguing.md). A check whose failure nobody has seen is a check nobody has read, and this one has now been read.

## What was wrong with the reasoning, not just the number

Last night's record explained the 2.29× gap between the two runs' solo rungs as *the swept variable moving the solo* — a footprint sweep changes what a session does alone, so the solo stops being a machine fingerprint.

**The footprint does not move the solo.** Two 25-second observations before any of this gave 19.92 units/s at 20 MiB and 20.04 at 33 — **0.6% apart**. The solo rung was doing exactly its job: it reported that the machine had changed, and the report was explained away.

The rule it was explained away with is real — [a solo rung is a fingerprint only across runs that share a workload](2026-08-01-080158-the-relation-at-a-quarter-duty.md) — but it was written for the duty knob, which moves a solo four-fold. **A rule about *whether a knob moves the baseline* was applied by reading the knob's name instead of measuring its effect**, and the effect is 0.6%.

## What this cannot say

- **One pairing, one state, one duty.** 20 against 33 MiB at 0.27 in the slow state.
- **The 20 MiB side is one-sided.** Its after-solos are the recovery and cannot be averaged in. The two before-solos agree to 0.51%, and the slowdown built on them is not a published figure.
- **Nothing here says which state a gate should be judged in.** The fast state is the rested box; the slow one is the box that has recently done the work the gate asks for. Both are this machine.
- **The mechanism is still unmeasured.** That a heavier session costs its neighbours less is now the direction of two comparisons rather than one, and neither says why.
- **`η` values from ladders are still not comparable to these.** The 80 MiB figure remains a redline fitted across rungs.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical · 31 GiB · Windows 11 (26200) |
| Power | on mains, SAMSUNG MODE, 100% |
| Background | 24% before the run |
| Harness | `sessionbench` 0.0.0 at `c833f24`, release, `hold --with-solo --solo-repeats 2 --solo-duration 60` |
| Shape | 2 solo × 60 s, 100 sessions × 300 s, 2 solo × 60 s, 30 s uncounted warm-up before each |
| Artifacts | `bench-out/1785684220-r20-slow-regime-daemon`, `1785680927-eta-at-33-daemon` |
