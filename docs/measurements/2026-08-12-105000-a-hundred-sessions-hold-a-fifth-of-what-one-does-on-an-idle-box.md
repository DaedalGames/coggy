# A hundred sessions hold a fifth of what one does, on an idle box

> **2026-08-12 12:05 — superseded. [Read the ceiling record instead](2026-08-12-114500-every-hundred-session-hold-today-is-a-third-of-what-the-box-used-to-do.md).** The collapse below is real and it is not about session count or `--resident`: the box has **two stable operating points at a hundred sessions, 4.7× apart**, and every hold behind this record was taken in the low one. Per-core efficiency is identical in both, compute per unit is constant at 13–20 ms, and what varies fivefold is the **pause** — the sessions oversleep by 2× in the fast state and 11× in the slow one. What survives here is the within-day arithmetic; what it meant does not.

> **2026-08-12 10:58 — the headline is withdrawn: this is a `--resident 1` result and the gate uses `--resident 20`.** Reading the eleven hundred-session holds already on disk, the same `--duty 0.27` command spans **2.396 to 15.525 job cores** — and the four highest all ran `--resident 20` on a quiet box, holding **15.34, 15.36, 15.51 and 15.53 cores**, which is 0.155 each and **57% of the requested duty, not 13%**. They very nearly saturate the machine.
>
> **Both of my holds used `--resident 1`**, which is the one parameter that differs. So the collapse measured below is a property of a nearly-memoryless session, not of a hundred sessions as such — a conclusion about the stand-in, drawn from a stand-in that was never checked against the archive.
>
> **What survives.** Session count still matters at `--resident 1`: 92%, 66%, 31%, 13% at 1, 10, 30, 100 is monotonic and real for that workload. The fixed-pause discriminator still shows the pause is not the mechanism. And the arithmetic refuting sleep granularity and core contention stands.
>
> **What does not.** The claim that a hundred sessions do not saturate this box, and everything downstream of it — including the reading that gate M1's work-rate ratio partly measures the workload backing off. **At the gate's own `--resident 20` the sessions reach 15.5 of 16 cores when the box is quiet**, which is what the gate assumes. The M1 concern raised below is withdrawn until measured at the workload the gate uses.
>
> **The likely mechanism, now visible and still untested**: `--resident` sizes the buffer a unit touches, so compute per unit scales with it. At 1 MiB the compute is short and the pause dominates; at 20 MiB it is long enough to hold the duty. That is a testable claim and this record does not test it.

> **2026-08-12 22:38 — and THAT is withdrawn in turn: `--resident 20` run today returns 3.63 job cores on a quiet box, inside the band `--resident 1` occupies.** The flag does nothing on this machine now, so the archive's split was confounded by day rather than caused by the flag. The refutation below was right and its reasoning was wrong; the reading above was wrong and its reasoning was right. **What separates them is a same-day pair varying the flag, which is the design neither had.**

> **2026-08-12 22:28 — the refutation below is itself withdrawn, and the original `--resident` reading was right.** The back-to-back pair that refuted it ran at `rest` ~ 9, where the machine holds both arms near 4.7 cores and no lever can show itself. Restricted to QUIET holds across the whole archive, `--resident 20` gives the job **15.34 and 15.51** cores against **4.76, 3.12, 4.59 and 4.55** for `--resident 1` — a **3.3x lever**, from ten `--resident 20` holds that were on disk throughout. [The correction is in the parking record](2026-08-12-143500-ten-of-sixteen-cores-are-parked-under-a-hundred-sessions.md). **An invariant exercised only where it cannot break has not been exercised**, and that is what the pair below did.

