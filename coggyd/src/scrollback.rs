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
    byte_capacity: usize,
    bytes: usize,
    read: u64,
    read_bytes: u64,
    evicted: u64,
    truncated: u64,
    failed_reads: u64,
}

impl Scrollback {
    /// A scrollback holding at most `capacity` lines and `byte_capacity` bytes
    /// of them.
    ///
    /// **Both, because either alone leaves the other unbounded.** A line count
    /// is what every grid terminal uses and it is right for them: their lines
    /// are as wide as the terminal, so counting lines counts bytes. On pipes
    /// there is no width, and the count stops bounding anything — the shape
    /// [tmux hit at about 48 GB](https://github.com/tmux/tmux/issues/4859).
    /// So bytes bound the content, and the line count keeps its job of
    /// bounding the fixed per-line cost, which is real and which a byte
    /// budget alone would let a flood of empty lines run up.
    ///
    /// A zero line capacity is allowed and means keep nothing while still
    /// counting everything, which is what a session nobody will read wants.
    pub fn new(capacity: usize, byte_capacity: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            capacity,
            byte_capacity,
            bytes: 0,
            read: 0,
            read_bytes: 0,
            evicted: 0,
            truncated: 0,
            failed_reads: 0,
        }
    }

    /// Records one line the session emitted, cut to [`MAX_LINE_BYTES`].
    pub fn push(&mut self, mut line: String) {
        self.read += 1;
        // Counted as it arrives, before the ceiling below cuts anything. This
        // is the output path's volume — what a session actually sent and this
        // daemon actually took — rather than what a policy chose to keep.
        self.read_bytes += line.len() as u64;
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
        // A line that cannot fit an empty buffer is dropped rather than
        // allowed to push the budget over, which draining the whole buffer
        // for it would still do.
        if self.capacity == 0 || line.len() > self.byte_capacity {
            self.evicted += 1;
            return;
        }
        while self.lines.len() >= self.capacity || self.bytes + line.len() > self.byte_capacity {
            let Some(gone) = self.lines.pop_front() else {
                break;
            };
            self.bytes -= gone.len();
            self.evicted += 1;
        }
        self.bytes += line.len();
        self.lines.push_back(line);
    }

    /// Bytes of session output currently held.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    /// Bytes the session sent and this daemon took, whether or not kept.
    ///
    /// **The output-path axis, which had no number under a daemon.** A
    /// benchmark holding the reading end of a session's pipe counts these
    /// itself; one measuring through a daemon cannot, and the only figures
    /// available to it were the daemon's own few hundred bytes of reporting
    /// or a zero. Both would describe a hundred sessions as having produced
    /// almost nothing.
    ///
    /// A line cut at [`MAX_LINE_BYTES`] contributes what survived rather than
    /// what was sent, since the excess is consumed by the reader and never
    /// reaches here. `truncated` says how often that happened.
    pub fn read_bytes(&self) -> u64 {
        self.read_bytes
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

    /// Records that a drain stopped on an error rather than end-of-file.
    ///
    /// Called at most once a stream, since the drain returns after it.
    pub fn fail_read(&mut self) {
        self.failed_reads += 1;
    }

    /// Streams whose drain gave up on an error instead of reaching EOF.
    ///
    /// **This is the gate's dropped output, and the only thing that can be.**
    /// The other three counters are policy — read and aged out, read and cut —
    /// while this one means the daemon stopped asking and whatever the session
    /// wrote afterwards is gone with no record of how much. A pipe does not
    /// lose data, it blocks; so between a session's `write` and this buffer
    /// there is nothing to drop, and the only loss available is the reader
    /// giving up. Non-zero is the condition failing, and it used to be
    /// unobservable because the error shared a branch with EOF.
    pub fn failed_reads(&self) -> u64 {
        self.failed_reads
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

    /// Wide enough that the line count is what bites, for the tests about it.
    const ROOMY: usize = 64 * 1024;

    #[test]
    fn under_the_ceiling_nothing_is_evicted() {
        let mut s = Scrollback::new(4, ROOMY);
        for i in 0..3 {
            s.push(format!("{i}"));
        }
        assert_eq!((s.read(), s.evicted(), s.retained()), (3, 0, 3));
    }

    #[test]
    fn the_byte_budget_bites_before_the_line_count_does() {
        // The whole point: a line count set for short lines does not bound
        // long ones. Ten lines allowed, but only twenty bytes of them.
        let mut s = Scrollback::new(10, 20);
        for _ in 0..4 {
            s.push("0123456789".into());
        }
        assert_eq!(s.read(), 4);
        assert_eq!(s.retained(), 2, "twenty bytes holds two ten-byte lines");
        assert_eq!(s.bytes(), 20);
        assert_eq!(s.evicted(), 2, "and the line count never came near ten");
    }

    #[test]
    fn a_line_too_big_for_the_whole_budget_is_dropped_rather_than_emptying_it() {
        // Draining the buffer would not make room, so the alternative to
        // dropping is exceeding the budget the buffer exists to hold.
        let mut s = Scrollback::new(10, 8);
        s.push("keep".into());
        s.push("far too long for this".into());
        assert_eq!(s.read(), 2);
        assert_eq!(s.evicted(), 1);
        assert_eq!(s.tail(9), vec!["keep"], "the survivor was not sacrificed");
        assert!(s.bytes() <= 8);
    }

    #[test]
    fn past_the_ceiling_the_oldest_goes_and_the_count_still_climbs() {
        let mut s = Scrollback::new(2, ROOMY);
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
        let mut s = Scrollback::new(0, ROOMY);
        for i in 0..7 {
            s.push(format!("{i}"));
        }
        assert_eq!((s.read(), s.retained()), (7, 0));
        assert_eq!(s.evicted(), 7);
        assert!(s.tail(3).is_empty());
    }

    #[test]
    fn a_tail_longer_than_the_buffer_is_the_whole_buffer() {
        let mut s = Scrollback::new(3, ROOMY);
        s.push("only".into());
        assert_eq!(s.tail(100), vec!["only"]);
    }
}
