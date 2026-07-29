# Working rules

[ROADMAP.md](ROADMAP.md) says what to build next and [docs/PLAN.md](docs/PLAN.md) says why. **This file owns how to work**, and deliberately repeats nothing those two already hold.

## Never build what already exists

The rule broken most often here, because writing code feels like progress and reading someone else's does not. A clean-room implementation of a solved problem is a defect, not craftsmanship — and the tell is that it arrives with no citation.

Before writing any non-trivial module, in this order:

1. **Search for prior art, and search past the obvious.** The references named in [ROADMAP's milestones](ROADMAP.md#sequencing-direction) and [sessionbench's prior-art table](sessionbench/README.md#what-we-take-from-prior-art) are the starting point, not the search. Also search crates.io and GitHub for the *mechanism*, not the goal — the operating system frequently already accounts for whatever you are about to count by hand.
2. **Check the licence.** MIT, Apache-2.0, BSD, MPL: depend on it or vendor it. GPL: fine, we are GPL-3.0-or-later. Anything that cannot reach us: read it, then rebuild the logic from its description rather than its source.
3. **Take the largest piece that fits** — the crate, then the module, then the algorithm, then the CLI shape, in that order of preference. Customising something we took is good. Not having looked is not.
4. **Cite what you took**, in the code comment and the commit body, including what you changed and why.

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
