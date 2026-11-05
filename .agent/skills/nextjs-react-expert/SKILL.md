---
name: nextjs-react-expert
description: "React, Next.js, and Vite SPA performance optimization. Covers waterfall elimination, bundle size reduction, React 19 hooks, and client-side desktop performance."
when_to_use: "When building React components, optimizing Vite or Next.js performance, eliminating waterfalls, reducing bundle size, or auditing frontend rendering."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.2.0"
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
> **Skill Synergy & Boundaries**:
> - **Global State Stores & Telemetry**: Delegates to [`zustand-patterns`](../zustand-patterns/SKILL.md).
> - **Styling Tokens & UI Design**: Delegates to [`tailwind-patterns`](../tailwind-patterns/SKILL.md) and [`frontend-design`](../frontend-design/SKILL.md).
> - **Desktop IPC & Window Events**: Delegates to [`tauri-pro`](../tauri-pro/SKILL.md).
> - **Browser E2E Verification**: Delegates to [`webapp-testing`](../webapp-testing/SKILL.md).
> - **Workflow Bindings**: Integrated into [`/refactor`](../../workflows/refactor.md) and [`/ui-ux-pro-max`](../../workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign priorities below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Impact / Priority | Trigger / When to Load |
|---|---|---|
| [`references/1-async-eliminating-waterfalls.md`](./references/1-async-eliminating-waterfalls.md) | 🔴 **CRITICAL** | Slow page loads, sequential `await` fetch waterfalls |
| [`references/2-bundle-bundle-size-optimization.md`](./references/2-bundle-bundle-size-optimization.md) | 🔴 **CRITICAL** | Large JS bundles (>200KB), barrel import bloat, dynamic imports |
| [`references/3-server-server-side-performance.md`](./references/3-server-server-side-performance.md) | 🟠 **HIGH** | Slow Server-Side Rendering (SSR), streaming Suspense boundaries |
| [`references/4-client-client-side-data-fetching.md`](./references/4-client-client-side-data-fetching.md) | 🟡 **MEDIUM-HIGH** | Client-side SWR/TanStack query deduplication, optimistic updates |
| [`references/5-rerender-re-render-optimization.md`](./references/5-rerender-re-render-optimization.md) | 🟡 **MEDIUM** | Excessive React re-renders, Zustand selector tuning, `React.memo` |
| [`references/6-rendering-rendering-performance.md`](./references/6-rendering-rendering-performance.md) | 🟡 **MEDIUM** | Virtualization (`@tanstack/react-virtual`), layout thrashing |
| [`references/7-js-javascript-performance.md`](./references/7-js-javascript-performance.md) | ⚪ **POLISH** | Fast loops, memoized expensive compute, micro-benchmarks |
| [`references/8-advanced-advanced-patterns.md`](./references/8-advanced-advanced-patterns.md) | 🔵 **ADVANCED** | `useLatest`, init-once refs, custom hook abstractions |
| [`references/9-cache-components.md`](./references/9-cache-components.md) | 🔴 **CRITICAL (Next 16+)** | `use cache`, `cacheLife`, Partial Prerendering (PPR), `cacheTag` |
| [`scripts/react_performance_checker.py`](./scripts/react_performance_checker.py) | 🛠️ **AUDITOR** | Automated waterfall, barrel-import, and memoization scanner |

---

## ⚡ 1. Top 5 Non-Negotiable Performance Rules

1. **Parallelize Data Fetching**: Never execute serial awaits for independent data (`Promise.all([fetchA(), fetchB()])`).
2. **Avoid Barrel File Imports**: Import specific modules directly (`import { Button } from '@radix-ui/react-button'` vs large monolithic barrels).
3. **Selector-Based State Subscription**: In Zustand/Redux, subscribe strictly to needed slices (`useStore(state => state.activeId)`).
4. **React 19 Transitions for Desktop UI**: Wrap non-urgent desktop view mutations in `startTransition` or React 19 action hooks (`useActionState`, `useOptimistic`) to keep input responsiveness at 60 FPS.
5. **Defer Off-Screen Components**: Use `React.lazy` or `dynamic()` for modals, drawers, and heavy canvas graphs.

---

## 🧠 2. Socratic Gate for React Optimization

Before modifying component trees or data loading architectures, verify:
- **Gate 1 (Data Dependencies)**: Can these independent network requests run in parallel via `Promise.all` or React Server Components?
- **Gate 2 (Bundle Boundary)**: Is this heavy component (>10KB) loaded on the critical render path or can it be deferred with `lazy()`?
- **Gate 3 (State Granularity)**: Does the component subscribe to the entire store object or a minimal atomic selector?
- **Gate 4 (Input Latency)**: Are heavy UI state updates wrapped in `startTransition` to prevent blocking typing inputs?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Sequential Awaits** | `await fetchA(); await fetchB();` doubles Time to First Byte (TTFB). | Parallelize with `Promise.all([fetchA(), fetchB()])`. |
| **Monolithic Barrel Imports** | `import { Icon } from 'lucide-react'` pulls hundreds of unused modules. | Direct module import or Vite tree-shaking aliases. |
| **`useEffect` Data Fetching** | Client fetch in `useEffect` creates layout shifts and duplicate requests. | Migrate to Server Components, TanStack Query, or SWR. |
| **Whole-Store Subscriptions** | `const state = useStore()` re-renders component on any store change. | Use atomic selectors: `const value = useStore(s => s.value)`. |
| **Blocking Transitions** | Heavy graph re-renders block user typing in search bars. | Wrap state update in `startTransition(() => ...)`. |

---

## 🛠️ 4. Deterministic Verification Gate

Audit React source code for waterfalls and bundle bloat:

```powershell
# 1. Run built-in self-tests
python .agent/skills/nextjs-react-expert/scripts/react_performance_checker.py --test

# 2. Audit a target frontend source directory
python .agent/skills/nextjs-react-expert/scripts/react_performance_checker.py src/
```