---
name: nextjs-react-expert
description: "React, Next.js, and Vite SPA performance optimization. Covers waterfall elimination, bundle size reduction, React 19 hooks, and client-side desktop performance."
when_to_use: "When building React components, optimizing Vite or Next.js performance, eliminating waterfalls, reducing bundle size, or auditing frontend rendering."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: high
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / nextjs-react-expert
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Sequential fetch waterfalls, bloated vendor bundles, or unmemoized re-render cascades.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# React & Next.js Performance Optimization Core

> **Philosophy**: Eliminate waterfalls first, optimize bundle payload second, eliminate re-render jank third.  
> **Native Target**: In Tadpole OS, the frontend is a **Vite + React 19 SPA inside a Tauri desktop shell**. SSR rules apply when building external apps; SPA & Tauri IPC rules apply for core OS.  
> **Workflow Bindings**: Integrated into [`/refactor`](file:///D:/TadpoleOS-Dev/.agent/workflows/refactor.md) and [`/ui-ux-pro-max`](file:///D:/TadpoleOS-Dev/.agent/workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign priorities below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Impact / Priority | Trigger / When to Load |
|---|---|---|
| [`references/1-async-eliminating-waterfalls.md`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/references/1-async-eliminating-waterfalls.md) | 🔴 **CRITICAL** | Slow page loads, sequential `await` fetch waterfalls |
| [`references/2-bundle-bundle-size-optimization.md`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/references/2-bundle-bundle-size-optimization.md) | 🔴 **CRITICAL** | Large JS bundles (>200KB), barrel import bloat, dynamic imports |
| [`references/3-server-server-side-performance.md`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/references/3-server-server-side-performance.md) | 🟠 **HIGH** | Slow Server-Side Rendering (SSR), streaming Suspense boundaries |
| [`references/4-client-client-side-data-fetching.md`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/references/4-client-client-side-data-fetching.md) | 🟡 **MEDIUM-HIGH** | Client-side SWR/TanStack query deduplication, optimistic updates |
| [`references/5-rerender-re-render-optimization.md`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/references/5-rerender-re-render-optimization.md) | 🟡 **MEDIUM** | Excessive React re-renders, Zustand selector tuning, `React.memo` |
| [`references/6-rendering-rendering-performance.md`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/references/6-rendering-rendering-performance.md) | 🟡 **MEDIUM** | Virtualization (`@tanstack/react-virtual`), layout thrashing |
| [`references/7-js-javascript-performance.md`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/references/7-js-javascript-performance.md) | ⚪ **POLISH** | Fast loops, memoized expensive compute, micro-benchmarks |
| [`references/8-advanced-advanced-patterns.md`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/references/8-advanced-advanced-patterns.md) | 🔵 **ADVANCED** | `useLatest`, init-once refs, custom hook abstractions |
| [`references/9-cache-components.md`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/references/9-cache-components.md) | 🔴 **CRITICAL (Next 16+)** | `use cache`, `cacheLife`, Partial Prerendering (PPR), `cacheTag` |
| [`scripts/react_performance_checker.py`](file:///D:/TadpoleOS-Dev/.agent/skills/nextjs-react-expert/scripts/react_performance_checker.py) | 🛠️ **AUDITOR** | Automated waterfall, barrel-import, and memoization scanner |

---

## ⚡ 1. Top 5 Non-Negotiable Performance Rules

1. **Parallelize Data Fetching**: Never execute serial awaits for independent data (`Promise.all([fetchA(), fetchB()])`).
2. **Avoid Barrel File Imports**: Import specific modules directly (`import { Button } from '@radix-ui/react-button'` vs large monolithic barrels).
3. **Selector-Based State Subscription**: In Zustand/Redux, subscribe strictly to needed slices (`useStore(state => state.activeId)`).
4. **Defer Off-Screen Components**: Use `next/dynamic` or `React.lazy` for modals, drawers, and heavy charts.
5. **Batch Telemetry & RAF**: Never fire state updates directly inside tight event loops; debounce or batch to `requestAnimationFrame`.

---

## 🧠 2. Socratic Gate for React Optimization

Before modifying component trees or data loading architectures, verify:
- **Gate 1 (Data Dependencies)**: Can these independent network requests run in parallel via `Promise.all` or React Server Components?
- **Gate 2 (Bundle Boundary)**: Is this heavy component (>10KB) loaded on the critical render path or can it be deferred with `dynamic()`?
- **Gate 3 (State Granularity)**: Does the component subscribe to the entire store object or a minimal atomic selector?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Sequential Awaits** | `await fetchA(); await fetchB();` doubles Time to First Byte (TTFB). | Parallelize with `Promise.all([fetchA(), fetchB()])`. |
| **Monolithic Barrel Imports** | `import { Icon } from 'lucide-react'` pulls hundreds of unused modules. | Direct module import or Next.js `optimizePackageImports`. |
| **`useEffect` Data Fetching** | Client fetch in `useEffect` creates layout shifts and duplicate requests. | Migrate to Server Components, TanStack Query, or SWR. |
| **Whole-Store Subscriptions** | `const state = useStore()` re-renders component on any store change. | Use atomic selectors: `const value = useStore(s => s.value)`. |
| **Unoptimized Images** | Raw `<img>` tags load full-res assets and trigger CLS shifts. | Use `next/image` with explicit `width`, `height`, and `priority`. |

---

## 🛠️ 4. Deterministic Verification Gate

Audit React source code for waterfalls and bundle bloat:

```powershell
# 1. Run built-in self-tests
python .agent/skills/nextjs-react-expert/scripts/react_performance_checker.py --test

# 2. Audit a target frontend source directory
python .agent/skills/nextjs-react-expert/scripts/react_performance_checker.py src/
```