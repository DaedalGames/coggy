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
# BOTH HOLDS RECORD THEIR OWN `rest_cores_median`, so the injection's real size
# is measured rather than assumed, and a browser arriving mid-test shows up in
# the artifact rather than silently spoiling the comparison.
param(
    [ValidateRange(1, 32)]
    [int]$Tenants = 6,
    [ValidateRange(5, 300)]
    [int]$Duration = 30,
    [double]$QuietBelow = 0.5,
    [int]$PollSeconds = 10,
    [int]$GiveUpMinutes = 30,
    [ValidateRange(1, 20)]
    [int]$MaxVoids = 6
)

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$bench = "$root\target\release\sessionbench.exe"
$spin = "$root\target\release\cpu-spin.exe"
$work = @('--units', '100000000', '--duty', '0.27')

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$null = New-Item -ItemType Directory -Force -Path "$root\bench-out"
Start-Transcript -Path "$root\bench-out\inject-$stamp.log" | Out-Null

function Get-TenantCores {
    $c = Get-Counter '\Process(*)\% Processor Time' -ErrorAction SilentlyContinue
    if ($null -eq $c) { return -1 }
    $busy = $c.CounterSamples |
        Where-Object { $_.InstanceName -notin @('_total', 'idle') -and $_.CookedValue -gt 50 } |
        Where-Object { $_.InstanceName -notlike 'sessionbench*' -and $_.InstanceName -notlike 'cpu-spin*' }
    if ($null -eq $busy) { return 0 }
    ($busy | Measure-Object CookedValue -Sum).Sum / 100
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
    "injecting $Tenants co-tenants around a $Duration s hold; quiet means under $QuietBelow cores"
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
        $t = Get-TenantCores
        # The sentinel is tested FIRST and the order is load-bearing: -1 is less
        # than any threshold, so checking quiet first would read an unreadable
        # counter as the quietest possible machine.
        if ($t -lt 0) { "{0:HH:mm:ss} counter unreadable, resetting" -f (Get-Date); $run = 0 }
        elseif ($t -lt $QuietBelow) { $run++; "{0:HH:mm:ss} quiet {1}/2 at {2:N2} cores" -f (Get-Date), $run, $t }
        else { if ($run -gt 0) { "{0:HH:mm:ss} not quiet at {1:N2} cores" -f (Get-Date), $t }; $run = 0 }
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

    Write-Host ("{0:HH:mm:ss} starting $Tenants co-tenants" -f (Get-Date))
    $procs = 1..$Tenants | ForEach-Object {
        Start-Process $spin -ArgumentList (@('--units', '100000000', '--duty', '0.27', '--resident', '1')) -PassThru -WindowStyle Hidden
    }
    Start-Sleep -Seconds 5

    $after = Invoke-Hold 'inject-after' $Duration
    if ($null -eq $after) { Write-Host 'tenanted hold unmeasurable, stopping rather than guessing'; exit 3 }

    Write-Host ("RESULT: {0:N3} -> {1:N3} units/s, {2:+0.0;-0.0}%  (rest {3:N2} -> {4:N2} cores)" -f `
        $before.rate, $after.rate, (100 * ($after.rate / $before.rate - 1)), $before.rest, $after.rest)
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
