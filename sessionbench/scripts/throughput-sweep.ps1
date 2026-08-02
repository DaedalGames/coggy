# Sweeps one workload knob and reads total throughput, with references between.
#
# WHAT IT MEASURES. Past saturation a session's rate is `eta*C / (w*N)`, so the
# total across a hold is `eta*C / w` — the session count and the duty cancel, and
# two holds run back to back share the machine's core count and its unit time.
# Their totals are therefore equal exactly when their `eta` are. **No solo
# baseline is involved**, which is the point: a solo hold is the noisiest number
# this instrument makes, spreading 25% across six holds on one afternoon, and
# every comparison that divided by one spent the day being refused.
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
# READ THE OUTPUT, NOT THE EXIT CODE. Piping this makes $LASTEXITCODE the last
# native command's, and neither `exit` nor `throw` survives that. A run that
# went ahead prints a line per hold; one that refused prints REFUSING and stops.

param(
    [Parameter(Mandatory)][ValidateSet('duty', 'resident')][string]$Sweep,
    [Parameter(Mandatory)][double[]]$Values,
    [Parameter(Mandatory)][double]$Reference,
    [int]$Sessions = 100,
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
$refWork = if ($Sweep -eq 'duty') { Work $Reference 20 } else { Work 0.27 $Reference }

# References at both ends and between every pair, so each swept point sits
# inside a bracket rather than beside one. The gap between two references is the
# drift, and a point is only readable when the effect exceeds it.
$plan = @(@{ n = 'ref0'; work = $refWork; tag = "reference" })
for ($i = 0; $i -lt $Values.Count; $i++) {
    $v = $Values[$i]
    $work = if ($Sweep -eq 'duty') { Work $v 20 } else { Work 0.27 $v }
    $plan += @{ n = "v$i"; work = $work; tag = "$Sweep=$v" }
    $plan += @{ n = "ref$($i+1)"; work = $refWork; tag = "reference" }
}

"`nsweeping $Sweep over $($Values -join ', ') against $Reference — $($plan.Count) holds of ${HoldSeconds}s"
$totals = @{}
foreach ($step in $plan) {
    $out = & $bench hold --label "sw-$($step.n)" --sessions $Sessions --interval 5 `
        --duration $HoldSeconds --rss-budget-gb 8 -- $spin @($step.work) 2>&1
    $m = $out | Select-String -Pattern 'rate       ([\d.]+)' | Select-Object -First 1
    if ($null -eq $m) { $out | Select-Object -Last 20; throw "$($step.n) produced no rate" }
    $per = [double]$m.Matches[0].Groups[1].Value
    $totals[$step.n] = $per * $Sessions
    "{0,-6} {1,-14} {2,8:N3} per session   {3,9:N1} total" -f $step.n, $step.tag, $per, ($per * $Sessions)
}

# --- the drift is what says whether any of it is readable.
$refs = $plan | Where-Object { $_.tag -eq 'reference' } | ForEach-Object { $totals[$_.n] }
$stat = $refs | Measure-Object -Average -Minimum -Maximum
$drift = ($stat.Maximum - $stat.Minimum) / $stat.Average * 100
"`nreferences: {0}" -f (($refs | ForEach-Object { '{0:N1}' -f $_ }) -join ', ')
"drift across them: {0:N2}%  — an effect smaller than this is not an effect" -f $drift

"`nDO NOT PRUNE bench-out until the record is written."
