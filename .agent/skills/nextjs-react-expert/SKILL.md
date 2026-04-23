> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: react-best-practices
description: React/Next.js performance optimization. Waterfalls, bundle size, rendering.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# Tadpole UI & React Performance (Vite 8 + React 19)

**eliminate waterfalls -> atomize state -> premium aesthetic.**

## 🎯 Content Map (Read Impact High -> Low)
1.  🔴 `1-async-eliminating-waterfalls.md` (CRITICAL): Network latency.
2.  🔴 `2-bundle-bundle-size-optimization.md` (CRITICAL): TTI/Load.
3.  🟠 `3-server-server-side-performance.md` (HIGH): SSR speed.
4.  🟡 `4-client-client-side-data-fetching.md`: SWR/Dedupe.
5.  🟡 `5-rerender-re-render-optimization.md`: Memo/Render.
6.  🟡 `6-rendering-rendering-performance.md`: Virtualization.
7.  ⚪ `7-js-javascript-performance.md`: Micro-ops.
8.  🔵 `8-advanced-advanced-patterns.md`: Rare patterns.

## 🎯 Core Stack (Tadpole OS)
- **Engine**: Vite 8 + React 19 + TypeScript 5.9.
- **State**: Zustand (Atomic stores in `src/stores/`).
- **Styling**: Tailwind CSS v4 (CSS-first tokens).
- **Animation**: Framer Motion 12.

## ❌ Anti-Patterns
- **Waterfalls**: Sequential `await` in stores. Use `Promise.all`.
- **Prop-Drilling**: Passing state through >2 layers. Use Zustand.
- **Context Overuse**: Use React Context sparingly; favor atomic store subscriptions.
- **Large Components**: Keep UI components < 150 lines.

---

## 🧠 Aletheia Reasoning Protocol (Performance)

### 1. Generator (Architecture)
*   **Boundary**: "Server vs Client?".
*   **Strategy**: "Fetch-on-render vs Render-as-you-fetch?".
*   **State**: "URL vs Global vs Local?".

### 2. Verifier (Bottleneck Hunt)
*   **Waterfall**: "Serial requests in Network tab?".
*   **Bundle**: "Lodash/Moment in bundle?".
*   **Flash**: "Rerenders highlighting whole page?".

### 3. Reviser (Optimization)
*   **Memo**: "Memoize charts/tables".
*   **Lazy**: "Dynamic import modals".

---

## 🛡️ Security & Safety Protocol (React)

1.  **Server Actions**: Verify auth INSIDE action.
2.  **XSS**: Sanitize `dangerouslySetInnerHTML`.
3.  **Secrets**: NO `NEXT_PUBLIC_` secrets.
4.  **Deps**: Audit regularly.

## Review Checklist
- [ ] No sequential fetches.
- [ ] Bundle < 200KB.
- [ ] No barrel imports.
- [ ] Dynamic imports for heavy comps.
- [ ] Server Components default.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
