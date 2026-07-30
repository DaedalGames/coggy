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
    /// Lines the session has written, which is its own count of work done.
    pub work_units: u64,
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
    /// Units the workload reported finishing, one per line of stdout.
    pub work_units: u64,
    /// The rate the work-rate condition is measured against.
    ///
    /// Comparable only against the same workload, since a unit means whatever
    /// that workload says it means.
    pub work_units_per_sec: f64,
    /// Cores the session occupied in steady state, where 1.0 is one full core.
    ///
    /// Recorded beside Defender's because `WorkRate` names a symptom: a redline
    /// limited by work rate has to say where the cores went, or the pair
    /// collapses into a number with a label on it. `None` when the run never
    /// outlasted [`STARTUP_WINDOW`].
    pub session_cores: Option<f64>,
    /// Cores Defender occupied over the same window.
    pub defender_cores: Option<f64>,
    pub defender: Option<DefenderCost>,
}

/// CPU over the window after startup, in cores.
///
/// One window for every steady figure. Averaging Defender over a shorter one
/// swung a hundred-session projection by a factor of three and a half between
/// otherwise identical runs, because its CPU arrives in bursts and a short mean
/// lands wherever the bursts fell.
#[derive(Debug, Clone, Copy)]
struct SteadyCpu {
    session: f64,
    defender: f64,
}

fn steady_cpu(samples: &[Sample]) -> Option<SteadyCpu> {
    let after: Vec<&Sample> = samples
        .iter()
        .filter(|s| Duration::from_millis(s.t_ms) >= STARTUP_WINDOW)
        .collect();
    if after.is_empty() {
        return None;
    }
    Some(SteadyCpu {
        session: mean(after.iter().map(|s| f64::from(s.cpu_percent) / 100.0)),
        defender: mean(
            after
                .iter()
                .map(|s| f64::from(s.defender_cpu_percent.unwrap_or(0.0)) / 100.0),
        ),
    })
}

/// Defender's cost, split the way the milestone asks for it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenderCost {
    pub startup_cpu_seconds: f64,
    /// The rate that decides whether Defender is a fixed toll or a running one.
    ///
    /// Derived from the same steady window as everything else, rather than
    /// integrated separately — one quantity computed twice is one quantity that
    /// can disagree with itself. `None` when the run never outlasted
    /// [`STARTUP_WINDOW`], which is distinct from zero: a short run has no
    /// steady state, while a zero would claim Defender stops charging.
    pub steady_cpu_seconds_per_min: Option<f64>,
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
    /// Cores a hundred of these would ask for, including Defender's share.
    ///
    /// `None` when the run had no steady state to project from.
    pub cores_needed: Option<f64>,
    pub cores_available: usize,
    /// Whether the machine is asked for more cores than it has.
    ///
    /// Not a fifth condition. It is the mechanism by which the work-rate
    /// condition trips, recorded so the redline can name a cause instead of
    /// only a symptom.
    pub cpu_oversubscribed: Option<bool>,
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
            work_units: output.units(),
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
    let summary = summarize(&samples, &output, duration);
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

fn summarize(samples: &[Sample], output: &Output, duration: Duration) -> Summary {
    let seconds = duration.as_secs_f64().max(f64::EPSILON);
    let output_bytes = output.total();
    let work_units = output.units();

    // Memory settles within seconds, so its steady figure reads off the final
    // quarter. CPU needs the wider post-startup window instead, for the reason
    // on SteadyCpu.
    let mut steady_rss: Vec<u64> = samples[samples.len() * 3 / 4..]
        .iter()
        .map(|s| s.rss_bytes)
        .collect();
    steady_rss.sort_unstable();
    let cpu = steady_cpu(samples);

    Summary {
        peak_rss_bytes: samples.iter().map(|s| s.rss_bytes).max().unwrap_or(0),
        steady_rss_bytes: steady_rss.get(steady_rss.len() / 2).copied().unwrap_or(0),
        peak_processes: samples.iter().map(|s| s.processes).max().unwrap_or(0),
        peak_pseudoconsoles: samples.iter().map(|s| s.pseudoconsoles).max().unwrap_or(0),
        output_bytes,
        output_bytes_per_sec: output_bytes as f64 / seconds,
        work_units,
        work_units_per_sec: work_units as f64 / seconds,
        session_cores: cpu.map(|c| c.session),
        defender_cores: cpu.map(|c| c.defender),
        defender: defender_cost(samples, cpu),
    }
}

