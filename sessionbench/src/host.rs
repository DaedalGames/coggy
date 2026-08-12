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
  # NOT Sort-Object InstalledOn. Some rows carry an InstalledOn this machine's
  # locale cannot parse -- on 2026-08-12 that threw "String was not recognized
  # as a valid DateTime" while still returning rows, so the order was a hope
  # rather than a property and "the newest few" meant nothing. Sorting on the
  # KB NUMBER is locale-free and monotonic with release order.
  $kb = Get-HotFix -ErrorAction Stop |
        Sort-Object { $_.HotFixID.Substring(2) -as [int] } -Descending |
        Select-Object -First 6 -ExpandProperty HotFixID
  "updates`t$($kb -join ',')"
} catch { "error`tGet-HotFix: $($_.Exception.Message)" }
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
  $samples = (Get-Counter '\Processor Information(*)\% Processor Performance' -ErrorAction Stop).CounterSamples
  $total = $samples | Where-Object { $_.InstanceName -eq '_total' } | Select-Object -First 1
  if ($total) { "processor_performance`t$([Math]::Round($total.CookedValue, 1))" }
  $cores = @($samples | Where-Object { $_.InstanceName -notmatch '_total' } | Sort-Object InstanceName)
  if ($cores.Count -gt 0) { "processor_performance_cores`t$((($cores | ForEach-Object { [Math]::Round($_.CookedValue, 1) }) -join ','))" }
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
    /// The most recent installed updates, newest first.
    ///
    /// **The second axis to break every cross-day comparison here, and the
    /// build string did not notice it.** On 2026-08-12 this box began parking
    /// cores: six quiet hundred-session holds, tenant absent in all of them,
    /// gave **16.11 and 16.00 machine cores on 3 and 11 August against 4.52 to
    /// 5.37 on the 12th**. Every one of those artifacts records the same
    /// `os_version`, `11 (26200)` — and two updates installed that day, which
    /// nothing in this struct could see.
    ///
    /// So a build string is not a configuration fingerprint: the archive could
    /// not tell two machines apart that reported the same one. This is the same
    /// argument [`HostFacts::on_battery`] carries, written after the same kind
    /// of day.
    ///
    /// Six is a compromise. The whole list would bloat every artifact; the
    /// newest few are what differ between two runs days apart.
    pub updates: Vec<String>,
    /// The ACPI thermal zone in degrees Celsius, and how fast the cores are
    /// actually clocking as a percentage of their nominal rate.
    ///
    /// **Recorded because the state they might name is worth 72% and nothing
    /// reports it.** This box runs a solo session at about **18.9** units/s
    /// rested and at **8.997 over twenty holds spanning 340 minutes**, in a
    /// slower state — each state flat across its own samples, and the slow
    /// one steadier than the fast. A gate bracket ran entirely inside it with
    /// nothing in its artifact to say so.
    ///
    /// **The 9.4 this said until six holds replaced it was the top of that
    /// band, not its centre.** What made the six readable is that each one
    /// also reported 1.07 to 1.92 cores held by anything else, so the state
    /// was [separated from a neighbour at the same
    /// instant](../../docs/measurements/2026-08-03-094550-the-slow-state-caught-on-a-quiet-machine.md)
    /// rather than inferred from a rate alone.
    ///
    /// **And the slow state is two states, which no field here separates and
    /// no solo hold can.** Two runs whose solo holds agree to half a percent,
    /// 9.752 and 9.801, held a hundred sessions at 246.4 and 902.8 units/s —
    /// one machine crippled under load, one within 0.5% of its rested
    /// reference with only the lone session down. They move gate M1's
    /// slowdown in opposite directions, 3.958 against a 1.54 that *passes* a
    /// condition asking for 2, so the 72% above belongs to the crippled one
    /// rather than to "the slow state". What separates them is a concurrent
    /// hold's own total throughput, [bimodal across all nine on disk with
    /// nothing in the 3.1× gap](../../docs/measurements/2026-08-03-173452-the-slow-state-flatters-the-gate.md)
    /// — a figure this struct cannot carry, because it is a property of a run
    /// rather than of a host.
    ///
    /// **Neither field distinguishes the two, and `thermal_c` is slow-moving
    /// rather than constant — which took a week to see.** It read **39.1 °C
    /// in every artifact** for the first week, idle and at the end of twenty
    /// minutes of a hundred saturating sessions alike, which was recorded
    /// here as *it does not vary*. On 2026-08-11 it reads **63.1 °C in every
    /// artifact** — 24 degrees up, and again identical across every hold of
    /// the day. So it moves between days and never within one.
    ///
    /// **It still names nothing.** Across 124 holds carrying both counters,
    /// solo holds under 50 °C average **12.422** against **12.634** over it,
    /// and the box was slow and loose at 39–40 °C too — one bracket spread
    /// 24.36% at 40.1 °C with the cores boosting at 175.6%. A figure that
    /// changes across days but never within one cannot separate conditions
    /// that arrive and leave in minutes. A zone tracking the package may
    /// exist under another instance; this is the first `MSAcpi` returns.
    ///
    /// **And both are point samples rather than run characteristics.** This
    /// struct is queried once — at the *end* of a hold, since `into_report`
    /// builds it after the run, and at the *start* of a ramp or an observe.
    /// Two twenty-minute holds of the same shape returned 144.3 and 182.1 for
    /// `processor_performance`, which is teardown timing rather than a
    /// difference between the runs. Sampling either per tick would cost a
    /// PowerShell process, which is why they are not.
    ///
    /// **Both are now ruled out on this box, and the second was tested.** The
    /// zone has one instance, returns 39.1 °C whatever the load, and there is
    /// no `Win32_TemperatureProbe` beside it. `processor_performance` was
    /// given a before-reading so a hold could show the machine changing under
    /// it, and the pair is noise: one session for twenty seconds went 171% →
    /// 154%, a hundred sessions for two minutes went 159% → 200%. The loaded
    /// run rose. On an idle box the counter follows whichever core happens to
    /// be boosting, so the *before* is the noisy half — and two twenty-minute
    /// holds of the same shape ended at 144.3 and 182.1, which is too wide to
    /// separate states with. The pair was built, measured and removed.
    ///
    /// They travel in every artifact because the query is already made and
    /// another machine may answer differently. Nothing here reads them.
    ///
    /// `None` where the counter is absent, which is ordinary: `MSAcpi` is not
    /// exposed by every firmware.
    pub thermal_c: Option<f64>,
    /// **One point sample per report, and it is a poor summary of a hold.**
    /// Taken once when the report is built, so a two-minute hold gets a
    /// reading from its first instant. On 2026-08-11 four holds in one set read
    /// 192.1, 183.5, 105.0 and 149.4 — and the 149.4 belongs to the hold that
    /// returned the *lowest* rate of the four. Do not derive a rate, a ratio or
    /// a normalisation from this column.
    ///
    /// **Sampling it per tick was costed and rejected.** It arrives from
    /// [`HOST_QUERY`], one PowerShell process that also asks Defender, the
    /// battery, the power plan and the thermal zone. Per-tick would mean a
    /// shell spawn every 5 s, and `Get-Counter` needs ~1.1 s for any
    /// `% ...` counter on its own — two internal samples about a second apart,
    /// since the counter is a rate — so roughly 30% of every tick would be
    /// spent inside a query, in the window being measured. What it would buy
    /// is narrow: the clock is **not** the mechanism behind the tenancy step.
    /// From the 2–4 core band to the 11–12.5 band it climbs 143.9% → 174.0%
    /// while the rate moves 1.1%, and between the two lowest bands it *falls*
    /// 124.3% → 121.3% while the rate rises 17.6%. A native PDH reader with
    /// negligible per-read cost would change the arithmetic; a shell spawn
    /// does not.
    pub processor_performance: Option<f64>,
    /// The same counter per core, because `_Total` is an average and the
    /// session runs on **one** core.
    ///
    /// **Added because a refutation turned out to rest on the aggregate.** A
    /// run on 2026-08-11 found `r(rate, processor_performance) = −0.429` and
    /// read it as the clock being refuted backwards. Read per core the same
    /// day, this box spans **91.1% to 123.8% — 36% — while `_Total` said
    /// 96.6%**, so the aggregate need not describe the core the work happened
    /// on. Worse, it has a mechanism for a spurious negative: `_Total`
    /// averages across cores, so as load rises and more cores wake it takes in
    /// newly-active ones at middling clocks and falls, while the busy core may
    /// be climbing. That reading would be *how many cores are awake* wearing a
    /// frequency's name.
    ///
    /// **All of them are kept rather than a summary.** Which core a session
    /// lands on is not recorded, so no single value can be chosen honestly:
    /// the maximum assumes the scheduler picked the fastest core, and [this
    /// box's cores differ by 2.1× under
    /// load](../../docs/measurements/2026-07-31-145412-the-cores-are-not-interchangeable.md),
    /// so the maximum may belong to something else entirely. Sixteen numbers
    /// in an artifact that already holds hundreds is cheap, and a clock at
    /// hold time cannot be recovered afterwards at any price.
    ///
    /// Empty where the query failed or the counter is absent, which is a
    /// different state from a machine that answered with nothing.
    #[serde(default)]
    pub processor_performance_cores: Vec<f64>,
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
                "updates" => {
                    facts.updates = value
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .collect();
                }
                "thermal_c" => facts.thermal_c = value.parse().ok(),
                "processor_performance" => facts.processor_performance = value.parse().ok(),
                "processor_performance_cores" => {
                    facts.processor_performance_cores = value
                        .split(',')
                        .filter_map(|v| v.trim().parse().ok())
                        .collect();
                }
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
