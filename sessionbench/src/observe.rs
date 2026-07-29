// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Watches a single session from start to finish.
//!
//! This runs before the ramp harness exists, and deliberately so. The ramp
//! applies the redline conditions; one session run to completion is what says
//! whether those conditions are pointed at the right thing. Multiplying a real
//! session by a hundred on paper costs an afternoon, and if the shape
//! contradicts the conditions they get fixed before anything is built to
//! enforce them.
//!
//! Nothing here knows what the session is. It takes a command, spawns it, and
//! records what holding it costs — the same question the daemon will ask later,
//! asked with no daemon in the way.
//!
//! Prior art: `psrecord` (BSD) settled the shape of this tool years ago —
//! attach or launch, `--interval`, `--duration`, children included — so the
//! flags match it rather than inventing a dialect. `procpath` is where the idea
//! of recording a queryable history instead of a summary comes from, which here
//! is `samples.jsonl`. Process membership comes from a job object rather than
//! anything written here; see `tree.rs`.

use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use crate::host::HostFacts;
use crate::machine::Machine;
use crate::provenance::Provenance;
use crate::redline::RSS_BUDGET_FRACTION;
use crate::tree::{ArmedTree, Attribution, Membership, ProcessSample};

/// Windows Defender's scanning service.
const DEFENDER_PROCESS: &str = "MsMpEng.exe";

/// How much of a run counts as startup when splitting Defender's cost.
///
/// Generation writes files for the whole session, so the question is not what
/// Defender costs to get going but whether it keeps charging afterwards. The
/// split is what makes those two separable.
const STARTUP_WINDOW: Duration = Duration::from_secs(30);

/// Session count the report projects to.
///
/// The number the whole plan is sized around, so a projection to any other
/// count would have to be re-read against it.
const PROJECTED_SESSIONS: u32 = 100;

/// How often the run prints a line while it is going.
const HEARTBEAT: Duration = Duration::from_secs(30);

/// The cursor-position query a pseudoconsole sends, and waits on.
///
/// ConPTY expects a terminal on the other end, so it asks where the cursor is
/// before letting the client proceed. Nothing answers in a headless run, and
/// the session deadlocks at startup having produced exactly these four bytes —
/// alive, silent, and burning no CPU, which reads like a slow workload rather
/// than a hang.
const DEVICE_STATUS_REQUEST: &[u8] = b"\x1b[6n";

/// The answer. Row 1, column 1.
///
/// The position is not used by anything here; only its arrival matters. A real
/// VT parser is the answer from M1 onwards, when `alacritty_terminal` or
/// `termwiz` joins for the grid model — replying to one sequence by hand is
/// what M0 needs and no more.
const CURSOR_POSITION_REPORT: &[u8] = b"\x1b[1;1R";

/// How the session's output is wired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    /// Direct pipes, no pseudoconsole. What the daemon intends to default to.
    Pipe,
    /// A pseudoconsole, which is what a terminal gives a session today.
    Pty,
}

impl SessionMode {
    pub fn label(self) -> &'static str {
        match self {
            SessionMode::Pipe => "pipe",
            SessionMode::Pty => "pty",
        }
    }
}

/// Everything a run needs to know before it starts.
pub struct ObserveConfig {
    pub label: String,
    pub out_dir: PathBuf,
    pub interval: Duration,
    pub mode: SessionMode,
    pub max_duration: Option<Duration>,
    pub command: Vec<String>,
}

/// One instant of the session's cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub t_ms: u64,
    pub rss_bytes: u64,
    pub processes: usize,
    pub pseudoconsoles: usize,
    pub cpu_percent: f32,
    /// `None` when Defender is not running, which is itself worth recording.
    pub defender_cpu_percent: Option<f32>,
    pub defender_rss_bytes: Option<u64>,
    pub available_memory_bytes: u64,
    pub output_bytes: u64,
    /// Every member, so the artifact can be re-read for which process names
    /// hold what share without taking the run again.
    pub members: Vec<ProcessSample>,
}

