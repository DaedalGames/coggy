# Sweeps one workload knob and reads total throughput, with references between.
#
# WHAT IT MEASURES. Past saturation a session's rate is `eta*C / (w*N)`, so the
# total across a hold is `eta*C / w` — the session count and the duty cancel, and
# two holds run back to back share the machine's core count and its unit time.
# Their totals are therefore equal exactly when their `eta` are. **No solo
# baseline is involved**, which is the point: a solo hold is the loudest term in
# a slowdown, spreading 4-8% across six holds where a hundred-session reference
# holds to 0.94%, because one session takes a disturbance at full strength and a
# hundred divide it. The 25% figure this comment used to quote is one afternoon
# whose background collapsed from 28% to 9% mid-run; three other sets the same
# day were quiet.
#
# WHY IT EXISTS AS A FILE. This shape was written inline four times in one day,
# into bench-out/*.ps1, which is gitignored — so each run took its script with
# it and the next rewrote the same thing. Both runs it is for wait on a window
# that has not opened, which is exactly the case the repository's rules put in
# scripts/ rather than in a note.
#
# THE TWO RUNS IT IS FOR:
#
#   -Sweep duty -Values 0.06,0.10,0.14,0.18 -Reference 0.27
#     Finds the knee. Below saturation the total rises with duty because every
#     session gets what it asks for; above it the total is flat at `eta*C / w`.
#     Where the curve bends gives `eta*C` with no solo and no slowdown in it,
#     which is the only independent route to eta's value that exists.
#
#   -Sweep resident -Values 20,40,80,160 -Reference 20
#     Finds how eta moves with footprint. Four figures in the records split by
#     workload rather than scattering — 0.73-0.78 at 20 MiB and 0.84-0.93 at 80
#     — and a real session holds 2.39 GiB, thirty times the heavier of them.
#
#   -Sweep sessions -Values 8,12,16,20,24,28 -Reference 100
#     Finds the knee in N, which is where the redline actually lives: total
#     throughput rises with the session count until the machine is claimed and
#     is flat after, so the redline is twice the bend. Both `w` and `d` cancel
#     out of `plateau / rising-slope`, so nothing here divides by a solo rung.
#     It also tests the assumption underneath that: the rise is only a straight
#     line if sessions below saturation cost each other nothing, and several
#     points on the way up are what say whether it curves.
#
#     **Put every point well under the knee.** At duty 0.27 the knee sits near
#     `eta*C/d` = 43, and a first draft of this example swept to 80 — where four
#     of its seven points are on the plateau, get dropped for being over 85% of
#     it, and leave three to carry the slope. A rounded corner takes the third
#     as well. Two thirds of the expected knee is the last point worth spending,
#     and the knee moves with duty, so recompute the ceiling when that changes.
#
# READ THE OUTPUT, NOT THE EXIT CODE. Piping this makes $LASTEXITCODE the last
# native command's, and neither `exit` nor `throw` survives that. A run that
# went ahead prints a line per hold; one that refused prints REFUSING and stops.

param(
    [Parameter(Mandatory)][ValidateSet('duty', 'resident', 'sessions')][string]$Sweep,
    [Parameter(Mandatory)][double[]]$Values,
    [Parameter(Mandatory)][double]$Reference,
    [int]$Sessions = 100,
    [double]$Duty = 0.27,
    [double]$Resident = 20,
    [int]$HoldSeconds = 240
)

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

cargo build --release -p coggyd -p cpu-spin -p sessionbench
$spin = "$root\target\release\cpu-spin.exe"
$bench = "$root\target\release\sessionbench.exe"

# --- survivors first. A killed sweep leaves its own, and they would be counted.
$stray = Get-Process coggyd, cpu-spin -ErrorAction SilentlyContinue
if ($stray) {
    "REFUSING: {0} stray process(es) already running" -f $stray.Count
    throw "the machine is not clean"
}

# --- and the plug. The same command returns 7.8x less on battery with every
# other recorded figure identical: a hundred sessions alive, RSS to a hundredth
# of a gibibyte, a steady climb rather than a stall, and a thermal zone at 42 C.
# A run that is merely slow produces a clean artifact, so this is the one
# precondition that cannot be left to whoever reads the result.
$health = & $bench doctor 2>&1
if ($health | Select-String -Pattern 'ON BATTERY' -Quiet) {
    "REFUSING: on battery — every figure would be a different machine's."
    ($health | Select-String -Pattern 'ON BATTERY').Line
    throw "plug it in"
}
($health | Select-String -Pattern 'busy before we start|on mains').Line

