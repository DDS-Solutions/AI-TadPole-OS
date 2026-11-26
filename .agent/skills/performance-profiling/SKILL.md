---
name: performance-profiling
description: "Performance profiling principles. Measurement, analysis, and optimization techniques. Core Web Vitals, Lighthouse audits, flame graphs, and bundle inspection."
when_to_use: "When diagnosing performance issues, running Lighthouse audits, analyzing bundle size, or optimizing Core Web Vitals."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / performance-profiling
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Premature unmeasured optimization, degraded Core Web Vitals, or undetected main-thread long tasks.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Performance Profiling & Optimization Protocol

> **Philosophy**: Measure, analyze, optimize — in that strict order. Never optimize without a verified baseline.  
> **Workflow Bindings**: Integrated into [`/audit`](../../workflows/audit.md), [`/enhance`](../../workflows/enhance.md), and [`/ui-ux-pro-max`](../../workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/vitals_and_devtools_recipes.md`](./references/vitals_and_devtools_recipes.md) | Core Web Vitals targets (LCP/INP/CLS), DevTools flame graph analysis, Lighthouse scripts | Performance auditing & bottleneck isolation |
| [`scripts/lighthouse_audit.py`](./scripts/lighthouse_audit.py) | Automated Lighthouse scoring CLI and JSON reporter with built-in self-tests | Executing headless audits against live URLs |

---

## ⚡ 1. The 4-Step Profiling Lifecycle

```
[1. BASELINE] ───> [2. IDENTIFY] ───> [3. OPTIMIZE] ───> [4. VALIDATE]
 Measure current    Locate specific   Targeted code-split  Re-measure to prove
 metrics / traces   long tasks (>50ms) memoize, or parallel latency gains
```

---

## 📊 2. Core Web Vitals Baseline (Good Targets)

- **LCP (Largest Contentful Paint)**: `< 2.5s` (Priority loading, image compression)
- **INP (Interaction to Next Paint)**: `< 200ms` (Eliminate main-thread blocking tasks)
- **CLS (Cumulative Layout Shift)**: `< 0.1` (Explicit dimensions on images and dynamic containers)

---

## 🧠 3. Socratic Gate for Performance Profiling

Before implementing an optimization or cache layer, verify:
- **Gate 1 (Empirical Baseline)**: Do you have a recorded measurement (time, bundle KB, or FPS) proving the bottleneck exists?
- **Gate 2 (Root Bottleneck)**: Is the delay caused by network latency, I/O blocking, excessive re-renders, or heavy computation?
- **Gate 3 (Regression Check)**: Did the change preserve behavioral parity without degrading code readability or memory footprints?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Speculative Optimization** | Adding complexity or memoization without profiling evidence. | Measure first with Lighthouse or CPU profiler; optimize only verified bottlenecks. |
| **Unbounded Long Tasks** | Synchronous compute running $>50$ms on main thread freezes UI. | Chunk work using `requestIdleCallback`, `scheduler.yield()`, or Web Workers. |
| **Monolithic Chunks** | Bundling entire node_modules into a single vendor JS bundle. | Configure dynamic imports and route-based code splitting. |
| **Unsized Images** | Images rendering without `width`/`height` trigger CLS layout jumps. | Explicitly define aspect ratios and reserved dimensions. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate performance metrics and audit endpoints:

```powershell
# 1. Run built-in self-tests
python .agent/skills/performance-profiling/scripts/lighthouse_audit.py --test

# 2. Audit a local preview endpoint
python .agent/skills/performance-profiling/scripts/lighthouse_audit.py http://localhost:5173
```