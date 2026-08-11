// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Verifies every cross-document link and anchor in the repository resolves.
//!
//! Cross-references used to be prose section numbers ("PLAN §4"), which no tool
//! can check — one of them pointed at the wrong section for weeks and nobody
//! noticed. References are now name-based anchors, and this test is what makes
//! that an improvement rather than a different style: renaming a heading fails
//! the build instead of silently misdirecting a reader.
//!
//! It lives as a test rather than a separate tool so `cargo test` covers it and
//! the repository stays one crate with no extra toolchain.
//!
//! **Why this is not `lychee`,** which is the maintained Rust link checker and
//! was the obvious thing to take instead. Its own documentation describes the
//! fragments it generates as *similar to* GitHub's auto-generated anchors,
//! without stating the rules. These documents are read on GitHub, so "similar"
//! passes links that GitHub breaks — and the one divergence already found here,
//! that GitHub does not collapse runs of whitespace, is exactly that shape.
//! What lychee is unambiguously better at is the external URLs this test skips
//! entirely; it belongs in CI for those the moment CI can run again.
//!
//! Known gap: GitHub disambiguates repeated headings within a file by
//! appending `-1`, `-2`, and this does not. A link to the second of two
//! identical headings would resolve against the first and pass.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// GitHub's anchor rule: drop emphasis and punctuation, lowercase, and turn
/// every space into a dash.
///
/// Whitespace runs are *not* collapsed. Removing the separator from
/// "M1 · Headless Daemon" leaves two spaces, so the anchor carries a double
/// dash: `m1--headless-daemon`.
fn slug(heading: &str) -> String {
    heading
        .chars()
        .filter(|c| !matches!(c, '`' | '*' | '_'))
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
        .collect::<String>()
        .trim()
        .chars()
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .collect::<String>()
        .to_lowercase()
}

/// Every file that can carry a cross-reference, which is not only the prose.
///
/// **Rust doc comments cite measurement records too, and nobody was checking
/// them.** Sixteen such links existed when this was widened, two of them added
/// the same day, pointing at records from a doc comment that `cargo test` read
/// past. The three callers are all link checks and none of them cares whether
/// the source it scans compiles.
fn markdown_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if !matches!(name.as_ref(), "target" | ".git") {
                markdown_files(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "md" || e == "rs")
            // This file's own test fixtures are strings that look like links
            // — `record.md`, `serialise.md` — and checking them checks the
            // sample data rather than the repository.
            && name != "docs.rs"
        {
            out.push(path);
        }
    }
}

/// Headings outside fenced code blocks, as anchor slugs.
fn anchors(body: &str) -> HashSet<String> {
    let mut fenced = false;
    let mut found = HashSet::new();
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let text = line.trim_start_matches('#').trim();
        if line.starts_with('#') && !text.is_empty() {
            found.insert(slug(text));
        }
    }
    found
}

/// Every `[label](target)` outside code blocks, as a pair.
///
/// The label is carried because it makes claims of its own — a link reading
/// *"[four readings give 1.865 GiB](record.md)"* asserts a figure, and the
/// assertion is checkable against the file it points at.
fn links(body: &str) -> Vec<(String, String)> {
    // **Fences are decided per line; links are not.** A label whose opening
    // bracket sits on one line and whose target sits on the next was found by
    // its target and handed the figure check an empty label, so the target got
    // verified while the claim beside it did not. 38 of 725 links here wrap and
    // 12 of those carry a figure. Doing both passes at once broke the fence
    // state, which cost this test three of the links it had: strip fenced
    // blocks line by line first, then scan the remainder as one string.
    let mut fenced = false;
    let mut prose = String::new();
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let t = line.trim_start();
        // A doc comment's marker is not part of the prose it carries.
        let t = t
            .strip_prefix("///")
            .or_else(|| t.strip_prefix("//!"))
            .unwrap_or(t);
        prose.push_str(t);
        prose.push(' ');
    }

    let chars: Vec<char> = prose.chars().collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i + 1 < chars.len() {
        let opens_link = chars[i] == ']' && chars[i + 1] == '(';
        let close = chars[i + 2..].iter().position(|c| *c == ')');
        if let (true, Some(end)) = (opens_link, close) {
            let target: String = chars[i + 2..i + 2 + end].iter().collect();
            // The label is whatever sits inside the nearest `[` before this
            // `]`; nested brackets are rare enough here that the nearest one
            // is the right one.
            let label = chars[..i]
                .iter()
                .rposition(|c| *c == '[')
                .map(|open| chars[open + 1..i].iter().collect())
                .unwrap_or_default();
            found.push((label, target.trim().to_string()));
            i += end + 2;
        }
        i += 1;
    }
    found
}

