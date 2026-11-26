---
name: wayfinder
description: "Plan large multi-session initiatives wrapped in fog as a map of linked decision tickets, resolving them sequentially without context degradation."
when_to_use: "When an epic or multi-session initiative is too large for a single context window, or wrapped in initial fog of war."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / wayfinder
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Charging blindly into foggy architecture without a decision map, or blowing context windows with multi-session scope.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Fog-of-War Wayfinding Protocol

> **Purpose**: Chart large, ambiguous multi-session epics by maintaining an explicit, version-controlled map of linked decisions in `.tmp/wayfinder-map-<epic>.md` rather than charging blindly at an unrefined destination.  
> **Workflow Bindings**: Integrated directly into [`/plan`](../../workflows/plan.md) and [`/brainstorm`](../../workflows/brainstorm.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/epic_maps_and_decision_trees.md`](./references/epic_maps_and_decision_trees.md) | Fog-of-War frontier schemas, ticket graduation workflows, and decision tree structures | Initializing or updating an epic decision map |
| [`scripts/wayfinder_linter.py`](./scripts/wayfinder_linter.py) | Automated linter verifying Destination, Settled Decisions, Fog of War, and Out-of-Scope boundaries | Auditing `.tmp/wayfinder-map-*.md` documents |

---

## 🧭 1. The Map Structure (`.tmp/wayfinder-map-<epic>.md`)

```markdown
# Epic Map: <Name>

## 1. Destination
<What reaching the end of this map looks like — 1-2 lines. Shapes every decision ticket.>

## 2. Architectural Constraints & Standing Notes
<Domain rules, architecture constraints, standing preferences for this epic.>

## 3. Decisions So Far
- [x] [<Closed Decision Title>] — <One-line gist of the settled decision and rationale>

## 4. Active Decision Ticket
- [ ] [<Current Decision Title>] — <Specific trade-off being resolved in this session>

## 5. Not Yet Specified (Fog of War)
<!-- Hazy upcoming questions that cannot yet be phrased precisely until earlier tickets settle -->

## 6. Out of Scope (Non-Goals)
<!-- Work ruled permanently beyond the destination -->
```

---

## ⚡ 2. Operating Principles

1. **Decide, Don't Build Prematurely**: Each wayfinder ticket settles an architectural fork or interface contract before production coding starts.
2. **Single Decision per Session**: Focus each agent turn on resolving exactly one active ticket to avoid token bloat and cognitive confusion.
3. **Graduate from the Fog**: As decisions settle, upcoming hazy questions under *Not Yet Specified* become crisp and graduate into the next ticket.

---

## 🧠 3. Socratic Gate for Wayfinding

Before tackling an ambiguous epic, verify:
- **Gate 1 (Destination Clarity)**: Is there a clear 1–2 sentence destination defining what completion looks like?
- **Gate 2 (Frontier Boundedness)**: Are we resolving only the single active decision rather than trying to settle the entire fog simultaneously?
- **Gate 3 (Decision Persistence)**: Are settled decisions recorded under *Decisions So Far* so incoming sessions start with full historical alignment?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Charging Blindly** | Starting to code a 5,000-line architecture with unanswered fundamental questions. | Stop and build a Wayfinder map first. |
| **Solving the Whole Epic at Once** | Trying to settle database, network, UI, and auth in one prompt turn exhausts tokens. | Resolve one decision ticket per session. |
| **Re-Litigating Settled Decisions** | Forgetting why a previous choice was made and re-debating it. | Record settled choices in *Decisions So Far* with rationale. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate epic map structure and completeness:

```powershell
# 1. Run built-in self-tests
python .agent/skills/wayfinder/scripts/wayfinder_linter.py --test

# 2. Lint a wayfinder map file
python .agent/skills/wayfinder/scripts/wayfinder_linter.py --file .tmp/wayfinder-map-local-llm.md
```