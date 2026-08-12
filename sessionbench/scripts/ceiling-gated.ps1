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
param([int]$Sessions = 100, [int]$Seconds = 60, [int]$MaxWaitMinutes = 40)

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
        if ($t -lt 0.5) { $quiet = $true; break }
        Start-Sleep -Seconds 20
    }
    if (-not $quiet) { "gave up: tenant never fell below 0.5 cores"; return }

    "CLEARED at {0} - holding {1} sessions at duty 1.0 for {2}s" -f (Get-Date -Format 'HH:mm:ss'), $Sessions, $Seconds
    & $bench hold --label 'ceiling-gated' --sessions $Sessions --interval 5 --duration $Seconds -- `
        $spin --units 100000000 --duty 1.0 --resident 1
    "sessionbench exit = $LASTEXITCODE"
}
finally {
    Start-Sleep -Seconds 3
    "survivors: " + @(Get-Process cpu-spin,sessionbench -ErrorAction SilentlyContinue).Count
    Stop-Transcript | Out-Null
}
