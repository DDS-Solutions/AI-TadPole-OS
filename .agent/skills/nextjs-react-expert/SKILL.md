---
name: react-best-practices
description: React and Next.js performance optimization from Vercel Engineering. Use when building React components, optimizing performance, eliminating waterfalls, reducing bundle size, reviewing code for performance issues, or implementing server/client-side optimizations.
when_to_use: "When building React components, optimizing Next.js performance, eliminating waterfalls, or reducing bundle size. For React/Next.js web projects."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / nextjs-react-expert
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Next.js & React Performance Optimization Core

> **Philosophy**: Eliminate waterfalls first, optimize bundle payload second, eliminate re-render jank third.
> **Standard**: Vercel 57-Rule Performance Engineering Guide.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core priorities below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Impact / Priority | Trigger / When to Load |
|---|---|---|
| [`1-async-eliminating-waterfalls.md`](./1-async-eliminating-waterfalls.md) | 🔴 **CRITICAL** | Slow page loads, sequential `await` fetch waterfalls |
| [`2-bundle-bundle-size-optimization.md`](./2-bundle-bundle-size-optimization.md) | 🔴 **CRITICAL** | Large JS bundles (>200KB), barrel import bloat, dynamic imports |
| [`3-server-server-side-performance.md`](./3-server-server-side-performance.md) | 🟠 **HIGH** | Slow Server-Side Rendering (SSR), streaming Suspense boundaries |
| [`4-client-client-side-data-fetching.md`](./4-client-client-side-data-fetching.md) | 🟡 **MEDIUM-HIGH** | Client-side SWR/TanStack query deduplication, optimistic updates |
| [`5-rerender-re-render-optimization.md`](./5-rerender-re-render-optimization.md) | 🟡 **MEDIUM** | Excessive React re-renders, Zustand selector tuning, `React.memo` |
| [`6-rendering-rendering-performance.md`](./6-rendering-rendering-performance.md) | 🟡 **MEDIUM** | Virtualization (`@tanstack/react-virtual`), layout thrashing |
| [`7-js-javascript-performance.md`](./7-js-javascript-performance.md) | ⚪ **POLISH** | Fast loops, memoized expensive compute, micro-benchmarks |
| [`8-advanced-advanced-patterns.md`](./8-advanced-advanced-patterns.md) | 🔵 **ADVANCED** | `useLatest`, init-once refs, custom hook abstractions |
| [`9-cache-components.md`](./9-cache-components.md) | 🔴 **CRITICAL (Next 16+)** | `use cache`, `cacheLife`, Partial Prerendering (PPR), `cacheTag` |

---

## ⚡ 1. Top 5 Non-Negotiable Performance Rules

1. **Parallelize Data Fetching**: Never execute serial awaits for independent data (`Promise.all([fetchA(), fetchB()])`).
2. **Avoid Barrel File Imports**: Import specific modules directly (`import { Button } from '@radix-ui/react-button'` vs large monolithic barrels).
3. **Selector-Based State Subscription**: In Zustand/Redux, subscribe strictly to needed slices (`useStore(state => state.activeId)`).
4. **Defer Off-Screen Components**: Use `next/dynamic` or `React.lazy` for modals, drawers, and heavy charts.
5. **Batch Telemetry & RAF**: Never fire state updates directly inside tight event loops; debounce or batch to `requestAnimationFrame`.

---

## 🛠️ 2. Execution & Audit Workflow

```
1. AUDIT WATERFALLS ➔ Run parallel fetches and add Suspense boundaries.
2. AUDIT BUNDLE      ➔ Split vendor chunks, dynamic import heavy dependencies.
3. PROFILE RENDERS   ➔ Isolate expensive subtree states; memoize list rows.
4. VERIFY METRICS    ➔ Check Core Web Vitals (LCP < 2.5s, INP < 200ms, CLS < 0.1).
```