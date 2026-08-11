# Read a duty sweep's finished holds, binned by tenancy, one sitting at a time.
#
# Pairs with `wait-for-quiet.ps1 -Duties 0.27,1.0`, which labels each hold
# `quiet-solo-<n>-r<res>-d<duty>-daemon`. Written while the first sweep was still
# running, deliberately, so the bands and the floor could not be chosen for the
# answer they produced.
#
# READS hold.json, NOT the transcript and NOT samples.jsonl. `occupancy` is
# computed by sampler.rs over a window that starts at the first sample where the
# job reaches its rough median, so the session's startup is excluded; an ad-hoc
# pass over the raw samples loses that guard.
#
# THE COMPARISON COLUMN IS rate/job, NEVER rate. `job` is the ACHIEVED duty, and
# it falls to 0.09-0.27 under load because the session is descheduled -- so raw
# rate confounds "how much CPU did it get" with "what did it do per core-second".
#
# TWO BUGS LIVED IN THE FIRST VERSION, both from the `-daemon` suffix: a glob of
# `*quiet-solo-*-d*` matched it and swept in every previous day's holds (69 rows
# across 13 sittings, most carrying no duty at all), and splitting on `-d`
# returned "aemon" as the duty for all of them. An anchored regex refuses
# anything that is not a duty-labelled hold rather than guessing at a filename's
# tail.
param(
    [string]$Root = 'C:\Users\LilMG\Desktop\coggy\bench-out',
    # A RATIO'S DENOMINATOR DECIDES ITS NOISE, and this one gets small. Holds
    # come back at job=0.05 cores at duty 0.27 and 0.18 at duty 1.0, the session
    # descheduled almost to nothing. `median_cores` is a median over 5-second
    # samples of a quantity near the sampler's own resolution, so a couple of
    # hundredths of absolute error is ~5% at job=0.5 and ~40% at job=0.05.
    # rate/job does not become WRONG there, it becomes UNINFORMATIVE -- and it
    # gets loud in exactly the high-tenancy band where the arms are compared.
    # Held out and listed rather than dropped, so the cut can be checked.
    [double]$Floor = 0.15,
    # A gap longer than this means the box was doing something else in between,
    # and a comparison across sittings varies time as well as the thing you meant.
    [int]$SittingGapSeconds = 300
)

$rows = @()
foreach ($d in Get-ChildItem $Root -Directory) {
    if ($d.Name -notmatch '^(\d+)-quiet-solo-\d+-r\d+-d([0-9.]+)-daemon$') { continue }
    $epoch = [long]$Matches[1]
    $duty = $Matches[2]
    $f = Join-Path $d.FullName 'hold.json'
    if (-not (Test-Path $f)) { continue }
    $j = Get-Content $f -Raw | ConvertFrom-Json
    $rate = $j.units_per_session_per_sec
    $job = $j.occupancy.median_cores
    $rest = $j.occupancy.rest_cores_median
    # An absent field must not become a zero: a 0.0 job reads as an infinitely
    # efficient session and a 0.0 rest as a perfectly idle machine.
    if ($null -eq $rate -or $null -eq $job -or $null -eq $rest -or $job -eq 0) {
        Write-Host "INCOMPLETE $($d.Name): rate=$rate job=$job rest=$rest"
        continue
    }
    $rows += [pscustomobject]@{
        T = $epoch; Duty = $duty; Rate = [double]$rate; Job = [double]$job
        Rest = [double]$rest; Per = [double]$rate / [double]$job; Name = $d.Name
    }
}
if ($rows.Count -eq 0) { Write-Host "no duty-labelled holds under $Root"; exit 1 }

$thin = @($rows | Where-Object { $_.Job -lt $Floor })
$rows = @($rows | Where-Object { $_.Job -ge $Floor } | Sort-Object T)
if ($thin.Count -gt 0) {
    Write-Host ("{0} hold(s) below job={1:N2} cores, held out of the bands (ratio too noisy to read):" -f $thin.Count, $Floor)
    foreach ($r in ($thin | Sort-Object T)) {
        Write-Host ("    {0,-5} rest {1,-6:N2} rate {2,-8:N3} job {3,-5:N2} -> {4:N1}" -f $r.Duty, $r.Rest, $r.Rate, $r.Job, $r.Per)
    }
    Write-Host ''
}

$sittings = @(); $cur = @(); $last = $null
foreach ($r in $rows) {
    if ($null -ne $last -and ($r.T - $last) -gt $SittingGapSeconds) { $sittings += , $cur; $cur = @() }
    $cur += $r; $last = $r.T
}
$sittings += , $cur

Write-Host ("{0} holds in {1} sitting(s)`n" -f $rows.Count, $sittings.Count)
$bands = @(@(0, 1.5), @(1.5, 3.5), @(3.5, 8.0), @(8.0, 99))
$i = 0
foreach ($s in $sittings) {
    $i++
    $span = ($s[-1].T - $s[0].T) / 60.0
    Write-Host ("=== sitting {0}: {1} holds over {2:N1} min" -f $i, $s.Count, $span)
    Write-Host ("    {0,-6} {1,-7} {2,-9} {3,-6} {4}" -f 'duty', 'rest', 'rate', 'job', 'rate/job')
    foreach ($r in ($s | Sort-Object Duty, Rest)) {
        Write-Host ("    {0,-6} {1,-7:N2} {2,-9:N3} {3,-6:N2} {4:N2}" -f $r.Duty, $r.Rest, $r.Rate, $r.Job, $r.Per)
    }
    # THE FLOOR CENSORS THE ARMS UNEQUALLY, so a band where one arm lost holds
    # and the other did not is not a comparison. At high tenancy duty 0.27 comes
    # back at 0.05-0.09 cores while duty 1.0 at the same tenancy holds 0.18-0.53:
    # a session demanding a full core keeps a larger share than one that sleeps
    # and loses its slot. The survivors of a censored arm are its LEAST-starved
    # holds, which biases its mean upward.
    Write-Host '    -- by tenancy band, rate/job, mean (n) [* = this arm lost holds to the floor] --'
    Write-Host ("        {0,-12} {1,-18} {2}" -f 'band', 'd0.27', 'd1')
    foreach ($b in $bands) {
        $cells = @()
        foreach ($duty in @('0.27', '1')) {
            $v = @($s | Where-Object { $_.Duty -eq $duty -and $_.Rest -ge $b[0] -and $_.Rest -lt $b[1] })
            $c = @($thin | Where-Object { $_.Duty -eq $duty -and $_.Rest -ge $b[0] -and $_.Rest -lt $b[1] }).Count
            $mark = if ($c -gt 0) { '*' * [Math]::Min($c, 3) } else { '' }
            $cells += if ($v.Count -gt 0) {
                '{0:N2} (n={1}){2}' -f (($v | Measure-Object Per -Average).Average), $v.Count, $mark
            } elseif ($c -gt 0) { "- ($c censored)" } else { '-' }
        }
        Write-Host ("        {0,-12} {1,-18} {2}" -f ('{0:N1}-{1:N1}' -f $b[0], $b[1]), $cells[0], $cells[1])
    }
    Write-Host ''
}
