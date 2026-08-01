// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading `coggyd`'s report line, which is the whole coupling between the
//! instrument and the thing it measures.
//!
//! **Every other target hands this process the reading end of each session's
//! output.** That is what lets a rung count units and notice a gap in them.
//! Under the daemon the reading end belongs to the daemon: a session's output
//! reaches its scrollback and stops there, and what comes back instead is one
//! line saying how much of it there was.
//!
//! So the numerator changes and the coupling appears. It is deliberately one
//! function in one file: a benchmark that reached into `coggyd` as a library
//! would share its drain, and [the instrument and its subject keeping separate
//! implementations](README.md#keeping-it-honest) is what stops the benchmark
//! measuring its own reader.
//!
//! **Tolerant of fields it does not know, strict about the ones it needs.** A
//! daemon that grows a counter should not break a ladder; a daemon that stops
//! reporting `read` must, because the alternative is a rung silently reading
//! zero units and calling it saturation.

/// One line of `coggyd --sessions N`'s periodic report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    /// Sessions the daemon is holding, finished or not.
    pub held: u64,
    /// Sessions with something still alive in their job.
    ///
    /// Distinct from `held` on the daemon's side and kept distinct here: a
    /// rung whose sessions have quietly exited is not a rung that held.
    pub running: u64,
    /// Lines the daemon read across every session it holds.
    ///
    /// **The unit count.** [A unit is a
    /// line](../../workloads/README.md#the-contract), so this is the same
    /// quantity the drains count for every other target, arrived at by a
    /// different route.
    pub read: u64,
    /// Bytes behind those lines.
    ///
    /// **Axis 4, which had no number under a daemon until the daemon reported
    /// it.** A benchmark holding a session's pipe counts these itself; through
    /// a daemon the only figures available were its own few hundred bytes of
    /// reporting, or a zero, and both describe a hundred sessions as having
    /// produced almost nothing.
    pub read_bytes: u64,
    /// Lines the daemon's scrollback aged out, and lines it cut short.
    ///
    /// Neither is dropped output in the gate's sense — both were read. They
    /// are carried so a report can say the shortfall was policy rather than
    /// leave a reader to assume it was not.
    pub evicted: u64,
    pub truncated: u64,
}

/// Parses a report line, or `None` if the line is not one.
///
/// The daemon also prints a startup line and a `cleared` line, and a rung
/// reads whatever arrives, so not-a-report is ordinary rather than an error.
pub fn parse_report(line: &str) -> Option<Report> {
    // `held 100 · running 100 · read 3 · evicted 0 · truncated 0`, taken by
    // name so a new field between two old ones changes nothing. Whole tokens
    // rather than substrings: a later `withheld 3` would otherwise answer to
    // `held`.
    let field = |name: &str| -> Option<u64> {
        let mut tokens = line.split_whitespace();
        while let Some(token) = tokens.next() {
            if token == name {
                return tokens.next()?.parse().ok();
            }
        }
        None
    };
    Some(Report {
        held: field("held")?,
        running: field("running")?,
        read: field("read")?,
        read_bytes: field("bytes")?,
        evicted: field("evicted")?,
        truncated: field("truncated")?,
    })
}

/// What a rung learns by watching a daemon's report lines go by.
///
/// **Two numbers, and the second is why this type exists.** The first is the
/// unit count, which every target has. The second is the fewest sessions seen
/// alive at any report — which pipe and pty never need, because the ramp
/// restarts what exits and the count asked for is the count held. A daemon
/// holds them instead and restarts nothing, so a rung has to watch, or it
/// divides a falling numerator by a denominator that never moves and reads a
/// working target as saturated.
#[derive(Debug, Default)]
pub struct Watch {
    latest: Option<Report>,
    fewest_running: Option<u64>,
}

impl Watch {
    /// Takes one line of the daemon's output, report or not.
    pub fn observe(&mut self, line: &str) {
        let Some(report) = parse_report(line) else {
            return;
        };
        self.fewest_running = Some(match self.fewest_running {
            Some(fewest) => fewest.min(report.running),
            None => report.running,
        });
        self.latest = Some(report);
    }

