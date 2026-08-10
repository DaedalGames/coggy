# Gate M1's hour — see task #45.
#
# A hundred sessions under coggyd for an hour, with a baseline on either side.
# End to end that is the hold plus ten two-minute baselines and their
# warm-ups -- about 48 minutes at the default twenty-minute hold, about 88
# if the hold is given the full hour -- and the machine must be left alone
# for all of it: the observer is not free.
#
# FIVE BASELINES A SIDE, NOT THREE, AND THE REASON IS ARITHMETIC RATHER THAN
# CAUTION. A solo hold's own sigma is 6% here, pooled over thirteen holds at
# matched tenancy, so three a side puts the standard error of the difference
# at 4.90% and the 5% allowance at 1.02 of it -- a coin weighted 2:1 that
# refuses roughly a third of runs on a machine that never moved, paid on the
# measurement hardest to repeat. Five brings that to 19% for four extra
# holds. Nine would reach 8% and costs eighteen baseline holds, which is
# more baseline than the hold it brackets on a box whose quiet stretches do
# not reach ninety seconds.
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
# THE PROBE CHECKS TWO THINGS AND ONLY ONE OF THEM IS A GAP. Two fresh solo
# holds agreeing says the machine is steady; it does not say which machine.
# This box runs about 18.9 units/s solo at this workload rested and ~9.0 after a
# saturating burst, and two probes taken inside the slow state agree with each
# other to 0.3% while the run they precede reports a slowdown 72% higher. The
# script now prints a NOTE when the level says post-saturation, and does not
# refuse on it -- the run is valid, it is just a figure for a different machine
# state, and refusing would have cancelled the run that found this.
#
# USE --duty, AND THE ROUTE TO THAT WAS WRONG TWICE. A mid-run reading said
# `--duty` backs off rather than competing -- 0.0655 cores a session against
# 0.27 requested, six of sixteen cores idle -- and that aborted a gate run and
# moved this script to `--wait-ms`. [It did not
# reproduce](../../docs/measurements/2026-08-01-210316-the-wait-mechanism-cancels.md):
# back to back at a hundred sessions the two flags take 15.32 and 15.57 cores
# and their slowdowns sit 2.16% apart with solo fingerprints 0.33% apart. The
# mechanism cancels, exactly as the workload's own source says it does.
#
# So pick on the surviving argument instead. `--wait-ms` holds a wall-clock
# constant, so its duty moves with the machine: calibrated warm and run cold it
# delivered 0.172 where 0.271 was asked for, because a 79% change in compute
# speed arrives as 13.6% of rate and is quiet enough to miss. `--duty` is
# self-calibrating and delivers the duty on any machine. **A gate stated in duty
# wants the flag that delivers a duty**, and since the slowdown is the same
# either way that costs nothing.
#
# `--wait-ms` is still the more faithful shape of a session waiting on a model,
# whose duty really does climb as its compute slows. It is the right flag for
# realism and the wrong one for a stated parameter.
#
# AND IT IS THE LOAD THAT STOPPED THIS BOX. A climbing duty on a slowing
# machine is a positive feedback -- less speed, more demand -- and the only
# run that ended in a hard stop was forty-one minutes of exactly that, its
# duty travelling from 0.172 toward 0.271 as the box heated. Five holds at a
# fixed 0.27 have finished clean at twenty minutes. So `--duty` is not only
# the flag that delivers a stated parameter; it is the one whose load does
# not run away, and the hour is untried under it rather than tried and
# failed.
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

# THE HOUR IS A PARAMETER BECAUSE THIS MACHINE MAY NOT HAVE ONE. A full-load
# run stopped the box dead at forty-one minutes -- event 41, bugcheck 0, no
# power button, no update, and no System log for the four minutes before it. On
# AC at 79%, so not a flat battery; a thin laptop holding sixteen cores at 100%
# will reach either the adapter's budget or its thermal limit, and both end
# exactly like that. A hundred `ping` sessions held an hour on this same box, so
# what it cannot sustain is the saturation the work-rate condition requires,
# not the session count.
#
# Shrink the measurement before blaming the machine. RSS and work rate settle in
# minutes; the only thing an hour buys is the sentence "held for an hour". Run
# twenty minutes to get every number at a third of the exposure, and say plainly
# in the record that the duration claim is unmet.
param([int]$DurationSeconds = 1200)

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

