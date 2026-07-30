// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Measurement library behind the `sessionbench` binary.
//!
//! Split from the CLI so backends stay testable without going through argument
//! parsing or report rendering. See `README.md` for the metric this produces.

pub mod axes;
pub mod format;
pub mod host;
pub mod machine;
pub mod observe;
pub mod provenance;
pub mod redline;
pub mod sampler;
pub mod session;
pub mod tree;

/// Label and value pairs a report renders as an aligned block.
///
/// Every module that appears in a report returns these rather than printing,
/// so the CLI owns layout and the library owns facts.
pub type Rows = Vec<(&'static str, String)>;
