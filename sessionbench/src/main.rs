// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

/// How long the uncounted first hold runs.
///
/// Long enough to spawn the run's full session count and let their pages fault
/// in, short enough that it is not worth skipping. Not derived from
/// `--solo-duration`, which a run without `--with-solo` never sets.
const WARMUP_SECONDS: f64 = 30.0;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use clap::{Parser, Subcommand};
use sessionbench::Rows;
use sessionbench::axes::{self, AxisStatus};
use sessionbench::format::human_bytes;
use sessionbench::host::{HeldExclusion, HostFacts};
use sessionbench::machine::{BackgroundLoad, Machine};
use sessionbench::observe::{self, ObserveConfig, RunReport};
use sessionbench::provenance::Provenance;
use sessionbench::ramp::{self, Drift, RampConfig, RampReport};
use sessionbench::sampler::Sampler;
use sessionbench::session::SessionMode;
use sessionbench::tree::Membership;

/// Measures redline: the maximum concurrent sessions this machine sustains,
/// and the condition that limits it.
#[derive(Parser)]
#[command(name = "sessionbench", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Report the machine and which measurement axes are available.
    Doctor {
        /// Exit non-zero when any axis is unavailable.
        ///
        /// Off by default so the report always prints. Turn it on when
        /// something downstream is about to trust the result.
        #[arg(long)]
        strict: bool,
    },

    /// Run one session to completion and record what holding it costs.
    ///
    /// The step that comes before the ramp. Flag names follow `psrecord`,
    /// which settled this shape long before we needed it.
    Observe {
        /// Name for this run, used in the output directory and the report.
        #[arg(long, default_value = "session")]
        label: String,

        /// Directory runs are written under.
        #[arg(long, default_value = "bench-out")]
        out: PathBuf,

        /// Seconds between samples.
        #[arg(long, default_value_t = 1.0, value_name = "SECONDS")]
        interval: f64,

        /// Stop after this long, whether or not the session has finished.
        #[arg(long, value_name = "SECONDS")]
        duration: Option<f64>,

        /// Give the session a pseudoconsole instead of pipes.
        ///
        /// Running the same workload both ways is the direct evidence for or
        /// against defaulting to pipes: the difference is a conhost per
        /// session, resident for as long as the session lives.
        #[arg(long)]
        pty: bool,

        /// The command to run, after `--`.
        #[arg(last = true, required = true, value_name = "COMMAND")]
        command: Vec<String>,
    },

    /// Hold N sessions under `coggyd` and measure them.
    ///
    /// **Gate M1's shape, and deliberately not a ladder.** A ramp asks how
    /// many sessions fit; this asks what a stated number costs, which is what
    /// the gate is written as. It is also not a mode of `observe`, which
    /// measures one session and multiplies — here the sessions are really
    /// there.
    ///
    /// One of the four conditions cannot be asked of a daemon and the report
    /// says so rather than passing it. Replacement is that one: nothing in the
    /// daemon restarts a session that exited. Dropped output was in this
    /// sentence until the daemon was asked the question a pipe actually poses
    /// — not *is there a gap in the ordinals*, which needs a stream nobody
    /// here holds, but *did a reader give up*, which it counts.
    Hold {
        #[arg(long, default_value = "hold")]
        label: String,

        #[arg(long, default_value = "bench-out")]
        out: PathBuf,

        /// How many sessions the daemon is asked to hold.
        #[arg(long, default_value_t = 100)]
        sessions: u32,

        /// Seconds between samples.
        #[arg(long, default_value_t = 5.0, value_name = "SECONDS")]
        interval: f64,

        /// How long to hold them. The gate asks for an hour.
        #[arg(long, default_value_t = 3600.0, value_name = "SECONDS")]
        duration: f64,

        /// The daemon binary. Built by `cargo build --release -p coggyd`.
        #[arg(long, default_value = "target/release/coggyd.exe")]
        daemon: PathBuf,

        /// Total RSS the gate allows across the daemon and everything it holds.
        #[arg(long, default_value_t = 4.0, value_name = "GB")]
        rss_budget_gb: f64,

        /// Take a solo hold before and after, and report the ratio between.
        ///
        /// The work-rate condition is per-session rate against the same
        /// workload held alone, so it needs a baseline. Two of them, on
        /// either side: two solo triples ten minutes apart differ by 8.5%
        /// where each spreads under 3.5%, so drift between runs is three
        /// times the noise inside one and a baseline taken only beforehand
        /// belongs to a machine that may have left.
        ///
        /// **Not "more than a concurrent run's whole effect"**, which this
        /// said until it was read against the gate: a hundred sessions slow a
        /// session by 130%, and 8.5% is nowhere near it. The comparison it
        /// swamps is a narrow one — a duty pairing chasing 6.9%.
        ///
        /// Costs `2 × --solo-repeats` extra holds of `--solo-duration` each.
        #[arg(long)]
        with_solo: bool,

        /// How long each solo hold runs, when `--with-solo` is given.
        #[arg(long, default_value_t = 120.0, value_name = "SECONDS")]
        solo_duration: f64,

        /// Solo holds per side, averaged into that side's baseline.
        ///
        /// **Three rather than one, because a single solo baseline is the
        /// loudest term in the whole comparison.** Twelve fresh one-session
        /// holds spanned 4.54% with nothing between them, against an allowance
        /// of 5% — so a bracket refuses itself on an unlucky pair often enough
        /// to lose an hour-long run to it. Four sets measured across one day
        /// put a hold's own spread at 4–8%, which three repeats bring to a
        /// standard error of 2.3–4.0% and inside the allowance.
        ///
        /// **Five, because three refuses a third of stable runs — and the
        /// paragraph above stops one step short of saying so.** Checking that
        /// the standard error fits under the allowance is not the question;
        /// what decides a threshold is how often the noise crosses it. Pooled
        /// **On a healthy machine a hold reproduces to about 0.4%**, so the
        /// allowance carries roughly twelve times the margin it needs and this
        /// paragraph's arithmetic almost never binds. The gate runs' own
        /// baselines say so: six solo holds at σ **0.42%** behind slowdown
        /// 2.0654, 0.39% behind 2.0799, 0.52% behind 3.2611.
        ///
        /// **A night of 6–16% σ was this box being sick, and it was nearly
        /// written up as a property of the instrument.** Twenty-five holds
        /// under a neighbour spread 9.44% and ten on an idle box 15.87%, from
        /// which a refusal table was derived, a default was changed, and two
        /// tasks were opened about a mis-specified allowance — all describing
        /// [one degraded
        /// evening](../../docs/measurements/2026-08-03-173452-the-slow-state-flatters-the-gate.md).
        ///
        /// **So read the spread as a health check rather than a constant.**
        /// Baselines agreeing inside about 1% mean the machine is fit to
        /// measure on; several percent means it is not, and no repeat count
        /// fixes that because more samples of a wandering box is not what is
        /// missing. Five a side rather than three is cheap insurance and not a
        /// remedy:
        /// nine a side is eighteen baseline holds and still refuses one stable
        /// run in six. The allowance being finer than a single hold's noise is
        /// [its own open question](../../docs/measurements/2026-08-03-173452-the-slow-state-flatters-the-gate.md).
        ///
        /// Three put the allowance at one standard error of the instrument's
        /// own noise, which is a coin weighted 2:1 rather than a check — paid
        /// on the run hardest to repeat. Five costs four extra holds; nine
        /// costs twelve and reaches 8%, so it is the right default for an
        /// hour-long run and too much baseline for a short
        /// [one](../../docs/measurements/2026-08-03-173452-the-slow-state-flatters-the-gate.md).
        ///
        /// **The other lever has the wrong sign.** Widening the 5% trades
        /// false refusals for accepting a machine that really moved, which is
        /// the failure this exists to prevent. Repeats shrink the noise; the
        /// allowance is the standard.
        ///
        /// **Nine was once rejected on evidence that was itself wrong.** A set
        /// spreading 28.5% implied a σ needing nine, and that set is [the only
        /// one whose background collapsed while it
        /// ran](../../docs/measurements/2026-08-02-221853-the-noisy-baseline-was-one-noisy-afternoon.md),
        /// from 28% to 9%. The rejection was right about the afternoon and
        /// wrong about the setting.
        ///
        /// **6% is this laptop at this workload.** The arithmetic travels and
        /// the constant does not: measure the local per-hold spread — thirteen
        /// holds, against one refused hour — before choosing `n` elsewhere.
        ///
        /// **And a longer hold does not help**, which is the part worth knowing
        /// before reaching for `--solo-duration` instead. What moves a solo
        /// hold is fixed for that hold's whole length — same share of CPU,
        /// different work done with it — so stretching one averages nothing.
        /// Separate launches are what sample it.
        #[arg(long, default_value_t = 5, value_name = "N")]
        solo_repeats: usize,

        /// The workload each session runs, after `--`.
        ///
        /// `${session}` in any argument expands to that session's own id, so a
        /// hundred sessions can be given a hundred paths from one command
        /// line without the workload learning anything about COGGY.
        #[arg(last = true, required = true, value_name = "COMMAND")]
        command: Vec<String>,
    },

    /// Measure what a Defender exclusion is worth, by running one workload
    /// twice.
    ///
    /// Adds a path exclusion for the directory the second run writes into and
    /// removes it afterwards. That is a change to this machine's real-time
    /// protection while the run lasts, held for the shortest window the
    /// measurement allows and removed even if the run fails.
    ExclusionDelta {
        #[arg(long, default_value = "exclusion")]
        label: String,

        #[arg(long, default_value = "bench-out")]
        out: PathBuf,

        #[arg(long, default_value_t = 1.0, value_name = "SECONDS")]
        interval: f64,

        /// How long each half runs.
        ///
        /// Both halves are cut at the same startup window, so this has to
        /// outlast it by enough to leave a steady state.
        #[arg(long, default_value_t = 90.0, value_name = "SECONDS")]
        duration: f64,

        /// How many watched/excluded pairs to run, alternating.
        ///
        /// One pair cannot separate the exclusion from whatever else the
        /// machine was doing: background activity here drifted by three times
        /// the signal in twenty seconds. Pairs run adjacent in time so slow
        /// drift lands on both halves, and the spread across pairs is what
        /// says whether the mean means anything.
        #[arg(long, default_value_t = 3)]
        repeats: u32,

        #[arg(long)]
        pty: bool,

        #[arg(last = true, required = true, value_name = "COMMAND")]
        command: Vec<String>,
    },

    /// Climb the session ladder until a redline condition breaks.
    ///
    /// Each rung holds its session count for the whole window, replacing any
    /// that finish, and is judged against all four conditions before the next
    /// is attempted.
    Ramp {
        /// Name for this ramp, used in the output directory and the report.
        #[arg(long, default_value = "ramp")]
        label: String,

        /// Directory ramps are written under.
        #[arg(long, default_value = "bench-out")]
        out: PathBuf,

        /// Seconds between samples.
        #[arg(long, default_value_t = 1.0, value_name = "SECONDS")]
        interval: f64,

        /// How long each rung is held before it is judged.
        ///
        /// The first third is spin-up and is not measured, since sessions
        /// spawned together fault in their pages together.
        #[arg(long, default_value_t = 90.0, value_name = "SECONDS")]
        hold: f64,

        /// Highest rung to attempt.
        ///
        /// The full ladder reaches 200 sessions and will take the machine with
        /// it for the duration. Lower this when something else needs the box.
        #[arg(long, default_value_t = 200)]
        max_sessions: u32,

        /// How tight to narrow the bracket before reporting a redline.
        ///
        /// The ladder is coarse and only locates the interval the ceiling
        /// falls in; each halving of that interval costs one more hold.
        #[arg(long, default_value_t = sessionbench::redline::DEFAULT_RESOLUTION)]
        resolution: u32,

        /// Give every session a pseudoconsole instead of pipes.
        #[arg(long)]
        pty: bool,

        /// Hold each rung's sessions under this daemon instead of spawning
        /// them here.
        ///
        /// Composes with `--pty` rather than replacing it: the daemon decides
        /// who owns the sessions, the mode decides how their output is wired.
        ///
        /// **A daemon rung measures two of the four conditions.** Dropped
        /// output needs the reading end of each session's stream, which the
        /// daemon holds, and nothing in it restarts a session that exited. The
        /// report labels both rather than passing them.
        #[arg(long, value_name = "PATH")]
        daemon: Option<PathBuf>,

        /// Skip both end-of-run controls, which cost one hold each.
        ///
        /// The ramp repeats its lowest saturated rung, which says whether the
        /// ladder measured one machine, and then its solo rung, which says
        /// whether the baseline every rate is read against reproduces — and
        /// that second figure is what decides whether this run may be set
        /// against another at all. Skipping is for a ramp being run for its
        /// shape rather than its number.
        #[arg(long)]
        skip_drift_check: bool,

        /// Hide the sessions' writes from real-time scanning for this ramp.
        ///
        /// The exclusion axis at the scale it belongs at: run the same ladder
        /// with and without, and compare the two redlines. **This changes the
        /// machine's real-time protection for the length of the ramp**, over a
        /// directory the benchmark created, and removes it afterwards.
        #[arg(long)]
        exclude_scratch: bool,

        /// The command each session runs, after `--`.
        #[arg(last = true, required = true, value_name = "COMMAND")]
        command: Vec<String>,
    },

    /// Say whether two ramps may be set against each other.
    ///
    /// A drift control tests one ladder against itself. Nothing tested two
    /// ladders against each other until this, and a pseudoconsole ramp run
    /// nine hours after its pipes counterpart returned a redline 13 sessions
    /// lower for reasons that had nothing to do with the transport.
    Compare {
        /// The ramp read as the baseline.
        left: PathBuf,
        /// The ramp read against it.
        right: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Doctor { strict } => doctor(strict),
        Command::Compare { left, right } => compare(&left, &right),
        Command::Hold {
            label,
            out,
            sessions,
            interval,
            duration,
            daemon,
            rss_budget_gb,
            with_solo,
            solo_duration,
            solo_repeats,
            command,
        } => {
            if !daemon.is_file() {
                anyhow::bail!(
                    "no daemon at {} — build it with `cargo build --release -p coggyd`",
                    daemon.display()
                );
            }
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or_default();
            let out_dir = out.join(format!("{stamp}-{label}-daemon"));
            std::fs::create_dir_all(&out_dir)?;

            let interval = Duration::from_secs_f64(interval);
            println!(
                "holding {sessions} session(s) under {} for {duration}s · sampling every {}s",
                daemon.display(),
                interval.as_secs_f64(),
            );
            println!("  {}", command.join(" "));

            // One path for all three holds, so a solo pass and the concurrent
            // one cannot drift apart in how they were taken.
            let take = |name: &str, count: u32, secs: f64| -> anyhow::Result<_> {
                println!("  {name}: {count} session(s) for {secs}s");
                let run = sessionbench::daemon::hold(
                    &daemon,
                    count,
                    &command,
                    out_dir.join(format!("{name}.log")),
                    interval,
                    Duration::from_secs_f64(secs),
                    Some(out_dir.join(format!("{name}-samples.jsonl"))),
                )?;
                let samples = run.samples.clone();
                let report = run.into_report(sessionbench::daemon::Ran {
                    label: format!("{label}-{name}"),
                    daemon: daemon.display().to_string(),
                    workload: command.clone(),
                    rss_budget_bytes: (rss_budget_gb * 1e9) as u64,
                    interval,
                    // Membership is not here on purpose. The tree is armed
                    // inside `hold`, so only the run knows which source it
                    // got — this used to hardcode JobObject under a comment
                    // claiming it recorded what the run used.
                    started_unix: stamp,
                });
                // Each hold's samples are on disk already, written as they were
                // taken rather than collected here — discarding the solo
                // passes' would throw away the only per-sample CPU figures the
                // baselines produce, and writing them a second time from this
                // Vec would give one run two files that can disagree.
                Ok((report, samples))
            };

            // **One hold that nobody counts, first, whatever else this run
            // does.** The opening hold has come back the outlier twice: a gate
            // run's nine solos put its first 8% below the other eight, and an
            // A·B·A control put its opening leg 6.3% under a repeat of itself
            // while the two later legs agreed to 0.7%. A cold file cache, a
            // background still settling from whatever built the binaries, and
            // a page-fault storm across a hundred fresh processes all land on
            // whichever hold goes first.
            //
            // **Averaging is the wrong tool for it.** More repeats dilute a
            // systematic first-hold deficit rather than removing it, and they
            // cost a hold each. One short throwaway costs less and removes it,
            // so this runs at the requested session count — warming the same
            // path the run will use — and its result is dropped on the floor.
            //
            // **Outside the bracket, because a plain hold has the same first.**
            // It sat inside the `--with-solo` arm for one commit, which would
            // have left the run that exists to test it running without one:
            // comparing two bare holds is exactly the shape that found the
            // problem, and exactly the shape that would have skipped the fix.
            let _ = take("warmup", sessions, WARMUP_SECONDS)?;

            let (report, samples) = if with_solo {
                // Named by index so a side of three leaves three artifacts
                // rather than three writes to one path.
                let solos = |side: &str| -> anyhow::Result<Vec<_>> {
                    (1..=solo_repeats.max(1))
                        .map(|i| Ok(take(&format!("solo-{side}-{i}"), 1, solo_duration)?.0))
                        .collect()
                };
                let before = solos("before")?;
                let (middle, samples) = take("concurrent", sessions, duration)?;
                let after = solos("after")?;
                let bracketed = sessionbench::daemon::bracket(before, middle, after);

                if let Some(why) = &bracketed.machine_moved {
                    println!("\nMACHINE MOVED: {why}");
                }
                // The verdict is deliberately not printed here. It appears
                // once, in the summary below, because the bracket writes it
                // back into the hold — and the two lines printing it
                // separately is how they came to disagree in the first place.
                // The two side spreads print beside the gap because the gap
                // means nothing without them: a solo hold's rate is mostly
                // which core its one session got, so a side that scatters as
                // far as the gap has not measured the gap.
                let percent =
                    |v: Option<f64>| v.map_or_else(|| "—".to_string(), |x| format!("{x:.1}%"));
                let cores =
                    |v: Option<f64>| v.map_or_else(|| "—".to_string(), |x| format!("{x:.2} cores"));
                // **The spread is a health check and the number alone does not
                // say so.** Asking whether 12.4 units/s is slow needs a
                // remembered 21.8; asking whether six holds agree needs
                // nothing, so it reads on hardware nobody has characterised.
                // Six holds span 0.42% on this box when it is well, the spread
                // behind the gate's 2.0654, against 9 to 16% on the night a
                // whole evening of arithmetic was built on the wrong premise.
                // More repeats do not fix the second.
                //
                // **1.5% is chosen from a gap, not rounded to.** Healthy
                // brackets here spread 0.42%, 0.39% and 0.52%; m1's 3.33%
                // is the one healthy run with a cold opening hold. Sick
                // ones spread 6.0, 9.4, 15.9, 17.0, 21.5 and 36.8. Nothing
                // observed sits between 0.52 and 6.0, so the bar is 2.9x
                // above the tightest population and 4.0x below the loosest
                // — and it is one box, so the number travels only as a
                // method: measure your own healthy spread and put the bar
                // in the gap.
                let health = |v: Option<f64>| match v {
                    Some(x) if x <= 1.5 => "  <- fit to measure on",
                    Some(_) => "  <- THE SPREAD IS THE MACHINE: unfit to measure on",
                    None => "",
                };
                let worst = bracketed
                    .before_spread_percent
                    .into_iter()
                    .chain(bracketed.after_spread_percent)
                    .reduce(f64::max);
                println!(
                    "\n  solo spread {} before · {} after{}\n  solo error  ±{} before · ±{} after\n  solo rest   {} before · {} after\n  solo gap    {}\n  slowdown    {}",
                    percent(bracketed.before_spread_percent),
                    percent(bracketed.after_spread_percent),
                    health(worst),
                    // The spread is what the holds did; the error is what the
                    // side's mean is worth. Only the second can be set against
                    // an allowance on a difference of means.
                    percent(bracketed.before_error_percent),
                    percent(bracketed.after_error_percent),
                    // **Before the gap, because the gap cannot see it.** Two
                    // baselines inside one disturbance agree with each other.
                    cores(bracketed.before_rest_cores),
                    cores(bracketed.after_rest_cores),
                    percent(bracketed.solo_gap_percent),
                    bracketed
                        .slowdown
                        .map_or_else(|| "—".to_string(), |s| format!("{s:.2}×")),
                );
                std::fs::write(
                    out_dir.join("bracket.json"),
                    serde_json::to_string_pretty(&bracketed)?,
                )?;
                (bracketed.concurrent, samples)
            } else {
                take("concurrent", sessions, duration)?
            };

            // **No top-level `samples.jsonl` here, unlike `observe` and
            // `ramp`.** It held a second copy of what `concurrent-samples.jsonl`
            // already has, and once the streamed copy exists the duplicate is
            // strictly worse: it is written at the end, which is precisely the
            // moment a crash takes. `hold.json` pairs with the concurrent
            // hold's own file.
            let _ = &samples;
            std::fs::write(
                out_dir.join("hold.json"),
                serde_json::to_string_pretty(&report)?,
            )?;

            if let Some(why) = &report.inconclusive {
                println!("\nINCONCLUSIVE: {why}");
            }
            println!(
                "\n  sessions   {} (fewest alive {:?})\n  window     {} ms counted of {} ms held\n  peak rss   {} of {}\n  units      {:?} in {} bytes\n  rate       {} units/s/session\n  total      {} units/s across all sessions\n  occupancy  {}{}\n  rss        {:?}\n  work rate  {:?}\n  dropped    {:?} ({} failed reads)\n  replaced   {:?}",
                report.sessions,
                report.fewest_running,
                // The rate's real denominator beside the one people assume it
                // is. Their gap is the teardown, which grows with the session
                // count, so two holds of different widths are not divisible
                // without seeing it.
                report
                    .counted_ms
                    .map_or_else(|| "—".to_string(), |c| c.to_string()),
                report.duration_ms,
                human_bytes(report.peak_rss_bytes),
                human_bytes(report.rss_budget_bytes),
                report.units,
                // A dash, not a zero. Its neighbour prints None when the
                // daemon never reported, and unwrapping this one to 0 beside
                // it read as a measured nothing.
                report
                    .output_bytes
                    .map_or_else(|| "—".to_string(), |b| b.to_string()),
                report
                    .units_per_session_per_sec
                    .map_or_else(|| "—".to_string(), |r| format!("{r:.3}")),
                // **The state fingerprint, which was on disk in nine runs
                // before anything printed it.** Read across every
                // hundred-session hold here this figure is bimodal — six
                // between 903 and 1055 units/s, three between 217 and 288,
                // nothing in the 3.1× gap — so a machine running at a quarter
                // speed is legible from one hold, with no baseline and no
                // bracket.
                //
                // **All nine of those were quiet, and the gap has since been
                // filled from both sides** — 344.9 under a tenant, and 756.1 on
                // the quietest box yet measured at 0.49 cores held. So the two
                // clusters described ten readings rather than the machine, and
                // what survives is weaker: a total far below ~900 means
                // something is wrong, and the occupancy line beside it says
                // whether the something is a neighbour. The per-session rate above cannot do that: two
                // solos agreeing to half a percent sat on boxes 3.7× apart
                // [here](../../docs/measurements/2026-08-03-173452-the-slow-state-flatters-the-gate.md).
                //
                // Printed rather than judged. The clusters are one laptop's,
                // and a threshold taken from them would travel to machines
                // whose boundary is somewhere else entirely.
                report
                    .units_per_session_per_sec
                    .map_or_else(|| "—".to_string(), |r| {
                        format!("{:.1}", r * f64::from(report.sessions))
                    }),
                // **Median first, and the loss beside it.** A mean alone read
                // three holds as a footprint effect on the workload when one of
                // them had simply lost the machine for two multi-minute
                // episodes; their medians agreed to 0.7%.
                report.occupancy.map_or_else(
                    || "—".to_string(),
                    |o| {
                        format!(
                            "{:.2} cores median, {:.2} mean, {:.3} lost · {:.2} cores held outside the job",
                            o.median_cores, o.mean_cores, o.lost_cores, o.rest_cores_median
                        )
                    },
                ),
                // **Only when the sampler cost a tenth of its own interval.**
                // A ramp has carried this per rung since it existed; a hold did
                // not, and the gate runs on holds. Silent below the threshold,
                // because a column that always reads 0 is a column nobody
                // checks — the same reason the occupancy loss beside it is.
                {
                    let worst = report.worst_tick.total_ms();
                    if worst * 10 >= report.interval_ms {
                        format!("  (worst tick {worst} ms of {} )", report.interval_ms)
                    } else {
                        String::new()
                    }
                },
                report.rss,
                report.work_rate,
                report.dropped_output,
                // A dash rather than a zero, for the reason above and more
                // sharply: zero is what this condition passes on, so a zero
                // printed when nothing was counted is the verdict's own
                // failure mode wearing the evidence's clothes.
                report
                    .failed_reads
                    .map_or_else(|| "—".to_string(), |f| f.to_string()),
                report.replacement,
            );
            println!("\nwritten to {}", out_dir.display());
            Ok(())
        }

        Command::Observe {
            label,
            out,
            interval,
            duration,
            pty,
            command,
        } => {
            let mode = if pty {
                SessionMode::Pty
            } else {
                SessionMode::Pipe
            };
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or_default();

            let out_dir = out.join(format!("{stamp}-{label}-{}", mode.label()));
            let config = ObserveConfig {
                scratch: out_dir.join("scratch"),
                out_dir,
                label,
                interval: Duration::from_secs_f64(interval),
                mode,
                max_duration: duration.map(Duration::from_secs_f64),
                command,
            };

            println!(
                "observing {} · {} · sampling every {interval}s",
                config.command.join(" "),
                config.mode.label(),
            );
            let report = observe::run(&config)?;
            print_run(&report, &config.out_dir);
            Ok(())
        }
        Command::ExclusionDelta {
            label,
            out,
            interval,
            duration,
            repeats,
            pty,
            command,
        } => exclusion_delta(label, out, interval, duration, repeats, pty, command),
        Command::Ramp {
            label,
            out,
            interval,
            hold,
            max_sessions,
            resolution,
            pty,
            daemon,
            skip_drift_check,
            exclude_scratch,
            command,
        } => {
            if let Some(path) = &daemon {
                if !path.is_file() {
                    anyhow::bail!(
                        "no daemon at {} — build it with `cargo build --release -p coggyd`",
                        path.display()
                    );
                }
                println!(
                    "  each rung held by {} — two of the four conditions are out of reach",
                    path.display()
                );
            }
            let mode = if pty {
                SessionMode::Pty
            } else {
                SessionMode::Pipe
            };
            let config = RampConfig {
                out_dir: out.join(format!("{}-{label}-{}", stamp(), mode.label())),
                label,
                interval: Duration::from_secs_f64(interval),
                hold: Duration::from_secs_f64(hold),
                max_sessions,
                resolution,
                exclude_scratch,
                skip_drift_check,
                mode,
                daemon,
                command,
            };

            println!(
                "ramping {} · {} · holding each rung {hold}s, sampling every {interval}s",
                config.command.join(" "),
                config.mode.label(),
            );
            let report = ramp::run(&config)?;
            print_ramp(&report, &config.out_dir);
            Ok(())
        }
    }
}