# --- and what that leaves the concurrent hold, which the probe below cannot
# see. The precondition is two SOLO holds agreeing within 5%, and agreement is
# not a level: two probes taken inside the same disturbance agree with each
# other and are both wrong.
#
# A SOLO IS NOT IMMUNE, WHICH IS NOT WHAT THIS COMMENT FIRST SAID. It claimed a
# solo needs one core and therefore runs at full speed under a tenant. Measured:
# 11.5 of 16 cores held costs a single session 26%, 17.561 units/s against
# 13.002. So the probes do slow -- just far less than the hundred-session hold
# they are guarding, and by an amount the 5% agreement gate never sees.
#
# WORSE, THE LEVEL CHECK BELOW MISNAMES IT. Probes averaging under 15 units/s
# are reported as the post-saturation machine state. A tenant lands in the same
# place -- three solos under one averaged exactly 15.0 -- and the two have
# different remedies: waiting an hour fixes the state and does nothing about a
# tenant. What separates them is this line: the slow state leaves the machine
# idle, a tenant does not.
#
# THE THRESHOLD IS NOT LOW, AND THAT IS DELIBERATE. Ordinary background here
# reads about 2.2 cores idle and collapses to roughly 0.6 once a hundred
# sessions compete -- it yields, so pre-screening a ratio on it measures the
# wrong thing, which is why the standing rule says a ratio is refused by its own
# bracket rather than by `doctor`. What this looks for is the other kind: a
# tenant that keeps computing under load. On 2026-08-03 this box sat at 10.4 of
# 16 for over twenty minutes, one headless browser holding 7.6 cores by itself,
# with `doctor` reading a steady 65-67% and nothing here saying so.
#
# A quarter of the machine is above anything idle background has reached on this
# box and below anything a real tenant leaves. It is a heuristic and it is not a
# refusal; the number that decides is the run's own `occupancy`, which reports
# the cores the job actually held and now carries the whole machine's CPU beside
# it. Read that line before the slowdown.
$free = 16 - [double]$busy
"free for the run: {0:N2} of 16 logical" -f $free
if ([double]$busy -gt 4.0) {
    "NOTE: something else holds {0:N2} cores, a quarter of the machine or more." -f ([double]$busy)
    "      That is a tenant rather than idle background, which yields under load."
    "      The slowdown will read high in proportion, and the solo probes below"
    "      will NOT notice, because a solo needs one core and {0:N2} are free." -f $free
    "      Name it before spending the hour:"
    "        Get-Counter '\Process(*)\% Processor Time'"
    "      Get-Process | Sort-Object CPU does not answer this -- that column is"
    "      cumulative lifetime seconds, not a current rate."
}

# --- the precondition: can two baselines agree today? --units 100000000
# because cpu-spin's default is 60 and it EXITS after them.
$work = @('--units', '100000000', '--duty', '0.27', '--resident', '20')
$probeRest = @()
$probe = foreach ($i in 1..2) {
    $out = & $bench hold --label "m1-probe-$i" --sessions 1 --interval 5 --duration 30 -- $spin @work 2>&1
    $r = Get-Capture $out 'rate       ([\d.]+)'
    if ($r -eq '') { $out | Select-Object -Last 20; throw "probe $i produced no rate" }
    $script:probeRest += [double](Get-Capture $out '([\d.]+) cores held outside the job')
    [double]$r
}
$gap = [Math]::Abs($probe[0] - $probe[1]) / (($probe[0] + $probe[1]) / 2) * 100
"probe baselines: {0} and {1} units/s · gap {2:N1}%" -f $probe[0], $probe[1], $gap