    /// Lines the daemon has read across its sessions, or `None` if it has not
    /// said yet.
    ///
    /// **Never zero for *has not reported*.** A rung taking zero units from a
    /// daemon that simply had not spoken would read as saturation, which is
    /// the same silent-zero this file's parser refuses one level down.
    pub fn units(&self) -> Option<u64> {
        self.latest.map(|r| r.read)
    }

    /// The fewest sessions seen alive, or `None` if no report arrived.
    ///
    /// Not narrowed to the measured window on purpose. A session that exits
    /// during spin-up is not one the ramp will replace, so the rung was never
    /// holding what it asked for — and a spin-up that quietly excluded that
    /// would hide exactly the case this watches for.
    pub fn fewest_running(&self) -> Option<u64> {
        self.fewest_running
    }

    pub fn latest(&self) -> Option<Report> {
        self.latest
    }
}

/// Feeds a daemon's output into a [`Watch`], and keeps a copy on disk.
///
/// **Not [`session::drain`](crate::session), and the reason is in that
/// function's own comments.** It carries at most 32 bytes of a line across
/// chunk boundaries, because for a workload only the ordinal at a line's
/// opening matters and the stream can arrive at gigabytes a second. A report
/// line is short, arrives once every ten seconds, and keeps the fields this
/// needs at its *end*. Opposite constraints, so a reader tuned for one is
/// wrong for the other rather than merely slower.
///
/// The copy on disk is kept for the same reason every other stream's is: a
/// rung's artifacts should let someone else reach the same numbers.
pub fn watch_output(
    source: impl std::io::Read + Send + 'static,
    log: std::path::PathBuf,
    watch: std::sync::Arc<std::sync::Mutex<Watch>>,
) -> std::thread::JoinHandle<std::io::Result<()>> {
    std::thread::spawn(move || {
        use std::io::{BufRead, Write};
        let mut sink = std::io::BufWriter::new(std::fs::File::create(&log)?);
        for line in std::io::BufReader::new(source).lines() {
            let line = line?;
            writeln!(sink, "{line}")?;
            watch
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .observe(&line);
        }
        sink.flush()
    })
}

/// A daemon started by the harness, held until dropped.
pub struct Held {
    pub child: std::process::Child,
    /// Held open for the life of the hold, never written to.
    ///
    /// **The whole stop condition, and it is why this cannot go through
    /// [`session::spawn`](crate::session).** That spawner gives every session
    /// `Stdio::null()`, which is right for a workload and fatal here: the
    /// daemon stops at end-of-file, so a null stdin ends it before the first
    /// sample. Holding a pipe open instead means the graceful stop is simply
    /// letting go of it — no signal, no console, and no separate process to
    /// hold the pipe, which is what an hour-long hold needed when this was
    /// driven by hand.
    stdin: Option<std::process::ChildStdin>,
    pub watch: std::sync::Arc<std::sync::Mutex<Watch>>,
    drain: Option<std::thread::JoinHandle<std::io::Result<()>>>,
}

impl Held {
    /// Starts `daemon` holding `sessions` copies of `workload`.
    ///
    /// `workload` may carry `${session}` where each session needs something of
    /// its own; the daemon expands it, so the workload learns nothing about
    /// COGGY and this process names no COGGY-specific path.
    ///
    /// Job membership is inherited at creation, so this must be called after
    /// the tree is armed or the daemon and everything under it sits outside
    /// the measurement.
    pub fn start(
        daemon: &std::path::Path,
        sessions: u32,
        workload: &[String],
        log: std::path::PathBuf,
    ) -> std::io::Result<Self> {
        let mut child = std::process::Command::new(daemon)
            .arg("--sessions")
            .arg(sessions.to_string())
            .arg("--")
            .args(workload)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take();
        let watch = std::sync::Arc::new(std::sync::Mutex::new(Watch::default()));
        let drain = child
            .stdout
            .take()
            .map(|out| watch_output(out, log, std::sync::Arc::clone(&watch)));

        Ok(Self {
            child,
            stdin,
            watch,
            drain,
        })
    }

