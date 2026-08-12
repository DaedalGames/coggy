# Watch the parked-core count over time, with the machine's own load beside it.
#
# WHY. On 2026-08-12 this box read 12 of 16 cores parked at idle and 9-10 under a
# hundred sleepless sessions, which pins it at ~5 usable cores and explains why
# every hold that day reached 3.34-4.81 job cores where 3 and 11 August reached
# 15.3-15.5. Nothing from those days records a parked count, because nothing was
# reading the counter. This builds that history going forward.
#
# It is deliberately cheap: two counter reads every 30 s and nothing else, so it
# can run alongside ordinary work without being the neighbour under study. It
# writes one JSONL line per poll, so an interrupted run keeps everything it saw.
[CmdletBinding()]
param([int]$Minutes = 20, [int]$IntervalSeconds = 30)

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$out = Join-Path $root "bench-out\parking-census-$stamp"
New-Item -ItemType Directory -Force -Path $out | Out-Null
$jsonl = Join-Path $out 'parking.jsonl'

$deadline = (Get-Date).AddMinutes($Minutes)
while ((Get-Date) -lt $deadline) {
    $park = (Get-Counter '\Processor Information(0,*)\Parking Status' -ErrorAction SilentlyContinue).CounterSamples |
        Where-Object { $_.InstanceName -notmatch '_total' }
    $busy = (Get-Counter '\Processor Information(0,_Total)\% Processor Time' -ErrorAction SilentlyContinue).CounterSamples[0].CookedValue
    $tenant = @(Get-Process chrome-headless-shell -ErrorAction SilentlyContinue).Count
    # An unreadable counter must not look like an unparked machine: report null,
    # which no comparison can silently accept, rather than a number.
    $parked = if ($null -ne $park -and $park.Count -gt 0) { @($park | Where-Object { $_.CookedValue -gt 0 }).Count } else { $null }
    $total = if ($null -ne $park) { $park.Count } else { $null }
    [ordered]@{
        at = (Get-Date -Format 'o')
        parked = $parked
        cores_total = $total
        machine_cores = if ($null -ne $busy) { [math]::Round($busy * 16 / 100, 3) } else { $null }
        tenant_processes = $tenant
    } | ConvertTo-Json -Compress | Out-File -FilePath $jsonl -Append -Encoding utf8
    "{0}  parked {1}/{2}  machine {3:N2}  tenant procs {4}" -f (Get-Date -Format 'HH:mm:ss'), $parked, $total, $(if ($null -ne $busy) { $busy * 16 / 100 } else { -1 }), $tenant
    Start-Sleep -Seconds $IntervalSeconds
}
"census complete: $jsonl"
