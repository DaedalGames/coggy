// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Holds `coggyd` under the harness for half a minute and samples it.
//!
//! **An example rather than a test, because it needs the daemon's binary.**
//! `cargo test` does not guarantee a sibling crate's executable exists, and a
//! test that silently skips when it is missing is a test nobody knows is not
//! running. What it asserts on is the effect: sessions alive while the pipe is
//! held, RSS attributed through the job the harness armed, and nothing left
//! behind.
//!
//! ```text
//! cargo build -p coggyd && cargo run -p sessionbench --example hold_daemon
//! ```

use std::path::PathBuf;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let daemon = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits one level below the repository root")
        .join("target/debug/coggyd.exe");
    if !daemon.is_file() {
        eprintln!("build it first: cargo build -p coggyd");
        std::process::exit(2);
    }

    let sessions = 4;
    let log = std::env::temp_dir().join("sessionbench-hold-daemon.log");
    let workload = [
        "ping".to_string(),
        "-n".into(),
        "120".into(),
        "127.0.0.1".into(),
    ];

    let run = sessionbench::daemon::hold(
        &daemon,
        sessions,
        &workload,
        log.clone(),
        Duration::from_secs(2),
        Duration::from_secs(26),
        None,
        // no abort ceiling: this example measures whatever the box is doing
        None,
    )?;

    println!("samples          {}", run.samples.len());
    println!("peak total rss   {} bytes", run.peak_rss_bytes());
    println!("fewest running   {:?}", run.fewest_running);
    println!("last report      {:?}", run.last);
    match run.unusable() {
        Some(why) => println!("UNUSABLE: {why}"),
        None => println!("usable"),
    }

    println!("--- what the daemon wrote");
    print!("{}", std::fs::read_to_string(&log)?);
    let _ = std::fs::remove_file(&log);
    Ok(())
}
