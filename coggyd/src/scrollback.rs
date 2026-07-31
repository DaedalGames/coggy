// coggyd — the session supervisor.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a session said, under a ceiling that does not move.
//!
//! **Two different things are called dropped output and the gate means one of
//! them.** [Condition 3](../../sessionbench/README.md#redline) fails when the
//! daemon never read a line — a pipe that filled, a drain that fell behind,
//! and [the workload contract's
//! ordinals](../../workloads/README.md#the-contract) show it as a gap below
//! the highest number seen. A scrollback that evicts its oldest line has not
//! failed that: the line was read, counted, and then aged out by policy.
//!
//! So this counts both, separately and always. `read` is what came out of the
//! session; `evicted` is what this buffer chose not to keep. A reader who
//! cannot tell them apart would see a ring buffer as a gate violation, which
//! it is not, or miss a real one behind an eviction figure, which it would be.
//!
//! Not shared with `sessionbench`, which drains pipes too. The instrument and
//! the subject keeping separate implementations is what stops the benchmark
//! measuring its own reader — [the integrity rule that
//! matters](../../sessionbench/README.md#keeping-it-honest) is about exactly
//! this kind of shortcut.

use std::collections::VecDeque;

/// Most bytes one line may keep.
///
/// A line-count ceiling is not a memory ceiling on its own: `read_until`
/// grows until it meets a newline, so one session emitting a gigabyte
/// without one takes the daemon down while every count still looks
/// healthy. Sixty-four kilobytes is far past any line a session means to
/// write and far below what a hundred of them can afford to hold.
pub const MAX_LINE_BYTES: usize = 64 * 1024;

/// A bounded record of a session's most recent output.
#[derive(Debug)]
pub struct Scrollback {
    lines: VecDeque<String>,
    capacity: usize,
    read: u64,
    evicted: u64,
    truncated: u64,
}

impl Scrollback {
    /// A scrollback holding at most `capacity` lines.
    ///
    /// Lines rather than bytes, because the unit a reader asks for is a line
    /// and because [a unit of work is a
    /// line](../../workloads/README.md#the-contract). A zero capacity is
    /// allowed and means keep nothing while still counting everything, which
    /// is what a session nobody will read wants.
    pub fn new(capacity: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            capacity,
            read: 0,
            evicted: 0,
            truncated: 0,
        }
    }

    /// Records one line the session emitted, cut to [`MAX_LINE_BYTES`].
    pub fn push(&mut self, mut line: String) {
        self.read += 1;
        if line.len() > MAX_LINE_BYTES {
            // Cut on a character boundary, since a String may not be
            // split mid-codepoint and the byte at the ceiling usually is not
            // one.
            let mut cut = MAX_LINE_BYTES;
            while cut > 0 && !line.is_char_boundary(cut) {
                cut -= 1;
            }
            line.truncate(cut);
            self.truncated += 1;
        }
        if self.capacity == 0 {
            self.evicted += 1;
            return;
        }
        if self.lines.len() == self.capacity {
            self.lines.pop_front();
            self.evicted += 1;
        }
        self.lines.push_back(line);
    }

    /// Lines the session emitted, whether or not they were kept.
    ///
    /// This is the figure condition 3 is about.
    pub fn read(&self) -> u64 {
        self.read
    }

    /// Lines this buffer aged out to stay under its ceiling.
    ///
    /// **Not a gate failure.** These were read and counted; the ceiling is a
    /// memory decision, and a hundred sessions each holding unbounded history
    /// is the thing it exists to prevent.
    pub fn evicted(&self) -> u64 {
        self.evicted
    }

    /// Lines that arrived longer than the per-line ceiling and were cut.
    ///
    /// **A third thing, and neither of the two above.** The line was read,
    /// so the gate is satisfied, and it was kept, so eviction did not touch
    /// it — but part of it is gone. Counting it under either of the others
    /// would hide a session emitting megabytes without a newline, which is
    /// the shape that makes a line-count ceiling stop being a memory ceiling.
    pub fn truncated(&self) -> u64 {
        self.truncated
    }

    pub fn retained(&self) -> usize {
        self.lines.len()
    }

    /// The most recent `n` lines, oldest first.
    pub fn tail(&self, n: usize) -> Vec<&str> {
        self.lines
            .iter()
            .skip(self.lines.len().saturating_sub(n))
            .map(String::as_str)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_the_ceiling_nothing_is_evicted() {
        let mut s = Scrollback::new(4);
        for i in 0..3 {
            s.push(format!("{i}"));
        }
        assert_eq!((s.read(), s.evicted(), s.retained()), (3, 0, 3));
    }

    #[test]
    fn past_the_ceiling_the_oldest_goes_and_the_count_still_climbs() {
        let mut s = Scrollback::new(2);
        for i in 0..5 {
            s.push(format!("{i}"));
        }
        // Read is what the gate reads; retained is what memory allows.
        assert_eq!(s.read(), 5);
        assert_eq!(s.retained(), 2);
        assert_eq!(s.evicted(), 3);
        assert_eq!(s.tail(9), vec!["3", "4"]);
    }

    #[test]
    fn a_zero_ceiling_keeps_nothing_and_still_counts_everything() {
        // A session nobody will read still has to be drained, or its pipe
        // fills and the session blocks — which is a real condition-3 failure.
        let mut s = Scrollback::new(0);
        for i in 0..7 {
            s.push(format!("{i}"));
        }
        assert_eq!((s.read(), s.retained()), (7, 0));
        assert_eq!(s.evicted(), 7);
        assert!(s.tail(3).is_empty());
    }

    #[test]
    fn a_tail_longer_than_the_buffer_is_the_whole_buffer() {
        let mut s = Scrollback::new(3);
        s.push("only".into());
        assert_eq!(s.tail(100), vec!["only"]);
    }
}