    /// Closes stdin and waits for the daemon to clear its pool.
    ///
    /// The graceful path, which runs `Drop for Session` inside the daemon.
    /// Killing instead also reclaims every tree — both were measured doing so
    /// — but only this one exercises the code that has to order the job
    /// release before the drains are joined.
    pub fn stop(mut self) -> std::io::Result<std::process::ExitStatus> {
        drop(self.stdin.take());
        let status = self.child.wait()?;
        if let Some(drain) = self.drain.take() {
            drain
                .join()
                .map_err(|_| std::io::Error::other("the daemon's reader panicked"))??;
        }
        Ok(status)
    }

    /// What the daemon has said so far.
    pub fn seen(&self) -> Watch {
        let held = self
            .watch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Watch {
            latest: held.latest,
            fewest_running: held.fewest_running,
        }
    }
}

/// What a sampled hold came back with.
#[derive(Debug, Clone)]
pub struct HeldRun {
    pub sessions: u32,
    pub samples: Vec<crate::sampler::Sample>,
    /// The last thing the daemon said, or `None` if it never reported.
    pub last: Option<Report>,
    /// Fewest sessions alive at any report.
    ///
    /// Below `sessions` means the hold stopped being a hold at that count,
    /// and nothing here restarts one — so the caller refuses the run rather
    /// than dividing by either number.
    pub fewest_running: Option<u64>,
    pub elapsed: std::time::Duration,
}

impl HeldRun {
    /// Peak total RSS across the samples, which is the figure a memory budget
    /// has to fit.
    pub fn peak_rss_bytes(&self) -> u64 {
        self.samples.iter().map(|s| s.rss_bytes).max().unwrap_or(0)
    }

    /// Why this run says nothing about the machine, when it does not.
    pub fn unusable(&self) -> Option<String> {
        if self.samples.is_empty() {
            return Some("no samples — the hold ended before the first tick".into());
        }
        let Some(fewest) = self.fewest_running else {
            return Some("the daemon never reported, so nothing says it held anything".into());
        };
        (fewest < u64::from(self.sessions)).then(|| {
            format!(
                "{} session(s) asked for and only {fewest} alive at some report — nothing here restarts one, so this was not a hold at {}",
                self.sessions, self.sessions,
            )
        })
    }
}

/// Holds `sessions` copies of `workload` under `daemon` and samples them.
///
/// **The order is the correctness.** Job membership is inherited at creation,
/// so the tree is armed before the daemon exists; a daemon started first sits
/// outside the measurement along with everything it spawns, and the run would
/// report a machine holding nothing.
pub fn hold(
    daemon: &std::path::Path,
    sessions: u32,
    workload: &[String],
    log: std::path::PathBuf,
    interval: std::time::Duration,
    duration: std::time::Duration,
) -> anyhow::Result<HeldRun> {
    use anyhow::Context;

    let armed = crate::tree::ArmedTree::arm(sysinfo::Pid::from_u32(std::process::id()));
    let started = std::time::Instant::now();
    let held = Held::start(daemon, sessions, workload, log).context("starting the daemon")?;
    let mut tree = armed.attach(sysinfo::Pid::from_u32(held.child.id()));

    let mut sampler = crate::sampler::Sampler::new();
    let mut samples = Vec::new();
    while started.elapsed() < duration {
        std::thread::sleep(interval);
        let seen = held.seen();
        // The sampler wants one counter. Units and bytes both come from the
        // daemon's own report rather than from a drain this process owns,
        // which is the whole difference between this target and every other.
        let output = crate::session::Output::from_counts(
            seen.latest().map_or(0, |r| r.read_bytes),
            seen.units().unwrap_or(0),
        );
        samples.push(sampler.take(&mut tree, &output, started.elapsed()));
    }

    // **Read after the stop, not before.** The daemon reports on a ten-second
    // clock and again at end-of-file, and that last one is the only complete
    // total — everything the sessions did since the previous tick is in it and
    // nowhere else. Taking the numbers while the hold was still running lost
    // 24% of the units on a half-minute example. The watch outlives the hold
    // for exactly this.
    let watch = std::sync::Arc::clone(&held.watch);
    held.stop().context("stopping the daemon")?;
    let seen = watch
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    Ok(HeldRun {
        sessions,
        samples,
        last: seen.latest(),
        fewest_running: seen.fewest_running(),
        elapsed: started.elapsed(),
    })
}

