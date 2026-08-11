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
# LAUNCH DETACHED, like anything that outlives a tool call:
#   Start-Process pwsh -ArgumentList '-NoProfile','-File','<this>' `
#       -RedirectStandardOutput quiet.log -RedirectStandardError quiet.err
#
# THAT REDIRECT IS A CONVENIENCE AND NOT THE RECORD. It names a fixed path, so
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
    # box. `Get-TenantCores` sums only instances above `CookedValue -gt 50`, so
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
    [int]$MaxHolds = 40
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
$holds = 0
# The outer loop is what makes this a harvester rather than a one-shot. A quiet
# window is common here and a RESTED one is not — 2 of 45 holds — so catching
# one means firing on every quiet window and keeping the one whose rate says
# the box was actually fast.
"waiting: fire under {0:N1} cores for {1} consecutive polls ({2}s apart); count interruptions above {3:N1}; keep a hold at or above {4:N1} units/s; giving up at {5:HH:mm} or {6} holds" `
    -f $FireBelow, $ConsecutiveQuiet, $PollSeconds, $CountBelow, $RestedAbove, $deadline, $MaxHolds

while ((Get-Date) -lt $deadline) {
$run = 0
$polls = 0
$inInterruption = $false
while ((Get-Date) -lt $deadline) {
    $t = Get-TenantCores
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
    if ($t -lt 0) {
        "{0:HH:mm:ss} COUNTER UNREADABLE — not a tenancy event, resetting" -f (Get-Date)
        $run = 0
    }
    elseif ($t -lt $FireBelow) {
        $inInterruption = $false
        $run++
        "{0:HH:mm:ss} quiet {1}/{2} at {3:N2} cores" -f (Get-Date), $run, $ConsecutiveQuiet, $t
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
$label = "quiet-solo-$holds"
$out = & $bench hold --label $label --sessions 1 --interval 5 --duration 30 -- $spin @work 2>&1
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
    "{0:HH:mm:ss} hold {1}: RATE UNPARSEABLE — stopping rather than guessing" -f (Get-Date), $holds
    exit 3
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