# --units 100000000 because cpu-spin's default is 60 and it EXITS after them,
# which would empty the hold two seconds in.
function Work([double]$duty, [double]$resident) {
    @('--units', '100000000', '--duty', "$duty", '--resident', "$resident")
}

# One swept axis at a time; the other two hold at their defaults. Sessions is
# not a workload argument, so a step carries its own count rather than sharing
# the script's -- which is the whole point of the `sessions` sweep.
function Step([string]$name, [double]$v, [string]$tag) {
    switch ($Sweep) {
        'duty' { @{ n = $name; work = (Work $v $Resident); sessions = $Sessions; tag = $tag } }
        'resident' { @{ n = $name; work = (Work $Duty $v); sessions = $Sessions; tag = $tag } }
        'sessions' { @{ n = $name; work = (Work $Duty $Resident); sessions = [int]$v; tag = $tag } }
    }
}

# References at both ends and between every pair, so each swept point sits
# inside a bracket rather than beside one. The gap between two references is the
# drift, and a point is only readable when the effect exceeds it.
$plan = @(Step 'ref0' $Reference 'reference')
for ($i = 0; $i -lt $Values.Count; $i++) {
    $plan += Step "v$i" $Values[$i] "$Sweep=$($Values[$i])"
    $plan += Step "ref$($i+1)" $Reference 'reference'
}

# --- for a sessions sweep, say before spending the time whether the points can
# reach the knee at all. The knee sits near `eta*C/d`, and a point at or past it
# lands on the plateau, gets dropped for being over 85% of it, and leaves fewer
# behind to carry the slope. A first draft of the documented example swept to 80
# against a knee near 43 and would have finished with three usable points, or
# two once the corner's rounding took the third.
if ($Sweep -eq 'sessions') {
    $expected = 0.733 * 16 / $Duty      # eta and C for this box; recompute elsewhere
    $tooHigh = @($Values | Where-Object { $_ -gt $expected * 0.67 })
    $usable = $Values.Count - $tooHigh.Count
    "`nknee expected near {0:N0} sessions at duty {1} — points above {2:N0} will land on the shoulder" -f `
        $expected, $Duty, ($expected * 0.67)
    if ($tooHigh.Count -gt 0) {
        "  {0} of {1} points are above it: {2}" -f $tooHigh.Count, $Values.Count, ($tooHigh -join ', ')
    }
    if ($usable -lt 3) {
        "REFUSING: only {0} point(s) sit low enough to carry the rise, and a slope through" -f $usable
        "two survives no rounding at the corner. Sweep counts under {0:N0}." -f ($expected * 0.67)
        throw "the sweep cannot reach the knee"
    }
}

# --- clean up after itself, in the order that works: `sessionbench` owns the
# job, so stopping it reaps the tree, while stopping the shell above it only
# orphans a hundred sessions.
#
# **It covers Ctrl-C and a throw, and not a killed host** — which is the case it
# was written for, and it does not cover it. Tested by force-killing the shell
# mid-sweep: forty-one processes survived, because `Stop-Process -Force` gives
# the host no chance to run anything. Nothing inside a script can defend against
# its own process disappearing. What covers that is the survivor check at the
# top of the next run, and knowing to stop `sessionbench` rather than the shell.
trap {
    "`nSWEEP INTERRUPTED — reaping"
    Get-Process sessionbench -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Seconds 4
    $left = Get-Process coggyd, cpu-spin -ErrorAction SilentlyContinue
    if ($left) {
        "  the job did not take them; killing {0} directly" -f $left.Count
        $left | Stop-Process -Force
        Start-Sleep -Seconds 2
    }
    $final = Get-Process coggyd, cpu-spin, sessionbench -ErrorAction SilentlyContinue
    if ($final) { "  STILL STRAY: {0} — kill by hand" -f $final.Count }
    else { "  no survivors" }
    break
}