/// Runs one workload twice, the second time with its directory excluded from
/// real-time scanning, and reports the difference.
///
/// The two halves write into sibling directories under one root, both fresh:
/// scanning charges far less the second time a file is written, so reusing the
/// path would credit the exclusion with the cache's work.
fn exclusion_delta(
    label: String,
    out: PathBuf,
    interval: f64,
    duration: f64,
    repeats: u32,
    pty: bool,
    command: Vec<String>,
) -> anyhow::Result<()> {
    let mode = if pty {
        SessionMode::Pty
    } else {
        SessionMode::Pipe
    };
    let root = out.join(format!("{}-{label}-{}", stamp(), mode.label()));
    let scratch_root = root.join("scratch");
    let tick = Duration::from_secs_f64(interval);
    let mut sampler = Sampler::new();
    let mut pairs = Vec::new();

    for pair in 1..=repeats.max(1) {
        // Fresh directories every time. Scanning charges far less the second
        // time a file is written, so a reused path would credit whichever half
        // went second with the cache's work.
        let half = |name: &str| ObserveConfig {
            label: format!("{label}-{name}-{pair}"),
            out_dir: root.join(format!("{pair:02}-{name}")),
            scratch: scratch_root.join(format!("{pair:02}-{name}")),
            interval: tick,
            mode,
            max_duration: Some(Duration::from_secs_f64(duration)),
            command: command.clone(),
        };

        println!("\n=== pair {pair} of {repeats} ===");
        let idle_watched = sampler.watch_defender(IDLE_BASELINE, tick);
        println!("idle: {}", describe_idle(idle_watched));
        let watched = observe::run(&half("watched"))?;

        let held = HeldExclusion::add(&scratch_root).map_err(|e| anyhow::anyhow!(e))?;
        let idle_excluded = sampler.watch_defender(IDLE_BASELINE, tick);
        println!(
            "idle: {}  (exclusion in place)",
            describe_idle(idle_excluded)
        );
        let excluded = observe::run(&half("excluded"));

        // Removed before the half's result is even examined. The drop guard
        // would catch it, but a machine should not sit unprotected for the
        // length of a report.
        let mut held = held;
        let removal = held.remove();
        let excluded = excluded?;
        removal.map_err(|e| anyhow::anyhow!(e))?;

        let pair = Pair {
            watched,
            excluded,
            idle_watched,
            idle_excluded,
        };
        println!("  {}", pair.describe());
        pairs.push(pair);
    }

    print_exclusion_delta(&pairs, &root);
    Ok(())
}

