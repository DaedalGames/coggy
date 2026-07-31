# What a baseline is worth · 2026-07-31 17:17:19

Every rate this instrument reports is read against one session measured once, and [`compare`](../../sessionbench/README.md#comparing-two-ramps) uses that same figure to decide whether two ramps may be set against each other. Its allowance was set at 5% from three ramps that happened to run back to back — one of which it admits, which is circular.

This measures the thing the allowance was guessing at.

## The solo rung reproduces to well under a percent, and then the machine wanders

Every ramp now holds one session again at the end. Two ramps carry that control, and they differ in how long they took to get back to it:

| Ramp | Ladder | First rung | Repeat | Spread |
|---|---|---|---|---|
| `solo-guard` | 2 rungs at 45 s, ≈2.5 min | 79.35 units/s | 79.64 | **+0.37%** |
| `calib-1` | 8 rungs at 60 s, ≈10 min | 77.24 units/s | 79.34 | **+2.72%** |

**The control measures noise plus whatever the machine did in the interval**, and the interval is what separates these two. Under half a percent over two and a half minutes is the measurement floor; the rest is the machine moving.

## Across ramps it is a band rather than a trend

Solo rates against `calib-1`'s baseline, where the shell-control trio ran roughly ten hours earlier:

| Ramp | Solo | Gap |
|---|---|---|
| `shell-bare` | 74.11 units/s | **+4.2%** |
| `shell-cmd` | 76.44 | +1.0% |
| `shell-pwsh` | 76.45 | +1.0% |
| Within the trio | | 0.0% to 3.2% |

**Ten hours apart is not worse than ten minutes apart.** This is not drift accumulating — it is a machine wandering inside a band a few percent wide, and where in that band a ramp lands is not a function of when it ran.

## So the allowance has a reason now

| | |
|---|---|
| Measurement floor | **0.37%** |
| Widest same-machine gap observed | **4.2%** |
| The allowance | **5%** |
| The case it exists to refuse | **51.6%** |

Five sits just above every gap this machine has produced while being itself, and more than ten times below [the pair that prompted the tool](2026-07-31-151821-the-pseudoconsole-row.md). That is the justification the number lacked: not *the spread is 3.1% so allow 5*, which read a single offset as noise, but *nothing the same machine does exceeds 4.2% and the thing being caught is an order of magnitude past it*.

**It stays provisional in one direction.** Every gap here comes from one machine on one afternoon, so the band is this laptop's rather than a property of the metric.

## The control was self-defeating until an hour before this

`calib-1` first reported **+0.0%**, and the reason was in the ramp rather than the machine. The solo repeat is also a one-session rung, so it took the branch that sets the baseline — overwriting the figure it exists to check, then comparing that figure with itself.

It also wrote the repeat into the record as `solo_units_per_sec`, which is what every rate in the report is divided by. **Reading `calib-1` before the fix put its baseline 2.7% high and flipped a verdict**: `shell-bare` against it reads 7.1% and refused, where the true first rung gives 4.2% and admits.

The tell was two console lines that could not both be true — a first rung of 77.24 and a control claiming 79.34 had not moved. [A control that cannot fail is not a control](../../CLAUDE.md), and this one was written three hours after that sentence was.

## What this rests on

- **Two ramps carry the control**, and one of them is short. The floor is one reading over one interval.
- **The band is one machine on one afternoon**, on a laptop whose background load ranged from 11% to 44% across the day.
- **The 51.6% case is a single pair**, nine hours apart, and it is the only refusal this tool has ever issued on real data.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 11% at `calib-1`'s start, the quietest reading of the day |
| Workload | `cpu-spin --units 1000000`, 80 MiB resident, duty 1.0, pipes |
| sessionbench | 0.0.0 at commit `9268c17` for `solo-guard`; `calib-1` predates the fix and its stored baseline is corrected here by hand |
| Defender | real-time protection on, no exclusions |
