# Is it the THREAD COUNT or the PACKING? Both arms hold 60 threads.
#
# WHY. Five processes of 27 threads unpark this box (11 parked cores -> 2) where
# five of one thread do not (-> 7). But those arms differ in TOTAL threads, 135
# against 5, so `more threads` explains it as well as `more threads per process`.
# Sixty processes of one thread is 60 total threads and is known NOT to unpark,
# which narrows it and does not settle it, 60 being well under 135.
#
# THE DISCRIMINATOR. Sixty threads in ONE process against sixty in SIXTY. Total
# demand and total thread count are identical; only the packing differs.
#   both arms unpark   -> total thread count, and 60 is above the threshold
#   only the packed one -> packing, and the browser's shape is what matters
#   neither unparks    -> the threshold is between 60 and 135 total threads
#
# Both arms are gated on a quiet neighbour and run back to back, so they share a
# window. A pair gathered in different sittings varies the afternoon too.
[CmdletBinding()]
param([int]$Seconds = 25, [int]$MaxWaitMinutes = 25)

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$dose = Join-Path $PSScriptRoot 'unpark-dose.ps1'
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$out = Join-Path $root "bench-out	hread-packing-$stamp"
New-Item -ItemType Directory -Force -Path $out | Out-Null
Start-Transcript -Path (Join-Path $out 'transcript.txt') | Out-Null
try {
    "ARM A  60 processes x  1 thread  = 60 threads"
    & $dose -Rungs 60 -Threads 1 -Seconds $Seconds -MaxWaitMinutes $MaxWaitMinutes
    "ARM B   1 process  x 60 threads  = 60 threads"
    & $dose -Rungs 1 -Threads 60 -Seconds $Seconds -MaxWaitMinutes $MaxWaitMinutes
    "ARM C   5 processes x 27 threads = 135 threads   (the known unparker, as a positive control)"
    & $dose -Rungs 5 -Threads 27 -Seconds $Seconds -MaxWaitMinutes $MaxWaitMinutes
} finally {
    Get-Process cpu-spin -ErrorAction SilentlyContinue | Stop-Process -Force
    Stop-Transcript | Out-Null
}
