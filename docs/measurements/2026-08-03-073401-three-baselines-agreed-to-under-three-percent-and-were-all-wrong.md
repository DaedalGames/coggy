# Three solo baselines agreed to 2.7% and every one was 38% low

The bracket refuses a ratio when its two sides disagree by more than 5%. This is the run where agreement was the wrong question: three back-to-back solo holds landed inside 2.7% of each other, comfortably passing, on a box with **eleven to thirteen cores held by something else**.

## The three holds

`hold --sessions 1 --interval 5 --duration 120 -- cpu-spin --units 100000000 --duty 0.27 --resident 20`, back to back, each reporting its own rest-of-machine figure.

| | rate | job (median) | **cores held outside the job** |
|---|---|---|---|
| fp2-1 | 13.449 units/s | 0.25 | **12.34** |
| fp2-2 | 13.085 | 0.24 | **13.20** |
| fp2-3 | 13.336 | 0.24 | **11.50** |

Spread on rate: **2.7%**, against a 5% allowance. Spread on the rest of the machine: 11.50 to 13.20 cores, present throughout.

**This box gives about 21.5 units/s solo when rested.** These three are 38% under it and agree with each other.

## Why the direction matters

`slowdown = solo ÷ concurrent`, and the gate's condition is `slowdown ≤ 2`. A tenant during the baselines lowers the numerator, so **the slowdown comes out smaller and the gate looks closer to passing than it is**. The failure is not a refused run; it is a run that passes on a baseline nobody could measure.

A tenant present through the concurrent hold as well would drag both, and the ratio would partly survive. The dangerous case is the one this box produces: a tenant that comes and goes in bursts of minutes, so which holds it lands on is chance.

## What agreement can and cannot say

Agreement is a statement about two numbers, not about the machine that produced them. Three holds inside 2.7% means the machine was **stable** across six minutes. It was stable and loaded. That is the same reading a rested machine gives, and no arrangement of the two rates distinguishes them.

The distinguishing figure is on the same line and is not a rate: [the cores held outside the job](2026-08-03-070018-a-solo-triple-spread-thirty-percent-and-the-column-said-why.md), which every hold now records and prints.

## Two attempts, opposite symptoms, same cause

| | rates | spread | rest of machine | what agreement said |
|---|---|---|---|---|
| first triple | 14.543 · 17.561 · 13.002 | **30.3%** | 11.46 · 2.99 · 11.60 | refuse — correctly, for the wrong reason |
| this triple | 13.449 · 13.085 · 13.336 | **2.7%** | 12.34 · 13.20 · 11.50 | accept — incorrectly |

The first triple had a burst arrive and leave inside it; this one sat inside a burst from end to end. **Agreement got one right and one wrong, and it got the right one right by accident** — its refusal message named drift, and nothing had drifted.

## What this does not establish

- **A rested fingerprint for this box.** Three attempts today, three tenants. The 21.5 figure is two days old and nothing since has been measured on a quiet machine for long enough to replace it.
- **The shape of the curve between them.** One quiet hold today gave 17.561 at 2.99 cores of background, these three give 13.3 at about 12.3, and 21.5 came from a rested box. Three points, one sample each.
- **Who the tenant is.** It appears and leaves on its own; `doctor` read 1.35 cores twenty minutes before this run and 13.75 as it started.

## Provenance

| | |
|---|---|
| Inputs | `bench-out/1785709115-fp2-1-daemon`, `1785709273-fp2-2-daemon`, `1785709432-fp2-3-daemon` |
| Machine | on mains, `doctor` 13.75 of 16 cores at launch |
| Commit | `665837d` |

## The reading was fixed before the run, and the outcome was not on the list

Three outcomes were written down beforehand: all three quiet and tight means a fingerprint, rates moving with the rest figure means a tenant, and rates moving while the rest stays quiet means suspect the box. What arrived was a fourth — **rates tight while the rest stayed loud** — which is the one combination that would have been invisible before this run's own instrument existed, and the only one that passes the check it should fail.
