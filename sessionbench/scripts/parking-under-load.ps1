# Does the machine park cores while a hundred CPU-bound sessions are runnable?
#
# WHY. On 2026-08-12 the parked count came back bimodal (0 for 39% of samples, 12
# for 27%) with dwell maxima of 178-257 s, and the parked side delivers 3.13
# cores against the unparked side's 11.28. That correlation cannot say which way
# it runs: cores may park BECAUSE the box went idle, or the box may deliver less
# BECAUSE cores parked. Both readings fit either story.
#
# This separates them by holding the load fixed. A hundred `--duty 0.27` sessions
# are runnable throughout, so if the parked count still reaches its high mode,
# parking is not a response to idleness — there is nothing idle about the machine
# while it happens. If the count instead pins near zero for the whole window, the
# idleness story survives and the ceiling needs a different explanation.
#
# The prediction is stated before the run and can fail either way, which is the
# point of writing it here rather than after.
[CmdletBinding()]
param([int]$Sessions = 100, [int]$Minutes = 6)

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$spin = Join-Path $root 'target\release\cpu-spin.exe'
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$out = Join-Path $root "bench-out\parking-under-load-$stamp"
New-Item -ItemType Directory -Force -Path $out | Out-Null
Start-Transcript -Path (Join-Path $out 'transcript.txt') | Out-Null
$jsonl = Join-Path $out 'samples.jsonl'

try {
    "launching $Sessions sessions at duty 0.27"
    for ($i = 0; $i -lt $Sessions; $i++) {
        Start-Process $spin -ArgumentList '--units','100000000','--duty','0.27','--resident','1' `
            -WindowStyle Hidden -RedirectStandardOutput 'NUL' | Out-Null
    }
    Start-Sleep -Seconds 8

    $deadline = (Get-Date).AddMinutes($Minutes)
    $n = 0
    while ((Get-Date) -lt $deadline) {
        $park = (Get-Counter '\Processor Information(0,*)\Parking Status' -ErrorAction SilentlyContinue).CounterSamples |
            Where-Object { $_.InstanceName -notmatch '_total' }
        $busy = (Get-Counter '\Processor Information(0,_Total)\% Processor Time' -ErrorAction SilentlyContinue).CounterSamples[0].CookedValue
        # The load must be asserted, not assumed: a sample taken after the
        # sessions died would read an idle box and be counted as evidence.
        $alive = @(Get-Process cpu-spin -ErrorAction SilentlyContinue).Count
        $tenant = @(Get-Process chrome-headless-shell -ErrorAction SilentlyContinue).Count
        $parked = if ($null -ne $park -and $park.Count -gt 0) { @($park | Where-Object { $_.CookedValue -gt 0 }).Count } else { $null }
        [ordered]@{
            at = (Get-Date -Format 'o')
            parked = $parked
            cores_total = if ($null -ne $park) { $park.Count } else { $null }
            machine_cores = if ($null -ne $busy) { [math]::Round($busy * 16 / 100, 3) } else { $null }
            sessions_alive = $alive
            tenant_processes = $tenant
        } | ConvertTo-Json -Compress | Out-File -FilePath $jsonl -Append -Encoding utf8
        if ($n % 20 -eq 0) { "{0}  parked {1}  machine {2:N2}  sessions {3}" -f (Get-Date -Format 'HH:mm:ss'), $parked, $busy, $alive }
        $n++
    }
}
finally {
    Get-Process cpu-spin -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Seconds 3
    "survivors: " + @(Get-Process cpu-spin -ErrorAction SilentlyContinue).Count
    "wrote $jsonl"
    Stop-Transcript | Out-Null
}
