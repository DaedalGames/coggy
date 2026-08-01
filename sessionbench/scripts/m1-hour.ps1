# Gate M1's hour — see task #45.
#
# A hundred sessions under coggyd for an hour, with a solo baseline on either
# side. Sixty-four minutes end to end, and the machine must be left alone for
# all of it: the observer is not free.
#
# Two of the gate's conditions come back out_of_reach whatever this does, and
# the report says so rather than passing them. This run answers RSS against
# the 4 GB budget and work rate as a ratio against its own bracketing solos.
#
# READ THE OUTPUT, NOT THE EXIT CODE. Piping this script makes $LASTEXITCODE
# the last native command's, and neither `exit` nor `throw` survives that. A
# run that went ahead prints "starting 64 minutes"; one that refused prints
# REFUSING and stops there.
#
# THE AS-IS COLUMN IS NOT THE 2026-08-01 HOUR-LONG HOLD. That run held `ping`
# at about 5.8 MiB a session; this one holds cpu-spin at --resident 20, so the
# totals differ by roughly 2 GiB of workload memory before anything about
# coggyd enters. What compares is the daemon's own cost per session held, the
# peak against the gate's stated 4 GB, and the two solo holds against each
# other.

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

cargo build --release -p coggyd -p cpu-spin
$spin = "$root\target\release\cpu-spin.exe"

# --- precondition. The RSS half barely notices background; the ratio half
# needs consecutive readings within a few percent of each other.
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
    "REFUSING: spread {0:N1}% — the solo halves would see different machines." -f $spread
    throw "background not steady enough"
}

# --- check for survivors before trusting anything, per the standing rule.
$stray = Get-Process coggyd, cpu-spin, ping -ErrorAction SilentlyContinue
if ($stray) {
    "REFUSING: {0} stray process(es) already running" -f $stray.Count
    throw "the machine is not clean"
}

# --units 100000000 because cpu-spin's default is 60 and it EXITS after them.
# At roughly 30 units/s an hour needs 108,000; this is the same "do not stop".
"starting 64 minutes — leave the machine alone"
cargo run --release -p sessionbench -- hold `
    --label m1 --sessions 100 --interval 5 --duration 3600 `
    --with-solo --solo-duration 120 `
    -- $spin --units 100000000 --duty 1.0 --resident 20

"`nDO NOT PRUNE bench-out until the record is written."