# --- and which machine state they agree in, which the gap cannot tell you.
# This box gives about 18.9 units/s solo at this workload when rested -- measured
# twice, two days apart, 0.15% apart. Three minutes of a hundred saturating
# sessions halves that for about ninety minutes, and two probes taken inside
# that state agree with each other to under a percent while the run they
# precede reports a slowdown 72% higher. So the agreement is necessary and
# says nothing about the state; the level does.
#
# **And the level does not either, which is why one more probe runs below.**
# Two runs whose solo holds agreed to half a percent, 9.752 and 9.801, held a
# hundred sessions at 246.4 and 902.8 units/s -- one box crippled under load,
# one 0.5% from its rested reference with only the lone session down. They move
# the slowdown opposite ways: 3.958, and a 1.54 that PASSES a condition asking
# for 2. Ninety seconds of a hundred sessions is what separates them, against
# an hour that would otherwise be spent measuring the afternoon.
$mean = ($probe[0] + $probe[1]) / 2

$cout = & $bench hold --label 'm1-probe-load' --sessions 100 --interval 5 --duration 60 -- $spin @work 2>&1
$ctotal = Get-Capture $cout 'total +([\d.]+)'
if ($ctotal -eq '') { $cout | Select-Object -Last 20; throw 'the load probe produced no total' }
$ctotal = [double]$ctotal
$crest = [double](Get-Capture $cout '([\d.]+) cores held outside the job')
$srest = ($probeRest | Measure-Object -Average).Average

"load probe: {0:N1} units/s across a hundred · {1:N2} cores held outside the job" -f $ctotal, $crest
"solo rest:  {0:N2} cores" -f $srest

