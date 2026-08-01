// coggyd — the session supervisor.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! The daemon process: hold N sessions until told to stop.
//!
//! **Deliberately not a socket API.** [M2 derives that surface backward from
//! the calls the harness makes](../../ROADMAP.md#m2--harness-contract), so
//! choosing verbs here would invent what that milestone exists to discover.
//! What this is instead is the thing [the comparison
//! set](../../sessionbench/README.md#what-we-measure-against) needs before it
//! can hold a row for us: a process that owns sessions, which a benchmark can
//! start and measure.
//!
//! Lifetime is stdin. The daemon holds its pool until end-of-file and then
//! clears it, which needs no signal crate and no console — [`timeout /t`
//! returning instantly without one](../../docs/measurements/2026-07-31-035111-between-builds.md)
//! is why a console-dependent stop condition is not used.

use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use coggyd::pool::Pool;

/// How often the daemon says what it is holding.
const REPORT_EVERY: Duration = Duration::from_secs(10);

/// How long the main loop sleeps between checks that it should stop.
///
/// Short enough that a closed pipe is noticed promptly, long enough that
/// holding a hundred idle sessions costs nothing measurable.
const TICK: Duration = Duration::from_millis(100);

fn main() -> Result<()> {
    let (sessions, command) = parse(std::env::args().skip(1).collect())?;

    // Every session through the template path, including a command with no
    // placeholder in it. Two spawn routes taken by session count is how one of
    // them stops being exercised.
    let mut pool = Pool::new();
    for _ in 0..sessions {
        pool.spawn_template(&command)
            .context("starting a session")?;
    }
    println!("holding {} session(s); stdin closes to stop", pool.len());

    // **The stop condition and the report clock are separate on purpose.**
    // They were one loop, reading stdin and checking the interval after each
    // read — which ties how often the daemon speaks to how much its caller
    // types. An hour-long hold whose stdin holder wrote nothing produced no
    // periodic line at all, and a benchmark scraping that line for a unit
    // count would have read a run of a hundred sessions as silent.
    //
    // So stdin gets a thread whose only job is to reach end-of-file, and the
    // clock runs here.
    let stopped = Arc::new(AtomicBool::new(false));
    let watcher = Arc::clone(&stopped);
    std::thread::spawn(move || {
        let mut sink = [0u8; 64];
        let mut stdin = std::io::stdin();
        loop {
            match stdin.read(&mut sink) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }
        watcher.store(true, Ordering::Release);
    });

    let mut last = Instant::now();
    while !stopped.load(Ordering::Acquire) {
        std::thread::sleep(TICK);
        if last.elapsed() >= REPORT_EVERY {
            report(&mut pool);
            last = Instant::now();
        }
    }

    report(&mut pool);
    pool.clear();
    println!("cleared");
    Ok(())
}

/// Says what is held, what is still running, and what came out of it.
///
/// Held and running, always: a finished session keeps its slot until it is
/// reaped, and a supervisor that printed one figure would be hiding which.
///
/// **The three output counters are here because a gate condition needs them.**
/// [M1 asks for no dropped output](../../ROADMAP.md#m1--headless-daemon), and
/// an hour-long hold could not be asked: the counters existed in the library
/// and stopped there, so no length of run produced the number. `read` is the
/// term the benchmark subtracts from what its workload emitted; `evicted` and
/// `truncated` are policy, and are printed beside it so a shortfall cannot be
/// blamed on the gate's failure when it was ours.
fn report(pool: &mut Pool) {
    let running = pool.running();
    let out = pool.output();
    println!(
        "held {} · running {running} · read {} · evicted {} · truncated {}",
        pool.len(),
        out.read,
        out.evicted,
        out.truncated
    );
}

/// `coggyd --sessions N -- <command>...`
fn parse(args: Vec<String>) -> Result<(usize, Vec<String>)> {
    let mut sessions = 1usize;
    let mut rest = Vec::new();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--sessions" => {
                let n = iter.next().context("--sessions wants a number")?;
                sessions = n.parse().with_context(|| format!("not a count: {n}"))?;
            }
            "--" => {
                rest = iter.collect();
                break;
            }
            other => bail!("unexpected argument {other}; usage: coggyd --sessions N -- <command>"),
        }
    }
    if rest.is_empty() {
        bail!("no command given; usage: coggyd --sessions N -- <command>");
    }
    Ok((sessions, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn a_command_after_the_separator_is_taken_whole() {
        let (n, cmd) = parse(args(&["--sessions", "4", "--", "cmd", "/c", "echo hi"])).expect("ok");
        assert_eq!(n, 4);
        assert_eq!(cmd, args(&["cmd", "/c", "echo hi"]));
    }

    #[test]
    fn the_session_count_defaults_to_one() {
        let (n, _) = parse(args(&["--", "cmd"])).expect("ok");
        assert_eq!(n, 1);
    }

    #[test]
    fn a_missing_command_is_refused_rather_than_run_as_nothing() {
        // Spawning zero sessions and reporting success would be the shape this
        // repository keeps meeting: an exit that says nothing happened wrong.
        assert!(parse(args(&["--sessions", "4"])).is_err());
        assert!(parse(args(&["--sessions", "4", "--"])).is_err());
    }

    #[test]
    fn an_unparseable_count_names_what_it_saw() {
        let err = parse(args(&["--sessions", "many", "--", "cmd"])).unwrap_err();
        assert!(format!("{err}").contains("many"), "{err}");
    }
}
