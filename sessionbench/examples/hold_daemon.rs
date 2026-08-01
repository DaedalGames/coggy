// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Starts `coggyd`, holds it for a few seconds, and reports what it said.
//!
//! **An example rather than a test, because it needs the daemon's binary.**
//! `cargo test` does not guarantee a sibling crate's executable exists, and a
//! test that silently skips when it is missing is a test nobody knows is not
//! running. This is run by hand, and what it asserts on is the effect: the
//! daemon lives while its stdin is held and stops when it is dropped.
//!
//! ```text
//! cargo build -p coggyd && cargo run -p sessionbench --example hold_daemon
//! ```

use std::path::PathBuf;

use sessionbench::daemon::Held;

fn main() -> std::io::Result<()> {
    let daemon = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits one level below the repository root")
        .join("target/debug/coggyd.exe");
    if !daemon.is_file() {
        eprintln!("build it first: cargo build -p coggyd");
        std::process::exit(2);
    }

    let log = std::env::temp_dir().join("sessionbench-hold-daemon.log");
    let workload = [
        "ping".to_string(),
        "-n".into(),
        "60".into(),
        "127.0.0.1".into(),
    ];

    let held = Held::start(&daemon, 3, &workload, log.clone())?;
    println!("started, pid {}", held.child.id());

    // Long enough for two of the daemon's ten-second reports.
    std::thread::sleep(std::time::Duration::from_secs(22));
    let mid = held.seen();
    println!(
        "while held:  units {:?}  fewest running {:?}",
        mid.units(),
        mid.fewest_running()
    );

    let status = held.stop()?;
    println!("stopped: {status}");
    println!("--- what it wrote");
    print!("{}", std::fs::read_to_string(&log)?);
    let _ = std::fs::remove_file(&log);
    Ok(())
}
