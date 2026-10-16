---
name: domain-modeling
description: Actively builds and sharpens the project's domain model, ubiquitous glossary (CONTEXT.md), and architectural decision records (docs/adr/).
when_to_use: "Use when defining domain terminology, resolving naming conflicts across stack boundaries, or recording architectural decisions."
allowed-tools: Read, Glob, Grep, Write, Edit
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / domain-modeling
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Domain Modeling & Glossary Alignment Protocol

Actively sharpen the project's domain model as code and architecture evolve. Ensure terms remain strictly consistent across Rust backend (`server-rs`) and React frontend stores (`Zustand`).

---

## File Structure

- `CONTEXT.md` (at workspace root): Ubiquitous domain glossary.
- `docs/adr/*.md`: Architectural Decision Records (ADRs).

---

## 1. Glossary Upkeep Rules

1. **Lazy Creation**: If `CONTEXT.md` does not exist, create it lazily when the first domain concept is defined.
2. **IPC & Schema Boundary Verification**: When creating a database column, API payload, or UI component, verify its name against `CONTEXT.md`.
3. **Term Resolution**: If a term is refined or sharpened during design, update `CONTEXT.md` immediately in the same turn.

---

## 2. ADR Creation Rules

Record an ADR under `docs/adr/` whenever:
- A significant architectural choice is made (e.g. database choice, IPC protocol, state management pattern).
- The user rejects a refactoring proposal for a load-bearing structural reason.

Format ADRs using standard Nygard template: Title, Status, Context, Decision, Consequences.