/// One watched run and the excluded run that followed it, with the idle
/// baseline each was measured against.
///
/// Paired and adjacent in time on purpose: background drift that a single
/// comparison cannot separate from the exclusion lands on both halves of a
/// pair, and what survives across pairs is what the exclusion did.
struct Pair {
    watched: RunReport,
    excluded: RunReport,
    idle_watched: Option<f64>,
    idle_excluded: Option<f64>,
}

impl Pair {
    /// Defender's steady rate for a run, in seconds per minute.
    fn rate(report: &RunReport) -> Option<f64> {
        report
            .summary
            .defender
            .as_ref()
            .and_then(|cost| cost.steady_cpu_seconds_per_min)
    }

    /// What the workload itself cost, with the room's share taken off.
    fn attributable(during: Option<f64>, idle: Option<f64>) -> Option<f64> {
        Some(during? - idle? * 60.0)
    }

    /// Seconds per minute the exclusion saved. Negative means it cost.
    fn saving(&self) -> Option<f64> {
        let watched = Self::attributable(Self::rate(&self.watched), self.idle_watched)?;
        let excluded = Self::attributable(Self::rate(&self.excluded), self.idle_excluded)?;
        Some(watched - excluded)
    }

    /// How far the two idle baselines moved. Noise the pair could not cancel.
    fn drift(&self) -> Option<f64> {
        Some((self.idle_excluded? - self.idle_watched?).abs() * 60.0)
    }

