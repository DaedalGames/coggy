// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Kills the daemon partway through a hold and checks the run refuses itself.
//!
//! **An example rather than a test, for the reason the sibling one gives**: it
//! needs `coggyd`'s binary, and a test that skips when that is missing is a
//! test nobody knows is not running.
//!
//! Before the guard this checks, a daemon that left at five seconds of an hour
//! produced fifty-nine minutes of empty samples, a session count frozen at
//! whatever its last report said, and no complaint.
//!
//! ```text
//! cargo build -p coggyd && cargo run -p sessionbench --example daemon_dies
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

    // A second thread kills every coggyd it can find, part way in.
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(8));
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "coggyd.exe"])
            .output();
        println!("  (killed the daemon at about 8s of a 30s hold)");
    });

    let log = std::env::temp_dir().join("sessionbench-daemon-dies.log");
    let run = sessionbench::daemon::hold(
        &daemon,
        4,
        &[
            "ping".to_string(),
            "-n".into(),
            "120".into(),
            "127.0.0.1".into(),
        ],
        log.clone(),
        Duration::from_secs(2),
        Duration::from_secs(30),
        None,
    )?;

    println!("samples          {}", run.samples.len());
    println!("fewest running   {:?}", run.fewest_running);
    println!("left early       {:?}", run.left_early);
    match run.unusable() {
        Some(why) => println!("REFUSED: {why}"),
        None => println!("USABLE — which would be the defect"),
    }
    let _ = std::fs::remove_file(&log);
    Ok(())
}
