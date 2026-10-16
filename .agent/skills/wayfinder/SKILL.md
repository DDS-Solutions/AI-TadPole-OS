---
name: wayfinder
description: Plan large multi-session initiatives wrapped in fog as a map of linked decision tickets, resolving them sequentially without context degradation.
when_to_use: "Use when an epic or task is too large for one context window or wrapped in initial fog of war."
allowed-tools: Read, Glob, Grep, Write
disable-model-invocation: true
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / wayfinder
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Fog-of-War Wayfinding Protocol

Chart large multi-session initiatives wrapped in fog by building a persistent map artifact rather than charging blindly at an unrefined destination.

---

## The Map Structure

Save the map to `.tmp/wayfinder-map-<epic-name>.md` or link it to an issue tracker issue labelled `wayfinder:map`.

```markdown
# Epic Map: <Name>

## Destination
<What reaching the end of this map looks like — 1-2 lines. Shapes every decision ticket.>

## Notes
<Domain rules, architecture constraints, standing preferences for this epic.>

## Decisions So Far
- [<Closed Ticket Title>](link) — <One-line gist of the decision/resolution>

## Not Yet Specified (Fog of War)
<!-- Dim view of upcoming in-scope questions that cannot yet be phrased precisely -->

## Out of Scope
<!-- Work ruled beyond the destination; closed, never graduates -->
```

---

## Operating Principles

1. **Plan, Don't Execute**: Each ticket resolves a *decision*, not a slice of code to build immediately.
2. **Refer by Name**: Always cite tickets by full descriptive title, not bare IDs (`#42`).
3. **Fog vs. Ticket**:
   - **Create a Ticket** when the decision question can be phrased precisely now (even if blocked).
   - **Keep in Not Yet Specified** when the question is hazy. Graduate it into a ticket as the frontier advances.
4. **Single Ticket per Session**: Resolve exactly one decision ticket per agent session to avoid context token saturation.