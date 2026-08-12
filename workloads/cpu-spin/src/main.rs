// cpu-spin — a workload for sessionbench.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! A session that only computes: no files, no disk at all, and no waiting
//! unless asked for one.
//!
//! This exists to separate two things the ramp otherwise cannot tell apart.
//! When per-session work rate falls, the cores went either to the sessions
//! competing with each other or to Defender scanning what they wrote — and a
//! workload that writes files always mixes the two. Running the same ramp
//! against this one and against `file-write` makes the difference between them
//! the scanning term, measured rather than inferred.
//!
//! It is also the harsher of the two. A hundred sessions that never yield will
//! find the core count long before a hundred that spend most of their time
//! asleep, which is the point: the ceiling being looked for is where work rate
//! collapses, and this reaches it soonest.
//!
//! See `../README.md` for the contract every workload keeps.

use std::io::Write;
use std::time::{Duration, Instant};

use clap::Parser;

/// Page size to stride when keeping the held memory resident.
const PAGE: usize = 4096;

/// How often the held memory is re-touched.
///
/// Windows trims a working set that goes quiet, and this is what stops that —
/// but the trimming it defends against happens over seconds. Touching once per
/// unit instead meant re-dirtying twenty megabytes eighty times a second, and
/// twenty-five such sessions saturated the memory bus so thoroughly that
/// *reading* a process's memory took twenty-eight seconds. A workload that
/// holds memory must not re-dirty it faster than the system would reclaim it,
/// or it stops measuring what it claims to.
const TOUCH_INTERVAL: Duration = Duration::from_secs(1);

const BYTES_PER_MIB: usize = 1024 * 1024;

/// Computes continuously, printing one line per unit finished.
#[derive(Parser)]
#[command(name = "cpu-spin", version, about)]
struct Args {
    /// Units to finish before exiting.
    #[arg(long, default_value_t = 60)]
    units: u32,

    /// Mixing rounds that make up one unit.
    ///
    /// A serial dependency chain, so this is a fixed amount of work rather
    /// than something the machine can widen its way through. Roughly 30 ms of
    /// one core at the default.
    #[arg(long, default_value_t = 4_000_000)]
    iterations: u64,

    /// Memory held resident for the whole run.
    #[arg(long, default_value_t = 80, value_name = "MIB")]
    resident: usize,

    /// Share of wall-clock time spent computing, from just above 0 to 1.
    ///
    /// A generation session is not flat out. It waits on a model, works, and
    /// waits again, and how much of the time it spends on each is the single
    /// number standing between the bracket this benchmark already produces —
    /// 25 sessions flat out, above 100 mostly waiting — and an answer for a
    /// real one. Measure a real session's duty, run this at that value, and
    /// the ramp reads off the redline.
    ///
    /// Self-calibrating: each unit is timed and the wait is derived from what
    /// that unit actually cost, so the ratio holds on a machine under load as
    /// well as an idle one.
    #[arg(long, default_value_t = 1.0)]
    duty: f64,

    /// Wait a fixed span after each unit instead of a proportional one.
    ///
    /// The shape a generation session really has. `--duty` keeps its ratio
    /// under load by stretching the pause to match a slower unit; a session
    /// waiting on a model gets the same wait however loaded the machine is, so
    /// its duty climbs as its compute slows.
    ///
    /// Solving both cases gives `slowdown = N·d/C` either way — the mechanism
    /// cancels, provided the wait really releases the core. This flag is what
    /// tests that, by pairing against a `--duty` run of the same solo duty.
    ///
    /// **A climbing duty is a positive feedback, and it is what stopped the
    /// measuring machine.** Less speed asks for more demand, which costs more
    /// speed. Forty-one minutes of a hundred sessions under this flag ended in
    /// a hard power-off with its duty travelling from 0.172 toward 0.271;
    /// holds at a fixed `--duty` have finished clean. Use this to pair against
    /// `--duty`, not to hold a machine for an hour.
    /// Report what the pause was asked for against what it cost, on stderr.
    ///
    /// **OFF BY DEFAULT BECAUSE IT CHANGES THE THING BEING MEASURED.** It adds
    /// two clock reads per unit and a line per report interval; with it off,
    /// not one instruction differs from before it existed.
    ///
    /// WHY IT EXISTS. `computed` is `working.elapsed()` — WALL TIME across the
    /// spin, so any preemption during the spin inflates it, and the pause is
    /// 2.7x that at duty 0.27. On 2026-08-12 a hundred of these oversleep by
    /// 10.7x while the box is 70% idle, and nine other causes were eliminated
    /// by measurement: the daemon, the harness, the observer, `--resident`, the
    /// pause mechanism, a job cap, timer resolution, and general wake latency
    /// (an outside sleeper degrades 1.19x to 1.37x against these sessions'
    /// 10.7x). What is left is this arithmetic, and nothing recorded whether
    /// the pause was computed from an inflated input or the sleep overshot.
    ///
    /// STDERR RATHER THAN STDOUT, because stdout is the unit stream and [a unit
    /// is a line](../../README.md#the-contract): a summary there would be
    /// counted as work.
    ///
    /// **THAT ONLY HELPS OUTSIDE THE DAEMON, and the flag is not free under it.**
    /// `coggyd` drains stdout AND stderr into one scrollback, so under the
    /// daemon these lines are captured — not discarded, which was the guess —
    /// and counted like any other output: one line per fifty units is about 2%
    /// on the unit rate. Measured 2026-08-12: a hundred sessions with the flag
    /// on reported 1.676 units/s/session and no timing line reached any
    /// artifact, because nothing exports the scrollback.
    ///
    /// So this flag answers the question outside the daemon and biases it
    /// inside. Reading `computed` in a daemon-hosted hold needs the scrollback
    /// exported or a channel that is not the session's output at all.
    #[arg(long)]
    report_timing: bool,