    fn describe(&self) -> String {
        format!(
            "saved {} · baselines moved {}",
            self.saving()
                .map_or("—".into(), |v| format!("{v:+.2} s/min")),
            self.drift().map_or("—".into(), |v| format!("{v:.2} s/min")),
        )
    }
}

/// How long to watch Defender with nothing running, before each half.
const IDLE_BASELINE: Duration = Duration::from_secs(20);

fn describe_idle(cores: Option<f64>) -> String {
    match cores {
        Some(cores) => format!(
            "{:.2} s per minute of Defender with no session up",
            cores * 60.0
        ),
        None => "Defender not running".into(),
    }
}

fn print_exclusion_delta(pairs: &[Pair], out_dir: &std::path::Path) {
    let savings: Vec<f64> = pairs.iter().filter_map(Pair::saving).collect();
    let drifts: Vec<f64> = pairs.iter().filter_map(Pair::drift).collect();

    let mut rows: Rows = pairs
        .iter()
        .enumerate()
        .map(|(index, pair)| {
            (
                "pair",
                format!(
                    "{}  {}  ·  {:.2} against {:.2} units/s",
                    index + 1,
                    pair.describe(),
                    pair.watched.summary.work_units_per_sec,
                    pair.excluded.summary.work_units_per_sec,
                ),
            )
        })
        .collect();

    if savings.is_empty() {
        rows.push(("verdict", "no pair produced a steady state".into()));
        block("defender exclusion delta", rows);
        println!("\nwritten to {}", out_dir.display());
        return;
    }

    let mean = savings.iter().sum::<f64>() / savings.len() as f64;
    let low = savings.iter().copied().fold(f64::INFINITY, f64::min);
    let high = savings.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let worst_drift = drifts.iter().copied().fold(0.0, f64::max);

    rows.push(("mean saving", format!("{mean:+.2} s per minute")));
    rows.push((
        "across pairs",
        format!("{low:+.2} to {high:+.2} s per minute"),
    ));
    rows.push((
        "worst baseline drift",
        format!("{worst_drift:.2} s per minute"),
    ));

    // Two ways the answer is not an answer, and both are about the spread
    // rather than the mean. A range that contains zero has not established a
    // direction; a range wider than its own centre has not established a size.
    // Averaging a noisy pair with another noisy pair produces a confident
    // number and no more information, which is the failure worth naming.
    let straddles_zero = low <= 0.0 && high >= 0.0;
    let spread_exceeds_signal = (high - low) > mean.abs();
    rows.push((
        "verdict",
        if straddles_zero || spread_exceeds_signal {
            "inconclusive — the spread across pairs is larger than what separates them".into()
        } else {
            format!("the exclusion saves {mean:.2} s of Defender CPU per minute")
        },
    ));

    block("defender exclusion delta", rows);

    if straddles_zero || spread_exceeds_signal {
        println!(
            "\nA quieter machine or more pairs is the only fix. Background drift reached\n{worst_drift:.2} s/min here, and no amount of averaging separates a signal from noise\nthat moves further than the signal does."
        );
    }
    println!("\nwritten to {}", out_dir.display());
}

