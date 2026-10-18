---
name: zustand-patterns
description: "Zustand state store architecture, slice patterns, selector memoization with useShallow, and outside-component actions."
when_to_use: "When authoring or refactoring Zustand state stores, writing React components subscribing to global state, or eliminating re-render cascades."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / zustand-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Whole-store destructuring, unnecessary re-render cascades, state synchronization bugs.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Zustand Store Architecture & Performance Protocol

> **Purpose**: Enforce atomic selector subscriptions, modular slice architectures, and deterministic state mutations across Tadpole OS frontend stores.
> **Workflow Binding**: Used during [`/refactor`](../../workflows/refactor.md) and [`/ui-ux-pro-max`](../../workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/zustand_slices_and_selectors.md`](./references/zustand_slices_and_selectors.md) | Slice pattern templates, `useShallow` multi-selector recipes, outside-component updates | Creating new stores or refactoring multi-property component hooks |
| [`scripts/zustand_auditor.py`](./scripts/zustand_auditor.py) | Deterministic scanner for whole-store destructuring anti-patterns | Auditing frontend component trees or self-test verification |

---

## 🏛️ Core Architectural Invariants

1. **Atomic Selector Subscriptions**: Never destructure directly from `useStore()`. Always pass an explicit selector: `useStore(state => state.property)`.
2. **Shallow Multi-Select**: When selecting multiple fields, use `useShallow` from `zustand/react/shallow` to avoid re-rendering on identical object shapes.
3. **Immutability Invariant**: Treat state as strictly immutable. Use spread operators or Immer middleware; never mutate state objects in place.
4. **Decoupled Actions**: Expose actions as discrete methods on the store or slice rather than ad-hoc inline setters.

---

## 🛡️ Socratic Gate: Store Scope & Selector Audits

Before adding or subscribing to a Zustand store:
1. **Global vs Local**: Does this state truly need cross-route permanence, or should it remain local component state (`useState`)?
2. **Selector Granularity**: Are you subscribing to the smallest possible primitive or object slice?
3. **Async Action Safety**: Do async actions handle network failure gracefully without leaving loading flags stuck?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| `const { a, b } = useStore()` | Component re-renders on ANY store change | Use discrete selectors: `const a = useStore(s => s.a)` |
| Direct state mutation (`state.list.push(item)`) | React fails to detect state change | Return new state: `list: [...state.list, item]` |
| Putting non-serializable objects in store | Breaks devtools and persistence middleware | Store serializable plain objects, IDs, or timestamps |

---

## 🛠️ Verification & Execution Commands

```powershell
# Audit component files for selector anti-patterns
python .agent/skills/zustand-patterns/scripts/zustand_auditor.py src

# Run self-test suite
python .agent/skills/zustand-patterns/scripts/zustand_auditor.py --test
```
