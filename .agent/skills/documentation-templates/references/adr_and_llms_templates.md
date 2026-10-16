> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / documentation-templates
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Missing ADR context or outdated AI context maps.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[DOCS]`)

# Architecture Decision Records (ADR) & LLMS.txt Templates (L3)

---

## 1. Architecture Decision Record (ADR) Template

```markdown
# ADR-00[X]: [Title of Architectural Decision]

## Status
[Proposed | Accepted | Deprecated | Superseded by ADR-00Y]

## Context & Problem Statement
What is the architectural problem? Which alternatives and trade-offs were evaluated?

## Decision
What is the final technical decision and rationale?

## Consequences
- **Positive**: [Immediate and systemic benefits]
- **Negative / Trade-offs**: [Incurred technical debt or maintenance costs]
- **Observability Impact**: [New telemetry tags or parity checks required]
```

---

## 2. Standard `llms.txt` Context Map

```markdown
# [Project Name] AI-Context Map

## Core Objective
One-sentence summary of the sovereign application purpose.

## Subsystem Architecture
- `server-rs/src/`: Rust Axum backend and state hubs.
- `src/`: React frontend components and Zustand state stores.
- `docs/`: Canonical architecture contracts and security registries.
```