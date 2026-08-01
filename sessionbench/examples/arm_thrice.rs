// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Arms the session tree three times in one process, as a bracketed hold does.
//!
//! `hold` arms inside itself, and `--with-solo` calls it three times. Windows
//! may refuse a second job to a process already in one, and the fallback is
//! silent by design — [`ArmedTree::arm`] returns a parent-walk tree with a
//! reason attached rather than failing. If that happens here, the solo halves
//! of a ratio and its concurrent middle were attributed by different methods.
//!
//! ```text
//! cargo run -p sessionbench --example arm_thrice
//! ```

use sessionbench::tree::ArmedTree;
use sysinfo::Pid;

fn main() {
    for pass in 1..=3 {
        let armed = ArmedTree::arm(Pid::from_u32(std::process::id()));
        println!(
            "pass {pass}: {:?}{}",
            armed.membership(),
            armed
                .fallback_reason
                .as_deref()
                .map(|r| format!(" — {r}"))
                .unwrap_or_default(),
        );
    }
}