/// Seconds since the epoch, for naming an output directory.
fn stamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

/// Reports hardware, provenance, and axis availability.
///
/// Run before trusting any result. The Defender axis needs elevation, and
/// without it a run silently measures five axes out of six yet still prints a
/// redline — which is not a smaller result but a wrong one. Unavailable axes
/// are named here rather than discovered afterward.
/// Read two ramp reports and say whether their redlines may be subtracted.
///
/// Exits non-zero when they may not, so a script that pairs ramps fails rather
/// than publishing a difference that is really the afternoon moving.
fn compare(left: &std::path::Path, right: &std::path::Path) -> anyhow::Result<()> {
    let read = |path: &std::path::Path| -> anyhow::Result<sessionbench::ramp::RampReport> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    };

    let comparison = sessionbench::compare::Comparison::of(&read(left)?, &read(right)?);

    println!(
        "{} against {}",
        comparison.left_label, comparison.right_label
    );
    println!(
        "  solo rungs   {:.2} against {:.2} units/s ({:+.1}%)",
        comparison.left_solo, comparison.right_solo, comparison.solo_gap_percent
    );
    println!(
        "  redlines     {} against {}",
        comparison
            .left_redline
            .map_or_else(|| "none reached".into(), |n| n.to_string()),
        comparison
            .right_redline
            .map_or_else(|| "none reached".into(), |n| n.to_string()),
    );
    println!(
        "  solo repeats {}",
        match (comparison.left_solo_spread, comparison.right_solo_spread) {
            (None, None) => "neither ramp held its solo rung again".to_string(),
            (l, r) => format!(
                "{} against {}",
                l.map_or_else(|| "—".into(), |v| format!("{v:+.1}%")),
                r.map_or_else(|| "—".into(), |v| format!("{v:+.1}%")),
            ),
        }
    );
    println!("\n{}", comparison.verdict());

    if !comparison.comparable() {
        anyhow::bail!("these two ramps cannot be set against each other");
    }
    Ok(())
}

