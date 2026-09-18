---
name: performance-profiling
description: Performance profiling principles. Measurement, analysis, and optimization techniques.
when_to_use: "When diagnosing performance issues, running Lighthouse audits, analyzing bundle size, or optimizing Core Web Vitals."
allowed-tools: Read, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / performance-profiling
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Performance Profiling & Optimization Protocol

> **Philosophy**: Measure, analyze, optimize — in that strict order. Never optimize without a baseline.
> **Workflow Binding**: Used directly during [`/audit`](../../workflows/audit.md), [`/enhance`](../../workflows/enhance.md), and [`/ui-ux-pro-max`](../../workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** profiling rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/vitals_and_devtools_recipes.md`](./references/vitals_and_devtools_recipes.md) | Core Web Vitals targets (LCP/INP/CLS), DevTools flame graph analysis, Lighthouse scripts | Performance auditing & bottleneck isolation |

---

## ⚡ 1. The 4-Step Profiling Lifecycle

```
1. BASELINE ➔ Measure current metrics with Lighthouse or cargo flamegraph.
2. IDENTIFY ➔ Locate specific bottlenecks (e.g. Long Tasks > 50ms, large bundle chunks).
3. OPTIMIZE ➔ Apply targeted code-splitting, memoization, or parallel fetch fixes.
4. VALIDATE ➔ Re-measure to prove latency or memory improvements.
```

---

## 📊 2. Core Web Vitals Baseline (Good Targets)

- **LCP (Largest Contentful Paint)**: `< 2.5s`
- **INP (Interaction to Next Paint)**: `< 200ms`
- **CLS (Cumulative Layout Shift)**: `< 0.1`

---

## 🛠️ 3. Execution Commands

```powershell
# 1. Run automated Lighthouse audit
python .agent/skills/performance-profiling/scripts/lighthouse_audit.py http://localhost:5173

# 2. Analyze frontend production bundle sizes
npm run build -- --profile
```