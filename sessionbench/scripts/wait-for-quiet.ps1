# Wait for the box to go quiet, then immediately run a pre-registered set.
#
# WHY THIS EXISTS, measured rather than guessed. On 2026-08-10 five separate
# quiet windows were lost between observing them and launching a run:
# `doctor` read 3-11% and by the time a command was issued the tenant was back
# at 78-86%. chrome-headless-shell holds 11-12 of 16 cores whenever it runs and
# its absences are under five minutes, so the latency of noticing IS the window.
#
# The waiter decides nothing. It removes the gap between the window opening and
# the run starting, then fires ONE 30-second solo hold. It fired a four-hold
# duration set until 2026-08-11; that question is closed and the set could not
# fit the window — see the comment above the hold.
#
# LAUNCH DETACHED, like anything that outlives a tool call -- AND DO NOT
# REDIRECT ITS OUTPUT:
#   Start-Process pwsh -ArgumentList '-NoProfile','-File','<this>' `
#       -WindowStyle Hidden -PassThru
#
# THE REDIRECT IS WHAT KILLS THE RUN. This comment showed one for a fortnight
# and cost four harvests: -RedirectStandardOutput forces UseShellExecute=$false,
# so the child INHERITS THE LAUNCHING CONSOLE rather than taking its own, and
# every process on a console gets the control event when it tears down. Three
# died leaving only absences -- no Stop-Transcript footer, no stderr, no event.
# The fourth was launched holding the process handle and named itself in three
# seconds: EXIT -1073741510, which is 0xC000013A, STATUS_CONTROL_C_EXIT.
# Nothing is lost, because this script writes its own timestamped transcript
# under bench-out/ -- which is the artifact actually read, and which held
# byte-identical content to the redirected log every time.
#
# A REDIRECT WAS NEVER THE RECORD EITHER. The one shown here named a fixed
# path, so
# every invocation overwrote the last and only the newest census survived —
# which cost something real: the aftermath of five of the eight band readings
# was unrecoverable, so a question this box had already answered had to be
# settled from three observations instead of eight. Sequential and silent,
# which is worse than the concurrent version of the same defect, where two
# gate runs sharing one temp path at least made the second redirect fail
# loudly. Nothing here fails; a later read just comes back empty.
#
# So the script keeps its own transcript under `bench-out/` with a timestamp
# in the name, and the launcher cannot get that wrong by choosing a path.
# Appending to one file would not do: a census means nothing without the
# parameters it ran under, and appending mixes runs that had different ones.
# A SINGLE READING OF 0.00 MEANS STARTING AS READILY AS GONE. The first
# attempt fired on exactly that and its first hold recorded 12.20 cores held,
# voiding the set. `Get-Counter` is a point sample, and a process ramping up
# reads zero on the way; requiring the quiet to persist for several polls is
# what distinguishes an absence from an instant.
# TWO BARS, BECAUSE FIRING AND COUNTING WANT DIFFERENT ONES. Firing is strict:
# a set costs five and a half minutes and voids if any hold sees more than
# ~2.5 cores held. Counting wants a bar above the readings that are not a
# settled tenant, so the transition log stays readable.
#
# Nine rejections on 2026-08-10 split with nothing between 2.20 and 8.14:
#   band    1.02  1.04  1.30  1.51  1.60  1.94  2.20
#   tenant  8.14 10.32 10.50 12.89
# One of those band rejections came after FIVE quiet polls, one short of
# firing, refused by two hundredths of a core.
#
# THAT SPLIT WAS READ AS AN IDLE FLOOR BREATHING BETWEEN 1.0 AND 2.0, AND IT
# IS NOT. On 2026-08-11 the band readings were all the TENANT IN TRANSITION:
# leading edges at 0.53, 0.55, 0.55 — each the last reading before an arrival —
# and trailing edges at 0.82 and 0.76, each the first poll after a departure.
# Seven of thirteen quiet-phase polls read exactly 0.00, which here means no
# process is over half a core, since this counter sums only instances above
# `CookedValue -gt 50`. So there is no floor band; the smallest non-zero value
# this instrument can emit is just above 0.50, because that is the threshold
# for being counted at all.
#
# The empty gap between 2.20 and 8.14 is a SAMPLING ARTIFACT rather than two
# populations: one cycle went 0.55 -> 11.38 in a single poll, crossing the
# whole range in eleven seconds. And a reading of 2.89 — below `CountBelow` —
# arrived twelve seconds before a 10.78-core arrival, so the counting bar is
# not a safe firing bar and the strict `FireBelow` is what caught both.
#
# THE ARM IS AS LONG AS THE WINDOW, WHICH MAKES FIRING NEAR-RANDOM.
# `ConsecutiveQuiet` 6 at `PollSeconds` 10 needs 50-60s of verified quiet
# before a hold starts. This box's measured gaps are 49, 50 and ~70s, against
# a tenant present 4m00s to 4m15s each cycle. See task #67.
#
# AND QUIET IS NOT THE STATE THE GATE WANTS. This waits for the tenant to
# leave; gate M1's baseline needs the box RESTED. Of 45 holds with known
# tenancy, 33 were slow, 7 tenanted, 3 between and 2 rested — so a caught
# window is usually the slow state at 9-11 units/s, useless as a baseline.
# The bands barely overlap (9-11 slow, 12-17 tenanted, 18.9-21.9 rested and
# quiet), so a fired hold's own rate says which one you got.
#
# "QUIET" HERE MEANS "NO BIG PROCESS", NEVER "IDLE", AND ON THIS BOX THE GAP
# IS ABOUT TWO CORES. The census half of `Get-Cores` sums only instances above
# `CookedValue -gt 50` — half a core each — so a machine carrying 1.87 cores
# spread across many small processes reads exactly 0.00. Measured
# 2026-08-11 at 10:57: `\Processor(_Total)` said 1.87 cores busy while NOT ONE
# process exceeded half a core.
#
# That is why every hold the harvester certified quiet landed at
# `rest_cores_median` between 0.99 and 2.44. Not a calibration error — the two
# instruments measure different things, and the certifying one cannot see the
# quantity that [turned out to matter
# most](../../docs/measurements/2026-08-11-103621-the-step-is-at-one-and-a-half-cores.md):
# a lone session's rate steps 33% across 1.36-1.46 cores held.
#
# So a run needing a genuinely low-tenancy arm cannot get one by waiting here.
# The floor is not the browser leaving; it is ~1.9 cores of small processes
# that never leave, and the lowest reading all day was 1.30 — at the transition
# rather than below it.
#
# THE POLL INTERVAL HAS A FLOOR OF ABOUT 2-3 SECONDS, MEASURED. `Get-Counter`
# costs ~1.1s per call whatever you ask it for — 1206 ms for the 346-instance
# `\Process(*)\% Processor Time` used here, 1093 ms for
# `\Processor(_Total)\% Processor Time`, 1085 ms for one named process. The
# expense is that `% Processor Time` is a RATE, so it takes two internal
# samples about a second apart to compute one. A 3s poll therefore spends 37%
# of its interval inside the call, a 1s poll is impossible, and N polls at 3s
# observe 3N seconds of elapsed time but only N seconds of machine.
#
# The instance form is the expensive one in CPU rather than latency: 0.216
# CPU-seconds per call enumerating every process twice, against `_Total`,
# which waits rather than computes. That matters because the poller runs
# inside the window being measured. Switching is a RECALIBRATION and not a
# substitution — `_Total` is a percentage of the whole machine (76.97% with
# the tenant at ~12.3 of 16 cores) where this sums only processes above half
# a core, so both bars would have to be re-derived from readings.
param(
    # 0.5, not 1.0, and the value is set by the instrument rather than by the
    # box. The census half of `Get-Cores` sums only instances above 50, so
    # the smallest non-zero number it can emit is just above 0.50 and there is
    # nothing between 0 and 0.5 to observe. A bar at 1.0 therefore admits
    # exactly one class of reading — the 0.5-1.0 sliver — and every value seen
    # there was a tenant edge: leading 0.53, 0.55, 0.55 and trailing 0.82,
    # 0.76, 0.67. One of those fired a hold during a live test.
    #
    # So any bar in (0, 0.5] is the same rule — require exactly 0.00, meaning
    # no process is over half a core — and it rejects every edge this counter
    # can see. This one cannot be tuned by watching readings, because the
    # readings it would need do not exist.
    #
    # THE SLIVER HOLDS TWO POPULATIONS AND THIS COUNTER CANNOT TELL THEM
    # APART. The six edge readings above were transients before an 11-core
    # arrival. A 42-minute run on 2026-08-11 produced twenty more — 17 of them
    # between 0.5 and 1.1 — and NONE was followed by an arrival: a steady
    # background rather than a tenant. The obvious discriminator is the
    # previous reading (ascending from 0.00 means arrival, steady means
    # background), and it does not work: the background reaches the counter as
    # 0.00 → 0.51 → 0.00 → 0.62, flickering across the half-core threshold
    # exactly as a leading edge does. Only what happens AFTER separates them.
    #
    # SO THE BAR IS A CHOICE OF PURPOSE, NOT A FACT. Strict (0.5) protects a
    # clean baseline and costs windows — this run took 13 holds in 30 minutes
    # and then none for 12 while a ~1.7-core process sat there. Loose (1.5)
    # harvests more and lands holds on both sides of the 1.4-core step, which
    # is how that step was measured in the first place. Every hold records its
    # own `rest_cores_median`, so a spoiled hold is detectable afterwards
    # either way. Set it for the run you are doing; there is no value that
    # serves both.
    [double]$FireBelow = 0.5,
    # THE SECOND BAR, AND IT IS THE ONE THAT MATCHES WHAT THE HOLD MEASURES.
    #
    # `FireBelow` above governs the CENSUS counter, which sums only instances
    # over half a core and is right for naming which process interrupted a
    # window. It cannot gate, because load spread across many small processes
    # reads exactly 0.00 to it. This bar reads `_Total - Idle - ours`, the same
    # quantity a hold reports as `rest_cores_median`, and firing needs BOTH --
    # so this can only ever withhold a window the old bar would have fired, and
    # every comment above about `FireBelow` keeps its meaning.
    #
    # 1.0 IS DERIVED FROM THREE HOLDS on 2026-08-11, not chosen: the injection
    # control voided two of its first three attempts, its gate reading 0.00
    # before baselines whose own rest came back at 3.26 and 1.43 against a guard
    # that rejects at 1.3 -- while the attempt that PASSED also read 0.00 at the
    # gate, against a true 0.75. So 1.0 admits the passing case with margin and
    # excludes both voided ones, and sits below the guard it feeds.
    [double]$FireBelowMachine = 1.0,
    [double]$CountBelow = 3.0,
    [int]$PollSeconds = 10,
    # Two, not six. Six polls at ten seconds needs 50-60s of verified quiet
    # before a hold starts, and this box's measured gaps are 49, 50 and ~70s —
    # so the arm was as long as the window and firing was closer to a coin
    # toss than a decision. Two polls is 10s of verification, and 10s + a 30s
    # hold fits every gap observed. It is not one poll, because the very first
    # attempt fired on a single 0.00 and its hold recorded 12.20 cores held:
    # a process ramping up reads zero on the way, so one confirmation is what
    # separates an absence from an instant. Beyond that, `rest_cores_median`
    # on the hold itself catches what verification would have.
    [int]$ConsecutiveQuiet = 2,
    [int]$HeartbeatPolls = 30,
    [int]$GiveUpMinutes = 120,
    # The rate above which a 30s solo hold says the box was RESTED rather than
    # merely unoccupied. Bands on this box: 9-11 slow, 12-17 with a neighbour,
    # 18.9-21.9 rested and quiet. 18 sits in the gap below the rested band's
    # floor and above every tenanted reading recorded (max 17.101).
    [double]$RestedAbove = 18.0,
    # A stop independent of the clock, so a box that never rests cannot spend
    # two hours firing holds. 40 x 30s is 20 minutes of load spread over the
    # window, which is light and bounded.
    [int]$MaxHolds = 40,
    # ONE FOOTPRINT MAKES THIS A READINESS PROBE; MORE THAN ONE MAKES IT A
    # HARVEST, and the difference is whether `RestedAbove` means anything.
    #
    # Default `20` is the probe `sessionbench/README.md` documents: fire on a
    # quiet window, hold, and exit 0 the moment the rate says the box is
    # rested. Pass `-Residents 20,1` and the holds alternate footprint, which
    # is [the test for whether the 1.4-core step is
    # memory-bound](../../docs/measurements/2026-08-11-103621-the-step-is-at-one-and-a-half-cores.md):
    # both arms sample the same wandering tenancy, so after enough holds there
    # are two rest-versus-rate curves. A large step in the 20 MiB curve and a
    # small one in the 1 MiB curve says the effect is memory-bound; equal steps
    # say it is not.
    #
    # **`RestedAbove` is skipped when more than one footprint is given**, and
    # it has to be: a 1 MiB session is a different workload with a different
    # rate level, so a threshold calibrated on 20 MiB would exit on the wrong
    # arm. A harvest runs to `MaxHolds` and is read afterwards. Only the SIZE
    # of each curve's step is comparable — never the two levels.
    #
    # This varies the one thing the box has no opinion about. Controlling
    # tenancy needs a state on demand that this machine will not give: at 1.87
    # cores busy not one process exceeded half a core, so the floor is small
    # processes that never leave. Tenancy varies anyway — 4 of 17 holds landed
    # below the transition and 13 above — so let it, record it, and vary the
    # footprint on purpose.
    # A STRING, SPLIT HERE, BECAUSE `-File` CANNOT BIND AN ARRAY AT ALL.
    # Measured on 2026-08-11 against a `[int[]]` parameter: `-R 20,1` binds the
    # single integer **201** (PowerShell strips the comma rather than splitting
    # on it) and `-R 20 1` binds **20**, dropping the second token silently.
    # The first form ran a hold at `--resident 201` and reported a perfectly
    # believable 13.005 units/s; it was caught only because the resident is in
    # the label, so the artifact read `quiet-solo-1-r201`.
    #
    # So the parameter takes text and this script does the splitting, which is
    # the one arrangement whose behaviour does not depend on how the caller was
    # invoked. `-Residents '20,1'` is now unambiguous.
    [ValidatePattern('^\d+(,\d+)*$')]
    [string]$Residents = '20',
    # THE SAME SHAPE FOR DUTY, AND IT TESTS THE LAST MECHANISM CLASS STANDING.
    # `cpu-spin --duty 0.27` spends 73% of its time NOT RUNNING: wake, work,
    # sleep, repeat. Everything that makes a WAKE expensive is a candidate for
    # the ~1.4-core step — idle C-states, timer coalescing, preemption quantum,
    # and `coggyd`'s own pipe reader — and all four survive the eliminations
    # that killed core clock, uncore, parking and placement.
    #
    # A workload at `--duty 1.0` NEVER SLEEPS, so it can have no wake cost.
    #   PREDICTION: at duty 1.0 the step vanishes. At 0.27 it is +10 to +49%.
    #   IF IT SURVIVES AT 1.0, the whole class is out and the effect is about
    #   running rather than waking.
    #
    # All 130 solo mains holds on disk are duty 0.27, so nothing in the archive
    # can separate the two. ONLY THE SIZE OF EACH DUTY'S STEP IS COMPARABLE —
    # duty changes the absolute rate, so the levels are not.
    [ValidatePattern('^[0-9.]+(,[0-9.]+)*$')]
    [string]$Duties = '0.27'
)

