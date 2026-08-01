# An hour of a hundred sessions · 2026-08-01 10:32:25

[The 25-second probe](2026-08-01-091356-a-hundred-sessions-in-the-daemon.md) named what it could not do: the gate wants an hour, and there was no repeat and no drift control because `coggyd` has no instrument of its own. This holds a hundred sessions for the hour and samples thirteen times.

The headline is not the number it went looking for. **Two of [gate M1](../../ROADMAP.md#m1--headless-daemon)'s three conditions cannot be asked of this process at all**, and finding that out is what the hour bought.

## The hold

```
waitfor /t 3600 coggyhour | coggyd --sessions 100 -- ping -n 3600 127.0.0.1
```

| | |
|---|---|
| Sessions alive, every one of thirteen samples | **100** |
| The daemon's own reading at the hour | **`held 100 · running 100`** |
| **Peak total RSS** | **624.9 MiB** — 15% of the 4 GB the gate allows |
| Survivors after a graceful stop | **0** `ping`, 0 `coggyd`, 0 `waitfor` |

**Peak is the right figure and it was not caught by luck.** The daemon's RSS rises while scrollback fills and holds constant content afterward, so the true peak sits where filling ends — `2000 lines ÷ 1 line/s` puts that at t≈2000 s, and the 624.9 sample is 160 s later. The sessions were flat from t=660. A five-minute grid found the peak because the peak's location is arithmetic.

**The teardown covers the other half of a claim the probe made.** That run force-killed the daemon and reclaimed every tree, which showed `KILL_ON_JOB_CLOSE` firing when the kernel closes the last handle. This one stopped at stdin end-of-file, so `Drop for Session` ran — job released first, then `wait`, then the drains joined. Zero survivors on both paths, and they share no code.

## What the daemon costs, which is not a percentage

The probe said **the daemon is 2% of what it holds**. That reading was taken at 25 s with the scrollback nearly empty; at the hour it is 5.8%. But the correction is smaller than the problem with the sentence — **a ratio is a fact about a pairing, and this one is dominated by how small a `ping` is.** Against nine real generation sessions the same daemon would be 0.16%. Nothing in any of those three numbers is a property of the daemon.

What is:

| Per session held | |
|---|---|
| Fixed — `Session`, two drain threads, a job handle | **110 KiB** |
| Scrollback at capacity — 2,000 lines × ~130 B | **253 KiB** |
| **Total** | **363 KiB** |

Two routes agree on the scrollback term. Dividing 253 KiB by 2,000 lines gives 130 B, and the daemon's growth measured straight across the filling window — 11.8 MiB to 34.8 MiB over 1,800 lines a session — gives 134 B.

**And that term is set by the workload, not by the daemon.** Line length is chosen by the session; the cap counts lines. At `MAX_LINE_BYTES` the same two constants give **131 MB a session**, so a hundred sessions reach 13.1 GB against a gate written for 4. This hold passes at 625 MiB because `ping` writes 47-byte lines.

**2026-08-01, later the same morning:** the scrollback now carries a byte budget beside the line count, which holds a hundred sessions to about 43 MB whatever they write. Searching for the mechanism rather than the goal is what found it — every grid terminal caps by lines and is right to, because a cell grid makes a line as wide as the terminal, and `coggyd` took the convention onto pipes where nothing bounds a line but `MAX_LINE_BYTES`. The shape is Ghostty's; the evidence it is needed is [tmux's request for `history-bytes`](https://github.com/tmux/tmux/issues/4859), opened after redraw traffic put about 48 GB into pane scrollbacks. The figures above stand as measured — this run held the old buffer.

## The cap holds, on a criterion registered before the samples arrived

Two conditions were written down while filling was still in progress, because three earlier readings of this run had already been overturned by the next sample:

- **Trend** — t=3540 no more than 0.5 MiB above the first post-fill sample. That threshold is 1.3 MiB/hour, which doubles the daemon in a day, so it means something.
- **Band** — every post-fill sample within ±2% of 36.3 MiB.

**Trend passed at −1.7 MiB.** The daemon did not grow.

**The band was violated downward**, and it is reported as registered rather than repaired: the hypothesis was one-sided — an unbounded cap grows — and a symmetric band was the wrong shape to write for it.

## Falling RSS with constant content is the machine, and that follows from the code

Post-fill the daemon reads 36.3, 35.4, 35.0, 35.3, 34.4, 34.6 MiB — down 5% over 23 minutes. **It cannot be the daemon releasing.** Past capacity the scrollback drops one line for every line it takes, `VecDeque` does not shrink, and the allocations are recycled, so the bytes it holds are constant by construction. A falling resident figure with constant allocation is the operating system trimming.

The sessions fell with it — 588.6 to 561.5 MiB — and the free-memory channel corroborates twice: 12.7 MiB shed while free rose 26 MB, then 9.8 MiB shed while something else took 1.14 GiB.

**So summed working set has a floor of about 2% over an hour**, which is a third of the whole daemon. A trim event moves it regardless of what the daemon does.

That floor is a reason to sample **private commit** alongside RSS next time rather than instead of it. RSS is the gate's own quantity and answers *does it fit*; commit is not trimmable and answers *does it grow*. This run asked the second question with the first question's instrument, which is why a trim broke a threshold.

## Two of the three gate conditions have no surface

| Condition | What running `coggyd` yields |
|---|---|
| 100 sessions, an hour, under 4 GB | `held` / `running` — answered |
| Work rate within 2× of solo | nothing; the daemon does not know what a session does |
| **No dropped output** | **nothing** |

The third is the surprise. `Scrollback` counts `read`, `evicted` and `truncated` separately, and [the README gives the reason](../../coggyd/README.md#output) — a line never read is a gate failure and a line aged out is policy, kept apart so neither hides behind the other. **That distinction never leaves the process.** The whole report is `held 100 · running 100`. No length of hold produces the number.

It has the same shape as the scrollback cap: the gate names one quantity and the daemon exposes another — bytes against a line count there, dropped output against held-and-running here.

**2026-08-01, later: one of the two was only waiting.** Work rate needed a second run rather than a different daemon — a solo hold to divide by — and `hold --with-solo` now takes one on either side of the concurrent hold. Dropped output is still out of reach for the reason above, so the count here is one of three rather than two.

Worth separating from the redline's four, which this table does not use and which are easy to confuse with it. **Replacement is not among the gate's three**; it is the redline's fourth, and a daemon has no counterpart for it either. So a ramp under the daemon loses two of four where a hold loses one of three, and the small numbers matching is what makes the lists blur.

## What this does not claim

- **`ping` is not a session.** It holds 5.8 MiB and does nothing, against [2.39 GiB for a generation session](2026-07-31-150258-g0-frozen.md). This is the daemon's own cost at a hundred sessions.
- **The work-rate half is still untouched**, and the machine was at 37% background afterwards, which would have made it a draft anyway.
- **One hour, once.** The cross-run error bar comes from two runs and is **8%** on the daemon's baseline once each reading is corrected for how full its scrollback was — not the 1.8% the totals agree to, which is agreement between two large numbers whose sessions dominate them.
- **The 100 in twelve of the thirteen samples is a machine-wide `ping` count**, not job membership. Choosing a silent stdin holder for clean attribution disabled the daemon's own periodic report, since it only checks its interval after a successful read. The authoritative reading exists once, at the end, which is the point the hour claim needs — but the continuity between samples is the weaker evidence of the two.

  **2026-08-01, later:** that was written up as a choice this run made and it was a defect in the daemon. Tying how often a supervisor speaks to how much its caller types is wrong for any caller, and it only looked like a trade-off because nothing yet needed the line. The stop condition and the report clock are separate now, so a silent holder gets the same reporting as a chatty one — and a rerun of this hold would carry the job-based count at every sample rather than once at the end.

## What was misread on the way

Each of these was stated confidently and overturned by the next sample, and all but the last are the same habit: computing a derived quantity from consecutive samples while more were still arriving.

| Read as | Overturned by |
|---|---|
| The first window's 154 B/line, as a constant | the third window at 108 |
| 154 → 143 → 108 as deceleration, with a mechanism attached | the fourth window at 171 |
| The totals agreeing to 1.8%, as this run's error bar | the daemon's own baselines, 8% apart |
| `Pool` reaching a session's scrollback, from having read its API | nothing yet — still unverified, and it sizes the fix |

The per-window rate has no trend: mean 137 B, spread ±22%, and the whole-span figure agrees with the mean. The noise was in the five-minute window, not in the process.

**The extrapolation was never needed.** The plateau those six readings were converging on is directly observed at t=2160, and by t=1860 the daemon already held 93% of its cap, so waiting had narrowed the answer to ±0.5 MiB for free.

## Provenance

| | |
|---|---|
| Machine | Intel Core Ultra 7 356H · 16 logical, three tiers · 31 GiB · Windows 11 (26200) |
| Background | 16% at launch, 6.43 GiB free; 37% after |
| Daemon | `coggyd` 0.0.0 at commit `555c083`, release build, unchanged since |
| Workload | `ping -n 3600 127.0.0.1`, one process per session, no shell |
| stdin holder | `waitfor /t 3600 coggyhour` — `timeout /t` [returns instantly without a console](2026-07-31-035111-between-builds.md), and a second `ping` would have been counted as a session |
| Sampling | 13 readings, t=60 s then every 5 min · `Get-Process` working sets · `Win32_OperatingSystem` free memory |
| Machine free memory | 6.426 GiB before, 6.518 after, swinging ±1.2 between — which is why it is a cause channel here and not a second measurement |
