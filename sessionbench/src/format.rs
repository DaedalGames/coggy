// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rendering shared by every report.

/// Bytes in binary units.
///
/// Binary everywhere, and this is the one place that decides it: the RSS
/// condition is a fraction of physical memory, so mixing GB into that
/// arithmetic moves a verdict by 7%. A machine sold as 32GB has about 31 GiB
/// to give, and the report says GiB because that is the number the condition
/// is computed from.
pub fn human_bytes(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}