/// Link targets — the `target` of every `[text](target)` outside code blocks.
fn link_targets(body: &str) -> Vec<String> {
    links(body).into_iter().map(|(_, target)| target).collect()
}

/// Figures in a piece of prose: `1.87`, `361`, `13,100`.
///
/// **Integers as well as decimals, and the two need different care.** A
/// decimal rarely collides by accident; `580` sits inside `15801` and `07`
/// sits inside every date in the repository. So dates are dropped here and
/// [`holds`] refuses a match that is part of a longer number — widening what
/// is checked has to come with narrowing what counts as found, or the check
/// starts passing for reasons unrelated to the claim.
///
/// Skipped: years, single digits, and bare integers of five or more digits,
/// which in these documents are timestamps and build numbers rather than
/// measurements.
fn figures(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let digit_at = |i: usize| chars.get(i).is_some_and(char::is_ascii_digit);

    let mut found = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while digit_at(i) {
            i += 1;
        }
        let integer_digits = i - start;

        // Comma groups, then at most one decimal part.
        let mut grouped = false;
        while chars.get(i) == Some(&',') && digit_at(i + 1) && digit_at(i + 2) && digit_at(i + 3) {
            i += 4;
            grouped = true;
        }
        let mut fractional = false;
        if chars.get(i) == Some(&'.') && digit_at(i + 1) {
            i += 1;
            while digit_at(i) {
                i += 1;
            }
            fractional = true;
        }

        // A run joined to another by a hyphen is a date or a version, not a
        // measurement: 2026-07-31 would otherwise contribute 07 and 31, and
        // two-digit numbers turn up somewhere in almost any document.
        let hyphenated = (start > 0 && chars[start - 1] == '-' && digit_at(start - 2))
            || (chars.get(i) == Some(&'-') && digit_at(i + 1));
        let token: String = chars[start..i].iter().collect();
        let year = integer_digits == 4 && !grouped && !fractional && token.starts_with("20");
        let too_plain = !grouped && !fractional && !(2..=4).contains(&integer_digits);

        if !hyphenated && !year && !too_plain {
            found.push(token);
        }
    }
    found
}

/// Whether `text` states `figure` as a number rather than inside a longer one.
///
/// `580` must not be found in `15801`, and `1.87` must not be found in
/// `1.874`. Comma grouping is accepted either way, since a target may write
/// the same quantity as `13,100` or `13100`.
fn holds(text: &str, figure: &str) -> bool {
    let ungrouped = figure.replace(',', "");
    [figure, ungrouped.as_str()]
        .iter()
        .any(|form| occurs(text, form))
}

fn occurs(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    haystack.match_indices(needle).any(|(at, _)| {
        let before = at == 0 || !matches!(bytes[at - 1], b'0'..=b'9' | b'.' | b',');
        let end = at + needle.len();
        let after = end >= bytes.len() || !bytes[end].is_ascii_digit();
        before && after
    })
}

/// Every component that owns a document is listed in the map that promises to
/// list them.
///
/// A list of components has no way to notice a new component, and this one was
/// wrong four times in a day: a findings count, a sentence naming three
/// comparison-set rows after a fourth was filled, a crate count, and the map
/// itself once `coggyd` existed. Each was fixed alone. The shape is what
/// recurs, so the check belongs here rather than in a reviewer's memory.
#[test]
fn the_documentation_map_lists_every_component_that_has_a_readme() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits one level below the repository root");
    let map = fs::read_to_string(root.join("README.md")).expect("a root README");

    let mut missing = Vec::new();
    for entry in fs::read_dir(root)
        .expect("readable repository root")
        .flatten()
    {
        let dir = entry.path();
        if !dir.is_dir() || !dir.join("README.md").is_file() {
            continue;
        }
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a directory name");
        if !map.contains(&format!("{name}/README.md")) {
            missing.push(name.to_string());
        }
    }

    assert!(
        missing.is_empty(),
        "these own a README and the documentation map does not list them: {missing:?}"
    );
}

