# Does a neighbour CAUSE the lone-session step, or merely coincide with it?
#
# Every low-tenancy hold on disk got that way by the browser LEAVING, never by
# anything being ADDED — so the +24% to +53% step measured across five sittings
# is correlational and nothing observational can change that. This adds a load
# we control, inside one window, and watches the same session's rate.
#
#   1. wait for quiet
#   2. hold 30s                       -> baseline
#   3. start N cpu-spin co-tenants
#   4. hold 30s                       -> tenanted
#   5. stop them, check for survivors
#
# RATE RISES -> the neighbour causes the step.
# RATE FLAT  -> the step coincides with tenancy rather than being produced by
#               it, and every figure measured so far is a correlation.
#
# SIX SESSIONS IS THE RIGHT SIZE, measured rather than guessed: driving this box
# with 2, 6 and 12 `cpu-spin` sessions moved the machine by +0.5, +1.6 and +3.1
# cores, and the transition sits between 1.36 and 1.46 cores held.
#
# THE CO-TENANTS RUN `--resident 1` SO THEY ADD CPU AND NOT MEMORY PRESSURE.
# The step is footprint-independent (+17.5% at resident 20 against +17.8% at
# resident 1) while the collapse above ~4 cores held is not, so a heavy injected
# load would confound the two.
#
# CONFOUND TO STATE WITH ANY NULL RESULT: `cpu-spin` at duty 0.27 is bursty
# where the browser is not. For a causality question any load serves, but a null
# would need repeating with a steady load before it means tenancy does not cause
# the step.
#
# LAUNCH DETACHED AND DO NOT REDIRECT ITS OUTPUT:
#   Start-Process pwsh -ArgumentList '-NoProfile','-File','<this>' `
#       -WindowStyle Hidden -PassThru
# A redirect forces UseShellExecute=$false, so the child inherits the
# launching console and dies on that console's control event. This script
# writes its own transcript under bench-out/, which is what to read. The
# full account, with the exit code that named it, is in CLAUDE.md beside
# `commit before any wait whose end you cannot see`.
#
# BOTH HOLDS RECORD THEIR OWN `rest_cores_median`, so the injection's real size
# is measured rather than assumed, and a browser arriving mid-test shows up in
# the artifact rather than silently spoiling the comparison.
param(
    [ValidateRange(1, 32)]
    [int]$Tenants = 6,
    [ValidateRange(5, 300)]
    [int]$Duration = 30,
    [double]$QuietBelow = 0.5,
    # THE BAR THAT MATCHES WHAT THE BASELINE GUARD REJECTS ON. `QuietBelow`
    # above governs the census counter, which sums only instances over half a
    # core and therefore reads 0.00 for load spread across small ones. The guard
    # below rejects a baseline at 1.3 cores of `rest_cores_median`, which is
    # machine-wide -- so a census-only gate passes runs the guard then throws
    # away, two of three on 2026-08-11 at readings of 0.00 against true 3.26 and
    # 1.43. 1.0 admits the attempt that passed (true 0.75) with margin and sits
    # below the 1.3 it feeds.
    [double]$QuietMachineBelow = 1.0,
    # MEASURE AT WHATEVER TENANCY EXISTS, instead of waiting for a low baseline.
    #
    # The guard below rejects a baseline at 1.3 cores because the step's
    # transition sits at 1.36-1.46, and crossing it is what the ORIGINAL
    # experiment measured. That requirement was inherited rather than chosen,
    # and it is not what every question needs.
    #
    # COMPARING TWO INJECTORS does not need the rising limb at all: it asks
    # whether `file-write` and `cpu-spin` produce the SAME step at the SAME
    # delta, so running both at whatever tenancy the box offers, back to back,
    # pairs them against EACH OTHER inside one window. That is stronger than the
    # low-baseline design, which pairs each against a different afternoon.
    #
    # It exists because this box does not go quiet. On 2026-08-11 evening three
    # runs across two and a half hours produced ONE baseline, and the browser
    # returned inside its thirty-second hold. Windows opened at 18:15, 22:10,
    # 22:20, 22:29 and 23:53, each a minute or two long, against a paired
    # measurement needing two consecutive quiet minutes.
    #
    # WHAT IT COSTS, and it must be stated with any result: a pair taken at 8
    # cores measures the COLLAPSE limb, where the recorded +95.4% and +100.1%
    # are both from the rising limb below ~1.4. The absolute figures are not
    # comparable with those; two injectors measured against each other are.
    [switch]$AnyBaseline,
    [int]$PollSeconds = 10,
    [int]$GiveUpMinutes = 30,
    [ValidateRange(1, 20)]
    [int]$MaxVoids = 6,
    # THE CONFOUND THIS EXISTS FOR: by default the co-tenants are `cpu-spin`,
    # the SAME BINARY as the measured session. Windows maps one image file once,
    # so seven processes share code pages and an instruction-cache footprint —
    # and a warm icache could help in a way a browser never would. The 2026-08-11
    # result of +95.4% has that confound and needs it removed before it means
    # "a neighbour" rather than "six copies of myself".
    #
    # A COPY OF THE BINARY AT A DIFFERENT PATH IS THE SHARPEST CONTROL. Same
    # instructions, same duty, same memory behaviour, different image identity,
    # so the pages are no longer shared and nothing else moves. Swapping to
    # `file-write` instead would change the load's character — disk I/O, a
    # different memory pattern — AND the sharing at once, which is two changes.
    #
    #   copy target\release\cpu-spin.exe target\release\cpu-spin-b.exe
    #   ... -Injector 'target\release\cpu-spin-b.exe'
    #
    # If the rise survives, self-similarity is out. If it collapses, the causal
    # result is about the injector rather than about tenancy.
    [string]$Injector = '',
    # WHAT TO PASS THE INJECTOR, because `-Injector` alone was a trap.
    #
    # The arguments were hardcoded to cpu-spin's `--units/--duty/--resident`,
    # so `-Injector` could only ever name another cpu-spin. `file-write` takes
    # `--files/--size/--interval` and `stdout-storm` its own set, so pointing
    # this at either would have spawned processes that clap rejects and that die
    # before the hold starts -- and a mangled switch producing error output is
    # exactly [how a ramp counted three error lines a second as completed
    # work](../../CLAUDE.md#verify-before-you-launch-too).
    #
    # A DELIMITED STRING, split in-script, for the reason `-Residents` and
    # `-Duties` are: `pwsh -File` cannot bind an array parameter, and an
    # argument list beginning with `-` is read as this script's own parameters.
    #
    # The default reproduces the two runs already recorded, so an invocation
    # that omits it is unchanged.
    [string]$InjectorArgs = '--units|100000000|--duty|0.27|--resident|1',
    # HOW MUCH CPU ONE CO-TENANT SHOULD HOLD, so the run can tell an injection
    # from six processes that merely exist. MEASURED 2026-08-11 by reading each
    # workload's OWN counter, which is immune to whatever else the box is doing
    # -- six processes, sampled twice nine seconds apart:
    #
    #   cpu-spin  --duty 0.27          1.247 / 1.036 cores   -> ~0.19 each
    #   file-write --interval 200      0.046 / 0.031         -> ~0.007 each
    #   file-write --interval  50      0.123 / 0.092
    #   file-write --interval  10      0.440 / 0.380         -> ~0.068 each
    #   stdout-storm any interval      0.000 / 0.000         -> NOTHING
    #
    # `stdout-storm` is the reason this parameter exists. With no reader draining
    # its stdout it blocks on the first full buffer, so six copies sit alive and
    # perfectly idle -- and the liveness check below passes them, because being
    # alive is a STATUS where holding CPU is the EFFECT. It is a fine session
    # workload, since `coggyd` drains it, and a useless standalone injector.
    #
    # `file-write` at the default interval is nearly as bad at 0.007 cores each:
    # it would inject a fortieth of what six cpu-spin do, and the delta guard
    # would blame the browser for the absence.
    [double]$ExpectedPerTenant = 0.19
)

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$bench = "$root\target\release\sessionbench.exe"
$spin = "$root\target\release\cpu-spin.exe"
# Empty means "same binary as the session", which is the default and the one
# carrying the shared-code-page confound. Resolved and existence-checked here
# rather than at first use, so a typo fails before a window is spent.
$inject = if ($Injector) { (Resolve-Path $Injector -ErrorAction Stop).Path } else { $spin }
$injectArgs = $InjectorArgs -split [regex]::Escape('|')
if (-not (Test-Path $inject)) { "REFUSING: injector not found at $inject"; exit 1 }
$work = @('--units', '100000000', '--duty', '0.27')

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$null = New-Item -ItemType Directory -Force -Path "$root\bench-out"
Start-Transcript -Path "$root\bench-out\inject-$stamp.log" | Out-Null

