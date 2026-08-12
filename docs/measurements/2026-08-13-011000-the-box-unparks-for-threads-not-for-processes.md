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
