# The same command on battery is 7.8× slower, and no artifact said so

A hundred sessions under `coggyd` at duty 0.27, the configuration run five times today, came back at **135 units/s against the 907 the same command gave seven hours earlier.** Nothing in the run was wrong. The machine had been unplugged.

## What the run showed

| | at noon, mains | in the evening, battery |
|---|---|---|
| Total throughput | **907.1** | **135.1** |
| Sessions held | 100 of 100 | 100 of 100 |
| Peak RSS | 2.37 GiB | 2.35 GiB |
| Tree CPU | ~15 of 16 cores | **3.0–3.8 cores** |

Every structural figure matches. All hundred sessions were alive at every report, memory landed within a hundredth of a gibibyte, and the unit count climbed steadily rather than stalling — 3,540 units every thirty seconds, flat across three minutes. **It was not broken, it was slow**, and a run that is merely slow produces a clean artifact.

The tell is the CPU: a hundred sessions wanting twenty-seven cores were taking three and a half, on a machine reporting `CurrentClockSpeed` at its base and a thermal zone at **42 °C**. Cool, idle, and pinned.

`Win32_Battery` had `BatteryStatus 1` — discharging. It read `2` for every earlier run today.

## The instrument recorded everything except the thing that mattered

`HostFacts` already captured elevation, Defender's state, its engine version and its exclusion lists. Every record carries the machine's cores and memory, the background load, the commit, the workload's argv and the shape of the run. **None of them carried the power state**, and it is the axis that moved the answer by 7.8× while every other recorded field stayed identical.

That is why the day's cross-run comparisons kept failing. [Two runs of the same configuration forty minutes apart differed by 14%](2026-08-02-114542-the-control-refused-its-own-run.md) and it was written up as drift of unattributed cause; the A·B·A·B control [drifted 6.29% before a warm-up hold and 0.37% after](2026-08-02-121359-eta-follows-the-awake-count.md). Warm-ups were a real fix for a real first-hold effect, and underneath them sat a second variable nobody was recording.

**A provenance block is a claim about what could have differed.** Everything in it was chosen because it might move a number; the power state moves numbers harder than anything else on the list and was not on it.

## What changed

`HostFacts` now queries `Win32_Battery` and the active power plan, so every `hold.json` and `ramp.json` carries whether the machine was on mains. Recording is not enough on its own — an artifact only speaks when someone opens it — so `doctor` prints the state where it cannot be missed:

```
ON BATTERY — SAMSUNG MODE — every figure from this machine is a different machine's (85% charged)
```

The sentence is deliberately absolute. A softer one invites the reading that a battery figure is a slightly worse figure, and this measurement says it is a different machine's.

## What this run cannot say

- **The 7.8× is one pair.** Noon against evening, one configuration. How the gap varies with duty, session count or charge level is unmeasured, and there is no reason to expect it constant.
- **The mechanism is inferred.** A base-clock reading and a cool thermal zone point at a power budget rather than heat, and nothing here separates the platform's power limit from the plan's own governor.
- **It does not invalidate the within-run findings, and that is checked rather than assumed.** Neither of those runs recorded a power field — it did not exist yet, and their directories are pruned — but the magnitude is its own fingerprint. Battery produces 135; [the awake-count run](2026-08-02-121359-eta-follows-the-awake-count.md) produced 1035 to 1057 and [the flat-`η` run](2026-08-02-194155-eta-is-flat-where-it-was-said-to-fall.md) 901 to 914. The two states are 7.8× apart and cannot overlap. And within each run the holds agree to between 0.37% and 1.46%, so a state that had flipped mid-run would have left one leg seven times off and did not. **Both ran entirely on mains.** What is invalidated is any figure compared across runs, which the day had already concluded for other reasons.


## The three scales, now that all of them are measured

| what varies | how much |
|---|---|
| Power state | **7.8×** |
| Run to run, same power state | up to **14%** |
| Hold to hold inside one run | **0.4 – 1.5%** |

Each is an order of magnitude apart from the next, which is why only the last supports a comparison and why the first went unnoticed for a day: it is so large that a figure carrying it does not look like a noisy version of the other state, it looks like a different measurement of something else.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Power | **on battery, 86% charged, SAMSUNG MODE** — the first record to say so |
| Background | 9% before the run |
| Thermal zone | 42.1 °C |
| Harness | `sessionbench hold` at commit `968e737`, release build |
| Daemon | `coggyd` 0.0.0, release build |
| Workload | `cpu-spin --units 100000000 --duty 0.27 --resident 20` |
| Shape | 180 s, 100 sessions, 30 s uncounted warm-up, no solo baselines |