> **2026-08-12 11:02 — that mechanism is refuted, and the withdrawal above blamed the wrong variable.** Both arms run back to back at 100 sessions, same window: `--resident 1` gives **4.675 cores** and `--resident 20` gives **4.811** — **2.9% apart**. `--resident` does not set compute per unit.
>
> **So the archive's 6.5× spread is the machine, not the parameter.** Those eleven holds also span `rest_cores_median` from 0.49 to 13.6 cores, and the four that reached 15.3–15.5 are the four taken on a quiet box. The withdrawal was correct that my result did not generalise; its explanation was a second conclusion drawn across sittings, made while correcting a conclusion drawn across sittings.
>
> **What is still unexplained**: the pattern is not monotonic in `rest`. A hold at `rest` 0.55 gave 4.76 cores and one at `rest` 0.49 gave 15.51, both quiet. Something separates them that neither `--resident`, the pause mechanism, nor the residual accounts for — and this box has documented states worth 2.2× thermally and 7.8× on the plug.

> **2026-08-12 11:06 — that "unexplained" pair is the pause flag, and the `--resident` verdict above is over-claimed.** Splitting the twelve large holds by mechanism: the 4.76 is the only `--wait-ms` hold and the 15.51 is `--duty`. Not a machine state — the flag. Fourth attribution error of the night in the same shape, this one caught before a run rather than after.
>
> **And the back-to-back `--resident` pair was taken at `rest` 9.9 and 8.9, where the machine already limits both arms to ~4.7 cores.** The archive's contrast sits at `rest` 0.5, where `--duty` holds reach 15.3–15.5. So the pair shows `--resident` is innocent **on a loud box** and says nothing about a quiet one — an invariant exercised only where it cannot break. The claim that `--resident` does not set compute per unit stands for that regime alone.
>
> **What the split does show, for `--duty` holds:** `rest` 13.6 → 2.4 cores; 8.5–10 → 4.3–7.5; 0.5–0.8 → **15.3–15.5**. A hundred sessions **do** saturate a quiet box. The one outlier, `rest` 3.24 → 3.635, is a hold whose own samples show the tenant arriving mid-way, so its median blends two machines.

> **2026-08-12 11:16 — a fifth attempt, and the quiet-box test is unobtainable here.** A self-gating run polled silently, cleared on a three-sample mean under 1.5 cores, and ran both arms — and the **holds themselves read `rest` 10.11 and 10.73**. The tenant returned during them. Both arms landed near 4 cores, which is the null this run pre-registered.
>
> **What it adds:** a fourth loud-box pair. At `rest` 9–11 the arms agree across two independent pairs — 4.675/4.811 and 4.042/3.608 — so `--resident` is settled as irrelevant *in that regime* and untested in the other.
>
> **What it establishes about the design:** this box cannot hold a hundred sessions quietly for sixty seconds. The gate cleared honestly on the machine it saw; the binding quantity is **window remaining after clearance**, which nothing predicts, and a hundred-session hold needs a minute of it. The archive's four holds at 15.3–15.5 cores with `rest` 0.49–0.77 prove it is possible, and five attempts tonight did not reproduce their conditions.
>
> **The gating design is worth keeping regardless**: waiting inside the detached run meant no watcher event woke the agent, and the observer column confirms it at **0.311 and 0.303 cores** across both holds against 1.58–1.99 while working.

**A single `cpu-spin --duty 0.27` session holds 0.248 cores. A hundred of them hold 0.036 each — 13% of solo — while the machine around them sits at 0.55 cores and the job totals 3.6 of sixteen. The fall is monotonic in session count, it survives replacing the adaptive pause with a fixed one, and core contention cannot explain it: the box is 70% idle throughout.**

## The curve

Each hold: `cpu-spin --units 100000000 --duty 0.27 --resident 1`, 40–120 s, exit-only watcher, mains, tenant absent.

| sessions | job total (cores) | each | achieved duty vs 0.27 |
|---|---|---|---|
| 1 | 0.248 | 0.2484 | **92%** |
| 10 | 1.779 | 0.1779 | **66%** |
| 30 | 2.524 | 0.0841 | **31%** |
| 100 | 3.635 | 0.0364 | **13%** |

