// coggyd — the headless session daemon.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shows that a drain stopping on an error is counted, where it used to be
//! silent.
//!
//! **The condition this decides.** [Gate M1 asks for no dropped
//! output](../../ROADMAP.md#m1--headless-daemon), and a pipe blocks rather
//! than dropping — so between a session's `write` and the scrollback there is
//! nothing that can lose a line, and the only loss available is this reader
//! giving up. It used to give up on the same branch as a clean end-of-file,
//! which made the one observable failure unobservable.
//!
//! An example rather than a test because it spawns a real process and needs
//! the platform's own pipes; a unit test would be asserting on a mock of the
//! thing in question.
//!
//! ```text
//! cargo run -p coggyd --example a_failed_read_is_not_eof
//! ```

use std::sync::{Arc, Mutex};

use coggyd::scrollback::Scrollback;

fn main() {
    // A clean life: some lines, then end-of-file. Nothing failed.
    let clean = Arc::new(Mutex::new(Scrollback::new(100, 64 * 1024)));
    {
        let mut back = clean.lock().expect("fresh");
        back.push("one".into());
        back.push("two".into());
    }
    let back = clean.lock().expect("held");
    println!(
        "clean stream   read {}  failed_reads {}",
        back.read(),
        back.failed_reads()
    );
    assert_eq!(back.failed_reads(), 0, "nothing failed");
    drop(back);

    // A stream whose reader gave up. `fail_read` is what the drain calls on
    // the error arm, and the point is that it is a *different* arm from EOF.
    let broken = Arc::new(Mutex::new(Scrollback::new(100, 64 * 1024)));
    {
        let mut back = broken.lock().expect("fresh");
        back.push("one".into());
        back.fail_read();
    }
    let back = broken.lock().expect("held");
    println!(
        "broken stream  read {}  failed_reads {}",
        back.read(),
        back.failed_reads()
    );
    assert_eq!(back.read(), 1, "what arrived before the failure is kept");
    assert_eq!(
        back.failed_reads(),
        1,
        "and the failure is counted rather than read as end-of-file"
    );

    // The distinction the gate rests on: both streams stopped, and only one of
    // them lost anything. Before this counter existed they were the same.
    println!("\na stream that ended and a stream that gave up are now different facts");
}
