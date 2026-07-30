// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Captures the parts of the provenance block that are only true at build time.
//!
//! Two facts belong here and nowhere else: the compiler that built this binary
//! is not necessarily the one `rustc -V` would find at run time, and the
//! resolved dependency versions are decided when the binary is linked.
//!
//! `vergen` emits both, along with the git facts we deliberately do not take
//! from it — see `provenance.rs` for why those are read at run time instead.

use vergen::{CargoBuilder, Emitter, RustcBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Emitter::default()
        .add_instructions(
            &RustcBuilder::default()
                .semver(true)
                .commit_hash(true)
                .commit_date(true)
                .build()?,
        )?
        .add_instructions(&CargoBuilder::default().dependencies(true).build()?)?
        .emit()?;
    Ok(())
}