"`nsweeping $Sweep over $($Values -join ', ') against $Reference — $($plan.Count) holds of ${HoldSeconds}s"
$totals = @{}
foreach ($step in $plan) {
    $out = & $bench hold --label "sw-$($step.n)" --sessions $step.sessions --interval 5 `
        --duration $HoldSeconds --rss-budget-gb 8 -- $spin @($step.work) 2>&1
    $m = $out | Select-String -Pattern 'rate       ([\d.]+)' | Select-Object -First 1
    if ($null -eq $m) { $out | Select-Object -Last 20; throw "$($step.n) produced no rate" }
    $per = [double]$m.Matches[0].Groups[1].Value
    # Total, not per session — it is what cancels the core count and the unit
    # time between adjacent holds, and it is the quantity whose bend is the knee.
    $totals[$step.n] = $per * $step.sessions
    "{0,-6} {1,-16} {2,4} sess {3,8:N3} each {4,9:N1} total" -f `
        $step.n, $step.tag, $step.sessions, $per, ($per * $step.sessions)
}

# --- the drift is what says whether any of it is readable.
$refs = $plan | Where-Object { $_.tag -eq 'reference' } | ForEach-Object { $totals[$_.n] }
$stat = $refs | Measure-Object -Average -Minimum -Maximum
$drift = ($stat.Maximum - $stat.Minimum) / $stat.Average * 100
"`nreferences: {0}" -f (($refs | ForEach-Object { '{0:N1}' -f $_ }) -join ', ')
"drift across them: {0:N2}%  — an effect smaller than this is not an effect" -f $drift

# --- for a sessions sweep, locate the bend rather than leaving it to the eye.
# Below saturation the total rises with slope d/w; above it is flat at eta*C/w.
# The knee is plateau / slope, and the redline is twice it. Both w and d cancel.
if ($Sweep -eq 'sessions') {
    $pts = @()
    for ($i = 0; $i -lt $Values.Count; $i++) { $pts += , @([double]$Values[$i], $totals["v$i"]) }
    $pts = $pts | Sort-Object { $_[0] }
    $plateau = ($refs | Measure-Object -Average).Average

    # The rising part is whatever sits far enough under the plateau to be on the
    # slope rather than on the shoulder. Two points at minimum, or say so.
    $rising = $pts | Where-Object { $_[1] -lt $plateau * 0.85 }
    "`nplateau (references): {0:N1}" -f $plateau
    if ($rising.Count -lt 2) {
        "KNEE UNREADABLE: only {0} point(s) below 85% of the plateau — sweep lower session counts." -f $rising.Count
    }
    else {
        # Slope through the origin, which is what `total = N*d/w` says it is.
        $slope = ($rising | ForEach-Object { $_[1] * $_[0] } | Measure-Object -Sum).Sum /
                 ($rising | ForEach-Object { $_[0] * $_[0] } | Measure-Object -Sum).Sum

        # **And then check the line it just drew.** Being under 85% of the
        # plateau does not put a point on the rise -- a smoke run with two
        # points at 2 and 4 sessions cleared that bar, sat on the shoulder
        # anyway, and returned a knee of 5.5 where the model says 11.7. A
        # curved rise has no one slope, and the residual is what says so.
        $worst = 0.0
        foreach ($p in $rising) {
            $err = [Math]::Abs($p[1] - $slope * $p[0]) / ($slope * $p[0]) * 100
            if ($err -gt $worst) { $worst = $err }
        }
        "rising slope over {0} point(s): {1:N2} units/s per session" -f $rising.Count, $slope
        "worst residual against it:     {0:N2}%" -f $worst
        foreach ($p in $rising) {
            $fit = $slope * $p[0]
            "   N={0,-5:N0} measured {1,8:N1}   line {2,8:N1}   {3,7:N1}% off" -f `
                $p[0], $p[1], $fit, (($p[1] - $fit) / $fit * 100)
        }
        if ($worst -gt 5.0) {
            "`nKNEE UNREADABLE: the rise bends by {0:N1}%, so it has no single slope." -f $worst
            "Sweep lower session counts until the low points fall on one line."
        }
        else {
            "`nknee    = plateau / slope = {0:N1} sessions" -f ($plateau / $slope)
            "redline = 2 x knee        = {0:N1} sessions" -f (2 * ($plateau / $slope))
        }
    }
}

"`nDO NOT PRUNE bench-out until the record is written."