/// What the run was, as opposed to what it found.
///
/// Separate from [`HeldRun`] because none of it is a measurement: it is the
/// provenance a reader needs to take the run again, and threading eight of
/// them through a call reads as eight results.
pub struct Ran {
    pub label: String,
    pub daemon: String,
    pub workload: Vec<String>,
    /// The gate's budget rather than the redline's share of the machine: M1
    /// states four gigabytes outright.
    pub rss_budget_bytes: u64,
    pub interval: std::time::Duration,
    pub membership: crate::tree::Membership,
    pub membership_fallback_reason: Option<String>,
    pub started_unix: u64,
}

impl HeldRun {
    /// Turns a run into the artifact, deciding each condition once.
    pub fn into_report(self, about: Ran) -> HoldReport {
        let Ran {
            label,
            daemon,
            workload,
            rss_budget_bytes,
            interval,
            membership,
            membership_fallback_reason,
            started_unix,
        } = about;
        let inconclusive = self.unusable();
        let peak_rss_bytes = self.peak_rss_bytes();
        let last = self.last;

        // Every verdict is NotExercised while the run itself is in doubt. A
        // rung that could not be measured has not passed anything, and this is
        // the one place that could quietly say otherwise.
        let rss = match (&inconclusive, peak_rss_bytes <= rss_budget_bytes) {
            (Some(_), _) => Verdict::NotExercised,
            (None, true) => Verdict::Held,
            (None, false) => Verdict::Broke,
        };

        HoldReport {
            label,
            daemon,
            workload,
            sessions: self.sessions,
            machine: crate::machine::Machine::detect(),
            provenance: crate::provenance::Provenance::current(),
            host: crate::host::HostFacts::query(),
            membership,
            membership_fallback_reason,
            started_unix,
            duration_ms: self.elapsed.as_millis() as u64,
            interval_ms: interval.as_millis() as u64,
            sample_count: self.samples.len(),
            peak_rss_bytes,
            rss_budget_bytes,
            min_available_memory_bytes: self
                .samples
                .iter()
                .map(|s| s.available_memory_bytes)
                .min()
                .unwrap_or(0),
            peak_processes: self.samples.iter().map(|s| s.processes).max().unwrap_or(0),
            fewest_running: self.fewest_running,
            units: last.map(|r| r.read),
            output_bytes: last.map(|r| r.read_bytes),
            evicted: last.map(|r| r.evicted),
            truncated: last.map(|r| r.truncated),
            rss,
            // **Not measured here, and not because it is hard.** The condition
            // is a ratio against the same workload run alone, and a solo
            // figure is a second run. Until this command takes one, saying
            // anything else would be inventing a baseline.
            work_rate: Verdict::NotExercised,
            // Neither of these is reachable through a daemon at all.
            dropped_output: Verdict::NotExercised,
            replacement: Verdict::NotExercised,
            inconclusive,
        }
    }
}

/// Whether a gate condition passed, failed, or was never in reach.
///
/// **Three states because two would lie.** [Two of the four
/// conditions](../../sessionbench/README.md#what-we-measure-against) cannot be
/// asked of a daemon at all: dropped output is found by watching ordinals in a
/// session's own stream, which ends in the daemon's scrollback, and nothing in
/// the daemon restarts a session that exited. A boolean would render both as
/// `true`, which is a pass earned by never having been asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Held,
    Broke,
    NotExercised,
}

