# Does any readable counter distinguish the two machine states?
#
# A three-minute saturating burst halves this box's solo rate for about ninety
# minutes, and nothing in any artifact says which state a run was in -- see
# docs/measurements/2026-08-03-004512-a-saturating-burst-halves-the-box-for-an-hour.md.
# `doctor` names the power state because Win32_Battery hands it over in one
# field; this asks whether the thermal state has an equivalent.
#
# Order matters: the fast state is the one you lose by measuring, so it goes
# first. The burst between the two phases is what induces the slow state, and
# it leaves the box slow for about an hour afterwards.
#
# READ THE OUTPUT, NOT THE EXIT CODE.

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$spin  = "$root\target\release\cpu-spin.exe"
$bench = "$root\target\release\sessionbench.exe"
$work  = @('--units', '100000000', '--duty', '0.27', '--resident', '20')

$stray = Get-Process coggyd, cpu-spin -ErrorAction SilentlyContinue
if ($stray) { "REFUSING: {0} stray process(es)" -f $stray.Count; throw "not clean" }

function Sample {
    $t = (Get-CimInstance -Namespace root/WMI -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction SilentlyContinue |
          Select-Object -First 1).CurrentTemperature
    $perf = (Get-Counter '\Processor Information(_Total)\% Processor Performance' -ErrorAction SilentlyContinue).CounterSamples[0].CookedValue
    $freq = (Get-Counter '\Processor Information(_Total)\Processor Frequency' -ErrorAction SilentlyContinue).CounterSamples[0].CookedValue
    [pscustomobject]@{ C = if ($t) { $t/10 - 273.15 } else { $null }; Perf = $perf; MHz = $freq }
}

function Phase([string]$name) {
    Write-Host "=== $name ==="
    $log = "$root\bench-out\thermal-$name.log"
    $p = Start-Process -FilePath $bench -ArgumentList (@('hold','--label',"thermal-$name",'--sessions','1',
        '--duration','60','--interval','5','--daemon',"$root\target\release\coggyd.exe",'--') + @($spin) + $work) `
        -RedirectStandardOutput $log -RedirectStandardError "$root\bench-out\thermal-$name.err" -PassThru -WindowStyle Hidden
    $rows = @()
    for ($i = 0; $i -lt 11; $i++) { Start-Sleep -Seconds 5; $rows += Sample }
    $p.WaitForExit(120000) | Out-Null
    $rate = (Select-String -Path $log -Pattern 'rate\s+([\d.]+)').Matches.Groups[1].Value
    $c    = ($rows | Where-Object { $_.C } | Measure-Object -Property C -Average).Average
    $perf = ($rows | Measure-Object -Property Perf -Average).Average
    $mhz  = ($rows | Measure-Object -Property MHz -Average).Average
    Write-Host ("  solo rate        {0} units/s" -f $rate)
    Write-Host ("  thermal zone     {0:N1} C" -f $c)
    Write-Host ("  % proc perf      {0:N1}" -f $perf)
    Write-Host ("  frequency        {0:N0} MHz" -f $mhz)
    [pscustomobject]@{ Rate = [double]$rate; C = $c; Perf = $perf; MHz = $mhz }
}

$fast = Phase 'fast'

"=== burst: 100 sessions, 180 s (this is what induces the slow state) ==="
& $bench hold --label thermal-burst --sessions 100 --duration 180 --interval 30 `
    --daemon "$root\target\release\coggyd.exe" -- $spin @work 2>&1 |
    Select-String -Pattern 'rate |peak rss' | ForEach-Object { "  " + $_.Line.Trim() }

$slow = Phase 'slow'

"`n=== verdict ==="
"  solo rate       {0,7:N2} -> {1,7:N2}   ({2:N2}x)" -f $fast.Rate, $slow.Rate, ($fast.Rate / $slow.Rate)
"  thermal zone    {0,7:N1} -> {1,7:N1}   ({2:+0.0;-0.0} C)" -f $fast.C, $slow.C, ($slow.C - $fast.C)
"  % proc perf     {0,7:N1} -> {1,7:N1}   ({2:N2}x)" -f $fast.Perf, $slow.Perf, ($fast.Perf / $slow.Perf)
"  frequency       {0,7:N0} -> {1,7:N0}   ({2:N2}x)" -f $fast.MHz, $slow.MHz, ($fast.MHz / $slow.MHz)
