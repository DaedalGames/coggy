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
    fn a_report_missing_the_unit_count_is_refused_rather_than_read_as_zero() {
        // The one that matters. A rung taking zero units from a line that
        // simply did not carry them would read as saturation, and the ladder
        // would return a redline from a daemon that was working fine.
        let without = "held 9 · running 9 · evicted 0 · truncated 0";
        assert!(parse_report(without).is_none());
    }
}
