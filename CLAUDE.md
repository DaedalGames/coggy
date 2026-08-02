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
- **Where a deciding figure is written decides whether it survives.** Three shapes turned up in one day. A figure that justified a design and lived only in a task note — the 8.5% behind bracketing, the 581.6 MiB behind "the tool runs at a hundred" — goes when the task closes, and the design stays without it. A figure written only beside the constant it set survives, but nothing else can reach it: `MIN_SAMPLES_PER_RUNG`'s starved-sampler run has no record, and its artifacts are pruned. And a figure that *is* recorded still needs the thing it decided to point at it, or the stronger evidence goes uncited while the weaker one gets quoted — PLAN cited where `2ηC/d` was derived while claiming it had been checked elsewhere. **The check is mechanical**: grep the repository for the figures a rule or a constant leans on, and for records nothing outside the index cites.
- **Every measured improvement gets a record with an as-is column beside the to-be one.** The as-is column is what makes it a measurement rather than an announcement. It belongs in [docs/measurements/](docs/measurements/) as `YYYY-MM-DD-HHMMSS-<slug>.md`, stamped with the moment the record was committed rather than the moment the run happened — a clock the next clone can still read, so the records sort into the order they were written on any machine. Publish it — an engineering figure has no reason to be private. Keep `.local` for what genuinely cannot be, which so far is unit economics and funding posture and nothing else.
- **A record is a log, and rewriting a log stops it being one.** Every other artifact here gets refreshed until it says what is true now; a measurement record says what was true when it was taken, including what it could not do yet. When a record's open question gets answered, **date the claim and append the answer** rather than replacing it — an attempt to swap *nothing checks this* for *this is checked* left one paragraph asserting both. What a run found, and what nobody had built at the time, are the same kind of fact.
- **A KPI that restates an existing record is a view of it, not a second record.** Fold what is new — usually the cost of the change — into the record that owns the measurement, and do not create the file.
- **Reach the same number twice by different routes, and the agreement is the check.** It keeps catching things: `η` above 1 said the core count was physical where the measurement was logical; one session's average over time matching four sessions' total at an instant said a median was a ceiling rather than a blend; and the job object's RSS moving 1.79 GiB while machine headroom moved 1.69 said neither was losing sessions nor counting strangers. Cheaper than a repeat and it catches a different class of error — a repeat tells you the number is stable, two routes tell you it is the number you meant.

  **A rearrangement is not a route, and it agrees to the last digit.** Three ways of getting `η` on this machine — dividing a slowdown, projecting the core crossing, and dividing total throughput by a solo — came to 0.7334, 0.7334 and 0.7333, which is what algebra does when every one of them reduces to `N·d/(C·s)`. That agreement was about to be written up as a confirmation. **Before believing two routes, substitute one into the other**: if the second collapses into the first, you have one measurement and a spreadsheet. The tell is that they share an input — all three of those divide by the same solo baseline, which is the noisiest number this instrument produces.

  **And when one run supplies a figure, put it beside the other runs before building on it.** A solo set spreading 25% became *the noisiest number this instrument produces* in six documents, motivated two designs that avoid it, and costed a third at nine repeats a side. [Three other sets the same day spread 3.9–8.4%](docs/measurements/2026-08-02-221853-the-noisy-baseline-was-one-noisy-afternoon.md), and the loud one is the only run whose background collapsed mid-run — a solo takes a disturbance at full strength where a hundred sessions divide it. The tell is a number that arrives once and is cited as a property. **The four sets existed all day; the check was one table.**

  **The same substitution catches a requirement that is really a restatement.** *Gate M1 needs about nineteen logical processors* was derived as `N·d ÷ 2η`, and `η` here is `N·d ÷ (C·s)`, so the requirement reduces to `C·s/2` — sixteen cores times this machine's own 2.301, halved. It reads as a fact about the gate and is a fact about this box. **A threshold computed from a quantity measured on the thing being judged is not a threshold**, and the giveaway is that the substitution leaves the subject's own numbers on both sides.
