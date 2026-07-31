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
- **Every measured improvement gets a record with an as-is column beside the to-be one.** The as-is column is what makes it a measurement rather than an announcement. It belongs in [docs/measurements/](docs/measurements/) as `YYYY-MM-DD-HHMMSS-<slug>.md`, stamped with the moment the record was committed rather than the moment the run happened — a clock the next clone can still read, so the records sort into the order they were written on any machine. Publish it — an engineering figure has no reason to be private. Keep `.local` for what genuinely cannot be, which so far is unit economics and funding posture and nothing else.
- **A KPI that restates an existing record is a view of it, not a second record.** Fold what is new — usually the cost of the change — into the record that owns the measurement, and do not create the file.
- **Reach the same number twice by different routes, and the agreement is the check.** It keeps catching things: `η` above 1 said the core count was physical where the measurement was logical; one session's average over time matching four sessions' total at an instant said a median was a ceiling rather than a blend; and the job object's RSS moving 1.79 GiB while machine headroom moved 1.69 said neither was losing sessions nor counting strangers. Cheaper than a repeat and it catches a different class of error — a repeat tells you the number is stable, two routes tell you it is the number you meant.
- A number that contradicts the plan rewrites the plan, never the other way around.
- **A conclusion drawn from a stand-in is a conclusion about the stand-in.** M0 spent a day establishing that memory had orders of headroom and cores were what bound the machine. Both were true of a workload holding 20 MiB and [false of an engine holding 1.69 GiB](docs/measurements/2026-07-31-022200-an-unreal-session.md), which reversed which condition trips first. Synthetic workloads exist to isolate one variable, and the moment their answer gets quoted as a fact about the real thing, name the workload in the same sentence.

  The same holds for rules rather than numbers: **an invariant only exercised by things that cannot break it has not been exercised.** [The workload contract](workloads/README.md#the-contract) forbids shared paths and the ramp handed every session one directory, which three synthetic workloads never noticed because each names its files uniquely. The contract was correct and unenforced for as long as nothing needed it.
- Claims in PLAN are marked **[measured]** or **[assumed]**. If you turn one into the other, edit the marker in the same change.

### Treat the machine as a variable

A long session of back-to-back ramps makes the box slower, and a slower box is not a different mood — it is a different machine, so late measurements stop being comparable with early ones. This is measured rather than suspected: **a rung that ran 40.02 units/s at the start of one ramp ran 38.11 at the end of it, 4.8% down**, and that alone dragged the fitted redline from ~33.6 to 32.3.

Every ramp now repeats its lowest saturated rung at the end and reports the gap. **Read that line before quoting anything else in the run.** Past a few percent the redline is reading low and the run is a draft.

**That control covers one ramp, and most claims here span two.** Pipes against pseudoconsoles, `cmd` against `pwsh`, one duty against another — each sets two ladders side by side and assumes they saw the same machine. A pseudoconsole ramp ran nine hours after its pipes counterpart and its solo session was **half the speed**, which no drift check could catch because both ramps held still internally. **Two ramps are comparable when their solo rungs agree**: that rung is one session with no contention doing the same work, so it is a machine fingerprint, and comparing redlines whose fingerprints differ measures the afternoon. `sessionbench compare` refuses the pair and exits non-zero. Run a comparison back to back or not at all.

**When a measurement will not fit the machine, shrink the measurement before blaming the machine.** A ten-session cook ramp needed 19 GiB against 8.7 free and was written up as blocked; four sessions answered the same question twenty minutes later, because aligned peaks would have shown 20 GiB and nothing came within a factor of two. The narrow memory was a condition rather than an obstacle — wide enough to leave the signal intact, narrow enough to make alignment visible. Reach for the smallest design that can still be wrong in the way you care about.

Whatever you watch a run with must emit a per-rung line, not only its verdict. A watch filtered down to redlines and failures cannot tell a working run from a hung one, and that ambiguity gets settled by reaching for the box — which is the one thing forbidden below.

While a measurement is running, do nothing else on this machine — no builds, no `git`, no `gh`, no file edits. The observer is not free. [The Defender estimate](docs/measurements/2026-07-30-142218-defender-at-scale.md) was wrong by two orders of magnitude partly because `gh` ran during the run that produced it, and the drift figure above came from a ramp with edits happening alongside it.

Between runs, keep the footprint from accumulating:

- **Prune `bench-out/`, once the record exists.** Each ramp leaves roughly 500 files, all on the scanned side of Defender's line, and thirty-five of them once reached 7 GB. Keep the newest run or two. It is gitignored and nothing committed points into it, which is exactly why an unwritten run has its only copy there — the record is the artifact, this is scratch.
- **Check for survivors** before trusting a run — `Get-Process cpu-spin,file-write,stdout-storm,sessionbench`. Teardown reaps its own scratch and its own sessions, and has been verified to leave nothing, but a killed ramp is a different story.
- **Do not reach for `cargo clean`.** It buys back a few GiB and costs a full rebuild, which is the slow thing the cleanup was supposed to fix.

## Verify before you report

Running `cargo build` proves the code compiles, which is not what anyone asked. Run the thing, read its output, and check the output against something independent.

**A check has to measure the thing it is checking**, and the cheapest way to get this wrong is to read whatever is nearest. Four in one day, each answering confidently from beside the instrument that decides:

- **The console is not the report.** A run streams progress and then writes a record; a rung line printed mid-ladder can be superseded and a refine pass revisits counts that looked settled. Two findings built on streamed lines were withdrawn when `ramp.json` disagreed — one a process-count anomaly the finished report showed as exactly 2.00 per session on every rung. Watch the console to know a run is alive; quote nothing from it.
- **A point sample is not a windowed mean.** A watcher waiting for `doctor` to call the machine quiet sampled instantaneous CPU instead, reported 8.4% and called it quiet while `doctor` read 23% of the same minute.
- **A frequency heuristic is not an efficiency class.** The pure-Rust way to find hybrid cores groups them by observed clock, and clocks converge on an idle machine — which is when `doctor` runs, so it would report one tier here and be believed.
- **A link checker is not GitHub.** Collapsing runs of spaces where GitHub replaces each one reported 23 sound anchors as broken; every one would have been "fixed" into an actual break.

**The tell is a confident answer from an instrument nobody checked against the one that decides**, and the adjacent instrument is always the convenient one. When several things look wrong at once, suspect the check before the subject — one place being wrong is usually that place, many places being wrong is usually you.

Before saying a change works:

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo +1.88.0 check --all-targets      # the declared MSRV, which the tracking toolchain never exercises
cargo run -p sessionbench -- doctor    # and read it
```

When an independent check disagrees with the code, suspect the check first — but say so either way. [CONTRIBUTING's three accident modes](CONTRIBUTING.md#the-one-rule-that-matters) are each a real mistake made in this repository.

### Verify before you launch, too

Two ramps in one afternoon measured nothing, and both were launched from a parameter that was never read back from whatever would consume it.

- `--max-sessions 40` fell between the ladder's own steps of 25 and 50, so it could not bracket and returned a floor. Those steps are a constant in `redline.rs` that nothing stopped anyone reading first.
- `cmd /c` arrived as `cmd C:/`, because Git Bash rewrites a leading-slash argument as a Windows path. Every session died instantly, the shell's error lines were counted as completed units, and four rungs came back "held" at zero bytes resident. `ramp.json` records the argv it really spawned.

**A smoke test written beside the script does not cover the script.** The one that passed here used `cmd //c` — differing from the real command in exactly the character that was broken, so it confirmed a shape nothing would run.

So for anything that will occupy the machine for more than a rung: read the parameter back from the code or the artifact that consumes it, and pre-flight the real command shape. A twenty-second `observe` that prints the recorded argv costs a six-hundredth of the ramp it protects.

## Stay inside the open gate

One question decides whether to start any piece of work: **does this move the number on the gate that is currently open?** If not, it belongs to a later milestone.

Two consequences that are easy to miss:

- **Order holds inside a milestone too.** M0 runs one session to completion *before* the ramp harness exists, because the single run is what says whether the redline conditions are pointed at the right thing.
- **Engines are workloads before they are integrations.** [M5](ROADMAP.md#m5--engine-surfaces-exploratory) is four milestones out whatever is installed, and nothing in the daemon will ever know an engine exists. But **Unreal Engine 5.8 and Unity 6000.4.7f1 are both installed here**, with Visual Studio and MSVC, so measuring against a real engine was available the whole time it was being called impossible — and [it reversed which condition binds](docs/measurements/2026-07-31-022200-an-unreal-session.md), then [showed that one engine installation builds one thing at a time](docs/measurements/2026-07-31-034150-unreal-builds-serialise.md). Blender and Godot are genuinely absent.

**This paragraph got it wrong twice, which is the point of it.** Unreal was called absent until a directory listing said otherwise, that listing became the rule above, and Unity was then left in the "absent" list by the same paragraph — unchecked for another day. Having the rule is not the same as noticing the moment it applies, and the moment is whenever a sentence is about to say a machine lacks something.
- **Check what the machine has before writing down what it lacks.** The claim above was carried through a day of decisions — it sent G0 to "blocked", picked a Rust build as a stand-in, and shaped this file. Confirming it cost one directory listing, and the cheapest moment to spend that was when the sentence was first written.

## Commits

[CONTRIBUTING's commit rules](CONTRIBUTING.md#commit-messages) are binding: Conventional Commits, one of eleven types, no co-author trailers of any kind. Commit when a change is coherent and green, not per edit.

**And commit before any wait whose end you cannot see** — a ramp, a long build, anything measured in hours. The session that resumes is not always the process that started: one ended mid-ramp and came back seven hours later, leaving an instrument fix uncommitted and three finished ramps recorded nowhere but `bench-out/`. Everything survived, and none of that was by design. Work that exists only in a working tree or a gitignored directory is invisible the moment the thing holding it stops.
