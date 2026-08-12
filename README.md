# COGGY

COGGS builds games. COGGY decides **how many it can build at once.**

COGGY is a session supervisor and resource governor for Windows-native agent workloads: a headless daemon that owns terminal sessions, a governor that keeps a hundred of them from killing the machine, and an audit surface for watching them. It is not an editor, not an agent, and not a generation harness.

## Status: M0 measured, M1 begun

**Almost nothing described above exists yet, and that order was the point:** a daemon built around an unmeasured assumption reproduces the lag it exists to remove. So M0 measured first, and M1 started with what the measurement said mattered — [`coggyd`](coggyd/) owns a session's lifetime and its output, because G0 found that killing the process you spawned does not end a session. The CLI, the socket API, the governor and the UI remain a target state.

The instrument was the only crate for as long as M0 ran, and `coggyd` joined it when M1 opened. Pointed at this project's own assumptions, it found that [three of the four costs it was designed around are not the bottleneck](docs/PLAN.md): processes are cheap in memory, Windows Defender was overstated by two orders of magnitude, and the output path has three to four orders of headroom.

**What binds is memory, and the ceiling is nine.** [G0 is frozen there](docs/measurements/2026-07-31-150258-g0-frozen.md): a generation session is an agent CLI holding 0.52 GiB for its whole life plus a game engine holding 1.87 GiB while it cooks, and a 31 GiB machine runs out at `21.97 ÷ 2.39`. Cores would not bind until about a hundred — [≈97 on a rested machine and 51 on one that has recently been saturated](docs/measurements/2026-08-03-024222-the-footprint-never-mattered.md), because the box's own state moves that ceiling further than the session's weight does. Either way it is an order above nine.

**That answer reversed twice while it was being measured**, which is the argument for M0 existing. Synthetic sessions holding 20 MiB said cores were the limit and memory had orders to spare; a real engine holding 1.87 GiB said the opposite; and the agent driving it — the largest resident cost of the three — had never been measured at all. **A governor built to the original list would have been tuned against three things that were not the problem, then sized against a ceiling eleven times too high.**

## [sessionbench](sessionbench/) — the measuring instrument

> The concurrent session scaling benchmark for Windows-native terminals.

Terminal benchmarks measure one session at a time. None of them plots the scaling curve against *N* concurrent sessions — the number that decides whether any of this works.

It reports **redline**: the largest concurrent session count a machine sustains, always paired with the condition that stopped it.

```
redline: 10 sessions (WorkRate) · stdout-storm · pipe · 16C/31GiB · Defender on
```

Requires Rust 1.88 or newer on Windows.

```
cargo run -p sessionbench -- doctor
```

See [sessionbench/README.md](sessionbench/README.md) for the metric definition, the comparison set, and the rules that keep the benchmark from grading its own homework.

## Documentation

| Document | Holds |
|---|---|
| [ROADMAP.md](ROADMAP.md) | What order it happens in, and the gate that closes each milestone |
| [coggyd/README.md](coggyd/README.md) | The daemon: what a session is, what owning one means, where its boundary is |
| [sessionbench/README.md](sessionbench/README.md) | The benchmark: metric, axes, comparison set, report format |
| [workloads/README.md](workloads/README.md) | What a workload is and what it must promise |
| [CLAUDE.md](CLAUDE.md) | How to approach the work: reuse before building, measure before optimizing |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute: setup, conventions, what gets closed — and the commit gate, which is [one script](sessionbench/scripts/gate.ps1) rather than four commands to read the output of |
| [docs/PLAN.md](docs/PLAN.md) | What is true: architecture, scope boundary, constraints |
| [docs/measurements/](docs/measurements/README.md) | Every run that decided something, oldest to newest, with what it could not establish |

Claims in PLAN are marked **[measured]** or **[assumed]**, so an assumption cannot be quietly designed around.

## License

[GPL-3.0-or-later](LICENSE).

GPL propagates through linking, so COGGY publishes no linkable core crate. Anything consuming it runs the executable and speaks to its socket — see [the linking boundary](docs/PLAN.md#the-linking-boundary).