/// The committed artifact for one held run.
///
/// **Deliberately carries no projection**, which is what separates this from
/// [`observe`](crate::observe). That command measures one session and
/// multiplies to say what N would cost; this holds N and measures them. A
/// projection field here would be a number with nothing behind it, and a field
/// that reads as a measurement without being one is the failure this
/// repository keeps finding.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HoldReport {
    pub label: String,
    pub daemon: String,
    pub workload: Vec<String>,
    pub sessions: u32,
    pub machine: crate::machine::Machine,
    pub provenance: crate::provenance::Provenance,
    pub host: crate::host::HostFacts,
    pub membership: crate::tree::Membership,
    pub membership_fallback_reason: Option<String>,
    pub started_unix: u64,
    pub duration_ms: u64,
    pub interval_ms: u64,
    pub sample_count: usize,
    /// Why the run says nothing about the machine, when it does not.
    pub inconclusive: Option<String>,
    pub peak_rss_bytes: u64,
    pub rss_budget_bytes: u64,
    /// Least the machine had free at any sample.
    pub min_available_memory_bytes: u64,
    pub peak_processes: usize,
    /// Fewest sessions the daemon reported alive.
    pub fewest_running: Option<u64>,
    pub units: Option<u64>,
    pub output_bytes: Option<u64>,
    /// Lines the scrollback aged out or cut, which are policy rather than the
    /// gate's dropped output.
    pub evicted: Option<u64>,
    pub truncated: Option<u64>,
    pub rss: Verdict,
    pub work_rate: Verdict,
    pub dropped_output: Verdict,
    pub replacement: Verdict,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The line as `coggyd` actually prints it.
    ///
    /// Copied from a run rather than composed here. A format invented in a
    /// test is a format nothing produces, so
    /// [`the_field_names_are_the_ones_the_daemon_documents`] reads them back
    /// out of the daemon's own README.
    /// Two sessions echoing `alpha` and `beta`: four lines, and 5+4 bytes
    /// twice. The arithmetic agreeing from the other side is why this is a
    /// captured line rather than a composed one.
    const REAL: &str = "held 2 · running 0 · read 4 · bytes 18 · evicted 0 · truncated 0";

    /// Every field this parser requires appears in `coggyd`'s worked example.
    ///
    /// The one coupling between the two crates, and the only thing holding it
    /// still is that both were written on the same afternoon. Renaming a field
    /// in the daemon would otherwise leave a ladder reading a line that no
    /// longer carries what it needs.
    #[test]
    fn the_field_names_are_the_ones_the_daemon_documents() {
        let readme = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("crate sits one level below the repository root")
                .join("coggyd/README.md"),
        )
        .expect("the daemon's README");

        let example = readme
            .lines()
            .find(|l| l.starts_with("held "))
            .expect("coggyd/README.md shows a report line starting with `held `");

        let parsed = parse_report(example).expect("the documented line parses");
        assert!(parsed.held > 0, "the example holds something: {example}");
    }

    #[test]
    fn a_real_report_line_parses() {
        assert_eq!(
            parse_report(REAL),
            Some(Report {
                held: 2,
                running: 0,
                read: 4,
                read_bytes: 18,
                evicted: 0,
                truncated: 0,
            })
        );
    }

    #[test]
    fn the_daemons_other_lines_are_not_reports() {
        assert!(parse_report("holding 3 session(s); stdin closes to stop").is_none());
        assert!(parse_report("cleared").is_none());
        assert!(parse_report("").is_none());
    }

    #[test]
    fn a_field_the_daemon_grows_later_does_not_break_the_ladder() {
        let grown =
            "held 9 · running 9 · read 71 · bytes 900 · evicted 2 · truncated 0 · admitted 9";
        assert_eq!(parse_report(grown).map(|r| (r.held, r.read)), Some((9, 71)));
    }

    #[test]
    fn a_watch_that_has_heard_nothing_says_so_rather_than_zero() {
        // Zero units reads as saturation. A daemon that has not spoken yet has
        // produced no measurement, and the two must not arrive as one number.
        let mut watch = Watch::default();
        assert_eq!(watch.units(), None);
        assert_eq!(watch.fewest_running(), None);

        watch.observe("holding 4 session(s); stdin closes to stop");
        assert_eq!(watch.units(), None, "the startup line is not a report");
    }

    #[test]
    fn the_watch_keeps_the_fewest_alive_it_ever_saw() {
        // The whole reason it exists: a rung that dipped is not a rung at the
        // count it asked for, and the latest report would say it recovered.
        let mut watch = Watch::default();
        watch.observe("held 4 · running 4 · read 10 · bytes 90 · evicted 0 · truncated 0");
        watch.observe("held 4 · running 2 · read 14 · bytes 126 · evicted 0 · truncated 0");
        watch.observe("held 4 · running 4 · read 30 · bytes 270 · evicted 0 · truncated 0");

        assert_eq!(watch.units(), Some(30), "units come from the latest");
        assert_eq!(
            watch.fewest_running(),
            Some(2),
            "and the dip is remembered, since nothing restarts a session here"
        );
    }

    #[test]
    fn the_drain_feeds_the_watch_and_leaves_the_lines_on_disk() {
        // Both halves asserted, because the artifact is what lets someone else
        // reach the same number and a drain that only counted would lose it.
        let stream = "holding 4 session(s); stdin closes to stop\n\
             held 4 · running 4 · read 10 · bytes 90 · evicted 0 · truncated 0\n\
             held 4 · running 3 · read 25 · bytes 225 · evicted 0 · truncated 0\n\
             cleared\n";
        let log = std::env::temp_dir().join(format!(
            "sessionbench-daemon-drain-{}.log",
            std::process::id()
        ));
        let watch = std::sync::Arc::new(std::sync::Mutex::new(Watch::default()));

        watch_output(
            std::io::Cursor::new(stream.as_bytes().to_vec()),
            log.clone(),
            std::sync::Arc::clone(&watch),
        )
        .join()
        .expect("the drain thread")
        .expect("the drain wrote its log");

        let seen = watch.lock().expect("not poisoned");
        assert_eq!(seen.units(), Some(25), "the latest report's read count");
        assert_eq!(seen.fewest_running(), Some(3), "and the dip it saw");

        let on_disk = std::fs::read_to_string(&log).expect("the log");
        assert_eq!(
            on_disk.lines().count(),
            4,
            "every line, reports and not: {on_disk}"
        );
        let _ = std::fs::remove_file(&log);
    }

    fn about() -> Ran {
        Ran {
            label: "t".into(),
            daemon: "coggyd".into(),
            workload: vec!["ping".into()],
            rss_budget_bytes: u64::MAX,
            interval: std::time::Duration::from_secs(2),
            membership: crate::tree::Membership::JobObject,
            membership_fallback_reason: None,
            started_unix: 0,
        }
    }

    fn run_of(sessions: u32, fewest: Option<u64>, samples: usize) -> HeldRun {
        HeldRun {
            sessions,
            // Built by hand rather than by deriving Default on Sample, which
            // would add a convenience to the real type that only a test wants.
            samples: (0..samples)
                .map(|i| crate::sampler::Sample {
                    t_ms: i as u64 * 2_000,
                    rss_bytes: 1_000_000,
                    processes: 4,
                    pseudoconsoles: 0,
                    cpu_percent: 0.0,
                    defender_cpu_percent: None,
                    defender_rss_bytes: None,
                    available_memory_bytes: 8_000_000_000,
                    output_bytes: 0,
                    work_units: 0,
                    members: Vec::new(),
                })
                .collect(),
            last: parse_report(REAL),
            fewest_running: fewest,
            elapsed: std::time::Duration::from_secs(60),
        }
    }

    #[test]
    fn a_run_in_doubt_passes_nothing() {
        // The one thing this function could quietly get wrong. A hold that
        // lost sessions still has a peak RSS under any budget -- fewer
        // sessions hold less -- so a verdict read off the number alone would
        // report the memory condition as satisfied by the failure.
        let short = run_of(4, Some(2), 30).into_report(about());
        assert!(short.inconclusive.is_some(), "the run is in doubt");
        assert_eq!(short.rss, Verdict::NotExercised, "so nothing is held");

        let whole = run_of(4, Some(4), 30).into_report(about());
        assert_eq!(whole.inconclusive, None);
        assert_eq!(whole.rss, Verdict::Held, "and a whole run is judged");
    }

    #[test]
    fn the_conditions_a_daemon_cannot_answer_are_never_held() {
        let run = run_of(4, Some(4), 30).into_report(about());
        // A boolean would have rendered all three as a pass.
        assert_eq!(run.work_rate, Verdict::NotExercised, "needs a solo run");
        assert_eq!(
            run.dropped_output,
            Verdict::NotExercised,
            "ordinals do not reach here"
        );
        assert_eq!(
            run.replacement,
            Verdict::NotExercised,
            "nothing restarts a session"
        );
    }

    #[test]
    fn a_report_missing_the_unit_count_is_refused_rather_than_read_as_zero() {
        // The one that matters. A rung taking zero units from a line that
        // simply did not carry them would read as saturation, and the ladder
        // would return a redline from a daemon that was working fine.
        let without = "held 9 · running 9 · bytes 90 · evicted 0 · truncated 0";
        assert!(parse_report(without).is_none());
    }
}
