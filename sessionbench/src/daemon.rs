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

#[cfg(test)]
mod tests {
    use super::*;

    /// The line as `coggyd` actually prints it.
    ///
    /// Copied from a run rather than composed here. A format invented in a
    /// test is a format nothing produces, so
    /// [`the_field_names_are_the_ones_the_daemon_documents`] reads them back
    /// out of the daemon's own README.
    const REAL: &str = "held 2 · running 0 · read 4 · evicted 0 · truncated 0";

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
        let grown = "held 9 · running 9 · read 71 · evicted 2 · truncated 0 · admitted 9";
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
        watch.observe("held 4 · running 4 · read 10 · evicted 0 · truncated 0");
        watch.observe("held 4 · running 2 · read 14 · evicted 0 · truncated 0");
        watch.observe("held 4 · running 4 · read 30 · evicted 0 · truncated 0");

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
             held 4 · running 4 · read 10 · evicted 0 · truncated 0\n\
             held 4 · running 3 · read 25 · evicted 0 · truncated 0\n\
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

    #[test]
    fn a_report_missing_the_unit_count_is_refused_rather_than_read_as_zero() {
        // The one that matters. A rung taking zero units from a line that
        // simply did not carry them would read as saturation, and the ladder
        // would return a redline from a daemon that was working fine.
        let without = "held 9 · running 9 · evicted 0 · truncated 0";
        assert!(parse_report(without).is_none());
    }
}