$ErrorActionPreference = 'Stop'
$root = 'C:\Users\LilMG\Desktop\coggy'
Set-Location $root
$bench = "$root\target\release\sessionbench.exe"
$spin = "$root\target\release\cpu-spin.exe"
$work = @('--units', '100000000')

# Refuse rather than measure on a dirty machine: a survivor from a killed run
# would be counted as a neighbour and disqualify the set for the wrong reason.
$stray = Get-Process coggyd, cpu-spin, sessionbench -ErrorAction SilentlyContinue
if ($stray) { "REFUSING: {0} stray process(es)" -f $stray.Count; exit 1 }

# AND REFUSE A SECOND WAITER, which the stray check above does not cover. Two
# instances polling at once can fire sets seconds apart, and each set would
# then measure the other's cpu-spin as tenancy at about 0.27 cores — far under
# the 2.5 that disqualifies a set, so the rule would pass a contaminated run.
# This happened on 2026-08-10; it was harmless only because the loser never
# fired, which was luck rather than design.
#
# A PID lock file rather than matching command lines: the first attempt looked
# for another `pwsh` whose command line mentions this script, and the PARENT
# that launches it matches that too — so it refused itself. A lock is the
# ordinary mechanism and does not need to tell a parent from a peer.
$lock = Join-Path $root '.wait-for-quiet.lock'
if (Test-Path $lock) {
    $holder = (Get-Content $lock -ErrorAction SilentlyContinue | Select-Object -First 1)
    $alive = $holder -and (Get-Process -Id $holder -ErrorAction SilentlyContinue)
    if ($alive) { "REFUSING: another waiter holds the lock (pid $holder)"; exit 1 }
    "stale lock from pid $holder, taking it"
}
Set-Content -Path $lock -Value $PID
# The script's own record, named so no invocation can overwrite another's.
# Under bench-out/ because it is gitignored scratch, and a census log is three
# orders below the ramps the pruning rule was written for.
$census = Join-Path $root ("bench-out\quiet-{0}.log" -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
$null = New-Item -ItemType Directory -Force -Path (Split-Path $census)
Start-Transcript -Path $census | Out-Null
try {


# WHAT THE HOLD WILL MEASURE, from the same query, at full resolution.
#
# `Get-Cores`'s census figure sums only instances over half a core, and that is the
# right shape for the CENSUS -- it names which processes interrupted a window.
# It is the wrong shape for a GATE, because load spread across many sub-half-core
# processes is invisible to it and it returns exactly 0.00.
#
# MEASURED, not suspected: on 2026-08-11 the injection control voided two of its
# first three attempts, and in both cases the gate had just printed "quiet 2/2 at
# 0.00 cores" before a baseline hold whose own `rest_cores_median` came back at
# 3.26 and 1.43, against a guard that rejects at 1.3. The hold that PASSED also
# read 0.00 at the gate against a true 0.75 -- so a gate reading of zero is
# consistent with anything from ~0.6 to ~1.5 true cores, a band that STRADDLES
# the guard. That is the whole void rate, and it is arithmetic rather than luck.
#
# It is BLIND rather than STALE, which the artifacts settle: the voided holds'
# per-tick machine CPU shows a STEADY 1.0-1.5 cores across the whole window, and
# steady load cannot have arrived after the gate. A live comparison at 14:26 on a
# box the gate called silent: _total 1558.8, idle 1482.4, difference 0.76 cores.
#
# `_Total` minus `Idle` is the machine, from the query already made, at no extra
# cost -- one `Get-Counter` is ~1.1 s whatever is asked of it, because a `%`
# counter is a rate needing two internal samples. Our own instances are excluded
# by the same names the census filter uses, since a hold's own session must not
# gate the next one.
#
# NOTE this is NOT a second route to the same number: `_Total` IS the sum over
# all instances, so `_total - idle` and `sum(all but idle)` are one identity and
# cannot disagree. It agrees with `rest_cores_median` because it measures the
# same thing, which is the point.
function Get-Cores {
    # ONE QUERY, BOTH FIGURES. The first version queried twice -- once for the
    # census, once for the machine -- and the second call INTERMITTENTLY FAILED,
    # returning the sentinel and printing COUNTER UNREADABLE on a box that was
    # merely being asked twice in a row. Two `%`-counter queries back to back is
    # the problem; the comment above already said the machine reading comes from
    # the query already made, and then a second function went and made one.
    # Halves the poll cost as well: ~1.1 s per call whatever is asked.
    $c = Get-Counter '\Process(*)\% Processor Time' -ErrorAction SilentlyContinue
    if ($null -eq $c) {
        Start-Sleep -Milliseconds 500
        $c = Get-Counter '\Process(*)\% Processor Time' -ErrorAction SilentlyContinue
    }
    # -1 in BOTH fields for unreadable: not a quiet machine, and it must not
    # fire. The branch testing it comes FIRST at the call site, and that order is
    # load-bearing -- a negative sentinel passes every `-lt` test there is.
    if ($null -eq $c) { return [pscustomobject]@{ Tenant = -1.0; Machine = -1.0 } }
    $s = $c.CounterSamples
    $total = ($s | Where-Object { $_.InstanceName -eq '_total' }).CookedValue
    $idle = ($s | Where-Object { $_.InstanceName -eq 'idle' }).CookedValue
    if ($null -eq $total -or $null -eq $idle) { return [pscustomobject]@{ Tenant = -1.0; Machine = -1.0 } }
    $mine = { $_.InstanceName -like 'sessionbench*' -or $_.InstanceName -like 'cpu-spin*' -or
        $_.InstanceName -like 'coggyd*' -or $_.InstanceName -like 'file-write*' -or
        $_.InstanceName -like 'stdout-storm*' }
    $ours = ($s | Where-Object $mine | Measure-Object CookedValue -Sum).Sum
    # A NEGATIVE MACHINE READING IS NONSENSE, NOT A MEASUREMENT, and it happens:
    # `% Processor Time` is a rate taken from two internal samples about a second
    # apart, so processes starting or exiting between them leave `_Total` and
    # `Idle` inconsistent. Observed 2026-08-11 at **-1.95 cores** six seconds
    # after six processes were spawned, while five readings on an undisturbed box
    # were all positive.
    #
    # It must not simply fall through: the call site tests `$m -lt 0` as the
    # UNREADABLE sentinel, so a churn artifact would be logged as a failed query
    # — and this census is the record of how long this box's quiet stretches are,
    # where a mislabelled event inflates a count of interruptions that never
    # happened. Same defect as #72, one layer down. Withholding the window is
    # right either way; saying WHY is what changes.
    # BUT ONLY A LARGE NEGATIVE. `_Total` and `Idle` are computed over slightly
    # offset internal samples, so their difference drifts by hundredths — and it
    # drifts closest to zero when the machine is NEAREST IDLE, which is the
    # reading a quiet gate exists to find. Rejecting every negative therefore
    # discards the best windows and resets the consecutive-quiet run: the 22:25
    # injection run logged 4 churn rejections against 2 quiet polls in forty
    # minutes, twice as many thrown away as let through.
    #
    # THE THRESHOLD IS IN THE COUNTER'S OWN UNITS, where 100 is one core, so -25
    # is a quarter of a core. The observed artifact was -195; a quiet box's
    # drift is single digits.
    $machine = $total - $idle - $ours
    if ($null -ne $total -and $null -ne $idle -and $machine -lt -25) {
        return [pscustomobject]@{ Tenant = -1.0; Machine = -2.0 }
    }
    if ($machine -lt 0) { $machine = 0.0 }
    # The census: only instances over half a core, which is what names WHICH
    # process interrupted a window. Blind to load spread across small ones.
    $busy = $s |
        Where-Object { $_.InstanceName -notin @('_total', 'idle') -and $_.CookedValue -gt 50 } |
        Where-Object { -not (& $mine) }
    $tenant = if ($null -eq $busy) { 0.0 } else { ($busy | Measure-Object CookedValue -Sum).Sum / 100 }
    [pscustomobject]@{ Tenant = $tenant; Machine = ($machine / 100) }
}

# Parsed once, here, rather than trusting the binder — see the parameter.
$residentList = @($Residents -split ',' | ForEach-Object { [int]$_ })
$dutyList = @($Duties -split ',' | ForEach-Object { [double]$_ })
foreach ($d in $dutyList) {
    if ($d -le 0 -or $d -gt 1) { "REFUSING: duty $d outside (0, 1]"; exit 1 }
}

# A HARVEST AND A PROBE WANT DIFFERENT FIRE BARS, and the default is the
# probe's. #74 established the bar has no correct value across the two modes:
# strict (0.5) protects a clean baseline and costs windows, loose (1.5)
# harvests and lands holds on both sides of the ~1.4-core transition. The
# parameter comment said "set it for the run you are doing" and that did not
# fire at the moment of use -- a footprint harvest launched on the default
# produced ZERO HOLDS IN 2.5 MINUTES, blocked at readings of 0.51.
#
# The script already knows which mode it is in, and already acts on it once:
# RestedAbove is skipped when there is more than one footprint. The bar and
# that threshold imply each other, so they move together.
#
# `ContainsKey` is exact rather than a sentinel comparison. A sentinel here
# would be the defect this repo already paid for, where an unreadable-counter
# value of -1 silently passed a less-than test.
if (($residentList.Count -gt 1 -or $dutyList.Count -gt 1) -and -not $PSBoundParameters.ContainsKey('FireBelow')) {
    $FireBelow = 1.5
    # The machine bar moves with it, or the loose census bar would be gated by a
    # strict machine bar and the harvest would be no looser than the probe. The
    # offset is the ~1.0-1.3 cores the census counter cannot see, measured the
    # same day: gate 0.00 against true 0.75, 1.43 and 3.26.
    if (-not $PSBoundParameters.ContainsKey('FireBelowMachine')) { $FireBelowMachine = 2.5 }
    "harvest mode: FireBelow raised to 1.5, FireBelowMachine to $FireBelowMachine (pass either to override)"
}
foreach ($r in $residentList) {
    if ($r -lt 1 -or $r -gt 64) { "REFUSING: resident $r outside 1-64"; exit 1 }
}
$deadline = (Get-Date).AddMinutes($GiveUpMinutes)
$holds = 0
# The outer loop is what makes this a harvester rather than a one-shot. A quiet
# window is common here and a RESTED one is not — 2 of 45 holds — so catching
# one means firing on every quiet window and keeping the one whose rate says
# the box was actually fast.
"waiting: fire under {0:N1} census cores AND {7:N2} machine-wide for {1} consecutive polls ({2}s apart); count interruptions above {3:N1}; keep a hold at or above {4:N1} units/s; giving up at {5:HH:mm} or {6} holds" `
    -f $FireBelow, $ConsecutiveQuiet, $PollSeconds, $CountBelow, $RestedAbove, $deadline, $MaxHolds, $FireBelowMachine

while ((Get-Date) -lt $deadline) {
$run = 0
$polls = 0
$inInterruption = $false
while ((Get-Date) -lt $deadline) {
    # One query, both readings: the census bar names the interrupting process,
    # the machine bar decides whether it is really quiet.
    $cores = Get-Cores
    $t = $cores.Tenant
    $m = $cores.Machine
    $polls++
    # A heartbeat whatever happens, because the branches below only speak when
    # the run counter moves — so an unbroken hour of load produced NO lines at
    # all, and a census could not tell it from an hour of nothing happening.
    if ($polls % $HeartbeatPolls -eq 0) {
        "{0:HH:mm:ss} heartbeat — {1} polls, tenant now {2:N2} cores, quiet run {3}/{4}" `
            -f (Get-Date), $polls, $t, $run, $ConsecutiveQuiet
    }
    # The sentinel is tested FIRST and this order is load-bearing: -1 is less
    # than any sane MaxTenantCores, so checking quiet first would read an
    # unreadable counter as the quietest possible machine and fire the set.
    if ($m -eq -2.0) {
        # Distinguished from an unreadable query so the census is not polluted
        # with interruptions that never happened.
        "{0:HH:mm:ss} COUNTER INCONSISTENT (negative machine reading, process churn) — not a tenancy event, resetting" -f (Get-Date)
        $run = 0
    }
    elseif ($t -lt 0 -or $m -lt 0) {
        "{0:HH:mm:ss} COUNTER UNREADABLE — not a tenancy event, resetting" -f (Get-Date)
        $run = 0
    }
    elseif ($t -lt $FireBelow -and $m -ge $FireBelowMachine) {
        # Quiet to the census and NOT to the machine: the exact state that voided
        # two of three injection attempts, and it used to fire silently.
        $inInterruption = $false
        $run = 0
        "{0:HH:mm:ss} no big process but {1:N2} cores busy machine-wide (bar {2:N2}) — not quiet, resetting" -f (Get-Date), $m, $FireBelowMachine
    }
    elseif ($t -lt $FireBelow) {
        $inInterruption = $false
        $run++
        "{0:HH:mm:ss} quiet {1}/{2} at {3:N2} cores ({4:N2} machine-wide)" -f (Get-Date), $run, $ConsecutiveQuiet, $t, $m
        if ($run -ge $ConsecutiveQuiet) {
            "{0:HH:mm:ss} WINDOW OPEN — quiet held for {1}s, starting" -f (Get-Date), ($run * $PollSeconds)
            break
        }
    }
    elseif ($t -ge $CountBelow) {
        # A real interruption: above the floor's band, so this is the neighbour.
        #
        # **Logged on the EDGE, not on every poll.** The first version fired
        # this branch whenever the reading was high, so a sustained tenant
        # wrote a line every ten seconds — 4 lines in the first 40 seconds of
        # its first run, which is 360 an hour and a census nobody can read. A
        # census wants one line per arrival plus the heartbeat, so the state
        # has to be remembered rather than re-derived each poll.
        if (-not $inInterruption) {
            "{0:HH:mm:ss} INTERRUPTION at {1:N2} cores after {2} quiet poll(s)" -f (Get-Date), $t, $run
            $inInterruption = $true
        }
        $run = 0
    }
    elseif ($run -gt 0) {
        # Between the bars. Read as the idle floor breathing when this was
        # written; the 2026-08-11 census says it is the TENANT IN TRANSITION —
        # a leading edge on the way up or a trailing edge on the way down, and
        # the smallest non-zero value this counter can emit is just above 0.50
        # because that is its per-process threshold. Resetting the run is right
        # either way, and more clearly so: an ascending edge is the window
        # ending. The label stays `floor` so old census logs remain greppable.
        "{0:HH:mm:ss} floor {1:N2} cores after {2} quiet poll(s) — not an interruption" -f (Get-Date), $t, $run
        $inInterruption = $false
        $run = 0
    }
    Start-Sleep -Seconds $PollSeconds
}
if ($run -lt $ConsecutiveQuiet) { "gave up without a window"; exit 2 }
$holds++

