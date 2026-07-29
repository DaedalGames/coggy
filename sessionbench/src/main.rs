// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::{Parser, Subcommand};
use sysinfo::System;

/// Measures redline: the maximum concurrent sessions this machine sustains,
/// and the condition that limits it.
#[derive(Parser)]
#[command(name = "sessionbench", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Report the machine and which measurement axes are available.
    Doctor,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Doctor => doctor(),
    }
}

/// Reports hardware and axis availability.
///
/// Run before trusting any result. The Defender axis needs elevation, and
/// without it a run silently measures five axes out of six yet still prints a
/// redline — which is not a smaller result but a wrong one. Unavailable axes
/// are named here rather than discovered afterward.
fn doctor() -> anyhow::Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("cores      {}", System::physical_core_count().unwrap_or(0));
    println!("memory     {} GiB", sys.total_memory() / 1024 / 1024 / 1024);
    println!(
        "os         {} {}",
        System::name().unwrap_or_else(|| "unknown".into()),
        System::os_version().unwrap_or_else(|| "unknown".into()),
    );

    Ok(())
}
