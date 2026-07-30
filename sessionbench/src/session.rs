// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Starting a session and keeping its output moving.
//!
//! Shared by the single-session observation and the ramp, because the two have
//! to start sessions identically or their numbers cannot be compared. The ramp
//! measures against a solo baseline taken by `observe`, and a difference in how
//! the session was launched would land in that comparison as if it were
//! contention.

use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::JoinHandle;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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

/// Counts what a session produced.
///
/// Output has to be drained whatever else happens: a session whose pipe fills
/// blocks on write, and a blocked session measures the observer rather than the
/// workload.
pub struct Output {
    stdout: Arc<AtomicU64>,
    stderr: Arc<AtomicU64>,
    /// Lines on stdout, which is how a workload says it finished something.
    ///
    /// Only stdout: a session that fails noisily would otherwise report a
    /// rising work rate on the way down.
    units: Arc<AtomicU64>,
}

impl Output {
    pub fn new() -> Self {
        Self {
            stdout: Arc::new(AtomicU64::new(0)),
            stderr: Arc::new(AtomicU64::new(0)),
            units: Arc::new(AtomicU64::new(0)),
        }
    }

    /// A frozen counter, for summing a pool of sessions into one view.
    ///
    /// Nothing writes to it: a ramp's sessions each own a live `Output`, and
    /// the sampler wants their total rather than a list.
    pub fn from_counts(bytes: u64, units: u64) -> Self {
        Self {
            stdout: Arc::new(AtomicU64::new(bytes)),
            stderr: Arc::new(AtomicU64::new(0)),
            units: Arc::new(AtomicU64::new(units)),
        }
    }

    pub fn total(&self) -> u64 {
        self.stdout.load(Ordering::Relaxed) + self.stderr.load(Ordering::Relaxed)
    }

    pub fn units(&self) -> u64 {
        self.units.load(Ordering::Relaxed)
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}

/// A running session and the machinery keeping it alive.
pub struct Spawned {
    pub session: Session,
    pub output: Arc<Output>,
    /// Threads copying the session's streams to disk. Joined after the session
    /// ends, so their byte counts are final before anything is reported.
    pub drains: Vec<JoinHandle<Result<()>>>,
}

impl Spawned {
    /// Waits for the drains after the session has been dropped.
    ///
    /// Dropping the session is what closes the pipes and lets the drains see
    /// end-of-file, so this cannot be called while it is still held.
    pub fn finish(drains: Vec<JoinHandle<Result<()>>>) -> Result<()> {
        for drain in drains {
            drain
                .join()
                .map_err(|_| anyhow::anyhow!("drain panicked"))??;
        }
        Ok(())
    }
}

pub enum Session {
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
    pub fn pid(&self) -> Option<u32> {
        match self {
            Session::Piped(child) => Some(child.id()),
            Session::Pty { child, .. } => child.process_id(),
        }
    }

    /// `Ok(None)` while still running; `Ok(Some(code))` once it has exited,
    /// where the inner `None` means it was terminated without a code.
    #[allow(clippy::option_option)]
    pub fn try_wait(&mut self) -> Result<Option<Option<i32>>> {
        Ok(match self {
            Session::Piped(child) => child.try_wait()?.map(|status| status.code()),
            Session::Pty { child, .. } => child
                .try_wait()?
                .map(|status| Some(status.exit_code() as i32)),
        })
    }

    pub fn kill(&mut self) -> Result<()> {
        match self {
            Session::Piped(child) => child.kill()?,
            Session::Pty { child, .. } => child.kill()?,
        }
        Ok(())
    }
}

/// The directory a session may write under.
///
/// Every rung of a ramp ends by killing its sessions, and `TerminateProcess`
/// runs no destructor, so a workload can never be relied on to clean up after
/// itself. Naming its scratch directory here is what lets the benchmark do it
/// instead — one ramp left 299 directories behind before this existed, and
/// that was a working ramp rather than a broken one.
pub const SCRATCH_VAR: &str = "SESSIONBENCH_SCRATCH";

/// Starts a session and the threads that keep its output moving.
///
/// `log_base` is a path prefix rather than a directory, so many sessions can
/// share one and still be told apart afterwards. `scratch` is where the
/// workload may write, and the caller owns removing it.
pub fn spawn(
    command: &[String],
    mode: SessionMode,
    log_base: &Path,
    scratch: &Path,
) -> Result<Spawned> {
    let (program, args) = command.split_first().context("no command to run")?;
    let output = Arc::new(Output::new());
    let log = |suffix: &str| -> PathBuf {
        let mut path = log_base.as_os_str().to_owned();
        path.push(suffix);
        PathBuf::from(path)
    };

    match mode {
        SessionMode::Pipe => {
            let mut child = std::process::Command::new(program)
                .args(args)
                .env(SCRATCH_VAR, scratch)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .with_context(|| format!("spawning {program}"))?;

            let drains = vec![
                drain(
                    child.stdout.take().context("stdout was not piped")?,
                    log("-stdout.log"),
                    Arc::clone(&output.stdout),
                    Some(Arc::clone(&output.units)),
                    None,
                ),
                drain(
                    child.stderr.take().context("stderr was not piped")?,
                    log("-stderr.log"),
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
            builder.env(SCRATCH_VAR, scratch);
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
                log("-output.log"),
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

/// Copies a session stream to disk, counting bytes and units.
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
