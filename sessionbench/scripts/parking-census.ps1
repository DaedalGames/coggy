# Watch the parked-core count over time, with the machine's own load beside it.
#
# WHY. On 2026-08-12 this box read 12 of 16 cores parked at idle and 9-10 under a
# hundred sleepless sessions, which pins it at ~5 usable cores and explains why
# every hold that day reached 3.34-4.81 job cores where 3 and 11 August reached
# 15.3-15.5. Nothing from those days records a parked count, because nothing was
# reading the counter. This builds that history going forward.
#
# SAMPLE FAST. The first version polled every 30 s, which is useless here: the
# parked count swings from 0 to 12 within seconds, so a 30-second poll aliases a
# fast signal into a sequence of unrelated instants and reports each as a level.
# That is exactly how this record's withdrawn claims were made. One read a second
# is still cheap, and the mean over a window is the quantity that means anything.
# It writes one JSONL line per poll, so an interrupted run keeps everything it saw.
#
# THE ACHIEVED INTERVAL IS NOT THE REQUESTED ONE. `Get-Counter` carries its own
# ~1 s floor and this loop makes two calls, so asking for 1 s delivers about 3.1.
# Every line carries its own timestamp, so the achieved interval is recoverable
# from the artifact rather than assumed from the parameter — which is the same
# reason a hold reports the duty it achieved rather than the flag it was given.
#
# WHAT IT FOUND, so the next reader does not re-derive it: the parked count here
# is BIMODAL, sitting at 0 or 12 in 21 of 26 samples. A mean over it describes no
# state the machine is ever in. Report the distribution, not the average.
[CmdletBinding()]
param([int]$Minutes = 20, [double]$IntervalSeconds = 1)

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$out = Join-Path $root "bench-out\parking-census-$stamp"
New-Item -ItemType Directory -Force -Path $out | Out-Null
$jsonl = Join-Path $out 'parking.jsonl'

$deadline = (Get-Date).AddMinutes($Minutes)
$n = 0
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
    # One console line a second would be noise; the JSONL is the artifact.
    if ($n % 30 -eq 0) {
        "{0}  parked {1}/{2}  machine {3:N2}  tenant procs {4}" -f (Get-Date -Format 'HH:mm:ss'), $parked, $total, $(if ($null -ne $busy) { $busy * 16 / 100 } else { -1 }), $tenant
    }
    $n++
    Start-Sleep -Seconds $IntervalSeconds
}
"census complete: $jsonl"