# A VERDICT IS NOT EVIDENCE — RECORD THE FIGURE IT WAS COMPUTED FROM.
#
# This script decided to reject a pair, printed the delta that decided it, and
# stored neither. On 2026-08-12 the void count per arm was HALF the finding of
# [the duration series](../../docs/measurements/2026-08-12-042000-longer-holds-do-not-cut-the-spread-and-they-halve-the-acceptance.md)
# — 7-of-8 acceptance against 4-of-8 — and had to be reconstructed by grepping
# sixteen transcripts, which survive only until `bench-out/` is pruned. The
# three deltas that voided (-0.94, +9.55, -10.41 cores) are the only record
# that a ~10-core disturbance arrived and left during the last two runs.
#
# A FAILED RUN LEFT NO ARTIFACT AT ALL, so an attempt that failed and an
# attempt never made looked identical.
#
# WRITTEN FROM THE `finally` BLOCK, which is the single path every `exit` and
# the `break` passes through. Setting `$outcome` at each terminal point and
# serialising once covers all ten of them, and covers any added later — the
# alternative is a write beside each exit, which is how a fix for a shape ends
# up applied to one line.
#
# `$outcome` STARTS AT 'Unknown' RATHER THAN AT A PLAUSIBLE VALUE, so a path
# that forgets to set it produces a word nothing consumes instead of a verdict
# that reads as real. The same reason `$pair` is $null until there is one: a
# `0.0` beside a failure silently satisfies a condition where a `$null` cannot.
$outcome = 'Unknown'
$voidLog = @()
$pair = $null

