// stdout-storm — a workload for sessionbench.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! A session whose cost is what it says, not what it computes or writes.
//!
//! This is the payload side of the format this project took from vtebench: a
//! workload directory holding an executable whose **stdout is the benchmark**.
//! The format was adopted early and the workload it exists for was never
//! written, so the condition that tolerates zero dropped output had never seen
//! any output worth dropping — every run reported zero because the other
//! workloads emit about twenty bytes per unit.
//!
//! It matters most under a pseudoconsole. A hundred sessions each pushing
//! megabytes a second through ConPTY is the shape terminal benchmarks have
//! measured for years, and the one this project claims a hundred of.
//!
//! See `../README.md` for the contract every workload keeps.

use std::io::Write;
use std::time::{Duration, Instant};

use clap::Parser;

/// Page size to stride when keeping the held memory resident.
const PAGE: usize = 4096;

/// How often the held memory is re-touched.
const TOUCH_INTERVAL: Duration = Duration::from_secs(1);

const BYTES_PER_MIB: usize = 1024 * 1024;
const BYTES_PER_KIB: usize = 1024;

/// Writes large lines to stdout as fast as it is allowed to.
#[derive(Parser)]
#[command(name = "stdout-storm", version, about)]
struct Args {
    /// Lines to write before exiting. Each line is one unit.
    #[arg(long, default_value_t = 100_000)]
    units: u32,

    /// Payload on each line, after the ordinal.
    #[arg(long, default_value_t = 8, value_name = "KIB")]
    line: usize,

    /// Wait between lines. Zero writes as fast as the reader accepts.
    #[arg(long, default_value_t = 0, value_name = "MS")]
    interval: u64,

    /// Memory held resident for the whole run.
    #[arg(long, default_value_t = 80, value_name = "MIB")]
    resident: usize,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut held = vec![0u8; args.resident * BYTES_PER_MIB];
    touch(&mut held, 0);

    // Printable and non-repeating within a line, so a reader that loses a
    // chunk cannot silently splice the remainder into something plausible.
    let payload: Vec<u8> = (0..args.line * BYTES_PER_KIB)
        .map(|i| b'!' + (i % 90) as u8)
        .collect();

    // One lock and one buffer for the whole run. Taking the lock per line
    // would measure the lock, and this workload exists to measure the pipe.
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut line = Vec::with_capacity(payload.len() + 32);
    let mut last_touch = Instant::now();

    for unit in 1..=args.units {
        line.clear();
        // The ordinal opens the line, which is what makes a gap detectable —
        // and detecting gaps is the entire reason this workload exists.
        write!(line, "{unit} ")?;
        line.extend_from_slice(&payload);
        line.push(b'\n');

        // A short write here would be a drop this process caused rather than
        // one the session did, so it is an error rather than a silent partial.
        out.write_all(&line)?;
        out.flush()?;

        if last_touch.elapsed() >= TOUCH_INTERVAL {
            touch(&mut held, unit as usize);
            last_touch = Instant::now();
        }
        if args.interval > 0 {
            std::thread::sleep(Duration::from_millis(args.interval));
        }
    }
    Ok(())
}

/// Writes one byte per page so the whole allocation stays resident.
fn touch(buffer: &mut [u8], round: usize) {
    let mark = (round % 251) as u8;
    for page in buffer.chunks_mut(PAGE) {
        page[0] = mark;
    }
}
