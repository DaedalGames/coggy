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

**The rule is retroactive.** It governs code already here, not only code about to be written, and "we already built it" is the weakest argument available — the work is sunk either way, and keeping the worse version costs from here on. Finding that something in this repository duplicates a maintained project is a reason to delete ours.

Keeping ours is allowed, and it needs a **specific** reason recorded next to the code: the thing the alternative gets wrong, in its own words where possible. "Similar to GitHub's anchors" is a reason; "ours is more tailored" is not. And a rejection is rarely total — take the part that fits and say which part you did not.

The same rule governs what we *do not* consume: [PLAN's anti-patterns](docs/PLAN.md#anti-patterns-these-kill-the-project) name the places where building it ourselves kills the project.

## Measure before you optimize

The project's own reason for existing. [M0](ROADMAP.md#current-priority-m0--attribution) is a measurement milestone precisely because a daemon built around an unmeasured assumption reproduces the lag it exists to remove.

- A performance claim without a `sessionbench` artifact is not a claim.
- **Every measured improvement gets a record with an as-is column beside the to-be one.** The as-is column is what makes it a measurement rather than an announcement. It belongs in [docs/measurements/](docs/measurements/) as `YYYY-MM-DD-HHMMSS-<slug>.md`, stamped with the moment the record was committed rather than the moment the run happened — a clock the next clone can still read, so the records sort into the order they were written on any machine. Publish it like everything else there — an engineering figure has no reason to be private, and two KPI files sat `.local` for a day for no reason anyone could name. Keep `.local` for what genuinely cannot be published, which so far is unit economics and funding posture and nothing else.
- **A KPI that restates an existing record is a view of it, not a second record.** Fold what is new — usually the cost of the change — into the record that owns the measurement, and do not create the file.
- A number that contradicts the plan rewrites the plan, never the other way around.
- Claims in PLAN are marked **[measured]** or **[assumed]**. If you turn one into the other, edit the marker in the same change.

### Do not degrade the machine you are measuring

A long session of back-to-back ramps makes the box slower, and a slower box is not a different mood — it is a different machine, so late measurements stop being comparable with early ones. This is measured rather than suspected: **a rung that ran 40.02 units/s at the start of one ramp ran 38.11 at the end of it, 4.8% down**, and that alone dragged the fitted redline from ~33.6 to 32.3.

Every ramp now repeats its lowest saturated rung at the end and reports the gap. **Read that line before quoting anything else in the run.** Past a few percent the redline is reading low and the run is a draft.

While a measurement is running, do nothing else on this machine — no builds, no `git`, no `gh`, no file edits. The observer is not free. [The Defender estimate](docs/measurements/2026-07-30-142218-defender-at-scale.md) was wrong by two orders of magnitude partly because `gh` ran during the run that produced it, and the drift figure above came from a ramp with edits happening alongside it.

Between runs, keep the footprint from accumulating:

- **Prune `bench-out/`.** Each ramp leaves roughly 500 files, all of them on the scanned side of Defender's line. Keep the newest run or two; it is gitignored and nothing committed points into it.
- **Check for survivors** before trusting a run — `Get-Process cpu-spin,file-write,stdout-storm,sessionbench`. Teardown reaps its own scratch and its own sessions, and has been verified to leave nothing, but a killed ramp is a different story.
- **Do not reach for `cargo clean`.** It buys back a few GiB and costs a full rebuild, which is the slow thing the cleanup was supposed to fix.

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
- **Engine adapters wait on sequence, and only partly on hardware.** [M5](ROADMAP.md#m5--engine-adapters-exploratory) is four milestones out whatever is installed, and anything that would have to be redone once an engine is attached waits with it. But **Unreal Engine 5.8 is installed here**, with Visual Studio and MSVC, so measuring against a real engine was available the whole time it was being called impossible — and [the reading reversed M0's conclusion about which condition binds](docs/measurements/2026-07-31-022200-an-unreal-session.md). Unity, Blender and Godot are genuinely absent.
- **Check what the machine has before writing down what it lacks.** The claim above was carried through a day of decisions — it sent G0 to "blocked", picked a Rust build as a stand-in, and shaped this file. Confirming it cost one directory listing, and the cheapest moment to spend that was when the sentence was first written.

## Commits

[CONTRIBUTING's commit rules](CONTRIBUTING.md#commit-messages) are binding: Conventional Commits, one of eleven types, no co-author trailers of any kind. Commit when a change is coherent and green, not per edit.