function Get-Cores {
    # ONE QUERY, BOTH FIGURES, AND A RETRY. Ported from wait-for-quiet.ps1 on
    # 2026-08-11 after this script burned its only window in twenty minutes.
    #
    # THE RETRY IS THE PART THAT MATTERED. The waiter has had it since
    # 2026-08-10 with a comment saying why -- the query fails transiently, and
    # under a reset-on-unreadable rule an intermittent failure keeps a poller
    # from ever firing on a box that is in fact quiet. This function returned -1
    # on the FIRST failure, so every transient cleared the consecutive-quiet run.
    # Measured: two wait-for-quiet runs of 40 and 15 minutes logged ZERO
    # unreadable counters while this script logged nine in twenty, and the
    # counter answered 3 of 3 from a shell immediately afterwards. The counter
    # was never the problem.
    #
    # AND THE CENSUS FIGURE CANNOT GATE. It sums only instances over half a
    # core, so a process hovering near 0.5 reads either 0.00 or 0.67 with
    # nothing between -- which is what this run saw at 18:15, one quiet poll
    # then a crossing, twice, while the machine sat at a smooth ~1-1.5 cores.
    # `_Total - Idle - ours` is what the baseline guard below actually rejects
    # on, so gating on it stops passing runs the guard then throws away.
    $c = Get-Counter '\Process(*)\% Processor Time' -ErrorAction SilentlyContinue
    if ($null -eq $c) {
        Start-Sleep -Milliseconds 500
        $c = Get-Counter '\Process(*)\% Processor Time' -ErrorAction SilentlyContinue
    }
    if ($null -eq $c) { return [pscustomobject]@{ Tenant = -1.0; Machine = -1.0 } }
    $s = $c.CounterSamples
    $total = ($s | Where-Object { $_.InstanceName -eq '_total' }).CookedValue
    $idle = ($s | Where-Object { $_.InstanceName -eq 'idle' }).CookedValue
    if ($null -eq $total -or $null -eq $idle) { return [pscustomobject]@{ Tenant = -1.0; Machine = -1.0 } }
    $mine = { $_.InstanceName -like 'sessionbench*' -or $_.InstanceName -like 'cpu-spin*' -or
        $_.InstanceName -like 'coggyd*' -or $_.InstanceName -like 'file-write*' -or
        $_.InstanceName -like 'stdout-storm*' }
    $ours = ($s | Where-Object $mine | Measure-Object CookedValue -Sum).Sum
    # A negative machine reading is process churn making `_Total` and `Idle`
    # inconsistent, not a failed query. -2.0 keeps the two distinguishable.
    $machine = $total - $idle - $ours
    # A SMALL NEGATIVE IS A QUIET BOX, NOT A BROKEN COUNTER. `_Total` and `Idle`
    # come from one query but are computed over slightly offset internal
    # samples, so their difference drifts by hundredths -- and it drifts closest
    # to zero when the machine is NEAREST IDLE, because that is when the two
    # figures are nearest each other.
    #
    # Rejecting every negative therefore threw away the quietest readings a gate
    # exists to find, and each rejection resets the consecutive-quiet run.
    # MEASURED on the 22:25 injection run: 4 churn rejections against 2 quiet
    # polls logged in forty minutes, so the guard discarded twice what it let
    # through and the run never assembled a second pair.
    #
    # The sentinel came from ONE reading of -1.95 cores, taken six seconds after
    # six processes were spawned on a box carrying 2.8 cores. That is real churn
    # and a magnitude no rounding reaches.
    #
    # THE THRESHOLD IS IN THE COUNTER'S OWN UNITS, where 100 is one core -- the
    # return divides by 100. So -25 is a quarter of a core, and writing -0.25
    # here would have meant a four-hundredth of one, a hundred times tighter
    # than intended and still rejecting the drift this exists to allow.
    if ($machine -lt -25) { return [pscustomobject]@{ Tenant = -1.0; Machine = -2.0 } }
    if ($machine -lt 0) { $machine = 0.0 }

    $busy = $s |
        Where-Object { $_.InstanceName -notin @('_total', 'idle') -and $_.CookedValue -gt 50 } |
        Where-Object { -not (& $mine) }
    $tenant = if ($null -eq $busy) { 0.0 } else { ($busy | Measure-Object CookedValue -Sum).Sum / 100 }
    [pscustomobject]@{ Tenant = $tenant; Machine = ($machine / 100) }
}

