# Vary tenancy INSIDE one window: take a hundred-session duty-0.27 hold while the
# tenant is absent, then another as soon as it returns, back to back.
#
# WHY. Two duty-0.27 windows hours apart disagreed by 52% in the wrong direction
# (2026-08-12). Comparing them varies time as well as tenancy, which is the
# confound this repository has already paid for twice. This runs both arms inside
# one sitting so the only thing that differs is the neighbour.
#
# The gate is a CENSUS, not `doctor` and not a residual: a residual is anonymous
# by construction, and both ad-hoc runs that night were tenanted while being read
# as idle. Here the tenant is named, so absence is asserted rather than inferred.
[CmdletBinding()]
param([int]$Sessions = 100, [int]$HoldSeconds = 25, [int]$MaxWaitMinutes = 45)

$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$spin = Join-Path $root 'target\release\cpu-spin.exe'
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$out = Join-Path $root "bench-out\tenancy-in-window-$stamp"
New-Item -ItemType Directory -Force -Path $out | Out-Null
Start-Transcript -Path (Join-Path $out 'transcript.txt') | Out-Null

function Tenant-Cores {
    # Named, not residual. Two reads a second apart give a rate rather than a total.
    $p = @(Get-Process chrome-headless-shell -ErrorAction SilentlyContinue)
    if ($p.Count -eq 0) { return 0.0 }
    $a = ($p | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
    Start-Sleep -Seconds 2
    $q = @(Get-Process chrome-headless-shell -ErrorAction SilentlyContinue)
    if ($q.Count -eq 0) { return 0.0 }
    $b = ($q | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
    return [math]::Max(0.0, ($b - $a) / 2.0)
}

function Take-Hold([string]$label) {
    for ($i = 0; $i -lt $Sessions; $i++) {
        Start-Process $spin -ArgumentList '--units','100000000','--duty','0.27','--resident','1' `
            -WindowStyle Hidden -RedirectStandardOutput 'NUL' | Out-Null
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
    Get-Process cpu-spin -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Seconds 3
    $surv = @(Get-Process cpu-spin -ErrorAction SilentlyContinue).Count
    $r = [ordered]@{
        label = $label; window_seconds = [math]::Round($el, 2); sessions = $Sessions
        machine_cores = [math]::Round($m * 16 / 100, 3)
        all_process_cores = [math]::Round($all, 3)
        job_cores = [math]::Round($job, 3)
        tenant_cores = [math]::Round($ten, 3)
        per_session = [math]::Round($job / $Sessions, 5)
        achieved_duty_pct = [math]::Round(($job / $Sessions) / 0.27 * 100, 2)
        survivors = $surv
        taken_at = (Get-Date -Format 'o')
    }
    "  {0,-8} job {1,6:N2}  tenant {2,6:N2}  machine {3,6:N2}  each {4:N4}  duty {5:N1}%  survivors {6}" -f `
        $label, $r.job_cores, $r.tenant_cores, $r.machine_cores, $r.per_session, $r.achieved_duty_pct, $surv
    return $r
}

$results = @()
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
    if (-not $quiet) { "gave up: tenant never fell below 0.5 cores"; return }

    "ARM A - tenant absent"
    $results += Take-Hold 'absent'

    # Straight into the second arm. If the tenant has not returned, say so rather
    # than waiting the pair out of one window, which is the whole point.
    "ARM B - immediately after, whatever the tenant is doing"
    $results += Take-Hold 'after'
}
finally {
    $payload = [ordered]@{
        holds = $results
        gap_seconds = if ($results.Count -eq 2) { ([datetime]$results[1].taken_at - [datetime]$results[0].taken_at).TotalSeconds } else { $null }
    }
    $payload | ConvertTo-Json -Depth 6 | Out-File (Join-Path $out 'tenancy.json') -Encoding utf8
    "wrote $out\tenancy.json"
    Stop-Transcript | Out-Null
}