/// What the run adds up to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub peak_rss_bytes: u64,
    /// Median RSS over the final quarter of the run.
    ///
    /// The figure that matters for residency. A peak is a moment; this is what
    /// the machine is asked to hold, with startup allocation behind it.
    pub steady_rss_bytes: u64,
    pub peak_processes: usize,
    pub peak_pseudoconsoles: usize,
    pub output_bytes: u64,
    pub output_bytes_per_sec: f64,
    pub defender: Option<DefenderCost>,
}

/// Defender's cost, split the way the milestone asks for it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenderCost {
    pub startup_cpu_seconds: f64,
    pub steady_cpu_seconds: f64,
    /// The rate that decides whether Defender is a fixed toll or a running one.
    pub steady_cpu_seconds_per_min: f64,
}

/// What a hundred of this session would cost if nothing interfered.
///
/// Linear and unvalidated by construction. Contention, cache pressure, and I/O
/// queueing all make the real curve worse, so this is a floor rather than an
/// estimate — useful because a floor that already breaks a condition settles
/// the question without running the ramp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projection {
    pub sessions: u32,
    pub rss_bytes: u64,
    pub rss_budget_bytes: u64,
    pub rss_condition_holds: bool,
    pub processes: usize,
    pub pseudoconsoles: usize,
    pub output_bytes_per_sec: f64,
}

/// The committed artifact for one run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub label: String,
    pub command: Vec<String>,
    pub mode: SessionMode,
    pub machine: Machine,
    pub provenance: Provenance,
    pub host: HostFacts,
    /// Whether the kernel or a parent walk decided which processes counted.
    pub membership: Membership,
    /// Why the job object was unavailable, when it was.
    pub membership_fallback_reason: Option<String>,
    pub started_unix: u64,
    pub duration_ms: u64,
    pub interval_ms: u64,
    pub sample_count: usize,
    /// `None` when the session was killed rather than exiting on its own.
    pub exit_code: Option<i32>,
    /// Whether `--duration` ended the run rather than the session finishing.
    pub stopped_at_limit: bool,
    pub summary: Summary,
    pub projection: Projection,
}

