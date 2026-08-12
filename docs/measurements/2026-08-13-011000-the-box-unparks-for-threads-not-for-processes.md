# The box unparks for threads, not for processes

**2026-08-13 01:00-01:02.** Three gated rungs inside one three-minute window, tenant censused at 0.00 cores throughout all three, same workload, same duty 1.0, same 25-second sampling span. The only thing varied is how the same demand is arranged.

| arm | processes | threads each | total threads | parked median | parked >=8 | machine cores |
| --- | --- | --- | --- | --- | --- | --- |
| control | 0 | - | 0 | 11 | 100% | 5.41 |
| count | 5 | 1 | 5 | 7 | 38% | 6.08 |
| **concentration** | 5 | **27** | **135** | **2** | **0%** | **9.12** |

**This closes the question that has been open since a hundred sessions failed to move the parked count at any rung.** Sixty `cpu-spin` processes asking for more than Chromium consumes left the box parked; five processes of twenty-seven threads unpark it almost completely. The demand is not what the policy reads. **What this box responds to is the arrangement of the demand, not its size** — which is why every count arm ever run here was answering a question the machine does not ask.

It also explains the neighbour without appealing to anything about browsers. `chrome-headless-shell` runs about five processes of roughly twenty-seven threads, which is the concentration arm's shape rather than a coincidence — the 27 was chosen to match it. So the tenant's effect on this box is reproducible by any workload willing to adopt its thread shape, and *what the neighbour is* turns out not to matter after all. That was [the sharpest open question left](2026-08-12-143500-ten-of-sixteen-cores-are-parked-under-a-hundred-sessions.md) and it dissolves rather than resolving: there is nothing special about the browser.

The three arms sit in one window with the tenant absent from every one, which is the condition [a comparison across windows cannot satisfy](2026-08-11-173433-the-tenancy-step-is-on-mains-and-absent-on-battery.md) and the reason this is a measurement rather than two afternoons compared.

## What this does not establish

**Total thread count and per-process concentration are still confounded**, and the table cannot separate them. The count arm has five threads and the concentration arm has 135, so *more threads* explains the result exactly as well as *more threads per process*. The known 60-process arm helps and does not settle it: sixty processes of one thread is sixty total threads and did not unpark, which is fewer than 135.

**The discriminator is one rung and it is written**: one process of sixty threads. Sixty processes of one thread is already known not to unpark, so if a single process holding the same sixty threads does unpark, concentration is the mechanism and total count is not. It was attempted twice at 01:02 and 01:05 and **the gate refused both**, Chromium having returned to five processes mid-attempt. The refusal is the instrument working — the [ungated attempt forty minutes earlier](2026-08-12-143500-ten-of-sixteen-cores-are-parked-under-a-hundred-sessions.md) ran the same shape against a box already at zero parked cores and learned nothing.

**Each arm is one rung of 25 seconds**, sampled several times within it because the parked count is bimodal, but not repeated. The control's 11 and the concentration arm's 2 are far enough apart to survive the [n=18 quiet-arm spread](2026-08-12-143500-ten-of-sixteen-cores-are-parked-under-a-hundred-sessions.md); the count arm's 7 at 38% is a middle reading on one sample and should not be quoted as a level.

**Nothing here reads the policy.** The mechanism is inferred from behaviour, and Windows' core-parking decision inputs are not something this measurement opened.

## 2026-08-13 01:42 — THE POSITIVE CONTROL FAILED, AND THE HEADLINE IS NOT ESTABLISHED

The discriminator above ran forty minutes later and carried the concentration arm again as a positive control, on the reasoning that an arm known to unpark should unpark. **It did not.** The same command, the same binary, the tenant censused at zero in both:

| time | arm | tenant cores | parked median | parked >=8 | machine cores |
| --- | --- | --- | --- | --- | --- |
| 01:00:24 | 5 x 27 threads | 0.00 | **2** | **0%** | **9.12** |
| 01:41:37 | 5 x 27 threads | -0.13 | **8** | **100%** | **5.83** |

The run's own pre-registered reading was *C parked too -> the window was not what it looked like; distrust the run, not the theory*. That was written to protect the discriminator, and it is the wrong disposal here, because the control is not a check on the window alone — **it is a replication of the headline, and it failed.** One measurement and one failed replication is not a finding.

So the claim at the top of this record stands as a description of two rungs taken at 01:00 and nothing more. What is still solid:

- **At 01:00, in one window, 5 x 27 threads read 2 parked cores and 9.12 machine cores where 5 x 1 read 7 and 6.08 and an empty box read 11 and 5.41.** Those three arms were minutes apart and the ordering is right. Something happened.
- **At 01:41, the same arm read 8 and 5.83.** So whatever happened at 01:00 is not a property of the command.

**The discriminator itself is void on both arms.** Arm A ran with Chromium at 10.38 cores and is marked `void_tenant_present`, so the 60-process arm never got its quiet window; arm B read 10 parked, which cannot be interpreted while the control that calibrates it is failing. Nothing about count-versus-packing was learned.

### What is now open, and it is larger than the discriminator

**Why did 5 x 27 unpark the box at 01:00 and not at 01:41?** Candidates, none tested:

- **Bimodality.** The parked count is known bimodal here, and the [quiet arm of the census spread 44% to 89% across three runs](2026-08-12-143500-ten-of-sixteen-cores-are-parked-under-a-hundred-sessions.md). Two single rungs can straddle that without either being wrong, which would make the 01:00 result a sample rather than an effect.
- **Recent tenancy.** Arm A at 01:39 ran with Chromium holding ten cores and the box fully unparked. Chromium then left. Arms B and C ran on a box that had just been unparked and re-parked, where the 01:00 arms ran on one parked flat for a minute. The record's own arrival and departure lags say departure-to-park is immediate, which argues against this and does not exclude a slower second-order effect.
- **Arm ordering inside the run.** The 01:00 sequence was empty, then concentrated, then spread. The 01:41 sequence was 60 processes, then concentrated. The concentration arm was preceded by a heavy load in one case and by nothing in the other.

**The cheap discriminator is repetition, not a new instrument.** Five alternating rungs of 0 and 5x27 in one gated window would say whether 9.12 recurs at all, and that is one script invocation. It was not run tonight because the tenant returned.

### The rule this cost

The concentration result was written into a record, indexed, and used to withdraw a live claim in ROADMAP within forty minutes of a single pair of rungs, on a box whose parked count is known to be bimodal and whose quiet-arm statistics were shown *in this same record, hours earlier* to swing by a factor of two. **A positive control was carried by luck** — it was added to protect a different measurement, and it is the only reason the failure was seen at all rather than being discovered by someone quoting the number weeks later.

