// file-write — a workload for sessionbench.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! A session shaped like the thing being measured: it holds memory for as long
//! as it runs, writes files the whole time, and says so one line at a time.
//!
//! Generation sessions are not compute spikes. They sit resident for hours with
//! a live heap and touch the disk continuously, which is what makes them
//! expensive to hold a hundred of. A workload that allocates nothing and writes
//! in one burst would measure a different machine.
//!
//! See `../README.md` for the contract every workload keeps.

use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;

/// Page size to stride when keeping the held memory resident.
///
/// Touching one byte per page is enough to fault it in and enough to keep
/// Windows from trimming it back out of the working set.
const PAGE: usize = 4096;

const BYTES_PER_MIB: usize = 1024 * 1024;
const BYTES_PER_KIB: usize = 1024;

/// Holds memory resident and writes files, printing one line per file.
#[derive(Parser)]
#[command(name = "file-write", version, about)]
struct Args {
    /// Files to write before finishing.
    #[arg(long, default_value_t = 60)]
    files: u32,

    /// Size of each file.
    #[arg(long, default_value_t = 64, value_name = "KIB")]
    size: usize,

    /// Wait between files.
    #[arg(long, default_value_t = 900, value_name = "MS")]
    interval: u64,

    /// Memory held resident for the whole run.
    ///
    /// Defaults to roughly what an agent CLI session occupies, so the memory
    /// being measured is the session's rather than the harness's.
    #[arg(long, default_value_t = 80, value_name = "MIB")]
    resident: usize,

    /// Directory to work under. A fresh subdirectory is made inside it.
    ///
    /// Defaults to `SESSIONBENCH_SCRATCH` when the benchmark set it, and the
    /// system temporary directory otherwise. Honouring that variable is what
    /// lets the benchmark clean up after a session it killed, which is how
    /// every ramp rung ends.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Leave the written files behind.
    #[arg(long)]
    keep: bool,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Named for this process so a hundred copies never collide, and fresh so
    // real-time scanning is measured rather than its cache.
    let root = args
        .out
        .or_else(|| std::env::var_os("SESSIONBENCH_SCRATCH").map(PathBuf::from))
        .unwrap_or_else(std::env::temp_dir)
        .join(format!("file-write-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;

    let mut held = vec![0u8; args.resident * BYTES_PER_MIB];
    touch(&mut held, 0);

    let payload = vec![b'x'; args.size * BYTES_PER_KIB];
    let mut stdout = std::io::stdout().lock();

    for unit in 1..=args.files {
        std::fs::write(root.join(format!("unit-{unit}.dat")), &payload)?;
        // Re-touching each round is what makes the memory *held* rather than
        // merely allocated. Windows trims a working set that goes quiet, and a
        // session whose heap is live does not go quiet.
        touch(&mut held, unit as usize);

        // One line, one unit of work, opening with the ordinal so a gap in the
        // sequence is visible afterwards. Flushed because a benchmark counting
        // these must see them as they happen, not when a buffer fills.
        writeln!(stdout, "{unit} unit-{unit}.dat")?;
        stdout.flush()?;

        std::thread::sleep(Duration::from_millis(args.interval));
    }

    if !args.keep {
        std::fs::remove_dir_all(&root)?;
    }
    Ok(())
}

/// Writes one byte per page so the whole allocation stays resident.
///
/// The value varies per round so the pages are genuinely dirtied; writing the
/// same byte back would still count as a write, but varying it removes the
/// question.
fn touch(buffer: &mut [u8], round: usize) {
    let mark = (round % 251) as u8;
    for page in buffer.chunks_mut(PAGE) {
        page[0] = mark;
    }
}