/// Runs the session and writes both artifacts into `config.out_dir`.
pub fn run(config: &ObserveConfig) -> Result<RunReport> {
    let machine = Machine::detect();
    let provenance = Provenance::current();
    let host = HostFacts::query();

    fs::create_dir_all(&config.out_dir)
        .with_context(|| format!("creating {}", config.out_dir.display()))?;
    let mut samples_file = BufWriter::new(File::create(config.out_dir.join("samples.jsonl"))?);

    let started_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();

    // Joining the job has to happen before the session exists: a process
    // inherits job membership from its parent only at creation.
    let armed = ArmedTree::arm(Pid::from_u32(std::process::id()));
    let membership = armed.membership();
    let membership_fallback_reason = armed.fallback_reason.clone();
    if let Some(reason) = &membership_fallback_reason {
        eprintln!("warning: falling back to parent-walk membership — {reason}");
    }

    let started = Instant::now();
    let Spawned {
        mut session,
        output,
        drains,
    } = spawn(config)?;
    let mut tree = armed.attach(Pid::from_u32(
        session
            .pid()
            .context("the session exited before it could be observed")?,
    ));

    let mut sys = System::new();
    // Primes the CPU counters. sysinfo reports usage as the delta between two
    // refreshes, so any process's first reading is zero.
    refresh(&mut sys);

    let mut samples: Vec<Sample> = Vec::new();
    let mut last_heartbeat = Instant::now();

    let (exit_code, stopped_at_limit) = loop {
        std::thread::sleep(config.interval);

        if let Some(code) = session.try_wait()? {
            break (code, false);
        }

        refresh(&mut sys);
        let members = tree.sample(&sys);
        let defender = sys
            .processes()
            .values()
            .find(|p| p.name().eq_ignore_ascii_case(DEFENDER_PROCESS));

        let sample = Sample {
            t_ms: started.elapsed().as_millis() as u64,
            rss_bytes: members.iter().map(|m| m.rss_bytes).sum(),
            processes: members.len(),
            pseudoconsoles: members
                .iter()
                .filter(|m| m.attribution == Attribution::Pseudoconsole)
                .count(),
            cpu_percent: members.iter().map(|m| m.cpu_percent).sum(),
            defender_cpu_percent: defender.map(|p| p.cpu_usage()),
            defender_rss_bytes: defender.map(|p| p.memory()),
            available_memory_bytes: sys.available_memory(),
            output_bytes: output.total(),
            members,
        };

        writeln!(samples_file, "{}", serde_json::to_string(&sample)?)?;
        samples_file.flush()?;

        if last_heartbeat.elapsed() >= HEARTBEAT {
            println!(
                "  {:>6}s  rss {:>10}  procs {:>3}  out {:>10}",
                sample.t_ms / 1000,
                human_bytes(sample.rss_bytes),
                sample.processes,
                human_bytes(sample.output_bytes),
            );
            last_heartbeat = Instant::now();
        }
        samples.push(sample);

        if config
            .max_duration
            .is_some_and(|limit| started.elapsed() >= limit)
        {
            session.kill()?;
            let code = session.try_wait()?.flatten();
            // Killing the root leaves its children running. The job object
            // knows every one of them, so the last sample doubles as the kill
            // list — used rather than KILL_ON_JOB_CLOSE, which fires when this
            // process's handle closes and would take the run down with it
            // before the report was written.
            reap(&mut sys, samples.last());
            break (code, true);
        }
    };

    // Dropping the session closes the pipes, which is what lets the drain
    // threads see EOF and stop counting.
    drop(session);
    for drain in drains {
        drain
            .join()
            .map_err(|_| anyhow::anyhow!("drain panicked"))??;
    }

    let duration = started.elapsed();
    let summary = summarize(&samples, output.total(), duration, config.interval);
    let projection = project(&summary, &machine);

    let report = RunReport {
        label: config.label.clone(),
        command: config.command.clone(),
        mode: config.mode,
        machine,
        provenance,
        host,
        membership,
        membership_fallback_reason,
        started_unix,
        duration_ms: duration.as_millis() as u64,
        interval_ms: config.interval.as_millis() as u64,
        sample_count: samples.len(),
        exit_code,
        stopped_at_limit,
        summary,
        projection,
    };

    fs::write(
        config.out_dir.join("run.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    Ok(report)
}

/// Terminates whatever the session left behind after its root was killed.
fn reap(sys: &mut System, last: Option<&Sample>) {
    let Some(sample) = last else { return };
    refresh(sys);
    for member in &sample.members {
        if let Some(process) = sys.process(Pid::from_u32(member.pid)) {
            process.kill();
        }
    }
}

fn refresh(sys: &mut System) {
    sys.refresh_memory();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );
}

fn summarize(
    samples: &[Sample],
    output_bytes: u64,
    duration: Duration,
    interval: Duration,
) -> Summary {
    let seconds = duration.as_secs_f64().max(f64::EPSILON);
    let tail_start = samples.len() * 3 / 4;
    let mut tail: Vec<u64> = samples[tail_start..].iter().map(|s| s.rss_bytes).collect();
    tail.sort_unstable();

    Summary {
        peak_rss_bytes: samples.iter().map(|s| s.rss_bytes).max().unwrap_or(0),
        steady_rss_bytes: tail.get(tail.len() / 2).copied().unwrap_or(0),
        peak_processes: samples.iter().map(|s| s.processes).max().unwrap_or(0),
        peak_pseudoconsoles: samples.iter().map(|s| s.pseudoconsoles).max().unwrap_or(0),
        output_bytes,
        output_bytes_per_sec: output_bytes as f64 / seconds,
        defender: defender_cost(samples, interval),
    }
}

/// Integrates Defender's CPU percentage into seconds, split at
/// [`STARTUP_WINDOW`].
///
/// Returns `None` when Defender never appeared, so a machine without it reads
/// as unmeasured rather than as costing nothing.
fn defender_cost(samples: &[Sample], interval: Duration) -> Option<DefenderCost> {
    if !samples.iter().any(|s| s.defender_cpu_percent.is_some()) {
        return None;
    }

    let per_sample = interval.as_secs_f64() / 100.0;
    let (mut startup, mut steady) = (0.0, 0.0);
    let mut steady_seconds = 0.0;
    for sample in samples {
        let cost = f64::from(sample.defender_cpu_percent.unwrap_or(0.0)) * per_sample;
        if Duration::from_millis(sample.t_ms) < STARTUP_WINDOW {
            startup += cost;
        } else {
            steady += cost;
            steady_seconds += interval.as_secs_f64();
        }
    }

    Some(DefenderCost {
        startup_cpu_seconds: startup,
        steady_cpu_seconds: steady,
        steady_cpu_seconds_per_min: if steady_seconds > 0.0 {
            steady * 60.0 / steady_seconds
        } else {
            0.0
        },
    })
}

fn project(summary: &Summary, machine: &Machine) -> Projection {
    let sessions = u64::from(PROJECTED_SESSIONS);
    let rss = summary.steady_rss_bytes.saturating_mul(sessions);
    let budget = (machine.total_memory_bytes as f64 * RSS_BUDGET_FRACTION) as u64;

    Projection {
        sessions: PROJECTED_SESSIONS,
        rss_bytes: rss,
        rss_budget_bytes: budget,
        rss_condition_holds: rss <= budget,
        processes: summary.peak_processes * PROJECTED_SESSIONS as usize,
        pseudoconsoles: summary.peak_pseudoconsoles * PROJECTED_SESSIONS as usize,
        output_bytes_per_sec: summary.output_bytes_per_sec * sessions as f64,
    }
}

/// Bytes at one decimal place, in the binary units the rest of the report uses.
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Counts every byte the session produced.
///
/// Output has to be drained whatever else happens: a session whose pipe fills
/// blocks on write, and a blocked session measures the observer rather than the
/// workload.
struct Output {
    stdout: Arc<AtomicU64>,
    stderr: Arc<AtomicU64>,
}

impl Output {
    fn total(&self) -> u64 {
        self.stdout.load(Ordering::Relaxed) + self.stderr.load(Ordering::Relaxed)
    }
}

enum Session {
    Piped(std::process::Child),
    Pty {
        child: Box<dyn portable_pty::Child + Send + Sync>,
        /// The end this process reads and writes, standing in for a terminal.
        _terminal_end: Box<dyn portable_pty::MasterPty + Send>,
        /// The pseudoconsole the session is attached to.
        ///
        /// Held for the session's life. Closing it right after spawning is the
        /// correct Unix move, since that is what lets the reader see
        /// end-of-file, but here it would tear down the console the child just
        /// attached to.
        _session_end: Box<dyn portable_pty::SlavePty + Send>,
    },
}

impl Session {
    fn pid(&self) -> Option<u32> {
        match self {
            Session::Piped(child) => Some(child.id()),
            Session::Pty { child, .. } => child.process_id(),
        }
    }

    /// `Ok(None)` while still running; `Ok(Some(code))` once it has exited,
    /// where the inner `None` means it was terminated without a code.
    #[allow(clippy::option_option)]
    fn try_wait(&mut self) -> Result<Option<Option<i32>>> {
        Ok(match self {
            Session::Piped(child) => child.try_wait()?.map(|status| status.code()),
            Session::Pty { child, .. } => child
                .try_wait()?
                .map(|status| Some(status.exit_code() as i32)),
        })
    }

    fn kill(&mut self) -> Result<()> {
        match self {
            Session::Piped(child) => child.kill()?,
            Session::Pty { child, .. } => child.kill()?,
        }
        Ok(())
    }
}

/// A running session and the machinery keeping it alive.
struct Spawned {
    session: Session,
    output: Arc<Output>,
    /// Threads copying the session's streams to disk. Joined after the session
    /// ends, so their byte counts are final before the report is written.
    drains: Vec<JoinHandle<Result<()>>>,
}

/// Starts the session and the threads that keep its output moving.
fn spawn(config: &ObserveConfig) -> Result<Spawned> {
    let (program, args) = config
        .command
        .split_first()
        .context("no command given to observe")?;
    let output = Arc::new(Output {
        stdout: Arc::new(AtomicU64::new(0)),
        stderr: Arc::new(AtomicU64::new(0)),
    });

    match config.mode {
        SessionMode::Pipe => {
            let mut child = std::process::Command::new(program)
                .args(args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .with_context(|| format!("spawning {program}"))?;

            let drains = vec![
                drain(
                    child.stdout.take().context("stdout was not piped")?,
                    config.out_dir.join("stdout.log"),
                    Arc::clone(&output.stdout),
                    None,
                ),
                drain(
                    child.stderr.take().context("stderr was not piped")?,
                    config.out_dir.join("stderr.log"),
                    Arc::clone(&output.stderr),
                    None,
                ),
            ];
            Ok(Spawned {
                session: Session::Piped(child),
                output,
                drains,
            })
        }
        SessionMode::Pty => {
            let pair = portable_pty::native_pty_system().openpty(portable_pty::PtySize {
                rows: 50,
                cols: 200,
                pixel_width: 0,
                pixel_height: 0,
            })?;
            // The crate names its two ends after the POSIX pty pair. Windows
            // has no such pair — a pseudoconsole and a set of pipes — so the
            // vocabulary is translated once, here, and nowhere else.
            let portable_pty::PtyPair {
                master: terminal_end,
                slave: session_end,
            } = pair;

            let mut builder = portable_pty::CommandBuilder::new(program);
            builder.args(args);
            // `CommandBuilder` inherits the environment but leaves the working
            // directory unset, which lands the child in the user's profile
            // instead. Pipe mode inherits it, and the two modes have to differ
            // by the pseudoconsole and nothing else.
            builder.cwd(std::env::current_dir()?);
            let child = session_end.spawn_command(builder)?;

            let reader = terminal_end.try_clone_reader()?;
            // One stream: a pseudoconsole merges stdout and stderr the way a
            // terminal does, which is part of what the two modes differ by.
            let drains = vec![drain(
                reader,
                config.out_dir.join("output.log"),
                Arc::clone(&output.stdout),
                Some(terminal_end.take_writer()?),
            )];
            Ok(Spawned {
                session: Session::Pty {
                    child,
                    _terminal_end: terminal_end,
                    _session_end: session_end,
                },
                output,
                drains,
            })
        }
    }
}

/// Copies a session stream to disk, counting bytes.
///
/// `answer_queries` is set only for a pseudoconsole, where the drain doubles as
/// the smallest possible terminal: it answers the cursor-position query so the
/// session is not left waiting on a reply that a headless run would otherwise
/// never send.
fn drain(
    mut source: impl Read + Send + 'static,
    path: PathBuf,
    counter: Arc<AtomicU64>,
    answer_queries: Option<Box<dyn Write + Send>>,
) -> JoinHandle<Result<()>> {
    std::thread::spawn(move || {
        let mut sink = BufWriter::new(File::create(&path)?);
        let mut responder = answer_queries;
        // Holds the bytes a query could have been split across. One shorter
        // than the sequence, so it can never hold a whole match and cause a
        // reply to be sent twice.
        let mut carry: Vec<u8> = Vec::new();
        let mut buffer = [0u8; 64 * 1024];

        loop {
            match source.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    counter.fetch_add(n as u64, Ordering::Relaxed);
                    sink.write_all(&buffer[..n])?;

                    if let Some(writer) = responder.as_mut() {
                        carry.extend_from_slice(&buffer[..n]);
                        let queries = carry
                            .windows(DEVICE_STATUS_REQUEST.len())
                            .filter(|window| *window == DEVICE_STATUS_REQUEST)
                            .count();
                        for _ in 0..queries {
                            writer.write_all(CURSOR_POSITION_REPORT)?;
                        }
                        if queries > 0 {
                            writer.flush()?;
                        }
                        let keep = DEVICE_STATUS_REQUEST.len() - 1;
                        if carry.len() > keep {
                            carry.drain(..carry.len() - keep);
                        }
                    }
                }
                // A closed pseudoconsole surfaces as an error rather than EOF.
                Err(_) => break,
            }
        }
        sink.flush()?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_render_in_binary_units() {
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(33_707_749_376), "31.4 GiB");
    }

    #[test]
    fn a_session_with_no_samples_summarizes_to_zero_rather_than_panicking() {
        let summary = summarize(&[], 0, Duration::from_secs(1), Duration::from_secs(1));
        assert_eq!(summary.peak_rss_bytes, 0);
        assert_eq!(summary.steady_rss_bytes, 0);
        assert!(summary.defender.is_none());
    }
}
