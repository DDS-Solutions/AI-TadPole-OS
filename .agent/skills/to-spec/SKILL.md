---
name: to-spec
description: "Converts informal conversation context or brainstorms into formal, published specification files under docs/specs/."
when_to_use: "When finalizing feature requirements, converting brainstorm outputs into specifications, or establishing an immutable technical contract before planning."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / to-spec
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Scope creep, ambiguous requirements, unversioned verbal contracts, or omitted boundary seams.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Conversation to Published Spec Protocol

> **Purpose**: Convert informal discussions, requirements, or brainstorm outputs into formal, version-controlled specification files saved under `docs/specs/<feature-name>.md`.  
> **Workflow Bindings**: Integrated directly into [`/brainstorm`](../../workflows/brainstorm.md) and [`/plan`](../../workflows/plan.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/specification_schema_and_examples.md`](./references/specification_schema_and_examples.md) | Standard specification templates, boundary seam schemas, and non-goal definitions | Drafting full architectural feature specifications |
| [`scripts/spec_linter.py`](./scripts/spec_linter.py) | Automated static linter verifying required sections, FR/NFR IDs, and out-of-scope boundaries | Validating specification documents in `docs/specs/` |

---

## 🧭 1. Canonical Specification Schema (`docs/specs/<name>.md`)

```markdown
# Feature Specification: <Name>

## 1. Overview & Objective
<High-level summary of problem statement and business/technical objective>

## 2. Ubiquitous Domain Terminology
<Key domain terms mapped to CONTEXT.md / docs/GLOSSARY.md>

## 3. Functional Requirements
- **FR-1**: <Requirement description>
- **FR-2**: <Requirement description>

## 4. Non-Functional Requirements (NFRs)
- **NFR-1**: Performance / Latency bounds (< 200ms)
- **NFR-2**: Security & Permission constraints (SEC-03)

## 5. System Boundary & Seams
- **IPC / API Schema**: <Expected signatures, request/response models>
- **State Changes**: <Store mutations, database migrations>

## 6. Out of Scope (Non-Goals)
<Explicitly excluded capabilities to prevent scope creep>
```

---

## 🧠 2. Socratic Gate for Specification Authoring

Before saving a specification, verify:
- **Gate 1 (Numbered Requirements)**: Are all functional requirements given explicit IDs (`FR-1`, `FR-2`) for traceability?
- **Gate 2 (Explicit Out-of-Scope)**: Are non-goals documented to prevent runaway scope during execution?
- **Gate 3 (Boundary Definitions)**: Are API payloads, database tables, and IPC commands explicitly typed?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Vague Prose Requirements** | "Make the system fast and responsive." | Write measurable criteria: `NFR-1: p99 latency < 200ms`. |
| **No Out-of-Scope Section** | Subagents and developers add speculative features. | Explicitly list excluded features in Section 6. |
| **Undocumented Seams** | Backend and frontend implement conflicting JSON payloads. | Define explicit TypeScript/Rust interface contracts in Section 5. |

---

## 🛠️ 4. Deterministic Verification Gate

Validate specification markdown files:

```powershell
# 1. Run built-in self-tests
python .agent/skills/to-spec/scripts/spec_linter.py --test

# 2. Lint a published feature specification
python .agent/skills/to-spec/scripts/spec_linter.py --file docs/specs/swarm_orchestration.md
```