fn doctor(strict: bool) -> anyhow::Result<()> {
    let machine = Machine::detect();
    let provenance = Provenance::current();
    let facts = HostFacts::query();
    let statuses = axes::availability(&facts);

    block("machine", machine.rows());
    // Sampled before anything else runs, so the figure describes the machine a
    // ramp would start on rather than one this command already disturbed.
    let background = BackgroundLoad::measure(Duration::from_secs(5));
    block("background", background.rows());

    // **Loudest line in the command, because it is the one that invalidates a
    // day's comparisons in silence.** A hundred sessions at duty 0.27 returned
    // 907 units/s at noon on mains and 135 the same evening on battery — 7.8×,
    // from a laptop held at its base clock. Nothing was hot, nothing was
    // running, and no artifact recorded which of the two machines it came from.
    match facts.on_battery {
        Some(true) => println!(
            "\n  ON BATTERY — {} — every figure from this machine is a different machine's{}",
            facts.power_plan.as_deref().unwrap_or("power plan unknown"),
            facts
                .charge_percent
                .map(|c| format!(" ({c}% charged)"))
                .unwrap_or_default()
        ),
        Some(false) => println!(
            "\n  on mains — {}",
            facts.power_plan.as_deref().unwrap_or("power plan unknown")
        ),
        None => println!("\n  power state unknown — see query errors"),
    }

    // **The other state, which nothing names.** A solo session runs about
    // 18.9 units/s on this box rested and 8.997 in a slower one, measured over
    // twenty holds across 340 minutes, and a bracket ran entirely inside the
    // slow one with nothing in its artifact to say so. A tenant lands between
    // the two at 13.8, so the order is the trap: a crowded rested box outruns
    // a quiet slow one. Whether either of these two figures distinguishes them
    // is unknown — the run that tried could not induce the slow state, so both
    // its phases sampled the fast one. They print because the state has to be
    // caught rather than ordered, and a number nobody is collecting cannot be
    // checked against the next occurrence.
    //
    // **And a solo rate does not name it either**, which is why the line below
    // no longer says it does. Two runs whose solo holds agree to half a
    // percent, 9.752 and 9.801, held a hundred sessions at 246.4 and 902.8
    // units/s — one machine crippled under load, one entirely normal with only
    // its lone session down. Nothing `doctor` reads separates those, and only
    // a concurrent hold does.
    match (facts.thermal_c, facts.processor_performance) {
        (None, None) => {}
        (c, p) => println!(
            "  thermal {} · cores clocking at {} of nominal — neither names the slow state on this box, and nor does a solo rate: only a hundred-session hold's own throughput separates a slow machine from a slow solo",
            c.map(|c| format!("{c:.1} °C")).unwrap_or("—".into()),
            p.map(|p| format!("{p:.0}%")).unwrap_or("—".into()),
        ),
    }
    block("provenance", provenance.rows());

    println!("\naxes");
    for (index, status) in statuses.iter().enumerate() {
        print_axis(index + 1, status);
    }

    if !facts.errors.is_empty() {
        println!("\nquery errors");
        for error in &facts.errors {
            println!("  {error}");
        }
    }

    println!();
    if !background.is_quiet() {
        println!(
            "this machine is not quiet: {:.2} of {} cores are spoken for before a session starts, \
             and the sessions will be taking those back from something",
            background.mean_cores, background.logical_cores,
        );
    }
    let unavailable = statuses.iter().filter(|s| !s.available).count();
    match unavailable {
        0 if provenance.is_reproducible() => println!("all six axes available"),
        0 => println!("all six axes available, but this build is not reproducible"),
        n => {
            println!("{n} of 6 axes unavailable — a redline taken now would be wrong, not smaller")
        }
    }

    if strict && unavailable > 0 {
        anyhow::bail!("{unavailable} of 6 axes unavailable");
    }
    Ok(())
}

