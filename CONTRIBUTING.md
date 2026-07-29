# Contributing

**This document owns how to work on COGGY.** [README](README.md#documentation) maps the rest.

## Working language

Issues, pull requests, code, and comments are in **English**, because it is the one language every contributor can reach. Machine translation is fine and nobody will comment on your grammar. Clear beats polished.

## Setup

You need **Rust 1.88 or newer** and **Windows**. `rust-toolchain.toml` tracks latest stable rather than pinning, so `rustup` fetches what it needs.

```
git clone https://github.com/DaedalGames/coggy
cd coggy
cargo build
cargo run -p sessionbench -- doctor
```

Windows-only is deliberate: ConPTY, Job Objects, and Defender are the subject matter.

## What gets closed

A PR that assumes away any of [PLAN's anti-patterns](docs/PLAN.md#anti-patterns-these-kill-the-project) gets closed. If you think one is wrong, open an issue arguing against the entry in PLAN — not a PR that quietly works around it.

Two more apply only to contributions:

- **A new crate not justified by a measurement**, without a matching amendment to [PLAN's architecture](docs/PLAN.md#architecture).
- **A hand-written version of something that already exists.** Name the prior art you checked, its licence, and what you changed. [Working rules](CLAUDE.md#never-build-what-already-exists) has the order to check in.

## The one rule that matters

**No measurement, no merge.**

Every performance claim needs a `sessionbench` artifact behind it: a run, on stated hardware, with its provenance block. Not a benchmark you ran once and described in prose.

Three ways the rule gets violated by accident:

- **Do not swallow stderr in a verification script.** `2>/dev/null` on a check turns a crash into a silent pass. If a script verifies something, its failures must be loud.
- **Do not let a pinned toolchain stand in for an MSRV check.** Building on your local toolchain proves nothing about the declared `rust-version`. CI runs `cargo +1.88.0 check` for that reason; if you raise a dependency, read its declared MSRV rather than assuming.
- **Do not report "it builds" as "it works."** Run the thing.

## Benchmark integrity

`sessionbench` measures what COGGY competes with, so it is held apart from COGGY on purpose. The three rules that do the holding are in [sessionbench/README.md](sessionbench/README.md#keeping-it-honest): the baseline freezes at M0, workloads know nothing about `coggyd`, and a lone improving axis is distrusted.

A PR that breaks one is closed even when the numbers look good.

## Code conventions

- `cargo fmt` and `cargo clippy` must be clean. Clippy runs with `-D warnings` in CI.
- Reference other documents by heading anchor, never by section number. `cargo test` resolves every link and anchor, so a rename fails the build instead of quietly misdirecting a reader.
- `unsafe` is forbidden at the crate level. If you need it, that is a discussion before it is a PR.
- Lowercase kebab-case for crate and directory names. [PLAN's naming rules](docs/PLAN.md#name-coggy-settled) explain why, and why the benchmark carries no product prefix.
- Comments explain why, not what.

## Commit messages

[Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/): `type(scope): description`, present tense, lowercase, no trailing period.

Use only the eleven types `@commitlint/config-conventional` accepts, so the log stays lintable if we add the tooling: `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`. Anything outside that set fails linting rather than warning. Breaking changes take `!` before the colon and a `BREAKING CHANGE:` footer.

The body is free-form: one bullet per durable change, each on a single line, omitting whatever the diff already proves.

**No co-author trailers, agents included.** Use whatever tooling you like; the author line names whoever ran the change and answers for it.

## Pull requests

Branch from `dev`. Before opening:

- [ ] `cargo fmt --check`, `cargo clippy --all-targets`, `cargo build` all pass
- [ ] Performance claims carry a `sessionbench` artifact with its provenance block
- [ ] Nothing in "What gets closed" applies
- [ ] Docs updated when behavior changed — PLAN for what is true, ROADMAP for order and gates

## License

Contributions are licensed **GPL-3.0-or-later**, matching the project.

GPL propagates through linking, so **COGGY never publishes a linkable core crate** — anything consuming it runs the executable and speaks to its socket. A PR exposing internals as a library dependency for outside consumers will be closed.