/// A figure quoted inside a link is a figure the target has to contain.
///
/// [The measurement index promises exactly this](../../docs/measurements/README.md)
/// — *nothing here is stated that is not measured there* — and it was broken in
/// the file that promises it. Two shapes, both found by the same sweep:
///
/// - **A record grew and its citers did not.** A fifth reading moved the engine
///   figure to `1.87 GiB ± 6.4%`; ROADMAP and the index went on quoting the
///   `1.865` and `6%` that four readings had given.
/// - **A compound link attributes both halves to one target.** *"[takes 3.27 GiB
///   and only one session can build at a time](serialise.md)"* sends a reader
///   after `3.27` to a record that never measured it.
///
/// **Measurement records are exempt, and that is not a convenience.** A record
/// is dated and says what was true when it was taken, so a citation it made is
/// still accurate when the record it points at grows past the figure later.
/// Living documents carry no date and promise to be current, so the same
/// staleness is a defect in them.
///
/// **It stops at the label, and that was measured rather than assumed.**
/// Widening to the sentence around each link would raise the figures checked
/// from 42 to 202 and flag 50 of the additions, essentially all of them false:
/// `4GB across 100 sessions is 40 MiB each` is the citing document's own
/// arithmetic, and `GPL-3.0` is a licence. A link inside a sentence does not
/// claim every number in it. The label is the span an author wrote to say
/// *this is over there*, so it is where a citation can be wrong — and a check
/// that needed an exception list would be a check nobody reads.
#[test]
fn a_figure_quoted_in_a_link_appears_in_the_document_it_points_at() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits one level below the repository root");

    let mut docs = Vec::new();
    markdown_files(root, &mut docs);

    /// Records live in `docs/measurements/` under a timestamped name.
    fn is_record(path: &Path) -> bool {
        let in_measurements = path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|d| d == "measurements");
        let timestamped = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(|c: char| c.is_ascii_digit()));
        in_measurements && timestamped
    }

    let mut stale = Vec::new();
    for doc in docs.iter().filter(|d| !is_record(d)) {
        let body = fs::read_to_string(doc).expect("readable markdown");
        let here = doc.parent().expect("file has a parent directory");

        for (label, target) in links(&body) {
            let path_part = target.split('#').next().unwrap_or_default();
            if !path_part.ends_with(".md") {
                continue;
            }
            let Ok(dest) = here.join(path_part).canonicalize() else {
                continue; // an unresolvable link is the other test's finding
            };
            if dest == doc.canonicalize().expect("readable path") {
                continue; // a link into the same file quotes itself
            }
            let Ok(cited) = fs::read_to_string(&dest) else {
                continue;
            };
            for figure in figures(&label) {
                if !holds(&cited, &figure) {
                    stale.push(format!(
                        "{}  claims {figure}  ->  {target}  (which does not contain it)",
                        doc.display()
                    ));
                }
            }
        }
    }

    assert!(
        stale.is_empty(),
        "{} link(s) quoting a figure their target does not hold:\n{}",
        stale.len(),
        stale.join("\n")
    );
}

