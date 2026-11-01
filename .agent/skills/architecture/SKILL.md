---
name: architecture
description: Architectural decision-making framework. Requirements analysis, trade-off evaluation, ADR documentation. Use when making architecture decisions or analyzing system design.
when_to_use: "When making architectural decisions, evaluating trade-offs, writing ADRs, or analyzing system design. NOT for direct code implementation."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / architecture
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Architecture Decision Framework

> "Requirements drive architecture. Trade-offs inform decisions. ADRs capture rationale."
> **Workflow Binding**: Used directly during the [`/plan`](../../workflows/plan.md), [`/brainstorm`](../../workflows/brainstorm.md), and [`/audit`](../../workflows/audit.md) workflows.

---

## 🎯 Progressive Disclosure & L3 Topic Index

Read **REQUIRED** core standards below; consult deep **L3 Topic Guides** on demand:

| Guide / Tool | Description | When to Read |
|---|---|---|
| [`references/context-discovery.md`](./references/context-discovery.md) | Questions to ask, project classification | Starting architecture design |
| [`references/trade-off-analysis.md`](./references/trade-off-analysis.md) | ADR templates, trade-off framework | Documenting decisions |
| [`references/pattern-selection.md`](./references/pattern-selection.md) | Decision trees, anti-patterns | Choosing patterns |
| [`references/examples.md`](./references/examples.md) | MVP, SaaS, Enterprise examples | Reference implementations |
| [`references/patterns-reference.md`](./references/patterns-reference.md) | Quick lookup for patterns | Pattern comparison |
| [`scripts/architecture_linter.py`](./scripts/architecture_linter.py) | Deterministic ADR structure and architecture boundary validator | Validating ADR documents or self-test verification |

---

## 🛡️ Socratic Gate: Architectural Trade-Off Alignment

Before committing to a structural or architectural decision:
1. **At Least 2 Competing Paradigms**: What are the alternatives (e.g. SQLite vs PostgreSQL, Tauri IPC vs HTTP REST)?
2. **Blast Radius Analysis**: What components, routes, or stores are impacted? (`npm run graph:blast`)
3. **Rollback & Reversibility**: If this architectural assumption fails in production, how painful is unwinding it?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Resume-Driven Architecture | Needless microservices and distributed complexity | Default to monolithic modular design with clean interfaces |
| Unrecorded Decisions | Future agents re-litigate decisions or regress | Write an ADR in `docs/adr/` capturing context and trade-offs |
| Leaky Abstractions | High coupling between layers | Enforce the 3-Layer Architecture (Directive / Orchestration / Execution) |
| Undocumented Failure Paths | Cascading failure under load | Define circuit breakers, timeouts, and fallback degradation |

---

## 🛠️ Verification & Execution Commands

```powershell
# Validate ADR structure
python .agent/skills/architecture/scripts/architecture_linter.py docs/adr/ADR-001.md

# Run self-test suite
python .agent/skills/architecture/scripts/architecture_linter.py --test
```