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

/// Link targets — the `target` of every `[text](target)` outside code blocks.
fn link_targets(body: &str) -> Vec<String> {
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
                found.push(chars[i + 2..i + 2 + end].iter().collect());
                i += end + 2;
            }
            i += 1;
        }
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
fn slug_matches_github_anchor_rules() {
    assert_eq!(slug("Architecture"), "architecture");
    assert_eq!(slug("Four core decisions"), "four-core-decisions");
    assert_eq!(slug("License: GPL-3.0-or-later"), "license-gpl-30-or-later");
    // Dropping the separator leaves two spaces, hence the double dash.
    assert_eq!(slug("M1 · Headless Daemon"), "m1--headless-daemon");
    assert_eq!(slug("Name: COGGY (settled)"), "name-coggy-settled");
    assert_eq!(slug("`redline` is a pair"), "redline-is-a-pair");
}