/// A link whose text names a document points at that document.
///
/// **The half of a citation that can be checked without reading it.** The
/// sibling test above catches a link quoting a figure its target does not
/// hold, and a citation carrying no figure was checked by nobody: `PLAN calls
/// it a five-minute implementation` pointed at PLAN's anti-patterns, and PLAN
/// has never held that phrase — it was ROADMAP's, from the scaffold commit
/// onward. The right document with a plausible-sounding section is the shape
/// this repository keeps producing.
///
/// Prose cannot be checked, but *`PLAN` in the link text and `ROADMAP.md` in
/// the destination* is a contradiction with no reading that makes it correct.
/// So this is deliberately narrow: it says nothing about whether the section
/// supports the claim, only that the file is the one the sentence names. A
/// looser version — do the link's words appear in the target section — was
/// tried first and returned seven suspects for one defect, which is a check
/// people learn to skip.
///
/// Records are included where the figure test excludes them. A record is a log
/// of what was true when it was taken, and no reading of that makes the
/// document name in its own sentence wrong.
#[test]
fn a_link_naming_a_document_points_at_that_document() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits one level below the repository root");

    let named: [(&str, &str); 4] = [
        ("PLAN", "PLAN.md"),
        ("ROADMAP", "ROADMAP.md"),
        ("CONTRIBUTING", "CONTRIBUTING.md"),
        ("CLAUDE", "CLAUDE.md"),
    ];

    /// The name has to stand as a word, so `PLANNED` and a path spelled out in
    /// the label are both left alone.
    fn names_document(label: &str, name: &str) -> bool {
        label.match_indices(name).any(|(at, _)| {
            let before = label[..at].chars().next_back();
            let after = label[at + name.len()..].chars().next();
            let boundary =
                |c: Option<char>| !c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '.');
            boundary(before) && boundary(after)
        })
    }

    let mut docs = Vec::new();
    markdown_files(root, &mut docs);

    let mut wrong = Vec::new();
    let mut checked = 0usize;
    for doc in &docs {
        let body = fs::read_to_string(doc).expect("readable markdown");
        for (label, target) in links(&body) {
            if target.starts_with("http") {
                continue;
            }
            let path_part = target.split('#').next().unwrap_or_default();
            if path_part.is_empty() {
                continue; // an anchor alone stays inside this file
            }
            for (name, file) in named {
                if !names_document(&label, name) {
                    continue;
                }
                checked += 1;
                if !path_part.ends_with(file) {
                    wrong.push(format!(
                        "{}  says {name}  ->  {target}",
                        doc.strip_prefix(root).unwrap_or(doc).display()
                    ));
                }
            }
        }
    }

    // A check nobody has seen fail is a check nobody has read. This one was
    // shown failing on the citation in the doc comment before it was fixed.
    //
    // Fifteen links name a document today, counted twice by different routes —
    // once here and once by a sweep over the same tree — after the second
    // route was taught to skip fenced blocks, which is where the first
    // disagreement lived. The floor sits below that and above zero, so
    // deleting the last such link fails loudly rather than passing vacuously.
    assert!(
        checked >= 12,
        "only {checked} links name a document, so this test is looking at nothing"
    );
    assert!(
        wrong.is_empty(),
        "{} link(s) naming one document and pointing at another:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// Every measurement record is reachable from the index that promises to list
/// them.
///
/// **Weaker than what a sweep can do by hand, and deliberately.** Reading
/// which records nothing *outside* the index cites is the useful question —
/// it found PLAN citing where the duty relation was derived while claiming it
/// had been checked, so the stronger evidence sat unquoted. But a record
/// written an hour ago is legitimately cited by nothing yet, and a test that
/// failed on that would be a test people learn to ignore.
///
/// So this asks only the mechanical half: a record the index does not list is
/// invisible, and nothing else here can find it either.
#[test]
fn the_measurement_index_lists_every_record() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits one level below the repository root");
    let dir = root.join("docs/measurements");
    let index = fs::read_to_string(dir.join("README.md")).expect("a measurement index");

    let mut unlisted = Vec::new();
    for entry in fs::read_dir(&dir).expect("readable measurements").flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.starts_with(|c: char| c.is_ascii_digit()) || !name.ends_with(".md") {
            continue;
        }
        if !index.contains(name) {
            unlisted.push(name.to_string());
        }
    }
    unlisted.sort();

    assert!(
        unlisted.is_empty(),
        "these records exist and the index does not list them: {unlisted:?}"
    );
}

#[test]
fn every_cross_reference_resolves() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits one level below the repository root");

    let mut docs = Vec::new();
    markdown_files(root, &mut docs);
    assert!(
        !docs.is_empty(),
        "found no markdown to check under {root:?}"
    );

    let mut anchor_cache: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    for doc in &docs {
        let body = fs::read_to_string(doc).expect("readable markdown");
        anchor_cache.insert(doc.clone(), anchors(&body));
    }

    let mut broken = Vec::new();
    for doc in &docs {
        let body = fs::read_to_string(doc).expect("readable markdown");
        let here = doc.parent().expect("file has a parent directory");

        for target in link_targets(&body) {
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
            {
                continue;
            }
            // **A rustdoc intra-doc link is not a path**, and `rustdoc` already
            // resolves it — `[Comparison](crate::compare::Comparison)` names an
            // item, not a file. Eleven of the fourteen reports the first widened
            // run produced were these; excluding them by the `::` that makes
            // them Rust leaves the three that were real.
            if target.contains("::") {
                continue;
            }

            let (path_part, anchor) = match target.split_once('#') {
                Some((p, a)) => (p, Some(a)),
                None => (target.as_str(), None),
            };

            let dest = if path_part.is_empty() {
                doc.clone()
            } else {
                here.join(path_part)
            };

            let Ok(dest) = dest.canonicalize() else {
                broken.push(format!("{}  ->  {target}  (no such file)", doc.display()));
                continue;
            };

            let Some(anchor) = anchor else { continue };
            if dest.extension().is_some_and(|e| e == "md") {
                let known = anchor_cache.entry(dest.clone()).or_insert_with(|| {
                    anchors(&fs::read_to_string(&dest).expect("readable markdown"))
                });
                if !known.contains(anchor) {
                    broken.push(format!(
                        "{}  ->  {target}  (no such heading)",
                        doc.display()
                    ));
                }
            }
        }
    }

    assert!(
        broken.is_empty(),
        "{} broken cross-reference(s):\n{}",
        broken.len(),
        broken.join("\n")
    );
}