The totals are sub-linear and plateau far short of the machine.

## Two explanations died before the discriminator ran

**Sleep granularity.** The obvious candidate is Windows' ~15.6 ms timer floor. It is refuted by arithmetic: the pause is `computed × (1 − duty) / duty` where `computed` is the *measured wall-clock* unit time, so under contention the pause **grows**. A floor truncates a pause that is too short; this one gets longer. And the solo reading of 0.248 shows the requested pause already clears the floor.

**Core contention.** At duty 0.27 roughly 27 of 100 sessions compute at once on sixteen cores, so each should run about 1.7× slow. The measured backoff is **7.5×**, and the machine is only ~23% busy while it happens.

## The discriminator: a fixed pause barely helps

`--wait-ms` sleeps a fixed span and cannot adapt, so if the adaptive pause were the mechanism it should hold its share.

| 100 sessions | each | job total | rest |
|---|---|---|---|
| `--duty 0.27` (adaptive) | 0.0364 | 3.635 | 3.239 |
| `--wait-ms 46` (**fixed**) | **0.0476** | 4.763 | **0.548** |

**A fixed pause buys 31%, not a return to 0.25.** Both mechanisms collapse, so the pause is not the cause — and the `--wait-ms` hold is the cleaner evidence, because the machine outside the job held only **0.548 cores**. A hundred sessions produced 4.76 cores of work on a sixteen-core box with essentially nothing else running.

## What that leaves

Each session gets about **13.5% of a core while computing**, where 100 sessions at 27% duty on sixteen cores predicts 59%. Something costs four times what core-sharing does, and it is not the machine being full.

Candidates this measurement does not separate: scheduler behaviour with a hundred processes waking on short cycles, timer coalescing under many sleepers, per-wake cost in the daemon's readers, or cache and TLB effects from a hundred resident images. **The mechanism is open.**

## Why it reaches past the workload

[Gate M1's work-rate condition](../../ROADMAP.md) is a ratio: per-session rate at a hundred sessions against the same workload held alone. If a session at a hundred achieves 13% of the duty it achieves alone — for reasons that are not the daemon, since the daemon is not what makes the box idle — then the ratio is partly measuring the workload's own behaviour at scale.

That does not make the gate wrong. It makes it measure something other than what its wording implies, and the distinction matters before a number from it is quoted as a property of `coggyd`.

## The confound was recorded as cleared, at the one count that cannot show it

The repository carries the oversleep confound as settled: *job cores read 0.99 at duty 1.0 uncontended and 0.21–0.27 at duty 0.27, so achieved duty matches requested.* Every one of those readings is **uncontended** — a single session, where the effect measured here cannot appear at all.

The check was correct and never exercised the thing it certified. That is the invariant rule in its own words: an invariant only exercised by things that cannot break it has not been exercised.

## What this does not establish

- **No mechanism.** Four candidates, none separated. The next step is per-wake attribution rather than another session count.
- **One workload, one duty, one box.** `cpu-spin` at `--resident 1`; nothing here says a heavier session behaves the same.
- **Short holds.** 40–120 s. The plateau is stable across samples within each hold but no hold ran long.
- **It does not re-derive any M1 figure.** It says what those figures partly contain, not what they should be.
- **The `--wait-ms` arm is not matched to solo.** Its solo value at 46 ms was not measured in this series, so its 0.0476 is compared against the duty arm rather than against its own baseline.

## Provenance

| | |
|---|---|
| Holds | `sessionbench hold --sessions {1,10,30,100} --interval 5 --duration {40,40,40,120} -- cpu-spin --units 100000000 --duty 0.27 --resident 1` |
| Discriminator | same at 100 sessions for 60 s with `--wait-ms 46` in place of `--duty` |
| Read from | each hold's `occupancy.median_cores`, `rest_cores_median` and `observer_cores_median` |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, tenant absent, 0 survivors after teardown |
