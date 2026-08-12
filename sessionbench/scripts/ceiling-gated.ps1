# The one pairing no run has had tonight: a SLEEPLESS load AND an absent tenant.
#
# WHY. A hundred sessions at duty 1.0 unpark this box — 12.64 machine cores
# against ~5 at duty 0.27 — but the hold that showed it had a neighbour holding
# 7.57, so the job took 5.07 and the work rate stayed in today's low band. That
# run cannot separate "unparking restores throughput" from "tenancy suppressed
# it", because only one of the two variables was controlled.
#
# The gate is a NAMED census of chrome-headless-shell rather than doctor or a
# residual, for the reason recorded on 2026-08-12: both ad-hoc runs that day were
# tenanted at 7.93 and 9.99 cores while being read as idle, which a residual
# cannot catch by construction.
#
# The prediction, stated before the run: if unparking restores throughput, job
# cores should approach the 15.3-15.5 of 3 and 11 August rather than today's
# 3.34-4.81. If they stay near 5 with the tenant absent AND the box unparked,
# neither parking nor tenancy explains the ceiling and it is something else.
[CmdletBinding()]
param([int]$Sessions = 100, [int]$Seconds = 60, [int]$MaxWaitMinutes = 40, [int]$MaxAttempts = 4, [double]$AbortRestAbove = 2.0, [double]$Duty = 1.0, [switch]$RequireTenant)

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$bench = Join-Path $root 'target\release\sessionbench.exe'
$spin = Join-Path $root 'target\release\cpu-spin.exe'
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$out = Join-Path $root "bench-out\ceiling-gated-$stamp"
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

try {
    "waiting for the tenant to leave (up to $MaxWaitMinutes min)"
    $deadline = (Get-Date).AddMinutes($MaxWaitMinutes)
    $quiet = $false
    while ((Get-Date) -lt $deadline) {
        $t = Tenant-Cores
        "  {0}  tenant {1:N2} cores" -f (Get-Date -Format 'HH:mm:ss'), $t
        # -RequireTenant inverts the gate. The mirror-image test needs the
        # NEIGHBOUR PRESENT, because continuity and tenancy predict opposite
        # machine totals for a gappy workload beside a busy Chromium, and every
        # run so far has varied them together.
        if ($RequireTenant) { if ($t -gt 5.0) { $quiet = $true; break } }
        elseif ($t -lt 0.5) { $quiet = $true; break }
        Start-Sleep -Seconds 20
    }
    if (-not $quiet) { "gave up: tenant never fell below 0.5 cores"; return }

    # A PRE-HOLD GATE TESTS AN INSTANT AND THE HOLD NEEDS A MINUTE. On 2026-08-12
    # this gate cleared at 15:55:00 with the tenant under 0.5 cores and the hold
    # that followed recorded rest 9.603 — the third time that night a clearance
    # was invaded inside the window it opened. So the hold has to refuse ITSELF:
    # `--abort-rest-above` ends it mid-flight when the residual climbs, and the
    # attempt is retried rather than published.
    for ($try = 1; $try -le $MaxAttempts; $try++) {
        "ATTEMPT {0}/{1} - cleared at {2}, holding {3} sessions at duty $Duty for {4}s" -f `
            $try, $MaxAttempts, (Get-Date -Format 'HH:mm:ss'), $Sessions, $Seconds
        & $bench hold --label "ceiling-gated-$try" --sessions $Sessions --interval 5 --duration $Seconds `
            --abort-rest-above $AbortRestAbove -- `
            $spin --units 100000000 --duty $Duty --resident 1
        "  sessionbench exit = $LASTEXITCODE"
        # Re-wait for quiet before the next attempt rather than launching into a
        # box the previous attempt just told us is busy.
        if ($try -lt $MaxAttempts) {
            $t = Tenant-Cores
            while ($t -ge 0.5 -and (Get-Date) -lt $deadline) { Start-Sleep -Seconds 20; $t = Tenant-Cores }
            if ((Get-Date) -ge $deadline) { "out of time after attempt $try"; break }
        }
    }
}
finally {
    Start-Sleep -Seconds 3
    "survivors: " + @(Get-Process cpu-spin,sessionbench -ErrorAction SilentlyContinue).Count
    Stop-Transcript | Out-Null
}