/// Zero for an empty run, so a session shorter than one sample reports nothing
/// rather than dividing by it.
fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let (sum, count) = values.fold((0.0, 0usize), |(sum, count), v| (sum + v, count + 1));
    if count == 0 { 0.0 } else { sum / count as f64 }
}

/// Defender's cost: CPU-seconds spent before the session was up, and the rate
/// it kept charging afterwards.
///
/// Returns `None` when Defender never appeared, so a machine without it reads
/// as unmeasured rather than as costing nothing. The steady rate comes from
/// `cpu` rather than being integrated here, so the two can never disagree.
fn defender_cost(samples: &[Sample], cpu: Option<SteadyCpu>) -> Option<DefenderCost> {
    if !samples.iter().any(|s| s.defender_cpu_percent.is_some()) {
        return None;
    }

    // Integrated against the gap each sample actually covers rather than the
    // requested interval, since a loaded machine delivers samples late and the
    // difference lands straight in the total.
    let startup: f64 = samples
        .iter()
        .scan(0u64, |previous, sample| {
            let elapsed = sample.t_ms.saturating_sub(*previous) as f64 / 1000.0;
            *previous = sample.t_ms;
            Some((sample, elapsed))
        })
        .filter(|(sample, _)| Duration::from_millis(sample.t_ms) < STARTUP_WINDOW)
        .map(|(sample, elapsed)| {
            f64::from(sample.defender_cpu_percent.unwrap_or(0.0)) / 100.0 * elapsed
        })
        .sum();

    Some(DefenderCost {
        startup_cpu_seconds: startup,
        steady_cpu_seconds_per_min: cpu.map(|c| c.defender * 60.0),
    })
}