#[test]
fn figures_reads_measurements_and_leaves_dates_and_versions_alone() {
    assert_eq!(figures("four readings give 1.865 GiB and 6%"), ["1.865"]);
    assert_eq!(figures("3.27 GiB and 1.24 cores"), ["3.27", "1.24"]);
    assert_eq!(figures("a teardown 361× slower"), ["361"]);
    assert_eq!(figures("about 13,100 MB across a hundred"), ["13,100"]);
    // A date would otherwise contribute 07 and 31, and a two-digit number
    // turns up somewhere in almost any document.
    assert_eq!(figures("Frozen on 2026-07-31 at nine"), [] as [&str; 0]);
    // Single digits, years and long bare runs are not measurements here.
    assert_eq!(
        figures("9 sessions in 2026 on build 26200"),
        [] as [&str; 0]
    );
    // A run already consumed is not re-entered from inside itself.
    assert_eq!(figures("12.5%"), ["12.5"]);
}

#[test]
fn holds_refuses_a_figure_that_is_part_of_a_longer_number() {
    assert!(holds("the CLI holds 580 MiB", "580"));
    assert!(!holds("port 15801 was open", "580"));
    assert!(holds("1.87 GiB", "1.87"));
    assert!(!holds("1.874 GiB", "1.87"));
    // A target may group the same quantity either way.
    assert!(holds("reached 13100 MB", "13,100"));
    assert!(holds("reached 13,100 MB", "13,100"));
}

#[test]
fn slug_matches_github_anchor_rules() {
    assert_eq!(slug("Architecture"), "architecture");
    assert_eq!(slug("Four core decisions"), "four-core-decisions");
    assert_eq!(slug("License: GPL-3.0-or-later"), "license-gpl-30-or-later");
    // Dropping the separator leaves two spaces, hence the double dash.
    assert_eq!(slug("M1 · Headless Daemon"), "m1--headless-daemon");
    assert_eq!(slug("Name: COGGY (settled)"), "name-coggy-settled");
    assert_eq!(slug("`redline` is a pair"), "redline-is-a-pair");
}

/// The minimum supported Rust version is stated in four documents, and
/// `Cargo.toml` is the only one that enforces it.
///
/// Nothing drifted yet — all four say 1.88 — which is exactly when the check is
/// cheap to add. A workspace `rust-version` bump lands in one file and leaves
/// the READMEs telling a contributor to install a toolchain that will no longer
/// build the tree, and `cargo` cannot notice: it reads the manifest and never
/// the prose. This is the same shape as the documentation map above, one field
/// wide.
#[test]
fn every_document_states_the_msrv_the_manifest_declares() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits one level below the repository root");
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("a workspace manifest");
    let declared = manifest
        .lines()
        .find_map(|line| line.trim().strip_prefix("rust-version"))
        .and_then(|rest| rest.split('"').nth(1))
        .expect("the workspace declares rust-version");

    let mut wrong = Vec::new();
    for doc in ["README.md", "CONTRIBUTING.md", "sessionbench/README.md"] {
        let body = fs::read_to_string(root.join(doc)).expect("a document that exists");
        // Every mention of a Rust version in these files is the MSRV, so any
        // other version named beside "Rust" is a stale copy rather than a
        // different fact.
        for (n, line) in body.lines().enumerate() {
            if !line.contains("Rust 1.") {
                continue;
            }
            if !line.contains(&format!("Rust {declared}")) {
                wrong.push(format!("{doc}:{} says {line:?}", n + 1));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the manifest declares Rust {declared}; these disagree: {wrong:#?}"
    );
}
