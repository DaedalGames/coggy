# Wait for the box to go quiet, then immediately run a pre-registered set.
#
# WHY THIS EXISTS, measured rather than guessed. On 2026-08-10 five separate
# quiet windows were lost between observing them and launching a run:
# `doctor` read 3-11% and by the time a command was issued the tenant was back
# at 78-86%. chrome-headless-shell holds 11-12 of 16 cores whenever it runs and
# its absences are under five minutes, so the latency of noticing IS the window.
#
# The waiter decides nothing. It removes the gap between the window opening and
# the run starting, and fires a fixed set whose reading rule is written down in
# task #59 before any number exists.
#
# LAUNCH DETACHED, like anything that outlives a tool call:
#   Start-Process pwsh -ArgumentList '-NoProfile','-File','<this>' `
#       -RedirectStandardOutput quiet.log -RedirectStandardError quiet.err
# A SINGLE READING OF 0.00 MEANS STARTING AS READILY AS GONE. The first
# attempt fired on exactly that and its first hold recorded 12.20 cores held,
# voiding the set. `Get-Counter` is a point sample, and a process ramping up
# reads zero on the way; requiring the quiet to persist for several polls is
# what distinguishes an absence from an instant.
param(
    [double]$MaxTenantCores = 1.0,
    [int]$PollSeconds = 10,
    [int]$ConsecutiveQuiet = 6,
    [int]$GiveUpMinutes = 120
)

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$bench = "$root\target\release\sessionbench.exe"
$spin = "$root\target\release\cpu-spin.exe"
$work = @('--units', '100000000', '--duty', '0.27', '--resident', '20')

# Refuse rather than measure on a dirty machine: a survivor from a killed run
# would be counted as a neighbour and disqualify the set for the wrong reason.
$stray = Get-Process coggyd, cpu-spin, sessionbench -ErrorAction SilentlyContinue
if ($stray) { "REFUSING: {0} stray process(es)" -f $stray.Count; exit 1 }

function Get-TenantCores {
    # Retry once, because the query fails transiently and a failure costs a
    # window: on 2026-08-10 it returned nothing twice in a minute, the second
    # time after four consecutive quiet polls, and with a reset-on-unreadable
    # rule an intermittently failing counter would keep the waiter from ever
    # firing on a box that was in fact quiet.
    $c = Get-Counter '\Process(*)\% Processor Time' -ErrorAction SilentlyContinue
    if ($null -eq $c) {
        Start-Sleep -Milliseconds 500
        $c = Get-Counter '\Process(*)\% Processor Time' -ErrorAction SilentlyContinue
    }
    # -1 means the counter could not be read, which is NOT a busy machine and
    # must not enter the transition log as one: this log is the record of how
    # long this box's quiet stretches are, and a failed query recorded as a
    # tenancy event would inflate the count of interruptions that never
    # happened. It still refuses to fire, since not seeing is not quiet.
    if ($null -eq $c) { return -1 }
    $busy = $c.CounterSamples |
        Where-Object { $_.InstanceName -notin @('_total', 'idle') -and $_.CookedValue -gt 50 } |
        Where-Object { $_.InstanceName -notlike 'sessionbench*' -and $_.InstanceName -notlike 'cpu-spin*' }
    if ($null -eq $busy) { return 0 }
    ($busy | Measure-Object CookedValue -Sum).Sum / 100
}

$deadline = (Get-Date).AddMinutes($GiveUpMinutes)
"waiting for a window: tenant under {0:N1} cores for {1} consecutive polls ({2}s apart), giving up at {3:HH:mm}" `
    -f $MaxTenantCores, $ConsecutiveQuiet, $PollSeconds, $deadline

$run = 0
while ((Get-Date) -lt $deadline) {
    $t = Get-TenantCores
    # The sentinel is tested FIRST and this order is load-bearing: -1 is less
    # than any sane MaxTenantCores, so checking quiet first would read an
    # unreadable counter as the quietest possible machine and fire the set.
    if ($t -lt 0) {
        "{0:HH:mm:ss} COUNTER UNREADABLE — not a tenancy event, resetting" -f (Get-Date)
        $run = 0
    }
    elseif ($t -lt $MaxTenantCores) {
        $run++
        "{0:HH:mm:ss} quiet {1}/{2} at {3:N2} cores" -f (Get-Date), $run, $ConsecutiveQuiet, $t
        if ($run -ge $ConsecutiveQuiet) {
            "{0:HH:mm:ss} WINDOW OPEN — quiet held for {1}s, starting" -f (Get-Date), ($run * $PollSeconds)
            break
        }
    }
    elseif ($run -gt 0) {
        "{0:HH:mm:ss} quiet broken at {1:N2} cores after {2} poll(s)" -f (Get-Date), $t, $run
        $run = 0
    }
    Start-Sleep -Seconds $PollSeconds
}
if ($run -lt $ConsecutiveQuiet) { "gave up without a window"; exit 2 }

# The pre-registered set: two arms, alternating, both inside one window, so the
# comparison varies duration and nothing else. Order alternates rather than
# grouping, so a drift partway through hits both arms equally.
foreach ($pair in 1..2) {
    foreach ($d in 30, 120) {
        $label = "dur-$d-$pair"
        & $bench hold --label $label --sessions 1 --interval 5 --duration $d -- $spin @work 2>&1 |
            Select-String -Pattern 'rate |cores held outside the job' |
            ForEach-Object { "{0}: {1}" -f $label, $_.Line.Trim() }
    }
}
"set complete"
