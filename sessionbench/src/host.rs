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
try {
  $battery = Get-CimInstance Win32_Battery -ErrorAction Stop | Select-Object -First 1
  if ($battery) { "on_battery`t$($battery.BatteryStatus -ne 2)"; "charge`t$($battery.EstimatedChargeRemaining)" }
  else { "on_battery`tfalse" }
} catch { "error`tWin32_Battery: $($_.Exception.Message)" }
try {
  $plan = Get-CimInstance -Namespace root\cimv2\power -ClassName Win32_PowerPlan -Filter "IsActive=true" -ErrorAction Stop
  "power_plan`t$($plan.ElementName)"
} catch { "error`tWin32_PowerPlan: $($_.Exception.Message)" }
try {
  $zone = Get-CimInstance -Namespace root/WMI -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction Stop | Select-Object -First 1
  if ($zone) { "thermal_c`t$([Math]::Round($zone.CurrentTemperature / 10 - 273.15, 1))" }
} catch { "error`tMSAcpi_ThermalZoneTemperature: $($_.Exception.Message)" }
try {
  $perf = (Get-Counter '\Processor Information(_Total)\% Processor Performance' -ErrorAction Stop).CounterSamples[0].CookedValue
  "processor_performance`t$([Math]::Round($perf, 1))"
} catch { "error`t% Processor Performance: $($_.Exception.Message)" }
"#;

/// What the host says about itself at the moment a report is taken.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostFacts {
    /// Whether the process can modify Defender exclusions. `None` when the
    /// query itself failed.
    pub elevated: Option<bool>,
    pub defender: DefenderFacts,
    /// Whether the machine was on battery, and which power plan was active.
    ///
    /// **The axis that broke every cross-run comparison in one day, and the
    /// only one this struct did not ask about.** A hundred sessions at duty
    /// 0.27 returned 907 units/s at noon on AC and 135 the same evening on
    /// battery — 7.8×, from a laptop pinned to its base clock with the CPU
    /// budget cut. Temperature was 42 °C, so it was not thermal, and nothing
    /// in the artifact said which machine had produced which number.
    ///
    /// `None` means the query failed, which on a desktop is also how *no
    /// battery exists* arrives — [`HostFacts::errors`] separates them.
    pub on_battery: Option<bool>,
    pub charge_percent: Option<u8>,
    pub power_plan: Option<String>,
    /// The ACPI thermal zone in degrees Celsius, and how fast the cores are
    /// actually clocking as a percentage of their nominal rate.
    ///
    /// **Recorded because the state they might name is worth 72% and nothing
    /// reports it.** This box runs a solo session at 21.5 units/s rested and
    /// 9.4 in a slower state that has now been seen three times, each state
    /// flat across its own samples. A gate bracket ran entirely inside the
    /// slow one with nothing in its artifact to say so.
    ///
    /// **Neither field has been shown to distinguish the two**, and the run
    /// that tried could not: a deliberate saturating burst failed to induce
    /// the slow state, so both of its phases sampled the fast one and agreed
    /// to 0.1%. The state cannot be ordered, so it has to be caught — which
    /// takes these travelling in every artifact until it next arrives.
    ///
    /// `None` where the counter is absent, which is ordinary: `MSAcpi` is not
    /// exposed by every firmware.
    pub thermal_c: Option<f64>,
    pub processor_performance: Option<f64>,
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
                "on_battery" => facts.on_battery = parse_bool(value),
                "charge" => facts.charge_percent = value.parse().ok(),
                "power_plan" => facts.power_plan = Some(value.to_string()),
                "thermal_c" => facts.thermal_c = value.parse().ok(),
                "processor_performance" => facts.processor_performance = value.parse().ok(),
                "error" => facts.errors.push(value.to_string()),
                _ => {}
            }
        }

        facts
    }
}

/// A Defender path exclusion, held only as long as this value is.
///
/// The removal is the reason the type exists. An exclusion is a hole in the
/// machine's real-time protection, and one left behind by a benchmark is a hole
/// nobody asked for and nobody will notice. It is removed on drop as well as on
/// request, verified rather than assumed, and a failure to remove is printed
/// where it cannot be missed — a silent failure here changes the machine's
/// security posture and says nothing.
///
/// Only ever point this at a directory the benchmark created for one run.
pub struct HeldExclusion {
    path: String,
    removed: bool,
}

impl HeldExclusion {
    /// Adds the exclusion and confirms Defender took it.
    ///
    /// Confirmed rather than trusted: `Add-MpPreference` can return quietly
    /// without the path appearing, and an exclusion that was never applied
    /// would produce a delta of zero that reads as "exclusions do not help".
    pub fn add(path: &std::path::Path) -> Result<Self, String> {
        let path = path.to_string_lossy().into_owned();
        run_powershell(&format!(
            "Add-MpPreference -ExclusionPath {} -ErrorAction Stop",
            quote(&path)
        ))?;

        if !HostFacts::query()
            .defender
            .exclusion_paths
            .iter()
            .any(|p| p.eq_ignore_ascii_case(&path))
        {
            // Try to undo whatever half-happened before giving up.
            let _ = run_powershell(&format!(
                "Remove-MpPreference -ExclusionPath {} -ErrorAction SilentlyContinue",
                quote(&path)
            ));
            return Err(format!(
                "Defender did not report {path} as excluded after adding it"
            ));
        }
        Ok(Self {
            path,
            removed: false,
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    /// Removes the exclusion and confirms it is gone.
    pub fn remove(&mut self) -> Result<(), String> {
        if self.removed {
            return Ok(());
        }
        run_powershell(&format!(
            "Remove-MpPreference -ExclusionPath {} -ErrorAction Stop",
            quote(&self.path)
        ))?;

        if HostFacts::query()
            .defender
            .exclusion_paths
            .iter()
            .any(|p| p.eq_ignore_ascii_case(&self.path))
        {
            return Err(format!("{} is still excluded after removing it", self.path));
        }
        self.removed = true;
        Ok(())
    }
}

impl Drop for HeldExclusion {
    fn drop(&mut self) {
        if let Err(error) = self.remove() {
            eprintln!(
                "\nWARNING: a Defender exclusion was left in place and must be removed by hand:\n  {}\n  {error}\n",
                self.path
            );
        }
    }
}

/// Single-quotes a value for PowerShell, doubling any quote inside it.
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Runs one PowerShell statement, returning its stderr as the error.
fn run_powershell(statement: &str) -> Result<(), String> {
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", statement])
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("powershell.exe: {error}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if output.status.success() && stderr.is_empty() {
        return Ok(());
    }
    Err(if stderr.is_empty() {
        format!("exited with {}", output.status)
    } else {
        stderr.to_string()
    })
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
