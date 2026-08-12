# How much of MY load does it take to unpark this box, and does any amount?
#
# WHY. Chromium holding ~10 cores leaves the box unparked in 96-98% of samples.
# A hundred `cpu-spin --duty 1.0` sessions — which ask for more than that — left
# it at 5.32 machine cores, i.e. parked. So it is not the SIZE of the demand.
# Something about the kind of load differs, and no measurement here has varied my
# own session count against the parked count while the box started parked.
#
# The prediction, before the run: if parking responds to demand at all, the
# parked count should fall as sessions rise. If it stays high at every rung
# including the largest, then this box does not unpark for this workload at any
# count, and the mechanism is about what the neighbour IS rather than what it
# consumes — which would be the sharpest open question left.
[CmdletBinding()]
param([int[]]$Rungs = @(0, 1, 5, 20, 60), [int]$Seconds = 25)

$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$spin = Join-Path $root 'target\release\cpu-spin.exe'
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$out = Join-Path $root "bench-out\unpark-dose-$stamp"
New-Item -ItemType Directory -Force -Path $out | Out-Null
Start-Transcript -Path (Join-Path $out 'transcript.txt') | Out-Null
$jsonl = Join-Path $out 'rungs.jsonl'

function Parked {
    $p = (Get-Counter '\Processor Information(0,*)\Parking Status' -ErrorAction SilentlyContinue).CounterSamples |
        Where-Object { $_.InstanceName -notmatch '_total' }
    if ($null -eq $p -or $p.Count -eq 0) { return $null }
    @($p | Where-Object { $_.CookedValue -gt 0 }).Count
}
function TenantCores {
    $t = @(Get-Process chrome-headless-shell -ErrorAction SilentlyContinue)
    if ($t.Count -eq 0) { return 0.0 }
    $a = ($t | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
    Start-Sleep -Seconds 1
    $u = @(Get-Process chrome-headless-shell -ErrorAction SilentlyContinue)
    if ($u.Count -eq 0) { return 0.0 }
    (($u | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum - $a)
}

try {
    foreach ($n in $Rungs) {
        for ($i = 0; $i -lt $n; $i++) {
            Start-Process $spin -ArgumentList '--units','100000000','--duty','1.0','--resident','1' `
                -WindowStyle Hidden -RedirectStandardOutput 'NUL' | Out-Null
        }
        Start-Sleep -Seconds 6
        # Several reads per rung: the parked count is bimodal and one sample of it
        # is a sample of one moment, which cost three withdrawn claims today.
        $pk = @(); $deadline = (Get-Date).AddSeconds($Seconds)
        while ((Get-Date) -lt $deadline) { $v = Parked; if ($null -ne $v) { $pk += $v }; Start-Sleep -Seconds 2 }
        $m = (Get-Counter '\Processor Information(0,_Total)\% Processor Time' -ErrorAction SilentlyContinue).CounterSamples[0].CookedValue
        $tc = TenantCores
        $alive = @(Get-Process cpu-spin -ErrorAction SilentlyContinue).Count
        $hi = @($pk | Where-Object { $_ -ge 8 }).Count
        [ordered]@{
            sessions = $n
            sessions_alive = $alive
            samples = $pk.Count
            parked_median = if ($pk.Count) { ($pk | Sort-Object)[[int]($pk.Count/2)] } else { $null }
            parked_ge8_pct = if ($pk.Count) { [math]::Round(100.0 * $hi / $pk.Count, 0) } else { $null }
            machine_cores = [math]::Round($m * 16 / 100, 3)
            tenant_cores = [math]::Round($tc, 3)
            at = (Get-Date -Format 'o')
        } | ConvertTo-Json -Compress | Out-File -FilePath $jsonl -Append -Encoding utf8
        "  sessions {0,3} (alive {1,3})  parked median {2,2}  >=8 in {3,3}%  machine {4,6:N2}  tenant {5,5:N2}" -f `
            $n, $alive, $(if ($pk.Count) { ($pk | Sort-Object)[[int]($pk.Count/2)] } else { -1 }), `
            $(if ($pk.Count) { [math]::Round(100.0*$hi/$pk.Count,0) } else { -1 }), ($m*16/100), $tc
        Get-Process cpu-spin -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Seconds 4
    }
}
finally {
    Get-Process cpu-spin -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Seconds 3
    "survivors: " + @(Get-Process cpu-spin -ErrorAction SilentlyContinue).Count
    "wrote $jsonl"
    Stop-Transcript | Out-Null
}
