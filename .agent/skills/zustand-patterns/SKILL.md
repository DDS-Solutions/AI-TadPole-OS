---
name: zustand-patterns
description: "Zustand state store architecture, slice patterns, selector memoization with useShallow, transient telemetry subscriptions, and outside-component actions."
when_to_use: "When authoring or refactoring Zustand state stores, writing React components subscribing to global state, or eliminating re-render cascades."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.2.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / zustand-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Whole-store destructuring, unnecessary re-render cascades, state synchronization bugs, frame drops during log streaming.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Zustand Store Architecture & Performance Protocol

> **Purpose**: Enforce atomic selector subscriptions, modular slice architectures, transient high-frequency streaming subscriptions, and deterministic state mutations across Tadpole OS frontend stores (Zustand v5).
> **Skill Synergy & Boundaries**:
> - **Desktop IPC & Streaming Channels**: Delegates to [`tauri-pro`](../tauri-pro/SKILL.md).
> - **Component Styling & Tokens**: Delegates to [`tailwind-patterns`](../tailwind-patterns/SKILL.md) and [`frontend-design`](../frontend-design/SKILL.md).
> - **Workflow Bindings**: Integrated directly into [`/refactor`](../../workflows/refactor.md) and [`/ui-ux-pro-max`](../../workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/zustand_slices_and_selectors.md`](./references/zustand_slices_and_selectors.md) | Slice pattern templates, `useShallow` recipes, transient telemetry subscriptions, session reset registry | Creating new stores or optimizing high-frequency telemetry renders |
| [`scripts/zustand_auditor.py`](./scripts/zustand_auditor.py) | Deterministic scanner for whole-store destructuring anti-patterns | Auditing frontend component trees or self-test verification |

---

## 🏛️ Core Architectural Invariants

1. **Atomic Selector Subscriptions**: Never destructure directly from `useStore()`. Always pass an explicit selector: `useStore(state => state.property)`.
2. **Shallow Multi-Select**: When selecting multiple fields, use `useShallow` from `zustand/react/shallow` to avoid re-rendering on identical object shapes.
3. **Transient Subscriptions for High-Frequency Telemetry**: For streaming data $> 10\text{ Hz}$ (logs, token meters, node coordinates), bypass React render cycles via `useStore.subscribe((slice) => { /* direct ref mutation */ })`.
4. **Immutability Invariant**: Treat state as strictly immutable. Use spread operators or Immer middleware; never mutate state objects in place.
5. **Decoupled Actions & Outside Access**: Expose actions as discrete methods on the store or slice; update from external event buses using `useStore.getState().action()`.
6. **Session Reset Registry**: All domain stores must expose a `reset()` method registered with the global teardown manager to prevent cross-session memory leaks.

---

## 🛡️ Socratic Gate: Store Scope & Selector Audits

Before adding or subscribing to a Zustand store:
1. **Global vs Local**: Does this state truly need cross-route permanence, or should it remain local component state (`useState`)?
2. **Selector Granularity**: Are you subscribing to the smallest possible primitive or object slice?
3. **High-Frequency Traffic**: If this state updates rapidly during agent execution, is it using a transient ref subscription to prevent 60 FPS frame drops?
4. **Session Teardown**: Does this store implement a `reset()` action for clean project unmounting?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| `const { a, b } = useStore()` | Component re-renders on ANY store change | Use discrete selectors or `useShallow` |
| Direct state mutation (`state.list.push(item)`) | React fails to detect state change | Return new state: `list: [...state.list, item]` |
| Putting high-frequency logs in React state | Chokes UI render thread, dropping frames | Use transient `useStore.subscribe` to DOM refs |
| Putting non-serializable objects in store | Breaks devtools and persistence middleware | Store serializable plain objects, IDs, or timestamps |
| Leaking state across project switch | Displays stale data from previous workspace | Call registered `reset()` handlers on project unload |

---

## 🛠️ Verification & Execution Commands

```powershell
# Audit component files for selector anti-patterns
python .agent/skills/zustand-patterns/scripts/zustand_auditor.py src

# Run self-test suite
python .agent/skills/zustand-patterns/scripts/zustand_auditor.py --test
```
