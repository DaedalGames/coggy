# The collapse limb starves the session below a readable ratio

**Injecting 1.67 cores onto a box already carrying 8.34 raised a lone session's rate 22.7% — and raised the session's own CPU from 0.06 to 0.08 cores at the same time, so per core-second it fell.** At that denominator neither reading can be defended, which is a limit on the measurement rather than a result about the machine.

## What was run

[`inject-tenant.ps1`](../../sessionbench/scripts/inject-tenant.ps1) with the new `-AnyBaseline`, 2026-08-12 00:11, mains. Six `cpu-spin --duty 0.27 --resident 1` co-tenants around a 30-second hold, the measured session unchanged at `cpu-spin --duty 0.27 --resident 20`.

| | rate | job cores | rest cores | rate ÷ job |
|---|---|---|---|---|
| before | 3.546 | **0.06** | 8.34 | 59.1 |
| after | **4.352** | **0.08** | 10.01 | 54.4 |

Delta 1.67 cores against 1.14 expected, inside the guard's 0.57–2.28 band; the co-tenants were measured holding 1.280 cores of their own.

## Why the +22.7% cannot be read as more work per core-second

The session's own CPU moved with the rate. **At `job = 0.06`, an error of one hundredth of a core is 17% of the ratio** — which is why [`duty-bands.ps1`](../../sessionbench/scripts/duty-bands.ps1) refuses anything below `job = 0.15`, and both arms here sit at less than half of that.

So two readings are equally available and neither is separable from the other:

- the session did more work per core-second, and the ratio's fall is noise;
- the session was simply granted a larger share of a busier machine, and the raw rise is scheduling.

**A ratio's denominator decides its noise**, and on the collapse limb the denominator is the thing that collapses.

## The design limit this establishes

`-AnyBaseline` was added because this box does not go quiet: across 2026-08-11 evening, windows opened at 18:15, 22:10, 22:20, 22:29 and 23:53, each a minute or two, while a paired measurement needs two consecutive quiet minutes. Three runs over two and a half hours produced one baseline, and the browser returned inside its thirty-second hold.

Measuring at whatever tenancy exists solves that and introduces this: **at 8–12 cores held, a lone session runs at 0.05–0.11 cores**, so the per-core-second column stops being readable. The design can compare **raw rates between two injectors at the same tenancy** — which is what [the differently-shaped-neighbour question](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md) actually asks — and cannot resolve efficiency.

## The file-write arm refused itself

The paired arm, launched two minutes later at a matched baseline of 8.46 cores, never produced a result:

```
REFUSING: 16 co-tenants are alive but hold 0.140 cores, against 1.09 expected
```

**`file-write` is I/O-bound, and its CPU draw is not linear in process count.** Six held 0.440 cores when the box carried 2.8; sixteen held 0.140 when it carried 8.46 — about 0.009 each against 0.073, an eightfold collapse per process. The plan of "sixteen file-write to match six cpu-spin" rested on a figure measured in a different condition.

The refusal is [the effect assertion](../../sessionbench/scripts/inject-tenant.ps1) added hours earlier for a different case entirely — six `stdout-storm` staying alive while blocked on a full buffer. Without it this arm would have reported a ratio for a 0.14-core injection, and it would have looked entirely plausible beside the reference arm's.

## What this does not establish

- **Nothing about the differently-shaped neighbour.** That question is untouched: the file-write arm produced no pair.
- **Nothing comparable with the rising limb.** The recorded [+95.4%](2026-08-11-133424-adding-a-neighbour-doubles-a-lone-session.md) and [+100.1%](2026-08-11-141925-the-neighbour-is-not-six-copies-of-the-workload.md) are both from baselines below 0.8 cores. A pair at 8.34 is a different point on the curve.
- **n = 1**, after one void where the browser moved 3.11 cores during the tenanted hold — more than the injection itself, which the delta guard refused.
- **The `-AnyBaseline` design needs a stable box, not merely a loaded one.** At 8+ cores the browser's own swings exceed a 1.1-core injection, so the signal sits inside the noise it is measured against.

## Provenance

| | |
|---|---|
| Reference arm | `inject-tenant.ps1 -Tenants 6 -ExpectedPerTenant 0.19 -AnyBaseline`, 00:11, after 1 void |
| Refused arm | same with `-Injector file-write.exe -Tenants 16 -ExpectedPerTenant 0.068`, 00:13 |
| Artifacts | `bench-out/1786461006-inject-before-daemon/`, `bench-out/1786461081-inject-after-daemon/`, `bench-out/1786461206-inject-before-daemon/` |
| Read from | `hold.json` — `units_per_session_per_sec`, `occupancy.median_cores`, `occupancy.rest_cores_median`, `host.on_battery` |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, 0 survivors after teardown |