# Returns @{rate; rest}, or $null when the rate could not be read. The caller
# must treat $null as "no measurement" rather than as a slow one — a 0.0 here
# would read as the slowest possible machine, which is the defect this
# repository has already paid for.
#
# **LOG LINES GO THROUGH `Write-Host`, AND THAT IS LOAD-BEARING.** In PowerShell
# every uncaptured expression in a function becomes part of its output, so a
# bare `"  $label : ..."` string makes the return value an ARRAY of [message,
# rate]. That happened on 2026-08-11: the `-f` format on the result threw, and
# the `$null -eq $before` guard passed an array straight through the check
# meant to stop it. Nothing reached stdout and the run reported nothing.
function Invoke-Hold($label, $seconds, $abortAbove) {
    # STOP A HOLD THE MACHINE HAS ALREADY SPOILED, at the sample that shows it.
    # Without a ceiling this script paid for a full hold before the guard below
    # could refuse the pair: three spoiled baselines on 2026-08-12 read 12.27 at
    # sample 0, 8.33 at sample 1 and 1.71 at sample 0, so two of three were
    # decided by the first sample and ran twenty-five more seconds anyway.
    #
    # $null means no ceiling, which is what a hold measuring whatever the box
    # offers wants — the `-AnyBaseline` path, and the gate's own holds.
    $ceiling = if ($null -ne $abortAbove) { @('--abort-rest-above', $abortAbove) } else { @() }
    $out = & $bench hold --label $label --sessions 1 --interval 5 --duration $seconds @ceiling `
        -- $spin @work --resident 20 2>&1
    $m = ($out | Select-String -Pattern '^\s*rate\s+([\d.]+)\s+units/s' | Select-Object -First 1)
    if (-not $m) {
        Write-Host "  $label : RATE UNPARSEABLE, captured output follows"
        $out | ForEach-Object { Write-Host "      $_" }
        return $null
    }
    # SURFACE THE TRUNCATION. The hold prints its own ABORTED line, and this
    # function CAPTURES the hold's output to parse a rate out of it — so
    # without this the console shows a normal-looking hold that silently ran
    # five seconds instead of thirty. The artifact carries `aborted_rest_cores`
    # either way; this is so a reader watching the run is not misled.
    $stopped = ($out | Select-String -Pattern '^\s*ABORTED .*' | Select-Object -First 1)
    if ($stopped) { Write-Host ("  {0}" -f $stopped.Matches[0].Value.Trim()) }
    $rest = ($out | Select-String -Pattern '([\d.]+) cores held outside the job' | Select-Object -First 1)
    $held = if ($rest) { [double]$rest.Matches[0].Groups[1].Value } else { [double]::NaN }
    Write-Host ("  {0} : {1} units/s, {2} cores held outside the job" -f $label, $m.Matches[0].Groups[1].Value, $held)
    @{ rate = [double]$m.Matches[0].Groups[1].Value; rest = $held }
}

$procs = @()
try {
    "injecting $Tenants co-tenants around a $Duration s hold; quiet means under $QuietBelow census cores AND $QuietMachineBelow machine-wide"
    "injector: $inject"
    $deadline = (Get-Date).AddMinutes($GiveUpMinutes)
    # STARTS AT 0 AND IS CHECKED AFTER THE INCREMENT, so `-MaxVoids 3` permits
    # THREE voids. It started at 1 until 2026-08-12, which permitted two — and
    # the give-up line printed $MaxVoids rather than the count, so a run that
    # voided twice reported `gave up after 3 voids`. A message that states a
    # PARAMETER cannot disagree with what happened, which is exactly why it
    # went unnoticed; it now prints $voids and can.
    $voids = 0
    # THE BASELINE QUALIFIES ROUGHLY ONE ATTEMPT IN THREE, so retrying is the
    # script's job rather than the operator's. The waiter's counter sees only
    # processes over half a core, while the guard below reads the hold's own
    # rest_cores, which includes this box's ~1-1.9 cores of small background
    # processes — so a window that looks quiet often is not, and the first
    # version cost a launch every time.
    while ((Get-Date) -lt $deadline) {
    # A WINDOWED MEAN, NOT TWO CONSECUTIVE POINT READINGS.
    #
    # The old rule wanted two polls in a row under the bar, and MEASURED ON
    # 2026-08-12 that is the wrong question to ask this box. Across one waiting
    # run every one of ten polls read below the 1.0 machine-wide bar — 0.94,
    # 0.88, 0.49, 0.78, 0.89, 0.90, 0.58, 0.83, 0.93, 0.62 — and most failed
    # anyway, because a single excursion breaks the streak. The machine sits at
    # ~0.8 and the SAMPLER occasionally reports above threshold, so the gate was
    # measuring its own sampling variance and calling it the machine's state.
    #
    # A mean over the same number of polls keeps the same bar and the same cost
    # while letting one high sample be outvoted rather than decisive — the same
    # reason a two-minute solo hold beats four point readings of `doctor`, and
    # the same reason pre-screening a ratio on point readings was measured to
    # cost runs in both directions.
    #
    # THREE SAMPLES RATHER THAN TWO, because a mean of two is a point reading
    # with extra steps: one excursion still moves it half its own size. Three
    # costs one extra poll — ten seconds against a window measured in minutes.
    # RESET BOTH, AT THE TOP OF EVERY ATTEMPT, AND THE FIRST ONE IS LOAD-BEARING.
    # `$run = 0` was dropped when the mean replaced the streak on 2026-08-12,
    # and the result was not a wrong number but NO GATE AT ALL: the first
    # clearance sets `$run = 2`, so on the next attempt the polling loop's
    # `-and $run -lt 2` was already false and it never ran. Five consecutive
    # baselines were then taken with no quiet check whatsoever, at 3.43, 13.18,
    # 13.02, 14.19 and 9.11 cores held, and the transcript shows NO poll lines
    # between them.
    #
    # It survived both deliberate breaks because each used `-MaxVoids 1` and so
    # only ever ran ONE attempt — the reset path was never exercised. A check
    # broken on purpose proves the branch you broke, not the loop around it.
    #
    # `$recent` is cleared for a second reason worth keeping separate: a hold
    # has just run, and this box has moved ten cores inside one thirty-second
    # hold, so samples from before it describe a machine that may be gone.
    $run = 0
    $recent = @()
    while ((Get-Date) -lt $deadline -and $run -lt 2) {
        $cores = Get-Cores
        $t = $cores.Tenant
        $m = $cores.Machine
        # The sentinels are tested FIRST and the order is load-bearing: a
        # negative is less than any threshold, so checking quiet first would
        # read an unreadable counter as the quietest possible machine. -2.0 is
        # tested before -1 because it also satisfies `-lt 0`.
        if ($m -eq -2.0) { "{0:HH:mm:ss} counter inconsistent (process churn), resetting" -f (Get-Date); $run = 0; $recent = @() }
        elseif ($t -lt 0 -or $m -lt 0) { "{0:HH:mm:ss} counter unreadable, resetting" -f (Get-Date); $run = 0; $recent = @() }
        # QUIET MEANS QUIET TO THE GUARD, which reads the machine rather than
        # the census. Both bars, so this can only withhold a window the census
        # bar alone would have passed and the baseline guard then voided.
        else {
            # THE CENSUS BAR IS GONE FROM THIS CONDITION AND THE MEAN REPLACES
            # THE STREAK.
            #
            # `$t` read 0.00 on every poll of three waiting runs while the
            # machine figure sat at 0.49-0.94 — so it never bound, and the gate
            # was effectively machine-only already. NOT because the counter is
            # dead: verifying this change on a LOADED box showed it reading
            # 10.60-11.98 against a machine figure of 11.38-12.95, tracking
            # closely. It sums only instances over half a core, so it sees one
            # big tenant perfectly and goes blind exactly when load is spread
            # thin — which is the quiet regime this gate exists to detect, and
            # the same blindness that once let a gate read 0.00 against a true
            # 3.26.
            #
            # It is NOT strictly redundant, and that is worth stating rather
            # than glossing: at the defaults the census bar is 0.5 against the
            # machine bar's 1.0, so a machine at 0.9 with a census of 0.6 used
            # to be refused and now passes. That is deliberate — the figure the
            # baseline guard is calibrated against is the machine one, so the
            # gate should ask the same question the guard will.
            #
            # Still printed, because a reading nobody consumes is what this
            # repository refuses to leave lying around, and a census sitting at
            # zero beside a live machine figure is the evidence for all of the
            # above.
            $recent += $m
            if ($recent.Count -gt 3) { $recent = $recent[-3..-1] }
            $mean = ($recent | Measure-Object -Average).Average
            if ($recent.Count -ge 3 -and $mean -lt $QuietMachineBelow) {
                $run = 2
                "{0:HH:mm:ss} quiet: mean {1:N2} over {2} polls (this one {3:N2} machine-wide, census {4:N2})" -f (Get-Date), $mean, $recent.Count, $m, $t
            }
            else {
                "{0:HH:mm:ss} not quiet: mean {1:N2} over {2} polls (this one {3:N2} machine-wide, census {4:N2})" -f (Get-Date), $mean, $recent.Count, $m, $t
            }
        }
        if ($run -lt 2) { Start-Sleep -Seconds $PollSeconds }
    }
    if ($run -lt 2) { Write-Host 'gave up without a quiet window'; $outcome = 'NoQuietWindow'; exit 2 }

    # THE BASELINE ARM'S CEILING IS THE GUARD IT WILL BE JUDGED BY. Without
    # `-AnyBaseline` the guard refuses a baseline at or above 1.3 cores, so a
    # hold that has already crossed it cannot produce a usable baseline no
    # matter how long it runs. With `-AnyBaseline` there is no such guard and
    # no ceiling belongs here.
    $baselineCeiling = if ($AnyBaseline) { $null } else { 1.3 }
    $before = Invoke-Hold 'inject-before' $Duration $baselineCeiling
    if ($null -eq $before) { Write-Host 'baseline unmeasurable, stopping rather than guessing'; $outcome = 'BaselineUnmeasurable'; exit 3 }

    # THE BASELINE MUST BE BELOW THE TRANSITION OR THERE IS NOTHING TO INJECT
    # INTO. The waiter certifies quiet from a counter that only sees processes
    # over half a core, so the browser can arrive *inside* the baseline hold —
    # which is what happened on 2026-08-11: the waiter fired at 0.00, the
    # baseline recorded 12.09 cores held, and injecting six spinners moved it
    # to 12.89. Both arms sat in the collapse region and the run reported a
    # plausible -9.6% that meant nothing. Refuse instead: a void test that says
    # so cannot be quoted later, and one that returns a number can.
    if ([double]::IsNaN($before.rest)) {
        Write-Host ("VOID {0}: baseline recorded no rest column — refusing rather than guessing" -f ($voids + 1))
        $voidLog += [pscustomobject]@{ index = ($voids + 1); kind = 'BaselineNoRestColumn'; delta = $null; expected = $null; baseline_rate = $before.rate; baseline_rest = $null; tenanted_rate = $null; tenanted_rest = $null }
        $voids++
        if ($voids -ge $MaxVoids) { Write-Host "gave up after $voids voided baselines"; $outcome = 'GaveUpOnVoids'; exit 4 }
        continue
    }
    if (-not $AnyBaseline -and $before.rest -ge 1.3) {
        Write-Host ("VOID {0}: baseline ran at {1} cores held, above the ~1.4 transition — nothing to inject into" -f ($voids + 1), $before.rest)
        $voidLog += [pscustomobject]@{ index = ($voids + 1); kind = 'BaselineAboveTransition'; delta = $null; expected = $null; baseline_rate = $before.rate; baseline_rest = $before.rest; tenanted_rate = $null; tenanted_rest = $null }
        $voids++
        if ($voids -ge $MaxVoids) { Write-Host "gave up after $voids voided baselines"; $outcome = 'GaveUpOnVoids'; exit 4 }
        continue
    }
    if ($AnyBaseline) {
        Write-Host ("  -AnyBaseline: measuring at {0:N2} cores held rather than waiting for a low one" -f $before.rest)
    }

    Write-Host ("{0:HH:mm:ss} starting $Tenants co-tenants: $inject $($injectArgs -join ' ')" -f (Get-Date))
    $procs = 1..$Tenants | ForEach-Object {
        Start-Process $inject -ArgumentList $injectArgs -PassThru -WindowStyle Hidden
    }
    Start-Sleep -Seconds 2

    # ASSERT ON THE EFFECT, NOT THE STATUS. `Start-Process` returns a handle for
    # a process that has already died, so a mis-flagged injector looks launched.
    # This matters the moment `-Injector` names anything but `cpu-spin`: the
    # arguments were hardcoded to `--units/--duty/--resident` until 2026-08-11,
    # and `file-write` takes `--files/--size/--interval`, so clap would reject
    # them and every co-tenant would exit before the hold began. The delta guard
    # below eventually catches that as a too-small injection, but it spends a
    # full cycle to say so and blames the browser for it. Cheaper and truer to
    # ask whether the things we started are still there.
    # The injector's own CPU, summed over its instances. Reading the MACHINE
    # before and after cannot do this job: on a box carrying 2.8 cores that
    # swings +/-0.4, an attempt to size six file-write this way returned 0.20,
    # 0.57 and an impossible -3.58, when their true draw was 0.046.
    $base = [IO.Path]::GetFileNameWithoutExtension($inject)
    $ctr = Get-Counter "\Process($base*)\% Processor Time" -ErrorAction SilentlyContinue
    $own = if ($null -eq $ctr) { [double]::NaN } else {
        (($ctr.CounterSamples | Where-Object { $_.InstanceName -notin @('_total', 'idle') } |
            Measure-Object CookedValue -Sum).Sum) / 100
    }
    $dead = @($procs | Where-Object { $_.HasExited })
    if ($dead.Count -gt 0) {
        Write-Host ("REFUSING: {0} of {1} co-tenants exited within 2s — the injector rejected its arguments" -f $dead.Count, $Tenants)
        Write-Host ("  tried: $inject $($injectArgs -join ' ')")
        Write-Host ("  exit codes: {0}" -f (($dead | ForEach-Object { $_.ExitCode }) -join ', '))
        $outcome = 'RefusedCoTenantsDied'
        $procs | Where-Object { -not $_.HasExited } | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }
        exit 5
    }
    # ALIVE IS NOT LOADED. Six stdout-storm with no reader stay alive and hold
    # 0.000 cores, so the check above passes them and the run measures nothing
    # while reporting a flat rate that reads as a true null. A floor at 40% of
    # expected is loose enough that a slow start or a differently-tuned injector
    # is not refused, and tight enough that a blocked one always is.
    $wanted = $Tenants * $ExpectedPerTenant
    if ([double]::IsNaN($own) -or $own -lt ($wanted * 0.4)) {
        Write-Host ("REFUSING: {0} co-tenants are alive but hold {1:N3} cores, against {2:N2} expected" -f $Tenants, $own, $wanted)
        $outcome = 'RefusedInjectionTooSmall'
        Write-Host ("  tried: $inject $($injectArgs -join ' ')")
        Write-Host '  alive is a status; holding CPU is the effect. A blocked injector produces a flat rate that reads as a true null.'
        $procs | Where-Object { -not $_.HasExited } | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }
        exit 6
    }
    Write-Host ("  co-tenants hold {0:N3} cores of their own, against {1:N2} expected" -f $own, $wanted)
    Start-Sleep -Seconds 3

    # FAIL BEFORE PAYING FOR THE HOLD, NOT AFTER READING IT.
    #
    # The delta guard below is correct and it is expensive: it discovers that
    # the tenant returned only once the tenanted hold has run to completion. On
    # 2026-08-12 that cost the cleanest rising-limb baseline this instrument has
    # taken — a baseline at 1.03 cores, then a tenanted arm at 13.23, a move of
    # 12.20 cores — and with it a share of a window that occurs about once an
    # hour. The run's other three attempts never got a usable baseline at all.
    #
    # WINDOW REMAINING IS THE BINDING RESOURCE, and nothing measures it: a gate
    # firing on two backward-looking polls says nothing about how much quiet
    # lies ahead. Failing fast is the only lever available on a quantity that
    # cannot be predicted.
    #
    # THIS CATCHES THE SPAWN WINDOW ONLY, which is about five seconds of the
    # forty a tenanted attempt costs. An arrival DURING the hold still reaches
    # the delta guard and still costs the full hold; catching that needs the
    # hold itself to abort, which means changing the measurement path every
    # figure in this repository runs through. Cheap first, and say what it
    # does not cover.
    #
    # THE CEILING IS THE DELTA GUARD'S OWN, so this cannot refuse a pair the
    # guard would have accepted — it can only reach the same verdict sooner.
    # SYMMETRIC, BECAUSE A DEPARTURE CONFOUNDS AS BADLY AS AN ARRIVAL. The first
    # version of this check tested only the ceiling, and the run that broke it on
    # purpose fell straight through to the delta guard — the tenant had LEFT,
    # -9.17 cores, and a one-sided check had nothing to say about it. That is the
    # same asymmetry the delta guard itself once had, where -Infinity as a floor
    # admitted a browser shedding 4.70 cores and the pair reported +27.0%.
    #
    # THE FLOOR IS LENIENT IN BOTH MODES, WHERE THE DELTA GUARD'S IS NOT, and
    # the difference is deliberate. Without `-AnyBaseline` the guard's floor is
    # `+0.5 * expected`: it requires tenancy to have RISEN by half the injection.
    # That is right AFTER a thirty-second hold and wrong five seconds after
    # spawn, when the machine counter has had barely a sample to register six
    # processes that are still starting. Applying it here would void good
    # attempts on the exact path the rising-limb work needs — the one that runs
    # WITHOUT `-AnyBaseline`.
    #
    # So this refuses only what is unambiguous at five seconds: a large arrival
    # or a large departure. Whether the injection itself registered is the delta
    # guard's question, asked later with a full hold behind it.
    $preFloor = $wanted * -0.5
    $nowCores = (Get-Cores).Machine
    if ($nowCores -ge 0) {
        $moved = $nowCores - $before.rest
        if ($moved -lt $preFloor -or $moved -gt ($wanted * 2.0)) {
            Write-Host ("VOID {0}: tenancy already moved {1:N2} cores before the hold started, where {2} spinners add about {3:N1} — refusing before paying for it" -f `
                ($voids + 1), $moved, $Tenants, $wanted)
            $voidLog += [pscustomobject]@{ index = ($voids + 1); kind = 'TenancyMovedBeforeHold'; delta = $moved; expected = $wanted; baseline_rate = $before.rate; baseline_rest = $before.rest; tenanted_rate = $null; tenanted_rest = $nowCores }
            $voids++
            $procs | Stop-Process -Force -ErrorAction SilentlyContinue
            $procs = @()
            if ($voids -ge $MaxVoids) { Write-Host "gave up after $voids voids"; $outcome = 'GaveUpOnVoids'; exit 4 }
            continue
        }
    }

    # THE TENANTED ARM'S CEILING IS THE DELTA GUARD'S OWN, expressed as an
    # absolute: the guard refuses when tenancy moved more than twice the
    # expected injection, so the hold can stop the moment it passes that rather
    # than at the end. This is what would have saved the cleanest rising-limb
    # baseline of 2026-08-12, whose tenanted arm ran a full thirty seconds
    # against a machine carrying twelve more cores than its baseline did.
    $after = Invoke-Hold 'inject-after' $Duration ($before.rest + $wanted * 2.0)
    if ($null -eq $after) { Write-Host 'tenanted hold unmeasurable, stopping rather than guessing'; $outcome = 'TenantedUnmeasurable'; exit 3 }

    # THE INJECTION MUST BE THE THING THAT CHANGED, AND NOTHING ELSE.
    # Six cpu-spin hold 1.04-1.25 cores of their own, measured 2026-08-11 from
    # their own counter; if the delta is far from
    # that, the browser arrived during the second hold and the comparison is
    # about the browser rather than about anything injected. That happened on
    # 2026-08-11: a baseline at 1.09 cores rose to 12.51, and the run reported
    # a perfectly believable +62.2% that was 11.4 cores of somebody else.
    #
    # This is the guard that matters most, because a FAILED injection and a
    # TRUE NULL produce identical output — a flat rate on a machine nobody
    # verified had changed. Refusing here is what makes a null mean something.
    $delta = $after.rest - $before.rest
    # THE BAND DERIVES FROM WHAT A TENANT ACTUALLY HOLDS, not from a constant.
    # It was `Tenants * 0.15` to `* 0.55`, which are cpu-spin's numbers wearing
    # no name: six of them hold ~1.14 cores, so the band was 0.9-3.3 and the two
    # recorded runs landed at 1.31 and 1.47. Point `-Injector` at anything else
    # and the band stops meaning anything — sixteen file-write hold ~1.1 cores
    # against a band of 2.4-8.8, so every attempt would void for a reason that
    # is about the constant rather than about the run.
    #
    # Half to twice the expected total keeps both recorded cpu-spin runs inside
    # and still rejects the 11.42-core delta that was a browser arriving.
    $expected = $Tenants * $ExpectedPerTenant
    # ON A LOADED BOX THE INJECTION DISPLACES RATHER THAN ADDS, so the lower
    # bound tests something that cannot be true there. Measured 2026-08-12 00:26:
    # six co-tenants VERIFIED holding 0.847 cores of their own moved machine-wide
    # tenancy by 0.15, because the box was already at 8.48 of 16 cores and they
    # took their share from the browser rather than from idle.
    #
    # The guard's PURPOSE survives: prove the injection is what changed and not
    # the browser. On a loaded box those are two questions with two instruments
    # — the co-tenants' own CPU (asserted above) proves the injection happened,
    # and a BOUNDED delta proves the browser stayed put. The void at 3.11 cores
    # was the browser moving; 0.15 and 1.67 were not.
    #
    # So `-AnyBaseline` keeps the ceiling and drops the floor. Without it, on a
    # quiet box where adding N cores raises the total by N, both bounds hold.
    # A DEPARTURE CONFOUNDS AS BADLY AS AN ARRIVAL, and -Infinity admitted every
    # one. Measured 2026-08-12 02:08: the browser shed 4.70 cores during a
    # tenanted arm carrying a 1.15-core injection, and the pair reported +27.0%
    # — the machine getting quieter, not the neighbour doing anything.
    #
    # Dropping the floor was right for the WRONG BOUND. On a loaded box a
    # displacing injection legitimately gives delta near zero, which is why
    # `expected * 0.5` was wrong; but -Infinity says any departure is fine. A
    # small negative tolerance takes both: enough for displacement and sampling
    # noise, not enough for a tenant leaving. At -0.5 * expected the previously
    # accepted -0.35 still passes and this -4.70 is refused.
    $floor = if ($AnyBaseline) { $expected * -0.5 } else { $expected * 0.5 }
    if ($delta -lt $floor -or $delta -gt ($expected * 2.0)) {
        Write-Host ("VOID {0}: tenancy moved {1:N2} cores where {2} spinners add about {3:N1} — the injection is not what changed" -f `
            ($voids + 1), $delta, $Tenants, $expected)
        Write-Host ("        baseline {0:N3} at {1:N2} cores, tenanted {2:N3} at {3:N2} cores" -f `
            $before.rate, $before.rest, $after.rate, $after.rest)
        # THE ONLY VOID KIND THAT CARRIES A DELTA, and the one worth the most:
        # a rejected pair says the machine moved, and by how much. Three of these
        # are the whole record of the disturbance that ended 2026-08-12's series.
        $voidLog += [pscustomobject]@{ index = ($voids + 1); kind = 'TenancyMoved'; delta = $delta; expected = $expected; baseline_rate = $before.rate; baseline_rest = $before.rest; tenanted_rate = $after.rate; tenanted_rest = $after.rest }
        $voids++
        if ($voids -ge $MaxVoids) { Write-Host "gave up after $voids voids"; $outcome = 'GaveUpOnVoids'; exit 4 }
        $procs | Stop-Process -Force -ErrorAction SilentlyContinue
        $procs = @()
        continue
    }

    Write-Host ("RESULT: {0:N3} -> {1:N3} units/s, {2:+0.0;-0.0}%  (rest {3:N2} -> {4:N2} cores, delta {5:N2})" -f `
        $before.rate, $after.rate, (100 * ($after.rate / $before.rate - 1)), $before.rest, $after.rest, $delta)
    Write-Host 'a rise means the neighbour CAUSES the step; flat means it only coincides with one'
    # `$pair` EXISTS ONLY ON THIS PATH, so a reader cannot find a rate beside a
    # failed outcome. The percent change is stored as well as its two inputs:
    # the extraction on 2026-08-12 recomputed it from the printed rates, and a
    # figure recomputed downstream is a figure that can disagree with the one
    # the run acted on.
    $pair = [pscustomobject]@{
        baseline_rate  = $before.rate
        tenanted_rate  = $after.rate
        percent_change = 100 * ($after.rate / $before.rate - 1)
        baseline_rest  = $before.rest
        tenanted_rest  = $after.rest
        delta          = $delta
        expected       = $expected
    }
    $outcome = 'Paired'
    break
    }
    if ((Get-Date) -ge $deadline) { Write-Host 'deadline reached'; $outcome = 'DeadlineReached'; exit 2 }
}
finally {
    # Only one attempt can ever have started spinners: the void path `continue`s
    # BEFORE they are launched, and the only path that launches them ends in
    # `break` or an `exit`. So there is nothing to leak between attempts, and
    # this stops the single set the successful attempt owns.
    if ($procs) { $procs | Stop-Process -Force -ErrorAction SilentlyContinue }
    Start-Sleep -Seconds 2
    $left = (Get-Process cpu-spin, sessionbench -ErrorAction SilentlyContinue | Measure-Object).Count
    "survivors after teardown: $left"

    # THE RUN'S OWN RECORD, written on every exit path because this block is on
    # every exit path. `duration_seconds` is here so an arm never has to be
    # inferred from log ORDER — a void retry writes an extra transcript and
    # shifts the alternation, which is the trap the 2026-08-12 extraction had to
    # be warned about by hand.
    try {
        [pscustomobject]@{
            stamp             = $stamp
            outcome           = $outcome
            duration_seconds  = $Duration
            tenants           = $Tenants
            expected_per      = $ExpectedPerTenant
            any_baseline      = [bool]$AnyBaseline
            injector          = $inject
            injector_args     = ($injectArgs -join ' ')
            void_count        = $voidLog.Count
            voids             = @($voidLog)
            pair              = $pair
            survivors         = $left
        } | ConvertTo-Json -Depth 5 | Set-Content -Path "$root\bench-out\inject-$stamp.json" -Encoding utf8
        "wrote bench-out\inject-$stamp.json ($outcome, $($voidLog.Count) voids)"
    }
    catch { "FAILED to write inject-$stamp.json: $($_.Exception.Message)" }

    try { Stop-Transcript | Out-Null } catch {}
}
