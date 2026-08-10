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
param(
    [double]$MaxTenantCores = 1.0,
    [int]$PollSeconds = 10,
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
    $c = Get-Counter '\Process(*)\% Processor Time' -ErrorAction SilentlyContinue
    if ($null -eq $c) { return 99 }   # cannot see, so assume busy
    $busy = $c.CounterSamples |
        Where-Object { $_.InstanceName -notin @('_total', 'idle') -and $_.CookedValue -gt 50 } |
        Where-Object { $_.InstanceName -notlike 'sessionbench*' -and $_.InstanceName -notlike 'cpu-spin*' }
    if ($null -eq $busy) { return 0 }
    ($busy | Measure-Object CookedValue -Sum).Sum / 100
}

$deadline = (Get-Date).AddMinutes($GiveUpMinutes)
"waiting for a window: tenant under {0:N1} cores, giving up at {1:HH:mm}" -f $MaxTenantCores, $deadline

while ((Get-Date) -lt $deadline) {
    $t = Get-TenantCores
    if ($t -lt $MaxTenantCores) {
        "{0:HH:mm:ss} WINDOW OPEN at {1:N2} cores held — starting" -f (Get-Date), $t
        break
    }
    Start-Sleep -Seconds $PollSeconds
}
if ((Get-Date) -ge $deadline) { "gave up without a window"; exit 2 }

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
