---
name: react-best-practices
description: React/Next.js performance optimization. Waterfalls, bundle size, rendering.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# Next.js & React Performance

**eliminate waterfalls -> optimize bundles -> micro-optimize.**

## 🎯 Content Map (Read Impact High -> Low)
1.  🔴 `1-async-eliminating-waterfalls.md` (CRITICAL): Network latency.
2.  🔴 `2-bundle-bundle-size-optimization.md` (CRITICAL): TTI/Load.
3.  🟠 `3-server-server-side-performance.md` (HIGH): SSR speed.
4.  🟡 `4-client-client-side-data-fetching.md`: SWR/Dedupe.
5.  🟡 `5-rerender-re-render-optimization.md`: Memo/Render.
6.  🟡 `6-rendering-rendering-performance.md`: Virtualization.
7.  ⚪ `7-js-javascript-performance.md`: Micro-ops.
8.  🔵 `8-advanced-advanced-patterns.md`: Rare patterns.

## 🚀 Quick Decision Tree
- **Slow Load**: Sec 1 (Waterfalls) + Sec 2 (Bundle).
- **Large Bundle**: Sec 2 (Dynamic Imports).
- **Slow SSR**: Sec 3 (Streaming).
- **Laggy UI**: Sec 5 (Re-renders) + Sec 6 (Virtualization).

## ❌ Anti-Patterns
- **Waterfalls**: Sequential `await`. Use `Promise.all`.
- **Bloat**: Importing full libs. Use named imports.
- **Barrels**: `index.ts` re-exports. Avoid in app code.
- **Client Components**: Use Server Components default.

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
