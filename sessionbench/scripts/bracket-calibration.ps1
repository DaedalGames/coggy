# Calibration for the bracket's solo allowance — see task #47.
#
# Per repeat: solo, eight concurrent, then THREE trailing solos. One trailing
# solo cannot tell a recovering point from a settled one; three give the
# recovery curve, and the curve's shape decides whether averaging two baselines
# is the right estimator at all. Flat means a step and averaging is wrong at
# any gap; climbing back means a ramp and averaging corrects it.
#
# THE SEQUENCE REPEATS, and that is the whole of how a 6.9% effect is separated
# from a machine that swings +-16% over a minute. Taken from `exclusion-delta`
# in this same crate, which had already solved it: adjacent halves share their
# background, repeats let the effect accumulate where noise does not, and the
# verdict is spread-across-repeats against the effect rather than a promise
# that the machine held still. Changed from its shape in one way — it pairs two
# states and this needs a curve, so a repeat is five holds rather than two.
#
# The earlier version instead waited for a quiet window: four `doctor` readings
# spread under 10%. It refused nineteen times across a day and never ran. That
# is the failure the borrowed design does not have, because it never asks.
#
# READ THE OUTPUT, NOT THE EXIT CODE. Piping this script makes $LASTEXITCODE
# the last native command's, and neither `exit` nor `throw` survives that --
# a refusal came back as 0 twice while testing. A run that went ahead prints
# a line per hold; one that refused prints REFUSING and nothing else.
#
# Committed rather than left in bench-out because that directory is ignored,
# and work that exists only there is gone the moment the thing holding it
# stops. A measurement waiting on a window may wait across sessions.

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

$spin = "$root\target\release\cpu-spin.exe"
if (-not (Test-Path $spin)) { throw "build it: cargo build --release -p cpu-spin" }
if (-not (Test-Path "$root\target\release\coggyd.exe")) { throw "build it: cargo build --release -p coggyd" }
# And the harness itself, which `cargo run --release` would otherwise build
# BETWEEN two holds -- a compile is exactly the thing the machine must not be
# doing while a rate is being read.
if (-not (Test-Path "$root\target\release\sessionbench.exe")) { throw "build it: cargo build --release -p sessionbench" }
$bench = "$root\target\release\sessionbench.exe"

# Pulls one capture out, or an empty string. `.Matches.Groups[1]` throws when
# nothing matched, and a throw fifty minutes in loses every hold before it --
# `rate` prints an em dash when the daemon never reported, which is exactly the
# case worth surviving to see.
function Get-Capture($text, $pattern) {
    $m = $text | Select-String -Pattern $pattern | Select-Object -First 1
    if ($null -eq $m) { return '' }
    $m.Matches[0].Groups[1].Value
}

function Get-Busy {
    $busy = Get-Capture (& $bench doctor 2>&1) 'busy before we start ([\d.]+)'
    if ($busy -eq '') { throw "doctor printed no background figure" }
    [double]$busy
}

# --- the one precondition left, and it is about the LOAD rather than steadiness.
# The burst has to be the thing that changed the machine, so its eight sessions
# need eight free cores. Half of sixteen busy is where that stops being true.
# Nothing else is gated: what the background did is measured by the repeats.
$before = Get-Busy
"background before: {0:N2} cores ({1:N0}%)" -f $before, ($before / 16 * 100)
if ($before -gt 8) {
    "REFUSING: {0:N1} of 16 cores busy — the eight-session burst would be competing, not saturating." -f $before
    throw "no room for the load"
}

# --- five holds a repeat. --units 10000 because cpu-spin's default is 60 and
# it EXITS after them, which would trip the fewest-alive guard two seconds in.
#
# Twenty seconds a solo, which is short enough that starting the daemon and its
# sessions is 2.2% of the counted window -- measured, and it CANCELS here
# because all four solos hold one session and pay it alike. The deficit is
# between them, not against the eight-session load, whose rate is never used.
# Short holds are also the only way to resolve a transient, which is the whole
# quantity being looked for.
$workload = @('--units', '10000', '--duty', '1.0', '--resident', '20')
$plan = @(
    @{ key = 's0';   sessions = 1; duration = 20 },
    @{ key = 'load'; sessions = 8; duration = 24 },
    @{ key = 's1';   sessions = 1; duration = 20 },
    @{ key = 's2';   sessions = 1; duration = 20 },
    @{ key = 's3';   sessions = 1; duration = 20 }
)

$rates = @{}
foreach ($step in $plan) { $rates[$step.key] = @() }

foreach ($repeat in 1..3) {
    "`n--- repeat $repeat"
    foreach ($step in $plan) {
        $label = "cal-r$repeat-$($step.key)"
        $out = & $bench hold `
            --label $label --sessions $step.sessions --interval 4 --duration $step.duration `
            -- $spin @workload 2>&1
        $rate = Get-Capture $out 'rate       ([\d.]+)'
        $alive = Get-Capture $out 'fewest alive (\w+\(?\d*\)?)'
        $window = Get-Capture $out 'window     (\d+ ms counted of \d+ ms held)'
        if ($rate -eq '') {
            $out | Select-Object -Last 20
            throw "$label produced no rate -- output above"
        }
        $rates[$step.key] += [double]$rate
        "{0,-12} {1,3} session(s)  {2,8} units/s/session  alive {3}  {4}" -f $label, $step.sessions, $rate, $alive, $window
    }
}

$after = Get-Busy
"`nbackground after: {0:N2} cores ({1:N0}%)" -f $after, ($after / 16 * 100)

# --- the curve, per repeat, as a deficit against that repeat's own s0.
"`ndeficit against each repeat's own pre-load solo, in percent:"
"{0,-8} {1,8} {2,8} {3,8}" -f 'repeat', 's1', 's2', 's3'
$deficits = @{ s1 = @(); s2 = @(); s3 = @() }
foreach ($i in 0..2) {
    $base = $rates['s0'][$i]
    $row = @()
    foreach ($k in 's1', 's2', 's3') {
        $d = ($base - $rates[$k][$i]) / $base * 100
        $deficits[$k] += $d
        $row += '{0:N2}' -f $d
    }
    "{0,-8} {1,8} {2,8} {3,8}" -f ($i + 1), $row[0], $row[1], $row[2]
}

# --- exclusion-delta's verdict rule, in its own words: a range that contains
# zero has not established a direction, and one wider than its own centre has
# not established a size. Averaging a noisy repeat with another noisy repeat
# produces a confident number and no more information.
"`n{0,-8} {1,8} {2,8} {3,8}" -f 'point', 'mean', 'low', 'high'
foreach ($k in 's1', 's2', 's3') {
    $s = $deficits[$k] | Measure-Object -Average -Minimum -Maximum
    $straddles = $s.Minimum -le 0 -and $s.Maximum -ge 0
    $wide = ($s.Maximum - $s.Minimum) -gt [Math]::Abs($s.Average)
    $verdict = if ($straddles -or $wide) { 'inconclusive' } else { 'established' }
    "{0,-8} {1,8:N2} {2,8:N2} {3,8:N2}  {4}" -f $k, $s.Average, $s.Minimum, $s.Maximum, $verdict
}

"`nA flat established curve is a step and the bracket must not average its two"
"baselines. One decaying toward zero is a ramp and averaging is the right"
"estimator. Anything inconclusive is neither, and repeats are what buys it."
"`nartifacts under bench-out/*-cal-*-daemon — read cpu_percent from each hold's"
"own <name>-samples.jsonl to separate 'did less with the same share' from"
"'was given less'. DO NOT PRUNE until the record is written."
