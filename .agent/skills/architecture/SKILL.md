---
name: architecture
description: Architectural decision-making framework. Requirements analysis, trade-off evaluation, ADR documentation. Use when making architecture decisions or analyzing system design.
when_to_use: "When making architectural decisions, evaluating trade-offs, writing ADRs, or analyzing system design. NOT for direct code implementation."
allowed-tools: Read, Glob, Grep
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SKILL]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Architecture Decision Framework

> "Requirements drive architecture. Trade-offs inform decisions. ADRs capture rationale."

## 🎯 Selective Reading Rule

**Read ONLY files relevant to the request!** Check the content map, find what you need.

| File | Description | When to Read |
|------|-------------|--------------|
| `context-discovery.md` | Questions to ask, project classification | Starting architecture design |
| `trade-off-analysis.md` | ADR templates, trade-off framework | Documenting decisions |
| `pattern-selection.md` | Decision trees, anti-patterns | Choosing patterns |
| `examples.md` | MVP, SaaS, Enterprise examples | Reference implementations |
| `patterns-reference.md` | Quick lookup for patterns | Pattern comparison |

---

## 🔗 Related Skills

| Skill | Use For |
|-------|---------|
| `@[skills/database-design]` | Database schema design |
| `@[skills/api-patterns]` | API design patterns |
| `@[skills/deployment-procedures]` | Deployment architecture |

---

## Core Principle

**"Simplicity is the ultimate sophistication."**

- Start simple
- Add complexity ONLY when proven necessary
- You can always add patterns later
- Removing complexity is MUCH harder than adding it

---

## Validation Checklist

Before finalizing architecture:

- [ ] Requirements clearly understood
- [ ] Constraints identified
- [ ] Each decision has trade-off analysis
- [ ] Simpler alternatives considered
- [ ] ADRs written for significant decisions
- [ ] Team expertise matches chosen patterns

---

## 🏛️ Deep Module Design Principles

When designing or refactoring system components, favor **Deep Modules** over shallow abstractions:

- **Module**: Scale-agnostic unit with an interface and implementation.
- **Interface**: Everything a caller must know (types, ordering constraints, error modes).
- **Depth**: High implementation capability behind a minimal interface.
- **Seam**: Clear location where behavior can be altered without touching callers.
- **Adapter**: Role-based implementation satisfying a seam interface.
- **Leverage**: Capability callers gain per unit of interface learned.
- **Locality**: Concentration of changes, bugs, and tests in one place.

Use `@[skills/improve-codebase-architecture]` to scan git hot spots and generate interactive visual HTML refactoring proposals.

[//]: # (Metadata: [SKILL])