- A number that contradicts the plan rewrites the plan, never the other way around.
- **A conclusion drawn from a stand-in is a conclusion about the stand-in.** M0 spent a day establishing that memory had orders of headroom and cores were what bound the machine. Both were true of a workload holding 20 MiB and [false of an engine holding 1.69 GiB](docs/measurements/2026-07-31-022200-an-unreal-session.md), which reversed which condition trips first. Synthetic workloads exist to isolate one variable, and the moment their answer gets quoted as a fact about the real thing, name the workload in the same sentence.

  **The stand-in is sometimes the window rather than the workload.** *The daemon is 2% of what it holds* was measured at twenty-five seconds with its buffers nearly empty; filled, the same daemon reads 5.8%. Neither figure is more correct — they are different states of the same thing, and the sentence named only the daemon. And a ratio is a fact about a pairing before it is a fact about either side: against nine real sessions that same daemon is 0.16%, so **what travels is the absolute — 363 KiB a session held — and the percentage travels nowhere.**

  The same holds for rules rather than numbers: **an invariant only exercised by things that cannot break it has not been exercised.** [The workload contract](workloads/README.md#the-contract) forbids shared paths and the ramp handed every session one directory, which three synthetic workloads never noticed because each names its files uniquely. The contract was correct and unenforced for as long as nothing needed it. **So break a new check once, on purpose, before trusting it** — the documentation-map test was shown naming `coggyd` with its row removed, `--locked` was shown refusing a lockfile missing that crate, and a solo control that could only ever report +0.0% was caught by two console lines that could not both be true. A check whose failure nobody has seen is a check nobody has read.

  **A measurement has the same failure, and it is quieter.** Three solo holds of `ping` returned 23 units each — flawless reproducibility from a workload that emits one line a second whatever the machine is doing. The same three with `cpu-spin` gave 585, 590 and 606. `ping` is the obvious thing to hold a hundred of while checking that sessions start and stop, and its unit count is elapsed time wearing a work rate's name. **Before believing a number that did not move, ask what would have moved it.** The same property is useful pointed the other way: because `ping` is a clock, [a hundred of it emitting 1.04 lines a session a second](docs/measurements/2026-08-01-163935-what-the-harness-says-about-itself.md) is a check on the session count and the window rather than on the machine.
- **A paragraph replaced leaves its summary behind.** The same edit that withdraws an explanation usually leaves the sentence restating it a few lines down, and both then sit in one file disagreeing: a README kept *lifetime, output and identity* after two sections were added, and a record kept *sessions more alike than an engine's* one paragraph after that reading was retracted. It is the counting rule's sibling — **when you change a thing, search the file for the sentence that describes it.**
- **And a record that grows leaves its citers behind**, which is the same shape across files rather than within one. A fifth reading moved the engine figure to 1.87 GiB ± 6.4%; ROADMAP and the measurement index went on quoting the 1.865 that four readings gave, because appending to a log is correct and nothing points from the log back to whoever quoted it. [A test now refuses a decimal quoted inside a link that its target does not hold](sessionbench/tests/docs.rs) — so what is left to you is the integers and the prose. **After appending to a record, grep the repository for the figure you superseded.**

  **The test only sees numbers, and a link can be wrong without one.** A sentence pointing at ROADMAP's M1 section for a claim about measurement allowances resolved cleanly — the anchor exists, so both link checks passed — and the target says nothing on the subject. That was written hours after the same defect was swept for and covered by a test, which is the honest limit of the test: **a citation carrying no figure is checked by nobody but you.**

  Four of those in one day, all the same shape: the right *document* and a plausible-sounding *section*. `#fixed-contracts` sounds like where a contract rule lives; the rule was in the workload contract. Guessing is what happens when the alternative is opening the file, so make the alternative cheap — `grep -n "^#" <file>` lists every heading in one line of output, and reading the section you are about to cite costs a second more. **Pick anchors from a list you are looking at, not from memory of what the document probably calls it.**
- **A sentence that counts a list will outlive its accuracy.** *Three accident modes*, *eight findings*, *those three rules*, *a session's lifetime, output and identity* — each was written beside the list it describes and each went wrong the next time the list grew, usually in the same change that grew it. The [map-completeness test](sessionbench/tests/docs.rs) catches a missing component, not a stale tally inside a page. So name the list or describe it, and leave the arithmetic to the reader who can see it.
- Claims in PLAN are marked **[measured]** or **[assumed]**. If you turn one into the other, edit the marker in the same change.

### Treat the machine as a variable

**Check the plug before the numbers.** A hundred sessions at duty 0.27 gave 907 units/s on mains and [135 seven hours later on battery](docs/measurements/2026-08-02-195840-the-same-command-on-battery.md) — 7.8×, from the same command with every other recorded figure identical: same session count, same RSS to a hundredth of a gibibyte, a steady climb rather than a stall, and a thermal zone at 42 °C. A run that is merely slow produces a clean artifact, so nothing looks wrong until a figure is set beside one from the other state. `doctor` says which state it is in and every artifact now carries it, which is what makes this a check rather than a thing to remember.

**Three scales, each an order of magnitude apart, and only the smallest supports a comparison.** The power state is worth 7.8×, one run against another on the same power is worth up to 14%, and one hold against another inside a run is worth 0.4% to 1.5%. So a claim built on two holds of one run stands, a claim built on two runs needs their fingerprints to agree first, and a claim across a power state is not a claim. The gap is also why the largest one hid for a day: a figure carrying it does not read as a noisy version of the other state, it reads as a measurement of something else entirely.

A long session of back-to-back ramps makes the box slower, and a slower box is not a different mood — it is a different machine, so late measurements stop being comparable with early ones. This is measured rather than suspected: **a rung that ran 40.02 units/s at the start of one ramp ran 38.11 at the end of it, 4.8% down**, and that alone dragged the fitted redline from ~33.6 to 32.3.

Every ramp now repeats its lowest saturated rung at the end and reports the gap. **Read that line before quoting anything else in the run.** Past a few percent the redline is reading low and the run is a draft.

**That control covers one ramp, and most claims here span two.** Pipes against pseudoconsoles, `cmd` against `pwsh`, one duty against another — each sets two ladders side by side and assumes they saw the same machine. A pseudoconsole ramp ran nine hours after its pipes counterpart and its solo session was **half the speed**, which no drift check could catch because both ramps held still internally. **Two ramps are comparable when their solo rungs agree**: that rung is one session with no contention doing the same work, so it is a machine fingerprint, and comparing redlines whose fingerprints differ measures the afternoon. `sessionbench compare` refuses the pair and exits non-zero. Run a comparison back to back or not at all.

**When a measurement will not fit the machine, shrink the measurement before blaming the machine.** A ten-session cook ramp needed 19 GiB against 8.7 free and was written up as blocked; four sessions answered the same question twenty minutes later, because aligned peaks would have shown 20 GiB and nothing came within a factor of two. The narrow memory was a condition rather than an obstacle — wide enough to leave the signal intact, narrow enough to make alignment visible. Reach for the smallest design that can still be wrong in the way you care about.

Whatever you watch a run with must emit a per-rung line, not only its verdict. A watch filtered down to redlines and failures cannot tell a working run from a hung one, and that ambiguity gets settled by reaching for the box — which is the one thing forbidden below.

**Ask what this machine can still answer before calling the window shut.** A loud box spoils *work rate*, which is the whole of a redline's noisy half. It barely touches RSS — an hour-long hold said so in its own opening line — and it touches a teardown less still, because processes leaving is not work being done. A day spent waiting for quiet had the graceful teardown at a hundred sessions available the whole time, untested, and it took fourteen seconds at 48% background. **A window is shut for a quantity, not for the afternoon.**

**And a ratio wants a steady machine rather than a quiet one.** The threshold above is written for an absolute work rate. A measurement that divides one hold by another hands both the same handicap, so a background sitting at a steady 40% leaves the shape of the answer intact — while [one *swinging* between 5.07 and 6.97 cores over a minute](docs/measurements/2026-08-01-163935-what-the-harness-says-about-itself.md) puts ±16% into a comparison chasing 6.9%, and every hold sees a different machine. So for a ratio, read a few consecutive `doctor` lines and look at their spread, not one line against ten percent. **And the baseline has to bracket the run rather than precede it** — [two solo triples ten minutes apart differ by 8.5% where each spreads under 3.5%](docs/measurements/2026-08-01-163935-what-the-harness-says-about-itself.md), so drift between runs is three times the noise inside one.

While a measurement is running, do nothing else on this machine — no builds, no `git`, no `gh`, no file edits. The observer is not free. [The Defender estimate](docs/measurements/2026-07-30-142218-defender-at-scale.md) was wrong by two orders of magnitude partly because `gh` ran during the run that produced it, and the drift figure above came from a ramp with edits happening alongside it.

Between runs, keep the footprint from accumulating:

- **Prune `bench-out/`, once the record exists.** Each ramp leaves roughly 500 files, all on the scanned side of Defender's line, and thirty-five of them once reached 7 GB. Keep the newest run or two. It is gitignored and nothing committed points into it, which is exactly why an unwritten run has its only copy there — the record is the artifact, this is scratch.
- **Check for survivors** before trusting a run — `Get-Process cpu-spin,file-write,stdout-storm,sessionbench`. Teardown reaps its own scratch and its own sessions, and has been verified to leave nothing, but a killed ramp is a different story.
- **Do not reach for `cargo clean`.** It buys back a few GiB and costs a full rebuild, which is the slow thing the cleanup was supposed to fix.

## Verify before you report

Running `cargo build` proves the code compiles, which is not what anyone asked. Run the thing, read its output, and check the output against something independent.

**A check has to measure the thing it is checking**, and the cheapest way to get this wrong is to read whatever is nearest. Four in one day, each answering confidently from beside the instrument that decides:

- **The console is not the report.** A run streams progress and then writes a record; a rung line printed mid-ladder can be superseded and a refine pass revisits counts that looked settled. Two findings built on streamed lines were withdrawn when `ramp.json` disagreed — one a process-count anomaly the finished report showed as exactly 2.00 per session on every rung. Watch the console to know a run is alive; quote nothing from it. **And a report still being written is not the report either.** In an hour-long hold sampled every five minutes, one window's growth rate was called a constant, then three windows were called a deceleration with a mechanism attached, and a fourth window 58% higher killed both — while the whole-span figure agreed with the segment mean the entire time. The tell is deriving a rate, a slope or a ratio from consecutive samples while more are still arriving. And the number all of that was extrapolating toward gets *measured* two samples later, which is the point: **do not model what the running measurement is about to hand you.**
- **A point sample is not a windowed mean.** A watcher waiting for `doctor` to call the machine quiet sampled instantaneous CPU instead, reported 8.4% and called it quiet while `doctor` read 23% of the same minute.
- **A frequency heuristic is not an efficiency class.** The pure-Rust way to find hybrid cores groups them by observed clock, and clocks converge on an idle machine — which is when `doctor` runs, so it would report one tier here and be believed.
- **A link checker is not GitHub.** Collapsing runs of spaces where GitHub replaces each one reported 23 sound anchors as broken; every one would have been "fixed" into an actual break.

**And a zero exit is not evidence of work.** Things that reported success while doing nothing: `cmd` handed a mangled switch printed three error lines a second and the ramp counted them as units; `timeout /t` returned instantly without a console and a supposed idle loop spun fifty times a second; `waitfor` rejected a hyphen in a signal name by exiting at once with status 0; a control compared a figure with itself and reported a perfect 0.0%; a failed job assignment returned an error while leaving the process running; and a PowerShell script refusing to run on an unusable machine said `exit 1` and reported 0, because piping its output makes `$LASTEXITCODE` the last *native* command's. The first five are other people's tools and the sixth was written an hour after reading this list, which is the more useful half of it. **Assert on the effect, not the status** — a file that appeared, a count that moved, a process that is still there.

**And in data, the same shape is a zero where a quantity does not exist.** Building one measurement path for a second kind of target met it four times in a day: a report line missing its unit count, a drop count nobody could take, a daemon that had not spoken yet, and an output-byte figure that had no source. Every one had `0` as the obvious value, and every one would have passed — zero drops satisfies a condition whose tolerance is zero, zero units reads as saturation, zero bytes says a hundred sessions produced nothing. **Make the type carry the absence**, and the check will not accept it by accident.

The fourth had a different answer, which is the part worth remembering: the number existed and was not reported. **Ask whether the quantity is really absent before wrapping it in an option** — for that one the fix was one `+=` in the daemon, where an `Option` would have marked an axis permanently unmeasurable.

  **A fifth instance moved the axis from values to processes.** A sampling loop asked its watcher for numbers and never asked whether the thing producing them was still alive; a daemon exiting at five seconds of an hour would have given fifty-nine minutes of empty samples and a session count frozen at its last report. **Something that stopped reporting looks exactly like something reporting the same thing** — so a loop reading a subject has to ask after the subject, not only after its numbers.

  **And a second loop had the same hole**, found hours later. Two sampling loops in two files each read a daemon and neither asked whether it was alive; fixing one is what made the other visible. **When a fix is for a shape rather than a line, grep for the shape.** The tell is a fix whose commit message describes a category — an absence reading as a value, a stopped thing reading as a steady one — and the fix touching one call site.

  **Having the rule did not fire it.** A commit teaching the daemon to count failed reads wired one of two entry points; three doc comments and a README row went on saying two of the four gate conditions were out of reach, and the README was then edited to say three, which made it claim something the ramp does not do. That is the rule's second miss, and the first is what wrote it.

  **So it was measured before a third rule was written on top.** 23 of 254 commits — **9.1%** — exist because an earlier change left something behind, and the obvious automation was rejected on its own numbers: *message uses category language* fires on 77% of commits and *touches one source file* on 24%, so the alarm fires on 18% to catch 9.1%, and most of the 9.1% is a document trailing code rather than a second call site. One in five commits alarming is a check people learn to skip, which is the same reason [a looser citation check was not made a gate](sessionbench/tests/docs.rs). **What is left is the habit, and the habit is cheapest at the moment the commit message is written**: if the message names a class, the diff should show more than one place, or say why one is all there is.

**The tell is a confident answer from an instrument nobody checked against the one that decides**, and the adjacent instrument is always the convenient one. When several things look wrong at once, suspect the check before the subject — one place being wrong is usually that place, many places being wrong is usually you.

Before saying a change works:

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo +1.88.0 check --all-targets      # the declared MSRV, which the tracking toolchain never exercises
cargo run -p sessionbench -- doctor    # and read it
```

When an independent check disagrees with the code, suspect the check first — but say so either way. [CONTRIBUTING's accident modes](CONTRIBUTING.md#the-one-rule-that-matters) are each a real mistake made in this repository.

### Verify before you launch, too

Two ramps in one afternoon measured nothing, and both were launched from a parameter that was never read back from whatever would consume it.

- `--max-sessions 40` fell between the ladder's own steps of 25 and 50, so it could not bracket and returned a floor. Those steps are a constant in `redline.rs` that nothing stopped anyone reading first.
- `cmd /c` arrived as `cmd C:/`, because Git Bash rewrites a leading-slash argument as a Windows path. Every session died instantly, the shell's error lines were counted as completed units, and four rungs came back "held" at zero bytes resident. `ramp.json` records the argv it really spawned.

**A smoke test written beside the script does not cover the script.** The one that passed here used `cmd //c` — differing from the real command in exactly the character that was broken, so it confirmed a shape nothing would run.

So for anything that will occupy the machine for more than a rung: read the parameter back from the code or the artifact that consumes it, and pre-flight the real command shape. A twenty-second `observe` that prints the recorded argv costs a six-hundredth of the ramp it protects.

## Stay inside the open gate

One question decides whether to start any piece of work: **does this move the number on the gate that is currently open?** If not, it belongs to a later milestone.

Consequences that are easy to miss:

- **Order holds inside a milestone too.** M0 runs one session to completion *before* the ramp harness exists, because the single run is what says whether the redline conditions are pointed at the right thing.
- **Engines are workloads before they are integrations.** [M5](ROADMAP.md#m5--engine-surfaces-exploratory) is four milestones out whatever is installed, and nothing in the daemon will ever know an engine exists. But **Unreal Engine 5.8 and Unity 6000.4.7f1 are both installed here**, with Visual Studio and MSVC, so measuring against a real engine was available the whole time it was being called impossible — and [it reversed which condition binds](docs/measurements/2026-07-31-022200-an-unreal-session.md), then [showed that one engine installation builds one thing at a time](docs/measurements/2026-07-31-034150-unreal-builds-serialise.md). Blender and Godot are genuinely absent.

**This paragraph got it wrong twice, which is the point of it.** Unreal was called absent until a directory listing said otherwise, that listing became the rule above, and Unity was then left in the "absent" list by the same paragraph — unchecked for another day. Having the rule is not the same as noticing the moment it applies, and the moment is whenever a sentence is about to say a machine lacks something.
- **Check what the machine has before writing down what it lacks.** The claim above was carried through a day of decisions — it sent G0 to "blocked", picked a Rust build as a stand-in, and shaped this file. Confirming it cost one directory listing, and the cheapest moment to spend that was when the sentence was first written.

## Commits

[CONTRIBUTING's commit rules](CONTRIBUTING.md#commit-messages) are binding: Conventional Commits, one of eleven types, no co-author trailers of any kind. Commit when a change is coherent and green, not per edit.

**Read `git status` before writing the message, not `git add` from memory.** Scoping the add to the paths you edited keeps a commit focused and silently drops what the change *caused* — `Cargo.lock` sat unstaged across five commits that added a crate, so a clean clone could not build any of them with `--locked`. No local gate says so, because `--locked` compares the manifests against the lockfile on disk and that one was always current. The commit is where a generated file gets lost, so that is where to look for it.

**A run, its launcher and its watcher each have a lifetime, and every accident here came from choosing them separately.** Four of them, in both directions:

- **The launcher was too short.** A gate hour started as an agent's background task, which carries a ten-minute ceiling; the probes and three two-minute baselines spent it and the run died the second the hour began. Detach it — `Start-Process` with the output redirected — so the thing holding it is not something with a timeout. **Killing that launcher does not stop the run**: `sessionbench` owns the job, so stop that instead, and it reaps the tree.
- **The watcher was too short.** The five-minute stallwatch lived only in session memory, so a reboot took it while the conversation was restored around it, and nothing ran for thirteen hours.
- **The watcher was too long.** A `Monitor` armed `persistent` tailed a log whose run had finished, and sat there for thirteen hours seeing nothing. `persistent` is for a watch with no end; a run with a known end takes a bounded timeout, and **the moment you read a run's result is the moment you retire its watcher** — make that a step of handling the result, not something to remember.
- **The watcher was bound to a path rather than to a run.** One waiting on a log file matched the *next* run's log after that file was deleted and recreated, and only by luck was it the line it wanted.

**All four produce silence, and silence is what a healthy watch produces too** — which is [the same shape the daemon's drain had](coggyd/src/lib.rs), where a failed read returned on the same branch as a clean end-of-file. So the check is never *is it quiet*; it is **whether the thing that would speak is still there and still pointed at the subject**.

**And commit before any wait whose end you cannot see** — a ramp, a long build, anything measured in hours. The session that resumes is not always the process that started: one ended mid-ramp and came back seven hours later, leaving an instrument fix uncommitted and three finished ramps recorded nowhere but `bench-out/`. Everything survived, and none of that was by design. Work that exists only in a working tree or a gitignored directory is invisible the moment the thing holding it stops.