# ONE HOLD, NOT A SET. This fired four alternating holds — 30, 120, 30, 120 —
# to compare durations inside one window. Two reasons that is now wrong.
#
# The question is closed. A 30s hold reads at most ~1.6% differently from a
# 120s one at matched tenancy, against a 25-35% claim that came from comparing
# ten holds in one window with fifty-two spanning a day. Six accounts agree.
# The set was re-measuring a settled thing.
#
# And it could not fit. The set costs about five minutes; this box's measured
# quiet gaps are 49, 50 and ~70 seconds, with the tenant present 4m00s-4m15s
# each cycle. Twelve sets fired and twelve voided, the neighbour usually
# arriving inside the FIRST hold. One 30s hold after ~10s of verification is
# 40s, which fits every gap observed.
#
# The safety net is that the hold records `occupancy.rest_cores_median`
# itself, so a spoiled hold is detectable afterwards rather than needing to be
# prevented beforehand — which is how the 34% step and the r = -0.950 relation
# were both obtained, from holds sorted by their rest column after the fact.
$resident = $residentList[($holds - 1) % $residentList.Count]
$duty = $dutyList[($holds - 1) % $dutyList.Count]
$label = "quiet-solo-$holds-r$resident-d$duty"
$out = & $bench hold --label $label --sessions 1 --interval 5 --duration 30 `
    -- $spin @work --duty $duty --resident $resident 2>&1
$out | Select-String -Pattern 'rate |cores held outside the job' |
    ForEach-Object { "{0}: {1}" -f $label, $_.Line.Trim() }

# QUIET IS NOT THE STATE THE GATE WANTS, so the hold's own rate decides whether
# this window was worth catching. On this box the bands barely overlap — 9-11
# slow, 12-17 with a neighbour, 18.9-21.9 rested and quiet — and of 45 holds
# with known tenancy only 2 were rested. So a caught window is usually the slow
# state, and firing reliably into it is not progress.
#
# `$rate` stays $null when the line cannot be parsed, and the branch order
# below is load-bearing: a 0.0 default would read as the slowest possible
# machine and keep waiting forever, which is the same defect as a sentinel
# comparable to the quantity it stands in for.
$rate = $null
$m = ($out | Select-String -Pattern '^\s*rate\s+([\d.]+)\s+units/s' | Select-Object -First 1)
if ($m) { $rate = [double]$m.Matches[0].Groups[1].Value }

if ($null -eq $rate) {
    # REFUSING IS RIGHT AND DISCARDING THE EVIDENCE IS NOT. This fired for real
    # on 2026-08-11 at hold 9: `sessionbench` exited during warmup after three
    # samples, wrote no `hold.json`, and left nothing on stderr — so the only
    # record of *why* was the output this branch had just failed to parse, and
    # the branch exited without printing it. A verdict is not evidence; the
    # thing the verdict was computed from has to travel with it.
    "{0:HH:mm:ss} hold {1}: RATE UNPARSEABLE — stopping rather than guessing" -f (Get-Date), $holds
    '--- captured output follows, which is the whole reason this is unrecoverable otherwise ---'
    $out | ForEach-Object { "    $_" }
    '--- end captured output ---'
    exit 3
}
if ($residentList.Count -gt 1 -or $dutyList.Count -gt 1) {
    # A harvest, not a probe: the arms have different rate levels, so no single
    # RestedAbove can judge them. Run to MaxHolds and read the artifacts.
    "{0:HH:mm:ss} hold {1}: {2:N3} units/s at resident {3} — harvesting, {4} of {5}" -f (Get-Date), $holds, $rate, $resident, $holds, $MaxHolds
    if ($holds -ge $MaxHolds) { "reached MaxHolds ($MaxHolds); read bench-out/*quiet-solo-*/hold.json"; exit 0 }
    continue
}
if ($rate -ge $RestedAbove) {
    "{0:HH:mm:ss} hold {1}: {2:N3} units/s — RESTED AND QUIET, this is the window" -f (Get-Date), $holds, $rate
    exit 0
}
"{0:HH:mm:ss} hold {1}: {2:N3} units/s — quiet but not rested, waiting for another" -f (Get-Date), $holds, $rate
if ($holds -ge $MaxHolds) { "reached MaxHolds ($MaxHolds) without a rested window"; exit 4 }
}
"gave up: deadline reached after $holds hold(s), none rested"
exit 2
}
finally {
    Remove-Item $lock -ErrorAction SilentlyContinue
    # Every exit from here is an `exit N` inside the try, so the transcript
    # only ever closes on this path. Errors are swallowed because a failure to
    # stop transcribing must not change the exit code the caller reads.
    try { Stop-Transcript | Out-Null } catch {}
}
