# The agent side of a session, which nothing had measured · 2026-07-31 05:03:22

A generation session is an agent CLI driving an engine. Every engine state has now been measured — [at rest](2026-07-31-035111-between-builds.md), [building](2026-07-31-034150-unreal-builds-serialise.md), [cooking](2026-07-31-045604-an-error-bar-for-the-engine.md). The agent had not, and "at rest a session holds 8.97 MiB" turned out to be a measurement of the batch file holding the engine's place.

## What the CLI itself costs

Taken from the agent process driving this repository, alive for 25.1 hours.

| | |
|---|---|
| **Resident** | **576 to 586 MiB**, stable across samples |
| **Cores, lifetime mean** | **0.112** — 10,128 CPU-seconds over 25.1 hours |
| Cores, during an active turn | 0.394, 0.447, 0.415 |

The lifetime mean is the figure that matters. A turn is a burst against long stretches of waiting on a model, and 0.112 is what that averages to over a day of ordinary use.

## Which moves the binding condition again

| | Per session | At a hundred | Against |
|---|---|---|---|
| Agent CLI, resident | 0.580 GiB | **58 GiB** | a 21.97 GiB budget |
| Agent CLI, cores | 0.112 | 11.2 | 16 available |

**Memory breaks at about 38 sessions on the agent alone**, before an engine is opened. Cores are comfortable: `2ηC/d` with `d = 0.112` gives well over two hundred, so the [duty relation](2026-07-30-154348-duty-is-derivable.md) says the CPU side is not what stops this.

Set beside the engine, the two costs behave differently. **The CLI's 580 MiB is held for the session's whole life; the engine's 1.865 GiB only while it cooks.** A hundred sessions therefore pay 58 GiB always and the cooking share on top, which is why the agent is the floor and the engine is what lands on it.

## The correction this makes

[Finding 1](README.md) read *at rest a session holds 8.97 MiB, less than the synthetic workload that stood in for it*. That figure was `cmd.exe` waiting between cooks — the workload's shell, not a session. **The real number is sixty-four times larger**, and it is the one that never goes away.

It is the same error as measuring memory against a 20 MiB synthetic workload and concluding memory was cheap, made one level in: the engine half was measured with care and the agent half was assumed to be nothing.

## What this rests on

- **One CLI, one session, one machine.** Claude Code specifically, and a different agent would hold a different amount.
- **Twenty-five hours of conversation.** Resident state grows with a session's history, so a fresh session holds less and this is closer to a ceiling than a typical value.
- **The lifetime mean spans idle and active alike**, which is what makes it the right `d` — but it is one session's mix of the two, not a generation harness's.
- **Nothing here is `sessionbench`'s doing.** The figures come from process accounting on a live session rather than from a controlled run, so they carry no drift check and no repeats beyond the three windows above.

## Provenance

| | |
|---|---|
| Machine | 16 physical / 16 logical cores · 31 GiB usable · Windows 11 (26200) |
| Session | agent CLI, 25.1 hours old, driving this repository |
| Sampling | three 20 s windows for the rate, process working set for residency |
| Lifetime | 10,128 CPU-seconds over 90,400 wall seconds |
