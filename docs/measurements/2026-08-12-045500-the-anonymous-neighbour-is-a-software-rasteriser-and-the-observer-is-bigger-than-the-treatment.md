# The anonymous neighbour is a software rasteriser, and the observer is bigger than the treatment

**Two facts that every artifact in the tenancy investigation was structurally unable to record. The ten-core disturbance is one process — Playwright's headless Chromium GPU process doing SwiftShader software rasterisation. And the agent taking the measurements holds 1.64 cores while working, against the 1.3 that six injected co-tenants contribute: the observer is larger than the treatment.**

## Why no artifact could have said this

`rest_cores_median` is `(machine_cpu_percent − cpu_percent) / 100` — a **subtraction**. It is an anonymous residual by construction, so it can report that ten cores are held elsewhere and can never report by what. Every hold in this investigation carries that column; none of them could answer a question that took one query.

The census counter cannot answer it either, for the opposite reason: it sums `\Process(*)\% Processor Time` only over instances above half a core, which is the blindness that once let a quiet gate read `0.00` against a true 3.26 cores.

What answers it is **diffing kernel `TotalProcessorTime` per process** across a window. That is CPU-time accounting rather than a sampled percentage, so it neither misses load spread thin nor loses load concentrated in one process.

## The disturbance is a software rasteriser

| process | pid | cores |
|---|---|---|
| **`chrome-headless-shell`** | 31880 | **9.74** |
| `claude` | 10692 | 0.97 |
| `claude` | 30728 | 0.93 |
| `chrome-headless-shell` | 41428 | 0.32 |
| everything else | | < 0.18 each |

Total attributed **12.51** cores against `doctor`'s **12.40** in the same minutes — two routes agreeing, which is the check.

The heavy process is not the browser's renderer. Its command line reads:

```
chrome-headless-shell.exe --type=gpu-process --headless --no-sandbox
  --enable-unsafe-swiftshader --use-gl=angle --use-angle=swiftshader-webgl
```

from `AppData\Local\ms-playwright\chromium_headless_shell-1228`. It is **Playwright**, and the ten cores are **SwiftShader** — doing on the CPU what a GPU would otherwise do.

That explains the shape the delta guard kept catching. Playwright launches and tears down browser instances per operation, so the load **arrives and departs** rather than sitting: `+9.55`, `−10.41` and `+10.75` cores in three separate voids, and five `chrome-headless-shell` processes present at one sample and zero four minutes later.

## The observer is bigger than the treatment

Attributing every process during a **solo hold** — one `cpu-spin` session under `sessionbench hold`:

| process | cores |
|---|---|
| **`claude` (two processes)** | **1.64** |
| `cpu-spin` — the measured session | 0.20 |
| `System` | 0.20 |
| `WmiPrvSE` | 0.12 |
| `Memory Compression` | 0.09 |
| **`sessionbench`, `coggyd`** | **below 0.02 — absent** |

Total 2.72 cores, of which the agent is 60%.

**A hypothesis died here and it was mine.** The extra load was expected to be the harness — `sessionbench` supervises from outside its own job object, so it lands in `rest` by construction, and `coggyd` does too. Both measure below the 0.02 cutoff. The residual is not the instrument.

**The figure is a working cost, not a footprint.** The same two processes read **0.04–0.17 cores** when idle minutes earlier. Quoting 1.64 as "the agent's footprint" would be a fact about a busy agent wearing the label of a resident one.

That is what makes it matter. [The standing rule](../../CLAUDE.md#measure-before-you-optimize) is that the observer is not free and nothing else should run during a measurement. It has never carried a number. **1.64 cores exceeds the ~1.3 that six `cpu-spin --duty 0.27` co-tenants inject**, so an agent working alongside a hold is a larger neighbour than the neighbour under study — and it enters `rest_cores_median` anonymously, indistinguishable from the browser.

## What caught the quiet window was the instrument for the noise

A sampler was armed to characterise the disturbance's *rhythm*. What it recorded was the disturbance *ending*:

| time | total cores | `chrome-headless-shell` |
|---|---|---|
| 04:34:04 | 10.00 | 7.53 (3 procs) |
| 04:34:16 | 2.60 | 0.12 (0 procs) |
| 04:34:28 | 1.64 | — (0 procs) |
| 04:34:52 → 04:37:53 | 0.51 – 1.27 | ~0.01 |

Four `doctor` point readings spaced across those minutes could have landed entirely on either side and reported a stable machine in either state. A windowed sampler cannot.

## The window was used and it closed first

An injection was launched into it within seconds, with the real quiet gate and **no `-AnyBaseline`**, to test a live contradiction: [two rising-limb pairs](2026-08-12-042000-longer-holds-do-not-cut-the-spread-and-they-halve-the-acceptance.md) had gone **−7.0%** and **−26.5%** where controlled injections recorded **+95.4%** and **+100.1%**.

It voided. The gate certified quiet at `0.41` cores machine-wide; the baseline hold thirty seconds later recorded **1.47 cores of rest**, above the 1.3 bar, and the next poll showed the browser returning at `1.12`. The window was about four and a half minutes and the run reached it with roughly one to spare.

## What this does not establish

- **The 1.47-core baseline is not attributed.** It could be the observer, the returning browser, or both; the per-process sample was taken during a *different* hold. What is established is that `sessionbench` and `coggyd` are not it.
- **One machine, one night, one tenant.** That this box's residual is Playwright says nothing about anyone else's.
- **Nothing is re-derived here.** No published tenancy figure changes, because the observer's contribution during past runs was not recorded and cannot be recovered — the column that would hold it is the anonymous one.
- **The contradiction is still open.** The rising-limb sign disagreement has one more failed attempt against it, not an answer.

## What it changes

Two instrument consequences, neither of them a re-derivation:

1. **A quiet window can be detected but not predicted**, and it is short. A run intending to catch one must sit waiting with a long give-up rather than be launched at one — this attempt reached a four-and-a-half-minute window with about a minute of margin, by luck.
2. **`rest_cores_median` cannot distinguish one ten-core neighbour from twenty half-core ones, or from the agent.** Where the identity matters, the per-process diff is one query and belongs beside the hold.

## Provenance

| | |
|---|---|
| Attribution method | `Get-Process` snapshot, diff of `TotalProcessorTime` across the window, ranked; 12 s for the tenant census, 20 s during the solo hold |
| Tenant census | 20 samples × 12 s, 04:34:04–04:37:53, `bench-out/tenant-census-20260812.log` |
| Solo hold | `sessionbench hold --label decompose --sessions 1 --interval 5 --duration 40 -- cpu-spin --duty 0.27 --resident 20` |
| Failed injection | `bench-out/inject-20260812-043826.json`, outcome `NoQuietWindow`, 1 void |
| Machine | 16 logical / 31 GiB / Windows 11, **mains**, 0 survivors after teardown |