fn print_run(report: &RunReport, out_dir: &std::path::Path) {
    let summary = &report.summary;
    let ending = match (report.stopped_at_limit, report.exit_code) {
        (true, _) => "stopped at the time limit".to_string(),
        (false, Some(0)) => "exit 0".to_string(),
        (false, Some(code)) => format!("exit {code}"),
        (false, None) => "terminated without an exit code".to_string(),
    };

    block(
        &format!(
            "observed {} · {} · {:.1}s · {ending}",
            report.label,
            report.mode.label(),
            report.duration_ms as f64 / 1000.0
        ),
        vec![
            ("samples", format!("{}", report.sample_count)),
            ("peak rss", human_bytes(summary.peak_rss_bytes)),
            (
                "steady rss",
                format!(
                    "{} (median of the last quarter)",
                    human_bytes(summary.steady_rss_bytes)
                ),
            ),
            ("rss drift", rss_drift(summary)),
            ("work drift", work_drift(summary)),
            (
                "machine headroom",
                format!(
                    "{} free at its lowest — the figure the RSS condition cannot see",
                    human_bytes(summary.min_available_memory_bytes)
                ),
            ),
            ("peak processes", format!("{}", summary.peak_processes)),
            ("peak conhost", format!("{}", summary.peak_pseudoconsoles)),
            (
                "work rate",
                format!(
                    "{:.2} units/s ({} units)",
                    summary.work_units_per_sec, summary.work_units
                ),
            ),
            (
                "cores",
                match (summary.session_cores, summary.defender_cores) {
                    (Some(session), Some(defender)) => {
                        format!("{session:.2} session + {defender:.2} defender")
                    }
                    _ => "no steady state to measure over".into(),
                },
            ),
            (
                "output",
                format!(
                    "{} at {}/s",
                    human_bytes(summary.output_bytes),
                    human_bytes(summary.output_bytes_per_sec as u64)
                ),
            ),
            (
                "defender",
                match &summary.defender {
                    Some(cost) => match cost.steady_cpu_seconds_per_min {
                        Some(rate) => format!(
                            "{:.2}s over startup, then {rate:.2}s per minute",
                            cost.startup_cpu_seconds
                        ),
                        None => format!(
                            "{:.2}s over startup; the run was too short to have a steady state",
                            cost.startup_cpu_seconds
                        ),
                    },
                    None => "not running".into(),
                },
            ),
            (
                "membership",
                match report.membership {
                    Membership::JobObject => "job object".into(),
                    Membership::ParentWalk => {
                        "parent walk — the kernel's list was unavailable".into()
                    }
                },
            ),
        ],
    );

    let projection = &report.projection;
    block(
        &format!(
            "\nprojection to {} sessions — linear, and therefore a floor rather than an estimate",
            projection.sessions
        ),
        vec![
            (
                "rss",
                format!(
                    "{} against a {} budget — {}",
                    human_bytes(projection.rss_bytes),
                    human_bytes(projection.rss_budget_bytes),
                    if projection.rss_condition_holds {
                        "holds"
                    } else {
                        "BREAKS the RSS condition"
                    }
                ),
            ),
            (
                "cores",
                match (projection.cores_needed, projection.cpu_oversubscribed) {
                    (Some(needed), Some(over)) => format!(
                        "{needed:.1} needed against {} available — {}",
                        projection.cores_available,
                        if over {
                            "OVERSUBSCRIBED, which is how the work-rate condition trips"
                        } else {
                            "fits"
                        }
                    ),
                    _ => "not projectable — the run had no steady state".into(),
                },
            ),
            ("processes", format!("{}", projection.processes)),
            ("conhost", format!("{}", projection.pseudoconsoles)),
            (
                "output",
                format!("{}/s", human_bytes(projection.output_bytes_per_sec as u64)),
            ),
        ],
    );

    println!("\nwritten to {}", out_dir.display());
}

/// Says whether the ladder measured one machine or several.
///
/// The lowest saturated rung, held once at the start and once after everything
/// else. Averaging over rungs takes noise out of the redline but carries drift
/// straight through — a machine that slows as the ramp runs steepens the fitted
/// slope and reports a ceiling that is too low, with no sign of it anywhere in
/// the numbers.
/// How much the session's own footprint moved between the two ends of a run.
///
/// The single-session counterpart to the ramp's repeated rung. A memory-limited
/// redline is the budget divided by this figure, so it is a ceiling only if the
/// session held the same amount throughout — and a run whose ends disagree is
/// reporting an average of two different sessions.
fn rss_drift(summary: &observe::Summary) -> String {
    if summary.early_rss_bytes == 0 {
        return "no early samples to compare against".to_string();
    }
    let moved = (summary.steady_rss_bytes as f64 - summary.early_rss_bytes as f64)
        / summary.early_rss_bytes as f64
        * 100.0;
    let verdict = if moved.abs() < 5.0 {
        "held"
    } else if moved > 0.0 {
        "still growing — this is not a steady figure"
    } else {
        "still settling — this is not a steady figure"
    };
    format!(
        "{} early, {:+.1}% by the end — {verdict}",
        human_bytes(summary.early_rss_bytes),
        moved
    )
}