fn project(summary: &Summary, machine: &Machine) -> Projection {
    let sessions = u64::from(PROJECTED_SESSIONS);
    let rss = summary.steady_rss_bytes.saturating_mul(sessions);
    let budget = (machine.total_memory_bytes as f64 * RSS_BUDGET_FRACTION) as u64;
    let cores_available = machine
        .physical_cores
        .unwrap_or(machine.logical_cores)
        .max(1);
    // Defender scales with the sessions rather than sitting flat, since what it
    // charges for is the files they write.
    let cores_needed = summary
        .session_cores
        .zip(summary.defender_cores)
        .map(|(session, defender)| (session + defender) * sessions as f64);

    Projection {
        sessions: PROJECTED_SESSIONS,
        rss_bytes: rss,
        rss_budget_bytes: budget,
        rss_condition_holds: rss <= budget,
        processes: summary.peak_processes * PROJECTED_SESSIONS as usize,
        pseudoconsoles: summary.peak_pseudoconsoles * PROJECTED_SESSIONS as usize,
        output_bytes_per_sec: summary.output_bytes_per_sec * sessions as f64,
        cores_needed,
        cores_available,
        cpu_oversubscribed: cores_needed.map(|needed| needed > cores_available as f64),
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
    /// Lines on stdout, which is how a workload says it finished something.
    ///
    /// Only stdout: a session that fails noisily would otherwise report a
    /// rising work rate on the way down.
    units: Arc<AtomicU64>,
}

impl Output {
    fn total(&self) -> u64 {
        self.stdout.load(Ordering::Relaxed) + self.stderr.load(Ordering::Relaxed)
    }

    fn units(&self) -> u64 {
        self.units.load(Ordering::Relaxed)
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
        units: Arc::new(AtomicU64::new(0)),
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
                    Some(Arc::clone(&output.units)),
                    None,
                ),
                drain(
                    child.stderr.take().context("stderr was not piped")?,
                    config.out_dir.join("stderr.log"),
                    Arc::clone(&output.stderr),
                    None,
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
                Some(Arc::clone(&output.units)),
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
    units: Option<Arc<AtomicU64>>,
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

                    if let Some(units) = units.as_ref() {
                        let lines = buffer[..n].iter().filter(|b| **b == b'\n').count();
                        units.fetch_add(lines as u64, Ordering::Relaxed);
                    }

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
        let output = Output {
            stdout: Arc::new(AtomicU64::new(0)),
            stderr: Arc::new(AtomicU64::new(0)),
            units: Arc::new(AtomicU64::new(0)),
        };
        let summary = summarize(&[], &output, Duration::from_secs(1));
        assert_eq!(summary.peak_rss_bytes, 0);
        assert_eq!(summary.steady_rss_bytes, 0);
        assert!(summary.session_cores.is_none());
        assert!(summary.defender.is_none());
    }

    #[test]
    fn a_run_too_short_for_a_steady_state_says_so_instead_of_reporting_zero() {
        let samples: Vec<Sample> = (0..5)
            .map(|i| Sample {
                t_ms: i * 1000,
                rss_bytes: 0,
                processes: 1,
                pseudoconsoles: 0,
                cpu_percent: 0.0,
                defender_cpu_percent: Some(10.0),
                defender_rss_bytes: Some(0),
                available_memory_bytes: 0,
                output_bytes: 0,
                work_units: 0,
                members: Vec::new(),
            })
            .collect();

        let cost =
            defender_cost(&samples, steady_cpu(&samples)).expect("Defender reported a reading");
        assert!(cost.startup_cpu_seconds > 0.0);
        // Five seconds never reaches the startup window, so there is no steady
        // state to report. Zero here would claim Defender stopped charging.
        assert!(cost.steady_cpu_seconds_per_min.is_none());
    }

    #[test]
    fn the_steady_window_survives_a_run_that_reaches_it() {
        let samples: Vec<Sample> = (0..40)
            .map(|i| Sample {
                t_ms: i * 1000,
                rss_bytes: 0,
                processes: 1,
                pseudoconsoles: 0,
                cpu_percent: 25.0,
                defender_cpu_percent: Some(50.0),
                defender_rss_bytes: Some(0),
                available_memory_bytes: 0,
                output_bytes: 0,
                work_units: 0,
                members: Vec::new(),
            })
            .collect();

        let cpu = steady_cpu(&samples).expect("the run outlasted the startup window");
        assert_eq!(cpu.session, 0.25);
        assert_eq!(cpu.defender, 0.5);
        // The rate the report prints is this same figure in other units, never
        // a second integration that could drift from it.
        let cost = defender_cost(&samples, Some(cpu)).expect("Defender reported a reading");
        assert_eq!(cost.steady_cpu_seconds_per_min, Some(30.0));
    }

    #[test]
    fn a_lone_sample_is_its_own_steady_state() {
        // The final quarter of a one-sample run is that sample, and a slice
        // index of zero has to survive rather than reach past the end.
        let output = Output {
            stdout: Arc::new(AtomicU64::new(20)),
            stderr: Arc::new(AtomicU64::new(0)),
            units: Arc::new(AtomicU64::new(2)),
        };
        let sample = Sample {
            t_ms: 0,
            rss_bytes: 1024,
            processes: 1,
            pseudoconsoles: 0,
            cpu_percent: 50.0,
            defender_cpu_percent: None,
            defender_rss_bytes: None,
            available_memory_bytes: 0,
            output_bytes: 20,
            work_units: 2,
            members: Vec::new(),
        };
        let summary = summarize(
            std::slice::from_ref(&sample),
            &output,
            Duration::from_secs(2),
        );
        assert_eq!(summary.steady_rss_bytes, 1024);
        assert_eq!(summary.work_units_per_sec, 1.0);
        // Memory settles in seconds, so one sample is a usable steady figure.
        // CPU does not, and one sample taken before the startup window is over
        // is not a steady state however the slice is indexed.
        assert!(summary.session_cores.is_none());
    }
}
