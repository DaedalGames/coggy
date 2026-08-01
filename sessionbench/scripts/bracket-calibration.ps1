# Calibration for the bracket's solo allowance — see task #47.
#
# Five holds back to back: solo, eight concurrent, then THREE trailing solos.
# One trailing solo cannot tell a recovering point from a settled one; three
# give the recovery curve, and the curve's shape decides whether averaging two
# baselines is the right estimator at all.
#
# Run it as one script rather than five typed commands: the gaps between
# typing are the very drift being measured.
#
# READ THE OUTPUT, NOT THE EXIT CODE. Piping this script makes $LASTEXITCODE
# the last native command's, and neither `exit` nor `throw` survives that --
# a refusal came back as 0 twice while testing. A run that went ahead prints
# a line per hold; one that refused prints REFUSING and nothing else.
#
# Committed rather than left in bench-out because that directory is ignored,
# and work that exists only there is gone the moment the thing holding it
# stops. A measurement waiting on a window may wait across sessions.
#
# Reads its own precondition first. A ratio wants a steady machine rather than
# a quiet one, so this checks the SPREAD of consecutive readings, not one
# reading against a threshold.

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

$spin = "$root\target\release\cpu-spin.exe"
if (-not (Test-Path $spin)) { throw "build it: cargo build --release -p cpu-spin" }
if (-not (Test-Path "$root\target\release\coggyd.exe")) { throw "build it: cargo build --release -p coggyd" }

# --- precondition: is the background steady enough to see 6.9%?
$readings = @()
foreach ($i in 1..4) {
    $line = cargo run -q -p sessionbench -- doctor 2>&1 |
        Select-String -Pattern 'busy before we start ([\d.]+)'
    $readings += [double]$line.Matches.Groups[1].Value
    if ($i -lt 4) { Start-Sleep -Seconds 10 }
}
$stat = $readings | Measure-Object -Average -Maximum -Minimum
$spread = ($stat.Maximum - $stat.Minimum) / $stat.Average * 100
"background: {0} cores · mean {1:N0}% · spread {2:N1}%" -f ($readings -join ', '), ($stat.Average / 16 * 100), $spread
if ($spread -gt 10) {
    "REFUSING: spread {0:N1}% against a 6.9% effect — each hold would see a different machine." -f $spread
    throw "background not steady enough"
}

# --- the five holds. --units 10000 because cpu-spin's default is 60 and it
# EXITS after them, which would trip the fewest-alive guard two seconds in.
$workload = @('--units', '10000', '--duty', '1.0', '--resident', '20')
$plan = @(
    @{ label = 'cal-s0';   sessions = 1; duration = 20 },
    @{ label = 'cal-load'; sessions = 8; duration = 24 },
    @{ label = 'cal-s1';   sessions = 1; duration = 20 },
    @{ label = 'cal-s2';   sessions = 1; duration = 20 },
    @{ label = 'cal-s3';   sessions = 1; duration = 20 }
)
foreach ($step in $plan) {
    $out = cargo run -q --release -p sessionbench -- hold `
        --label $step.label --sessions $step.sessions --interval 4 --duration $step.duration `
        -- $spin @workload 2>&1
    $rate = ($out | Select-String -Pattern 'rate       ([\d.]+)').Matches.Groups[1].Value
    $alive = ($out | Select-String -Pattern 'fewest alive (\S+)').Matches.Groups[1].Value
    "{0,-9} {1,3} session(s)  {2,8} units/s/session  fewest alive {3}" -f $step.label, $step.sessions, $rate, $alive
}

"`nartifacts under bench-out/*-cal-*-daemon — read cpu_percent from each hold's"
"own <name>-samples.jsonl to separate 'did less with the same share' from"
"'was given less'. DO NOT PRUNE until the record is written."
