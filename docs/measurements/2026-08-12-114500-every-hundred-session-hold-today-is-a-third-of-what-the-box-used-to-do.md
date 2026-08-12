# Every hundred-session hold today is a third of what the box used to do

**Seventeen hundred-session holds are on disk. The four best reach 15.3–15.5 job cores at 7.6–10.5 units per session per second, and all four were taken on 3 and 11 August. Every one of the eight taken today sits between 3.34 and 4.81 cores at 1.42–2.19 units — across residuals from 0.55 to 10.7. The ceiling has moved by three to five times, and nothing in the workload, the parameters or the residual accounts for it.**

## The table that shows it

| job cores | when | `rest` | units/s/session | label |
|---|---|---|---|---|
| 15.525 | 08-03 03:57 | n/a | 10.50 | tickpair |
| 15.505 | 08-11 02:34 | 0.49 | 7.56 | statepair2 |
| 15.359 | 08-03 04:42 | n/a | 10.28 | clockpair |
| 15.340 | 08-03 17:19 | 0.77 | 9.03 | slowstate-ratio |
| 7.491 | 08-11 02:51 | 8.53 | 4.32 | m1-probe-load |
| 7.453 | 08-03 18:45 | 8.56 | 3.45 | tenant-100 |
| 7.124 | 08-11 03:10 | 8.88 | 5.30 | m1-abs-1200 |
| **4.811** | **08-12 11:00** | 8.92 | 1.83 | resident20 |
| **4.763** | **08-12 10:47** | **0.55** | 1.84 | waitms100 |
| **4.675** | **08-12 10:58** | 9.88 | 1.71 | resident1 |
| 4.335 | 08-11 02:10 | 10.24 | 2.97 | statepair-conc |
| **4.042** | **08-12 11:10** | 10.11 | 1.58 | residq1 |
| **3.635** | **08-12 10:34** | 3.24 | 1.42 | obs100 |
| **3.608** | **08-12 11:12** | 10.73 | 1.42 | residq20 |
| **3.550** | **08-12 11:39** | 2.83 | 2.19 | resids20 |
| **3.335** | **08-12 11:38** | **1.84** | 2.11 | resids1 |
| 2.396 | 08-03 07:49 | 13.60 | 2.17 | loadgen |

**Today's eight are bolded. Every one is at or below 4.81.**

## The residual does not explain it

Today's holds span `rest` from **0.55 to 10.73** and their job cores span **3.34 to 4.81** — a 20× range in the machine's other load producing a 1.4× range in the job. On 3 and 11 August the same workload reached 15.3–15.5 twice at `rest` 0.49 and 0.77.

So a quiet box today gives 3.3–4.8 cores where a quiet box nine days ago gave 15.3–15.5. **Today's ceiling is not set by the tenant.**

## Nor do the parameters

Every hold in the table runs `cpu-spin --units 100000000 --duty 0.27`, and `--resident` was tested directly across three regimes today — 1 against 20 at `rest` ~9, ~10 and ~2 — agreeing within 3%, 12% and 6%. One hold used `--wait-ms 46` instead of `--duty` and landed at 4.76, inside today's band.

## What it costs, retroactively

**Every measurement taken today was taken in this state.** That includes:

- The session-count curve — 92%, 66%, 31%, 13% of requested duty at 1, 10, 30, 100 — whose collapse may be this state rather than a property of session count.
- The claim that a hundred sessions do not saturate this box, withdrawn earlier today on archive evidence and now ambiguous again: they do not saturate it **today**, and they did on 3 August.
- The five rising-limb attempts, whose baselines and voids were all read against a machine in this condition.

It does **not** invalidate the within-today comparisons, which are the ones taken back to back: the `--resident` pairs, the observer attributions, and the gate-rule replay all compare arms measured minutes apart in the same state.

## What this does not establish

- **No cause, but the code is now excluded rather than merely doubted.** `git log --since="2026-08-11" -- coggyd/ workloads/` returns **nothing**: the side that *produces* the work is byte-identical to the code behind the 15.5-core holds. Every change since is in `sessionbench`, which observes. So a regression in the daemon or the workload is ruled out, and what remains is the machine or the observer's own path.
- **Not the documented slow state, necessarily.** That one is worth 2.2× and lasts about ninety minutes; this is 3–5× and has held for at least seventy minutes across eight holds.
- **Not the plug.** Every hold today is on mains, recorded in each artifact.
- **Thermal reads the same.** `doctor` reported 44.1 °C throughout today, as it did on earlier days — which is the reason that sensor is already recorded here as unable to name this box's states.
- **Seventeen holds, three days.** The 08-03 and 08-11 groups each contain both high and low readings, so the day alone does not predict the value either.

## What to do with it

1. **Do not compare today's hundred-session figures with the archive's.** They are not the same machine.
2. **The daemon and the workload are already eliminated by `git log`** — one command, no rebuild, and it should have been the first check rather than the fourth.
3. **The next hundred-session hold on a future day is the test.** If it returns to 15, this was a state that passed. If it stays at 4, the remaining suspects are the machine and `sessionbench`'s own observing path — which can be separated by running one hold from the 08-11 binary, since the daemon it drives is unchanged.
4. **A reboot is the cheapest probe** and has not been tried.

## Provenance

| | |
|---|---|
| Source | every `hold.json` under `bench-out/` with `sessions >= 50`, seventeen holds |
| Fields | `occupancy.median_cores`, `occupancy.rest_cores_median`, `units_per_session_per_sec`, directory unix stamp |
| Machine | 16 logical / 31 GiB / Windows 11, **mains** for every hold in the table |
