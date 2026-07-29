# COGGY

COGGS builds games. COGGY decides **how many it can build at once.**

COGGY is a session supervisor and resource governor for Windows-native agent workloads: a headless daemon that owns terminal sessions, a governor that keeps a hundred of them from killing the machine, and an audit surface for watching them. It is not an editor, not an agent, and not a generation harness.

## Status: pre-M0

**Nothing described above exists yet.** The daemon, the CLI, the governor, and the UI are a target state, not shipped code — because the account of *why* a hundred sessions is hard is still a set of assumptions, and a daemon built around an unmeasured assumption reproduces the lag it exists to remove.

So exactly one crate exists, and it is the instrument:

## [sessionbench](sessionbench/) — the measuring instrument

> The concurrent session scaling benchmark for Windows-native terminals.

Terminal benchmarks measure one session at a time. None of them plots the scaling curve against *N* concurrent sessions — the number that decides whether any of this works.

It reports **redline**: the largest concurrent session count a machine sustains, always paired with the condition that stopped it.

```
redline: 84 sessions (RSS) · Windows Terminal + pwsh 7 · 16C/64GiB · Defender on
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
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to work on it |
| [docs/PLAN.md](docs/PLAN.md) | What is true: architecture, scope boundary, constraints |

Claims in PLAN are marked **[measured]** or **[assumed]**, so an assumption cannot be quietly designed around.

## License

[GPL-3.0-or-later](LICENSE).

GPL propagates through linking, so COGGY publishes no linkable core crate. Anything consuming it runs the executable and speaks to its socket — see [the linking boundary](docs/PLAN.md#the-linking-boundary).