    #[arg(long, value_name = "MS", conflicts_with = "duty")]
    wait_ms: Option<u64>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut held = vec![0u8; args.resident * BYTES_PER_MIB];
    touch(&mut held, 0);

    let mut stdout = std::io::stdout().lock();
    let mut state = 0x243F_6A88_85A3_08D3_u64;
    let mut last_touch = Instant::now();

    // Clamped rather than rejected: a duty of zero is a session that never
    // works, which is not a session.
    let duty = args.duty.clamp(0.01, 1.0);

    // Aggregates for `--report-timing`, summed rather than sampled so a report
    // covers every unit in its interval rather than whichever one it landed on.
    let mut acc_computed = Duration::ZERO;
    let mut acc_asked = Duration::ZERO;
    let mut acc_slept = Duration::ZERO;
    let mut acc_units: u64 = 0;

    for unit in 1..=args.units {
        let working = Instant::now();
        state = spin(state, args.iterations);
        let computed = working.elapsed();
        if last_touch.elapsed() >= TOUCH_INTERVAL {
            touch(&mut held, unit as usize);
            last_touch = Instant::now();
        }

        // The ordinal opens the line so a gap is visible afterwards, and the
        // state is printed so no optimizer can decide the loop was pointless.
        //
        // **A `--threads` flag would have to share this counter.** `session.rs`
        // tracks the largest ordinal any line opened with and reads a gap below
        // that maximum as dropped output — which is a third of gate M1. Threads
        // each running `1..=units` would restart from one, and every line below
        // the running maximum would report as loss on a session that lost
        // nothing. Any threaded version needs a single `AtomicU64` handing out
        // ordinals, not a per-thread range.
        writeln!(stdout, "{unit} {state:016x}")?;
        stdout.flush()?;

        let asked = match args.wait_ms {
            Some(ms) => Duration::from_millis(ms),
            None if duty < 1.0 => computed.mul_f64((1.0 - duty) / duty),
            None => Duration::ZERO,
        };
        if args.report_timing {
            // Two clock reads, and only when asked for.
            let before = Instant::now();
            if !asked.is_zero() {
                std::thread::sleep(asked);
            }
            acc_slept += before.elapsed();
            acc_computed += computed;
            acc_asked += asked;
            acc_units += 1;
            if acc_units >= REPORT_UNITS {
                let ms = |d: Duration| d.as_secs_f64() * 1000.0 / acc_units as f64;
                eprintln!(
                    "timing units {acc_units} computed_ms {:.3} asked_ms {:.3} slept_ms {:.3} oversleep {:.2}",
                    ms(acc_computed),
                    ms(acc_asked),
                    ms(acc_slept),
                    if acc_asked.is_zero() {
                        0.0
                    } else {
                        acc_slept.as_secs_f64() / acc_asked.as_secs_f64()
                    },
                );
                acc_computed = Duration::ZERO;
                acc_asked = Duration::ZERO;
                acc_slept = Duration::ZERO;
                acc_units = 0;
            }
        } else if !asked.is_zero() {
            std::thread::sleep(asked);
        }
    }
    Ok(())
}

/// Units between `--report-timing` lines.
///
/// **Sized so a report actually fires in the regime it exists to measure**, and
/// 200 was not: at ~0.5 s a unit under a hundred sessions that is ninety-five
/// seconds, so a 25-second hold emitted nothing and a 100-second probe was
/// killed on the boundary having reported once or not at all. A check that
/// cannot fire in the case it was built for has not been built.
///
/// At 50 it is about 25 s under a hundred sessions and about 5 s solo — several
/// reports inside any hold this repository takes, and still one line per fifty
/// units rather than per unit, so the reporting is not the measurement.
const REPORT_UNITS: u64 = 50;

/// A serial chain of splitmix64 rounds.
///
/// Each round consumes the previous one's output, so the work cannot be
/// vectorised or reordered into something shorter — which is what makes one
/// unit mean the same amount of work on a busy machine as on an idle one.
fn spin(seed: u64, iterations: u64) -> u64 {
    let mut x = seed;
    for _ in 0..iterations {
        x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        x ^= x >> 30;
        x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^= x >> 31;
    }
    x
}

/// Writes one byte per page so the whole allocation stays resident.
fn touch(buffer: &mut [u8], round: usize) {
    let mark = (round % 251) as u8;
    for page in buffer.chunks_mut(PAGE) {
        page[0] = mark;
    }
}
