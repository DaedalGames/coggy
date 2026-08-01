# Gate M1's hour — see task #45.
#
# A hundred sessions under coggyd for an hour, with a baseline on either side.
# Roughly seventy-six minutes end to end once the baselines are counted, and
# the machine must be left alone for all of it: the observer is not free.
#
# Two of the gate's conditions come back out_of_reach whatever this does, and
# the report says so rather than passing them. This run answers RSS against
# the 4 GB budget and work rate as a ratio against its own bracketing solos.
#
# LAUNCH IT DETACHED. Whatever starts this must outlive it, and the first
# attempt did not: an agent's background task carries a ten-minute ceiling, so
# the probes and three two-minute baselines used the whole budget and the run
# was killed the second the hour began. Nothing was left behind -- a hundred
# sessions went down with it and no process survived, which is the job objects
# working -- but seventy-six minutes of machine time bought six minutes of
# baseline. Start it so its lifetime is nobody else's:
#
#   Start-Process pwsh -ArgumentList '-NoProfile','-File','<this>' `
#       -RedirectStandardOutput m1.log -RedirectStandardError m1.err
#
# then read m1.log. The run prints a line per phase, which is what tells a
# working run from a hung one.
#
# READ THE OUTPUT, NOT THE EXIT CODE. Piping this script makes $LASTEXITCODE
# the last native command's, and neither `exit` nor `throw` survives that. A
# run that went ahead prints "starting" with a minute count; one that refused
# prints REFUSING and stops there.
#
# THE AS-IS COLUMN IS NOT THE 2026-08-01 HOUR-LONG HOLD. That run held `ping`
# at about 5.8 MiB a session; this one holds cpu-spin at --resident 20, so the
# totals differ by roughly 2 GiB of workload memory before anything about
# coggyd enters. What compares is the daemon's own cost per session held, the
# peak against the gate's stated 4 GB, and the two solo sides against each
# other.
#
# IT IS --wait-ms AND NOT --duty, WHICH COST A RUN TO LEARN. `--duty` derives
# each pause from how long that unit actually took, so contention stretches the
# unit and stretches the pause with it: the workload backs off instead of
# competing. Measured mid-run at a hundred sessions -- 0.0655 cores a session
# against 0.27 requested, the daemon at 0.006, and 6.7 of sixteen cores sitting
# idle while a hundred sessions that wanted twenty-seven of them slept. A
# workload that cannot saturate cannot produce a work-rate ceiling.
#
# `--wait-ms` holds the pause fixed, which is the shape a session waiting on a
# model has -- its duty climbs as its compute slows. 67 ms against a 24.9 ms
# unit gives the same 0.27 solo duty and lets the count decide the rest.
#
# The workload's own source says all of this three lines above the flag. The
# value was read back from what consumes it and the flag was not.
#
# THE DUTY IS 0.27 AND THAT IS NOT A DETAIL. Read the gate's own arithmetic
# before spending an hour on it: one session at duty d wants d cores, a hundred
# want 100d, and sixteen is what there is. Past saturation each session gets
# 0.16 cores instead of d, so the slowdown is d/0.16 -- and the gate asks for
# 2x. That makes d <= 0.32 the condition for M1 to be passable on this machine
# at all, and `--duty 1.0`, which this script used to pass, a guaranteed break
# discoverable by division. 0.27 is [the measured driven duty of a real agent
# turn](../../docs/measurements/2026-07-31-054657-the-driven-duty.md), giving a
# predicted 1.69x. The margin is thin on purpose: a gate nothing can fail is
# not a gate, and this one is decided by the workload it is handed.
#
# THE PRECONDITION CHECKS THE QUANTITY THAT DECIDES, which took a day to get
# right. An earlier version wanted four `doctor` readings within 10% of each
# other; its sibling script asked the same and was refused nineteen times in a
# day without ever running. Background steadiness over thirty seconds does not
# predict a gap between two baselines an hour apart, and what actually loses
# this run is those baselines disagreeing -- which is measured by taking two of
# them. A minute here against seventy-six is worth it BECAUSE the run is long;
# the same trade is why the calibration script has no precondition at all.

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

# Everything built before anything is measured. `cargo run` would otherwise
# compile between two holds, which is the one thing the machine must not do.
cargo build --release -p coggyd -p cpu-spin -p sessionbench
$spin = "$root\target\release\cpu-spin.exe"
$bench = "$root\target\release\sessionbench.exe"

function Get-Capture($text, $pattern) {
    $m = $text | Select-String -Pattern $pattern | Select-Object -First 1
    if ($null -eq $m) { return '' }
    $m.Matches[0].Groups[1].Value
}

# --- survivors first, per the standing rule. A killed ramp leaves its own.
$stray = Get-Process coggyd, cpu-spin, ping -ErrorAction SilentlyContinue
if ($stray) {
    "REFUSING: {0} stray process(es) already running" -f $stray.Count
    throw "the machine is not clean"
}

# --- background, recorded rather than gated. It goes in the record's
# provenance and it is not what decides.
$busy = Get-Capture (& $bench doctor 2>&1) 'busy before we start ([\d.]+)'
"background: {0} of 16 logical" -f $busy

# --- the precondition: can two baselines agree today? --units 100000000
# because cpu-spin's default is 60 and it EXITS after them.
$work = @('--units', '100000000', '--wait-ms', '67', '--resident', '20')
$probe = foreach ($i in 1..2) {
    $out = & $bench hold --label "m1-probe-$i" --sessions 1 --interval 5 --duration 30 -- $spin @work 2>&1
    $r = Get-Capture $out 'rate       ([\d.]+)'
    if ($r -eq '') { $out | Select-Object -Last 20; throw "probe $i produced no rate" }
    [double]$r
}
$gap = [Math]::Abs($probe[0] - $probe[1]) / (($probe[0] + $probe[1]) / 2) * 100
"probe baselines: {0} and {1} units/s · gap {2:N1}%" -f $probe[0], $probe[1], $gap
if ($gap -gt 5) {
    "REFUSING: two fresh solo holds sit {0:N1}% apart, past the 5% the run will be judged by." -f $gap
    "Placement noise alone is worth about 4.5% here; more than that is the machine."
    throw "the baseline cannot support the judgement"
}

"`nstarting — about 76 minutes, leave the machine alone"
& $bench hold `
    --label m1 --sessions 100 --interval 5 --duration 3600 `
    --with-solo --solo-duration 120 --solo-repeats 3 `
    -- $spin @work

"`nDO NOT PRUNE bench-out until the record is written."
