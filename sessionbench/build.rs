// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Captures the parts of the provenance block that are only true at build time.
//!
//! Neither the toolchain nor the measurement crates are pinned, so every run has
//! to record what produced it. Two of those facts cannot be recovered later: the
//! compiler that built this binary is not necessarily the one `rustc -V` would
//! find at run time, and the resolved dependency versions are decided when the
//! binary is linked. The commit and the dirty flag are deliberately *not* here —
//! see `provenance.rs` for why they are read at run time instead.

use std::path::Path;
use std::process::Command;

/// Crates whose version can change a measured number.
///
/// `sysinfo` reads every RSS and CPU figure; `portable-pty` decides whether a
/// session gets a conhost. The rest of the tree renders reports and parses
/// arguments, so its versions cannot move a curve and are left out to keep the
/// block readable.
const MEASUREMENT_CRATES: &[&str] = &["sysinfo", "portable-pty"];

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    let lock = Path::new(&manifest)
        .parent()
        .expect("crate sits one level below the workspace root")
        .join("Cargo.lock");

    println!("cargo:rerun-if-changed={}", lock.display());
    println!("cargo:rustc-env=SESSIONBENCH_RUSTC={}", rustc_version());
    println!(
        "cargo:rustc-env=SESSIONBENCH_DEPS={}",
        measurement_crate_versions(&lock)
    );
}

/// The full `rustc -V` string, so the report names a release rather than a
/// channel that has since moved.
fn rustc_version() -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    Command::new(rustc)
        .arg("-V")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

/// Resolved versions of [`MEASUREMENT_CRATES`], as `name=version` pairs.
///
/// Read straight from the lockfile rather than through a TOML parser: the two
/// keys involved are one line each, and a benchmark should not grow a
/// dependency to describe its own dependencies.
fn measurement_crate_versions(lock: &Path) -> String {
    let Ok(body) = std::fs::read_to_string(lock) else {
        return "unknown".into();
    };

    let mut found = Vec::new();
    let mut name = None;
    for line in body.lines() {
        if let Some(value) = quoted_value(line, "name") {
            name = MEASUREMENT_CRATES.iter().find(|c| **c == value).copied();
        } else if let (Some(crate_name), Some(version)) = (name, quoted_value(line, "version")) {
            found.push(format!("{crate_name}={version}"));
            name = None;
        }
    }

    if found.is_empty() {
        return "unknown".into();
    }
    found.join(",")
}

/// The value of a `key = "value"` lockfile line, if this line is that key.
fn quoted_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.strip_prefix(key)?
        .trim_start()
        .strip_prefix('=')?
        .trim()
        .strip_prefix('"')?
        .strip_suffix('"')
}
