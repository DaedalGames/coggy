# Measure oversleep AND achieved duty in ONE window, at a hundred sessions.
#
# WHY. Oversleep at a hundred sessions was measured at 10.73x, which predicts an
# achieved duty of 1/(1 + 2.70*10.73) = 3.3%. A censused-empty window measured
# 12.7%. Both are sound and they were taken in different sittings, so comparing
# them varies time as well as the quantity — the confound this repository has
# paid for repeatedly. One window settles it.
#
# If oversleep accounts for the achieved duty, the pause is the mechanism and the
# question closes. If it does not, something other than the pause is keeping a
# hundred CPU-bound sessions off eleven idle cores.
#
# The gate is a NAMED census of the tenant, not doctor and not a residual.
[CmdletBinding()]
param([int]$Sessions = 100, [int]$HoldSeconds = 30, [int]$Reporters = 12, [int]$MaxWaitMinutes = 40)

$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$spin = Join-Path $root 'target\release\cpu-spin.exe'
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$out = Join-Path $root "bench-out\oversleep-in-window-$stamp"
New-Item -ItemType Directory -Force -Path $out | Out-Null
Start-Transcript -Path (Join-Path $out 'transcript.txt') | Out-Null

function Tenant-Cores {
    $p = @(Get-Process chrome-headless-shell -ErrorAction SilentlyContinue)
    if ($p.Count -eq 0) { return 0.0 }
    $a = ($p | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
    Start-Sleep -Seconds 2
    $q = @(Get-Process chrome-headless-shell -ErrorAction SilentlyContinue)
    if ($q.Count -eq 0) { return 0.0 }
    $b = ($q | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
    return [math]::Max(0.0, ($b - $a) / 2.0)
}

$result = [ordered]@{ outcome = 'Unknown' }
try {
    "waiting for the tenant to leave (up to $MaxWaitMinutes min)"
    $deadline = (Get-Date).AddMinutes($MaxWaitMinutes)
    $quiet = $false
    while ((Get-Date) -lt $deadline) {
        $t = Tenant-Cores
        "  {0}  tenant {1:N2} cores" -f (Get-Date -Format 'HH:mm:ss'), $t
        if ($t -lt 0.5) { $quiet = $true; break }
        Start-Sleep -Seconds 20
    }
    if (-not $quiet) { $result.outcome = 'NoQuietWindow'; return }

    # A subset reports timing to its own stderr file. The rest are plain, so the
    # census sees the same hundred-session load either way.
    for ($i = 0; $i -lt $Sessions; $i++) {
        $args = @('--units', '100000000', '--duty', '0.27', '--resident', '1')
        if ($i -lt $Reporters) {
            $args += '--report-timing'
            Start-Process $spin -ArgumentList $args -WindowStyle Hidden `
                -RedirectStandardOutput 'NUL' -RedirectStandardError (Join-Path $out "timing-$i.err") | Out-Null
        } else {
            Start-Process $spin -ArgumentList $args -WindowStyle Hidden -RedirectStandardOutput 'NUL' | Out-Null
        }
    }
    Start-Sleep -Seconds 8

    $snapA = @{}; foreach ($p in Get-Process -ErrorAction SilentlyContinue) { try { $snapA[$p.Id] = $p.TotalProcessorTime.TotalSeconds } catch {} }
    $t0 = Get-Date
    $m = (Get-Counter '\Processor Information(0,_Total)\% Processor Time' -SampleInterval $HoldSeconds -MaxSamples 1).CounterSamples[0].CookedValue
    $t1 = Get-Date
    $spinIds = @(Get-Process cpu-spin -ErrorAction SilentlyContinue).Id
    $tenantIds = @(Get-Process chrome-headless-shell -ErrorAction SilentlyContinue).Id
    $snapB = @{}; foreach ($p in Get-Process -ErrorAction SilentlyContinue) { try { $snapB[$p.Id] = $p.TotalProcessorTime.TotalSeconds } catch {} }
    $el = ($t1 - $t0).TotalSeconds
    $job = 0.0; $ten = 0.0; $all = 0.0
    foreach ($k in $snapB.Keys) {
        if (-not $snapA.ContainsKey($k)) { continue }
        $d = ($snapB[$k] - $snapA[$k]) / $el
        if ($d -le 0) { continue }
        $all += $d
        if ($spinIds -contains $k) { $job += $d }
        elseif ($tenantIds -contains $k) { $ten += $d }
    }
    $alive = @(Get-Process cpu-spin -ErrorAction SilentlyContinue).Count
    Get-Process cpu-spin -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Seconds 3

    # Parse the timing lines the reporters wrote during that same window.
    $os = @()
    Get-ChildItem (Join-Path $out 'timing-*.err') -ErrorAction SilentlyContinue | ForEach-Object {
        foreach ($line in (Get-Content $_.FullName -ErrorAction SilentlyContinue)) {
            if ($line -match 'oversleep\s+([0-9.]+)') { $os += [double]$Matches[1] }
        }
    }
    $meanOs = if ($os.Count -gt 0) { ($os | Measure-Object -Average).Average } else { $null }

    $perSession = $job / $Sessions
    $achieved = $perSession / 0.27
    # Predicted achieved duty from the measured oversleep, same window:
    #   duty = 1 / (1 + ((1-d)/d) * oversleep),  (1-d)/d = 2.7037 at d = 0.27
    $pred = if ($null -ne $meanOs) { 1.0 / (1.0 + 2.7037 * $meanOs) } else { $null }

    $result = [ordered]@{
        outcome = 'Taken'
        window_seconds = [math]::Round($el, 2)
        sessions = $Sessions
        reporters = $Reporters
        sessions_alive_at_end = $alive
        machine_cores = [math]::Round($m * 16 / 100, 3)
        all_process_cores = [math]::Round($all, 3)
        job_cores = [math]::Round($job, 3)
        tenant_cores = [math]::Round($ten, 3)
        # NAMED FOR UNITS. `achieved_duty` is an absolute duty, directly
        # comparable with predicted_duty_from_oversleep; `..._vs_requested` is a
        # ratio. The first version gave the ratio the bare name and the run read
        # 0.1263 against a predicted 0.2653 — two different quantities under
        # names that invited exactly that comparison.
        per_session_cores = [math]::Round($perSession, 5)
        achieved_duty = [math]::Round($perSession, 5)
        achieved_duty_vs_requested = [math]::Round($achieved, 4)
        on_cpu_fraction_during_spin = $null
        oversleep_samples = $os.Count
        oversleep_mean = if ($null -ne $meanOs) { [math]::Round($meanOs, 3) } else { $null }
        predicted_duty_from_oversleep = if ($null -ne $pred) { [math]::Round($pred, 4) } else { $null }
        taken_at = (Get-Date -Format 'o')
    }
    # The decisive ratio: what the session believes it computed, against what the
    # kernel says it consumed. Both are in this artifact, so record the division
    # rather than leaving it to whoever reads them.
    if ($null -ne $pred -and $pred -gt 0) { $result.on_cpu_fraction_during_spin = [math]::Round($perSession / $pred, 4) }
    "  job {0:N2}  tenant {1:N2}  machine {2:N2}  duty {3:N4} vs predicted {4}  oversleep {5}  on-cpu during spin {6}" -f `
        $result.job_cores, $result.tenant_cores, $result.machine_cores, $result.achieved_duty, $result.predicted_duty_from_oversleep, $result.oversleep_mean, $result.on_cpu_fraction_during_spin
}
finally {
    $result | ConvertTo-Json -Depth 6 | Out-File (Join-Path $out 'oversleep.json') -Encoding utf8
    "wrote $out\oversleep.json"
    Stop-Transcript | Out-Null
}
