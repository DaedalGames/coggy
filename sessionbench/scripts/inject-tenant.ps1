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
    if (($total - $idle - $ours) -lt 0) { return [pscustomobject]@{ Tenant = -1.0; Machine = -2.0 } }
    $busy = $s |
        Where-Object { $_.InstanceName -notin @('_total', 'idle') -and $_.CookedValue -gt 50 } |
        Where-Object { -not (& $mine) }
    $tenant = if ($null -eq $busy) { 0.0 } else { ($busy | Measure-Object CookedValue -Sum).Sum / 100 }
    [pscustomobject]@{ Tenant = $tenant; Machine = ($total - $idle - $ours) / 100 }
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
function Invoke-Hold($label, $seconds) {
    $out = & $bench hold --label $label --sessions 1 --interval 5 --duration $seconds `
        -- $spin @work --resident 20 2>&1
    $m = ($out | Select-String -Pattern '^\s*rate\s+([\d.]+)\s+units/s' | Select-Object -First 1)
    if (-not $m) {
        Write-Host "  $label : RATE UNPARSEABLE, captured output follows"
        $out | ForEach-Object { Write-Host "      $_" }
        return $null
    }
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
    $voids = 1
    # THE BASELINE QUALIFIES ROUGHLY ONE ATTEMPT IN THREE, so retrying is the
    # script's job rather than the operator's. The waiter's counter sees only
    # processes over half a core, while the guard below reads the hold's own
    # rest_cores, which includes this box's ~1-1.9 cores of small background
    # processes — so a window that looks quiet often is not, and the first
    # version cost a launch every time.
    while ((Get-Date) -lt $deadline) {
    $run = 0
    while ((Get-Date) -lt $deadline -and $run -lt 2) {
        $cores = Get-Cores
        $t = $cores.Tenant
        $m = $cores.Machine
        # The sentinels are tested FIRST and the order is load-bearing: a
        # negative is less than any threshold, so checking quiet first would
        # read an unreadable counter as the quietest possible machine. -2.0 is
        # tested before -1 because it also satisfies `-lt 0`.
        if ($m -eq -2.0) { "{0:HH:mm:ss} counter inconsistent (process churn), resetting" -f (Get-Date); $run = 0 }
        elseif ($t -lt 0 -or $m -lt 0) { "{0:HH:mm:ss} counter unreadable, resetting" -f (Get-Date); $run = 0 }
        # QUIET MEANS QUIET TO THE GUARD, which reads the machine rather than
        # the census. Both bars, so this can only withhold a window the census
        # bar alone would have passed and the baseline guard then voided.
        elseif ($t -lt $QuietBelow -and $m -lt $QuietMachineBelow) {
            $run++
            "{0:HH:mm:ss} quiet {1}/2 at {2:N2} cores ({3:N2} machine-wide)" -f (Get-Date), $run, $t, $m
        }
        else {
            if ($run -gt 0) { "{0:HH:mm:ss} not quiet at {1:N2} cores ({2:N2} machine-wide)" -f (Get-Date), $t, $m }
            $run = 0
        }
        if ($run -lt 2) { Start-Sleep -Seconds $PollSeconds }
    }
    if ($run -lt 2) { Write-Host 'gave up without a quiet window'; exit 2 }

    $before = Invoke-Hold 'inject-before' $Duration
    if ($null -eq $before) { Write-Host 'baseline unmeasurable, stopping rather than guessing'; exit 3 }

    # THE BASELINE MUST BE BELOW THE TRANSITION OR THERE IS NOTHING TO INJECT
    # INTO. The waiter certifies quiet from a counter that only sees processes
    # over half a core, so the browser can arrive *inside* the baseline hold —
    # which is what happened on 2026-08-11: the waiter fired at 0.00, the
    # baseline recorded 12.09 cores held, and injecting six spinners moved it
    # to 12.89. Both arms sat in the collapse region and the run reported a
    # plausible -9.6% that meant nothing. Refuse instead: a void test that says
    # so cannot be quoted later, and one that returns a number can.
    if ($before.rest -ge 1.3 -or [double]::IsNaN($before.rest)) {
        Write-Host ("VOID {0}: baseline ran at {1} cores held, above the ~1.4 transition — nothing to inject into" -f $voids, $before.rest)
        $voids++
        if ($voids -ge $MaxVoids) { Write-Host "gave up after $MaxVoids voided baselines"; exit 4 }
        continue
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
        Write-Host ("  tried: $inject $($injectArgs -join ' ')")
        Write-Host '  alive is a status; holding CPU is the effect. A blocked injector produces a flat rate that reads as a true null.'
        $procs | Where-Object { -not $_.HasExited } | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }
        exit 6
    }
    Write-Host ("  co-tenants hold {0:N3} cores of their own, against {1:N2} expected" -f $own, $wanted)
    Start-Sleep -Seconds 3

    $after = Invoke-Hold 'inject-after' $Duration
    if ($null -eq $after) { Write-Host 'tenanted hold unmeasurable, stopping rather than guessing'; exit 3 }

    # THE INJECTION MUST BE THE THING THAT CHANGED, AND NOTHING ELSE.
    # Six spinners are measured to add ~1.6 cores; if the delta is far from
    # that, the browser arrived during the second hold and the comparison is
    # about the browser rather than about anything injected. That happened on
    # 2026-08-11: a baseline at 1.09 cores rose to 12.51, and the run reported
    # a perfectly believable +62.2% that was 11.4 cores of somebody else.
    #
    # This is the guard that matters most, because a FAILED injection and a
    # TRUE NULL produce identical output — a flat rate on a machine nobody
    # verified had changed. Refusing here is what makes a null mean something.
    $delta = $after.rest - $before.rest
    if ($delta -lt ($Tenants * 0.15) -or $delta -gt ($Tenants * 0.55)) {
        Write-Host ("VOID {0}: tenancy moved {1:N2} cores where {2} spinners add about {3:N1} — the injection is not what changed" -f `
            $voids, $delta, $Tenants, ($Tenants * 0.27))
        Write-Host ("        baseline {0:N3} at {1:N2} cores, tenanted {2:N3} at {3:N2} cores" -f `
            $before.rate, $before.rest, $after.rate, $after.rest)
        $voids++
        if ($voids -ge $MaxVoids) { Write-Host "gave up after $MaxVoids voids"; exit 4 }
        $procs | Stop-Process -Force -ErrorAction SilentlyContinue
        $procs = @()
        continue
    }

    Write-Host ("RESULT: {0:N3} -> {1:N3} units/s, {2:+0.0;-0.0}%  (rest {3:N2} -> {4:N2} cores, delta {5:N2})" -f `
        $before.rate, $after.rate, (100 * ($after.rate / $before.rate - 1)), $before.rest, $after.rest, $delta)
    Write-Host 'a rise means the neighbour CAUSES the step; flat means it only coincides with one'
    break
    }
    if ((Get-Date) -ge $deadline) { Write-Host 'deadline reached'; exit 2 }
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
    try { Stop-Transcript | Out-Null } catch {}
}
