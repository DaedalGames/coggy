# COGGY

COGGS builds games. COGGY decides **how many it can build at once.**

COGGY is a session supervisor and resource governor for Windows-native agent workloads: a headless daemon that owns terminal sessions, a governor that keeps a hundred of them from killing the machine, and an audit surface for watching them. It is not an editor, not an agent, and not a generation harness.

## Status: M0 — measuring, before building

**Nothing described above exists yet.** The daemon, the CLI, the governor, and the UI are a target state, not shipped code, because a daemon built around an unmeasured assumption reproduces the lag it exists to remove.

So exactly one crate exists, and it is the instrument — and it has now been pointed at the assumptions. [Three of the four costs this project was designed around have been measured, and none of them is the bottleneck](docs/PLAN.md): processes are cheap in memory, Windows Defender was overstated by two orders of magnitude, and the output path has three to four orders of headroom. What actually limits the machine is plainer than any of them — sessions competing for cores — and it scales with how much of the time a session spends computing rather than waiting.

That result is the argument for M0 existing. **A governor built to the original list would have been tuned against three things that were not the problem.**

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
| [sessionbench/README.md](sessionbench/README.md) | The benchmark: metric, axes, comparison set, report format |
| [workloads/README.md](workloads/README.md) | What a workload is and what it must promise |
| [CLAUDE.md](CLAUDE.md) | How to approach the work: reuse before building, measure before optimizing |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute: setup, conventions, what gets closed |
| [docs/PLAN.md](docs/PLAN.md) | What is true: architecture, scope boundary, constraints |

Claims in PLAN are marked **[measured]** or **[assumed]**, so an assumption cannot be quietly designed around.

## License

[GPL-3.0-or-later](LICENSE).

GPL propagates through linking, so COGGY publishes no linkable core crate. Anything consuming it runs the executable and speaks to its socket — see [the linking boundary](docs/PLAN.md#the-linking-boundary).
