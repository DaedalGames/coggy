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
        } else if path.extension().is_some_and(|e| e == "md") {
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
    let mut fenced = false;
    let mut found = Vec::new();
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i + 1 < chars.len() {
            let opens_link = chars[i] == ']' && chars[i + 1] == '(';
            let close = chars[i + 2..].iter().position(|c| *c == ')');
            if let (true, Some(end)) = (opens_link, close) {
                let target: String = chars[i + 2..i + 2 + end].iter().collect();
                // The label is whatever sits inside the nearest `[` before
                // this `]`; nested brackets are rare enough in these
                // documents that the nearest one is the right one.
                let label = chars[..i]
                    .iter()
                    .rposition(|c| *c == '[')
                    .map(|open| chars[open + 1..i].iter().collect())
                    .unwrap_or_default();
                found.push((label, target));
                i += end + 2;
            }
            i += 1;
        }
    }
    found
}

/// Link targets — the `target` of every `[text](target)` outside code blocks.
fn link_targets(body: &str) -> Vec<String> {
    links(body).into_iter().map(|(_, target)| target).collect()
}

/// Decimal figures in a piece of prose: `1.87`, `3.27`, `12.5`.
///
/// Decimals only. A bare integer is too often a count, a year or a milestone
/// number to be worth chasing, and the figures that go stale here carry a
/// point.
fn figures(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        let decimal_follows =
            chars.get(i) == Some(&'.') && chars.get(i + 1).is_some_and(char::is_ascii_digit);
        if !decimal_follows {
            continue;
        }
        i += 1;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        found.push(chars[start..i].iter().collect());
    }
    found
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
                if !cited.contains(&figure) {
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
fn figures_reads_decimals_and_leaves_counts_alone() {
    assert_eq!(figures("four readings give 1.865 GiB and 6%"), ["1.865"]);
    assert_eq!(figures("3.27 GiB and 1.24 cores"), ["3.27", "1.24"]);
    // Counts, years and milestone numbers carry no point and are not chased.
    assert_eq!(
        figures("nine sessions on 16 cores in 2026, M0"),
        [] as [&str; 0]
    );
    // A run already consumed is not re-entered from inside itself.
    assert_eq!(figures("12.5%"), ["12.5"]);
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
