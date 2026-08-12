// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rendering shared by every report.

/// Bytes in binary units.
///
/// Binary everywhere, and this is the one place that decides it: the RSS
/// condition is a fraction of physical memory, so mixing GB into that
/// arithmetic moves a verdict by 7%. A machine sold as 32GB has about 31 GiB
/// Renders an absent measurement as a dash, never as a zero.
///
/// **The convention was an idiom repeated sixteen times and named nowhere**, so
/// nothing could call it — which is why the test for it restated the rendering
/// as its own closure and would have stayed green through any change to what
/// ships. A test cannot call something that was never named.
///
/// The distinction is the one this repository keeps paying for: a zero is a
/// measurement that came back zero, and a dash is a measurement that was not
/// taken. Rendering the second as the first is how an absence becomes a passing
/// value.
pub fn or_dash<T>(value: Option<T>, show: impl FnOnce(T) -> String) -> String {
    value.map_or_else(|| "—".to_string(), show)
}

/// to give, and the report says GiB because that is the number the condition
/// is computed from.
pub fn human_bytes(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}
