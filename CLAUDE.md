# Working rules

[ROADMAP.md](ROADMAP.md) says what to build next and [docs/PLAN.md](docs/PLAN.md) says why. **This file owns how to work**, and deliberately repeats nothing those two already hold.

## Never build what already exists

The rule broken most often here, because writing code feels like progress and reading someone else's does not. A clean-room implementation of a solved problem is a defect, not craftsmanship — and the tell is that it arrives with no citation.

Before writing any non-trivial module, in this order:

1. **Read this repository's own reference list first.** [ROADMAP's milestones](ROADMAP.md#sequencing-direction) name what each one consumes before what it writes, and [sessionbench's prior-art table](sessionbench/README.md#what-we-take-from-prior-art) names what the benchmark takes from whom. Both were written by someone who had already done the survey. Opening a web search before reading them repeats work that is sitting in the repo, and can talk you past a decision the repo already made.
2. **Then search outward, for the mechanism rather than the goal.** The list says *what* to consume; a search is for *how*. What you need rarely appears under the name of the problem — the platform has usually already solved it under the name of something else.
3. **Check the licence.** MIT, Apache-2.0, BSD, MPL: depend on it or vendor it. GPL: fine, we are GPL-3.0-or-later. Anything that cannot reach us: read it, then rebuild the logic from its description rather than its source.
4. **Take the largest piece that fits** — the crate, then the module, then the algorithm, then the CLI shape, in that order of preference. Customising something we took is good. Not having looked is not.
5. **Cite what you took**, in the code comment and the commit body, including what you changed and why.

**The reliable tell** is writing a data structure to track something the operating system already tracks. Process membership, memory accounting, and lifetime are all kernel bookkeeping, and a hand-rolled version of any of them is a bug waiting for a long enough run.

The same rule governs what we *do not* consume: [PLAN's anti-patterns](docs/PLAN.md#anti-patterns-these-kill-the-project) name the places where building it ourselves kills the project.

## Measure before you optimize

The project's own reason for existing. [M0](ROADMAP.md#current-priority-m0--attribution) is a measurement milestone precisely because a daemon built around an unmeasured assumption reproduces the lag it exists to remove.

- A performance claim without a `sessionbench` artifact is not a claim.
- A number that contradicts the plan rewrites the plan, never the other way around.
- Claims in PLAN are marked **[measured]** or **[assumed]**. If you turn one into the other, edit the marker in the same change.

## Verify before you report

Running `cargo build` proves the code compiles, which is not what anyone asked. Run the thing, read its output, and check the output against something independent.

Before saying a change works:

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo +1.88.0 check --all-targets      # the declared MSRV, which the tracking toolchain never exercises
cargo run -p sessionbench -- doctor    # and read it
```

When an independent check disagrees with the code, suspect the check first — but say so either way. [CONTRIBUTING's three accident modes](CONTRIBUTING.md#the-one-rule-that-matters) are each a real mistake made in this repository.

## Stay inside the open gate

One question decides whether to start any piece of work: **does this move the number on the gate that is currently open?** If not, it belongs to a later milestone.

Two consequences that are easy to miss:

- **Order holds inside a milestone too.** M0 runs one session to completion *before* the ramp harness exists, because the single run is what says whether the redline conditions are pointed at the right thing.
- **Engine adapters are blocked on hardware, not on sequence.** Unity, Unreal, Blender, and Godot are not installed here, so [M5](ROADMAP.md#m5--engine-adapters-exploratory) waits for a configured machine. Anything that would have to be redone once a real engine is attached waits with it.

## Commits

[CONTRIBUTING's commit rules](CONTRIBUTING.md#commit-messages) are binding: Conventional Commits, one of eleven types, no co-author trailers of any kind. Commit when a change is coherent and green, not per edit.
