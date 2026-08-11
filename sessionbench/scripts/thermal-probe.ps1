# Can a saturating burst induce the slow machine state, and does any counter
# name it?
#
# THIS HAS RUN ONCE AND THE ANSWER TO THE FIRST HALF WAS NO. At -BurstSeconds
# 180 the box stayed fast throughout: solo 21.45 units/s before and 21.43
# after, thermal zone 39.1 both times, % Processor Performance 173.1 and 173.3.
# So the second half went unanswered -- with both phases in the same state the
# counters agree to 0.1%, which says nothing either way.
#
# The state itself is measured and not in doubt: two levels 2.2x apart, each
# flat across its own samples, seen three times. Its cause is not. A burst sat
# in the twelve-minute gap where the change happened and this script was
# written to test it; see
# docs/measurements/2026-08-03-004512-a-saturating-burst-halves-the-box-for-an-hour.md.
#
# WHAT IS LEFT TO VARY IS THE DURATION, which is why it is a parameter. Three
# minutes is not enough; the evening that produced the state had run far more
# than that in total. Anything longer costs the box for about an hour if it
# works, so pick the number deliberately.
#
# Order matters: the fast state is the one you lose by measuring, so it goes
# first.
#
# READ THE OUTPUT, NOT THE EXIT CODE.

param(
    [int]$BurstSeconds = 180,
    [int]$BurstSessions = 100
)

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
    # THE LAUNCH MUST STAY ASYNCHRONOUS -- the sampling loop below runs *during*
    # the hold, so a synchronous call would collect nothing.
    #
    # But `-WindowStyle Hidden` beside a redirect READS AS SAFE AND IS NOT: the
    # redirect forces UseShellExecute=$false, the hidden window buys nothing,
    # and the child INHERITS THIS CONSOLE. Four harvests died of that on
    # 2026-08-11, the one launched holding its process handle reporting
    # EXIT -1073741510 = 0xC000013A = STATUS_CONTROL_C_EXIT. This probe waits on
    # its child inside one shell so it was never at risk the same way, and that
    # is exactly why the pattern survived here -- it is where the next script
    # copies it from.
    $p = Start-Process -FilePath $bench -ArgumentList (@('hold','--label',"thermal-$name",'--sessions','1',
        '--duration','60','--interval','5','--daemon',"$root\target\release\coggyd.exe",'--') + @($spin) + $work) `
        -PassThru -WindowStyle Hidden
    $rows = @()
    for ($i = 0; $i -lt 11; $i++) { Start-Sleep -Seconds 5; $rows += Sample }
    $p.WaitForExit(120000) | Out-Null
    # AND THE FIGURES NOW COME FROM THE ARTIFACT RATHER THAN THE CONSOLE, which
    # is what made the redirect look necessary. `hold.json` is what the run
    # actually recorded; the printed line is a summary of it that a refine pass
    # or a superseded rung can disagree with.
    $dir = Get-ChildItem "$root\bench-out" -Directory -Filter "*thermal-$name*" |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    $json = if ($dir) { Get-Content "$($dir.FullName)\hold.json" -Raw | ConvertFrom-Json } else { $null }
    if (-not $json) { Write-Host "  NO hold.json for thermal-$name -- refusing to report a phase that left no artifact"; return }
    $rate = $json.units_per_session_per_sec
    # **What else held the machine while this phase ran.** The whole probe
    # attributes a solo rate drop to the burst, and a tenant produces the same
    # drop: 11.5 of 16 cores held cost a single session 26% on 2026-08-03, which
    # is larger than most of what this is looking for. Every hold prints the
    # figure; without reading it, a busy afternoon reads as an induced state.
    # NaN rather than 0 where the field is absent: a zero here would read as a
    # perfectly idle machine, which is the reading this exists to refuse.
    $rest = if ($null -ne $json.occupancy.rest_cores_median) { [double]$json.occupancy.rest_cores_median } else { [double]::NaN }
    $c    = ($rows | Where-Object { $_.C } | Measure-Object -Property C -Average).Average
    $perf = ($rows | Measure-Object -Property Perf -Average).Average
    $mhz  = ($rows | Measure-Object -Property MHz -Average).Average
    Write-Host ("  solo rate        {0} units/s" -f $rate)
    Write-Host ("  thermal zone     {0:N1} C" -f $c)
    Write-Host ("  % proc perf      {0:N1}" -f $perf)
    Write-Host ("  frequency        {0:N0} MHz" -f $mhz)
    Write-Host ("  cores elsewhere  {0:N2}" -f $rest)
    [pscustomobject]@{ Rate = [double]$rate; C = $c; Perf = $perf; MHz = $mhz; Rest = $rest }
}

$fast = Phase 'fast'

"=== burst: {0} sessions, {1} s (180 s was not enough) ===" -f $BurstSessions, $BurstSeconds
& $bench hold --label "thermal-burst-$($BurstSessions)x$($BurstSeconds)s" --sessions $BurstSessions --duration $BurstSeconds --interval 30 `
    --daemon "$root\target\release\coggyd.exe" -- $spin @work 2>&1 |
    Select-String -Pattern 'rate |peak rss' | ForEach-Object { "  " + $_.Line.Trim() }

$slow = Phase 'slow'

"`n=== verdict ==="
if ([Math]::Abs($fast.Rate / $slow.Rate - 1) -lt 0.15) {
    "  THE BURST DID NOT INDUCE THE STATE. Both phases are the same machine, so"
    "  the counters below are two readings of one state and settle nothing."
}
# **Read this line before the rate.** A rate that fell while the cores held by
# everything else rose has not measured the burst; it has measured a neighbour.
"  cores elsewhere {0,7:N2} -> {1,7:N2}   ({2:+0.00;-0.00})" -f $fast.Rest, $slow.Rest, ($slow.Rest - $fast.Rest)
if (($slow.Rest - $fast.Rest) -gt 1.0) {
    "  NOTE: {0:N2} more cores were held by something else during the slow phase." -f ($slow.Rest - $fast.Rest)
    "        The rate below cannot separate that from the burst. Name it with"
    "        Get-Counter '\Process(*)\% Processor Time' and run the probe again."
}
"  solo rate       {0,7:N2} -> {1,7:N2}   ({2:N2}x)" -f $fast.Rate, $slow.Rate, ($fast.Rate / $slow.Rate)
"  thermal zone    {0,7:N1} -> {1,7:N1}   ({2:+0.0;-0.0} C)" -f $fast.C, $slow.C, ($slow.C - $fast.C)
"  % proc perf     {0,7:N1} -> {1,7:N1}   ({2:N2}x)" -f $fast.Perf, $slow.Perf, ($fast.Perf / $slow.Perf)
"  frequency       {0,7:N0} -> {1,7:N0}   ({2:N2}x)" -f $fast.MHz, $slow.MHz, ($fast.MHz / $slow.MHz)