/// How much the session's work rate moved between the two ends of a run.
///
/// The same control as `rss_drift`, on the axis G0 leans on hardest: this run's
/// cores figure becomes `d` in `2ηC/d`, and a machine that changed while it was
/// being observed hands that field a number with nothing beside it to say so.
fn work_drift(summary: &observe::Summary) -> String {
    if summary.early_work_units_per_sec <= 0.0 {
        return "no early window to compare against".to_string();
    }
    let moved = (summary.late_work_units_per_sec - summary.early_work_units_per_sec)
        / summary.early_work_units_per_sec
        * 100.0;
    let verdict = if moved.abs() < 5.0 {
        "held, so the cores figure describes one machine"
    } else if moved > 0.0 {
        "**sped up — the machine freed cores mid-run and `d` is read low**"
    } else {
        "**slowed — something took cores mid-run and `d` is read high**"
    };
    format!(
        "{:.2} units/s early, {:+.1}% by the end — {verdict}",
        summary.early_work_units_per_sec, moved
    )
}

/// Both controls, on the console, where a run is read as it finishes.
///
/// They answer different questions and only one of them used to be printed
/// here. Drift says whether this ladder measured one machine; the solo spread
/// says whether its baseline reproduces, which is what decides whether this
/// ladder may be set against another at all.
fn print_controls(report: &RampReport) {
    if let Some(spread) = report.solo_spread_percent() {
        println!(
            "  solo check:  1 session ran {:.2} units/s early and {:.2} at the end ({spread:+.1}%)",
            report.solo_units_per_sec,
            report
                .solo_check
                .as_ref()
                .map_or(0.0, |s| s.units_per_session_per_sec),
        );
    }
    match report.drift() {
        // A repeat the instrument could not measure is not a machine that
        // slowed to nothing, and reporting it as one would put a drift on the
        // board describing the observer.
        Some(Drift::Unmeasurable(reason)) => {
            println!("  drift check: could not be re-measured — {reason}")
        }
        Some(Drift::Measured {
            sessions,
            early_units_per_sec,
            late_units_per_sec,
            slower_percent,
        }) => println!(
            "  drift check: {sessions} sessions ran {early_units_per_sec:.2} units/s early and {late_units_per_sec:.2} at the end ({slower_percent:+.1}% slower)"
        ),
        None => {}
    }
}

fn print_ramp(report: &RampReport, out_dir: &std::path::Path) {
    let target = report
        .command
        .first()
        .map(|c| {
            std::path::Path::new(c)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| c.clone())
        })
        .unwrap_or_default();
    let defender = match report.host.defender.realtime_protection {
        Some(true) => "Defender on",
        Some(false) => "Defender off",
        None => "Defender unknown",
    };

    let unmeasurable = report.steps.last().and_then(|s| s.inconclusive.as_ref());
    println!();
    match (&report.redline, unmeasurable) {
        (Some(redline), _) => {
            println!(
                "redline: {} sessions ({:?}) · {target} · {} · {} · {defender}",
                redline.sessions,
                redline.limited_by,
                report.mode.label(),
                report.machine.label(),
            );
            match &redline.fitted {
                Some(fit) => {
                    println!(
                        "  fitted at {:.1} through {} saturated rungs, gaining {:.4} slowdown per session",
                        fit.crossing, fit.rungs, fit.slowdown_per_session
                    );
                    if fit.ladder_sessions != redline.sessions {
                        println!(
                            "  the ladder's own search stopped at {} — the gap is how flat the curve is where the budget cuts it",
                            fit.ladder_sessions
                        );
                    }
                }
                None if !redline.limited_by.is_edge() => println!(
                    "  {:?} is a budget across a slope, not an edge — this count moves if the budget does",
                    redline.limited_by
                ),
                None => {}
            }
        }
        // Not a redline, and not a smaller one either. The ladder stopped
        // because the instrument ran out rather than because the machine did.
        (None, Some(reason)) => println!(
            "no redline: the ramp stopped at {} sessions without a usable reading — {reason}",
            report.steps.last().map(|s| s.sessions).unwrap_or(0),
        ),
        // The ladder ran out with every condition still holding, which locates
        // the ceiling above what was tried rather than at it.
        (None, None) => println!(
            "no redline reached: every rung up to {} sessions held · {target} · {} · {} · {defender}",
            report.steps.last().map(|s| s.sessions).unwrap_or(0),
            report.mode.label(),
            report.machine.label(),
        ),
    }
    // Outside the match: the control says whether the ladder measured one
    // machine, which is worth knowing whether or not it found a redline.
    print_controls(report);

    println!("\nrungs");
    for step in &report.steps {
        let verdict = if step.broken.is_empty() {
            "held".to_string()
        } else {
            format!("broke on {:?}", step.broken)
        };
        println!(
            "  {:>3}  rss {:>10}  free {:>10}  {:.2} units/s/session  {:.1} cores{}  {:>2} replaced  {} dropped  {verdict}",
            step.sessions,
            human_bytes(step.total_rss_bytes),
            human_bytes(step.min_available_memory_bytes),
            step.units_per_session_per_sec,
            step.session_cores + step.defender_cores,
            // **Only when the rung lost the machine**, because a column that is
            // almost always blank is read, and one that is almost always 0.05
            // is not. The threshold is a tenth of a core, well above the
            // 0.052-0.061 an undisturbed twenty-minute hold shows and well
            // below the 1.173 an interrupted one does.
            step.occupancy
                .filter(|o| o.lost_cores >= 0.1)
                .map_or_else(String::new, |o| format!(" (−{:.2} lost)", o.lost_cores)),
            step.replacements,
            // A rung that could not look reads "— dropped", never "0 dropped".
            step.dropped_units
                .map_or_else(|| "—".to_string(), |n| n.to_string()),
        );
    }

    println!("\nwritten to {}", out_dir.display());
}

/// Two columns wide enough for the longest label in any block.
const LABEL_WIDTH: usize = 20;

fn block(title: &str, rows: Rows) {
    println!("\n{title}");
    for (label, value) in rows {
        println!("  {label:<LABEL_WIDTH$} {value}");
    }
}

fn print_axis(number: usize, status: &AxisStatus) {
    let verdict = if status.available {
        "available"
    } else {
        "UNAVAILABLE"
    };
    let label = status.axis.label();
    match &status.note {
        Some(note) => println!("  {number}  {label:<LABEL_WIDTH$} {verdict:<12} {note}"),
        None => println!("  {number}  {label:<LABEL_WIDTH$} {verdict}"),
    }
}