# The pair, read together. Neither number alone survives: a tenanted hundred
# lands at 344.9 and 297.3, and a QUIET one has landed at 756.1, so the span
# between the clusters is not empty and these labels are verdicts rather than
# descriptions. What each branch decides is still right; what none of them can
# claim is that the box has exactly three states.
if ($crest -gt 2.5 -or $srest -gt 2.5) {
    "STATE: a tenant is present ({0:N2} cores during the load probe). Nothing" -f $crest
    "       below names the machine's own state while someone else holds it."
    "       Find it with Get-Counter '\Process(*)\% Processor Time'."
} elseif ($ctotal -lt 600) {
    "STATE: machine-slow -- a hundred sessions produced {0:N1} against 903-1055" -f $ctotal
    "       rested. The slowdown will read HIGH and the gate will fail by more"
    "       than it should. Waiting is the only lever."
} elseif ($mean -lt 15) {
    "STATE: solo-slow -- the hundred ran {0:N1}, which is normal, while one" -f $ctotal
    "       session runs {0:N1} against ~18.9 rested. The slowdown will read LOW" -f $mean
    "       and CAN PASS THE GATE FOR THE WRONG REASON. Do not spend the hour."
} else {
    "STATE: rested -- solo {0:N1}, hundred {1:N1}. This is the state the hour" -f $mean, $ctotal
    "       needs, and it has been the rare one."
}
# **A mean of two that disagree names nothing**, and this block used to run
# before the agreement check below. On 2026-08-03 a quiet box gave 15.216 then
# 12.636 in consecutive holds -- 18.6% apart, mean 13.9, which points at the
# tenanted band while neither reading was tenanted. Say the pair, and let the
# refusal below have the last word on whether the mean was worth forming.
"probes: {0:N3} and {1:N3} units/s, {2:N1}% apart" -f $probe[0], $probe[1], $gap
if ($gap -gt 5) {
    "  they disagree, so the level below names a band the machine may not be in"
}
# **A 30-second probe may not be placed against a 120-second band.** Eight
# probes of this exact length averaged 15.10 units/s on an afternoon when
# 120-second solo holds ran about 11.2 -- 35% apart, same box and workload,
# so a short hold reads a different LEVEL rather than a noisier version of
# the same one. The bands below were measured at 120 s. The comparison is
# kept because it is the only reading available before the run, and it is
# labelled rather than trusted; the precondition above is unaffected, since
# it compares two probes of equal length with each other.
if ($mean -lt 15) {
    "NOTE: these probes are 30 s and the bands below were measured at 120 s,"
    "      where the same box read about 35% lower. Treat the band as a hint."
    "NOTE: {0:N1} units/s is not the rested state (~19-21). Three bands were" -f $mean
    "      measured on this box on 2026-08-03, all at this workload:"
    "        rested          18.9   (quiet, 1-3 cores held; TWO HOLDS ONLY)"
    "        a tenant        13.8   (10-13 cores held elsewhere)"
    "        the slow state   9.1   (quiet, and half speed anyway)"
    "      So the level names the cause on its own here, and the background"
    "      line above confirms it: a tenant leaves cores held, the slow state"
    "      does not. Note the order -- a CROWDED RESTED box outruns a QUIET"
    "      SLOW one, so a middling figure is not a middling machine."
    if ($mean -lt 11.5) {
        "      This one is a slow state -- and there are TWO, which this level"
        "      cannot tell apart. On 2026-08-03 solo holds of 9.752 and 9.801,"
        "      half a percent apart, sat on boxes whose hundred sessions ran"
        "      246.4 and 902.8 units/s: one crippled under load, one 0.5% from"
        "      its rested reference with only the lone session down."
        "      They push the slowdown OPPOSITE WAYS -- 3.958, and 1.54 which"
        "      PASSES a gate asking for 2, for the wrong reason. So do not"
        "      expect a high reading here, and do not read a low one as recovery."
        "      Waiting an hour is still the only lever; the run's own total"
        "      throughput is what says which state it was taken in."
    } elseif ($mean -lt 16.5) {
        "      This one is a tenant. Find it before spending the hour."
    }
    "      Let the box sit for an hour if the figure is meant to be compared"
    "      with a rested one."
}
# TWO PROBES MAKE THIS A SPREAD, so it doubles as the fitness check the
# bracket prints. A healthy box here reproduces a solo hold to 0.42% across
# six of them -- the spread behind the gate 2.0654, with 0.39% and 0.52%
# behind the others. Anything past about 1.5% is the machine rather than the
# instrument, and no repeat count fixes it: more samples of a wandering box
# is not what is missing.
if ($gap -gt 1.5 -and $gap -le 5) {
    "NOTE: {0:N1}% between two fresh solo holds is above the ~1.5% a fit machine shows." -f $gap
    "      Healthy brackets here spread 0.42%. The run will proceed and its"
    "      baselines may not mean much; read the solo spread in the report."
}
if ($gap -gt 5) {
    "REFUSING: two fresh solo holds sit {0:N1}% apart, past the 5% the run will be judged by." -f $gap
    "THE SPREAD IS THE MACHINE, not placement noise. A fit box here reproduces"
    "a solo hold to 0.42% across six holds; on 2026-08-10 the same binary and"
    "workload spread 5 to 37% for about ten hours, on an idle machine. Waiting"
    "is the only lever and nothing here says how long."
    throw "the baseline cannot support the judgement"
}

# --- the label carries the duration, because that is the only thing the hour
# buys and nothing else in the artifact distinguishes it. This script is named
# for an hour and defaults to a third of one, deliberately (see the header), so
# two runs of it produce the same numbers under the same name and only one of
# them can claim the gate's third condition.
$label = "m1-{0}s" -f $DurationSeconds
if ($DurationSeconds -ge 3600) {
    "duration: {0}s — meets the gate's hour. A human should be present: this box" -f $DurationSeconds
    "          stopped dead at forty-one minutes of a rising load, and that is the"
    "          threshold this run is the first to test at a fixed duty."
} else {
    "duration: {0}s — SHORT OF THE GATE'S HOUR. Every other number is valid at" -f $DurationSeconds
    "          this exposure; the duration condition is unmet and the record has"
    "          to say so. Pass -DurationSeconds 3600 for the gate run."
}
"`nstarting — about {0:N0} minutes, leave the machine alone" -f (($DurationSeconds + 720) / 60)
& $bench hold `
    --label $label --sessions 100 --interval 5 --duration $DurationSeconds `
    --with-solo --solo-duration 120 --solo-repeats 5 `
    -- $spin @work

"`nDO NOT PRUNE bench-out until the record is written."
