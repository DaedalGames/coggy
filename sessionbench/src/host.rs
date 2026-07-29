// sessionbench — the concurrent session scaling benchmark.
// Copyright (C) 2026 Daedal Games
// SPDX-License-Identifier: GPL-3.0-or-later

//! Host facts that only a shell can answer: Defender configuration, and
//! whether this process could change it.
//!
//! Defender state belongs in every report because the same workload measured
//! with and without exclusions is the exclusion-delta axis, and because a
//! reader cannot reproduce a redline without knowing what was scanning the disk
//! while it was taken.
//!
//! One `powershell.exe` invocation answers all of it. Reaching for the API
//! directly would mean `unsafe`, which the crate forbids, and the query runs
//! once per report rather than once per sample.

use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

/// Emits one `key<TAB>value` line per fact.
///
/// Line pairs rather than `ConvertTo-Json`: PowerShell collapses a
/// single-element array to a scalar on serialisation, so a machine with exactly
/// one exclusion would deserialise differently from one with two. Tabs are safe
/// because no Windows path can contain one.
///
/// The `if ($item)` guards are load-bearing. An unset exclusion list is `$null`,
/// and `@($null)` is a one-element array holding nothing — so counting the
/// wrapped array reports one exclusion on a machine that has none.
const QUERY: &str = r#"
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal $identity
"elevated`t$($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))"
try {
  $status = Get-MpComputerStatus -ErrorAction Stop
  "present`ttrue"
  "realtime`t$($status.RealTimeProtectionEnabled)"
  "antivirus`t$($status.AntivirusEnabled)"
  "engine`t$($status.AMEngineVersion)"
} catch { "error`tGet-MpComputerStatus: $($_.Exception.Message)" }
try {
  $prefs = Get-MpPreference -ErrorAction Stop
  foreach ($item in @($prefs.ExclusionPath))    { if ($item) { "exclusion_path`t$item" } }
  foreach ($item in @($prefs.ExclusionProcess)) { if ($item) { "exclusion_process`t$item" } }
} catch { "error`tGet-MpPreference: $($_.Exception.Message)" }
"#;

/// What the host says about itself at the moment a report is taken.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostFacts {
    /// Whether the process can modify Defender exclusions. `None` when the
    /// query itself failed.
    pub elevated: Option<bool>,
    pub defender: DefenderFacts,
    /// Anything the query could not answer, kept rather than discarded so a
    /// partial result never reads as a complete one.
    pub errors: Vec<String>,
}

/// Defender configuration as it stood during the run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefenderFacts {
    /// `None` means the query failed, which is a different state from a machine
    /// that genuinely has no Defender.
    pub present: Option<bool>,
    pub realtime_protection: Option<bool>,
    pub antivirus_enabled: Option<bool>,
    pub engine_version: Option<String>,
    pub exclusion_paths: Vec<String>,
    pub exclusion_processes: Vec<String>,
}

impl HostFacts {
    /// Runs the query. Never fails: an unanswerable question is recorded in
    /// [`HostFacts::errors`] and leaves its field `None`.
    pub fn query() -> Self {
        let mut facts = Self::default();

        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", QUERY])
            .stdin(Stdio::null())
            .output();

        let output = match output {
            Ok(output) => output,
            Err(err) => {
                facts.errors.push(format!("powershell.exe: {err}"));
                return facts;
            }
        };

        // Kept even on success. PowerShell writes non-terminating errors here
        // while still exiting zero, and a check that hides its own failures is
        // worse than no check.
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        if !stderr.is_empty() {
            facts.errors.push(stderr.to_string());
        }

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let Some((key, value)) = line.trim_end_matches('\r').split_once('\t') else {
                continue;
            };
            match key {
                "elevated" => facts.elevated = parse_bool(value),
                "present" => facts.defender.present = parse_bool(value),
                "realtime" => facts.defender.realtime_protection = parse_bool(value),
                "antivirus" => facts.defender.antivirus_enabled = parse_bool(value),
                "engine" => facts.defender.engine_version = Some(value.to_string()),
                "exclusion_path" => facts.defender.exclusion_paths.push(value.to_string()),
                "exclusion_process" => facts.defender.exclusion_processes.push(value.to_string()),
                "error" => facts.errors.push(value.to_string()),
                _ => {}
            }
        }

        facts
    }
}

/// PowerShell renders booleans as `True` / `False`.
fn parse_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "True" | "true" => Some(true),
        "False" | "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powershell_booleans_parse_and_anything_else_stays_unknown() {
        assert_eq!(parse_bool("True"), Some(true));
        assert_eq!(parse_bool("False"), Some(false));
        // An empty property renders as an empty string rather than an error, so
        // it has to land on "unknown" instead of defaulting to false.
        assert_eq!(parse_bool(""), None);
    }
}
