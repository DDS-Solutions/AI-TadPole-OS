---
name: domain-modeling
description: "Actively builds and sharpens the project's domain model, ubiquitous glossary (CONTEXT.md / docs/GLOSSARY.md), and architectural decision records (docs/adr/)."
when_to_use: "Use when defining domain terminology, resolving naming conflicts across stack boundaries, or recording architectural decisions."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / domain-modeling
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Ubiquitous language drift, semantic naming mismatches across stack boundaries, unrecorded domain entity mutations.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Domain Modeling & Ubiquitous Language Protocol

> **Purpose**: Maintain absolute conceptual fidelity and ubiquitous nomenclature across all repository tiers—from low-level Rust kernel structs and SQLite migrations to OpenAPI contracts and frontend React stores.

---

## 📚 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core modeling principles below; consult deep **L3 Reference Guides** on demand:

| L3 Resource | Scope & Purpose | Trigger / When to Load |
|---|---|---|
| [`domain_modeling_guide.md`](file:///D:/TadpoleOS-Dev/.agent/skills/domain-modeling/references/domain_modeling_guide.md) | Bounded contexts taxonomy, entity vs value object classification, glossary upkeep SOP | Designing new entities or mapping cross-tier contracts |
| [`domain_glossary_validator.py`](file:///D:/TadpoleOS-Dev/.agent/skills/domain-modeling/scripts/domain_glossary_validator.py) | Deterministic ubiquitous language & prohibited alias audit tool | Pre-commit validation of terminology parity |
| [`docs/GLOSSARY.md`](file:///D:/TadpoleOS-Dev/docs/GLOSSARY.md) | Authoritative Tadpole OS Technical Specifications & Glossary | Canonical definition of hubs, structs, and constants |
| [`documentation-templates`](file:///D:/TadpoleOS-Dev/.agent/skills/documentation-templates/SKILL.md) | Architecture Decision Record (ADR) blueprints | Formalizing structural decisions under `docs/adr/` |

---

## 🏛️ 1. Core Principles of Ubiquitous Language

1. **One Name, One Meaning**: If an autonomous objective is designated as a `Mission`, it must be named `Mission` across all layers:
   - Rust Struct: `pub struct Mission` ([server-rs/src/agent/types/mission.rs](file:///D:/TadpoleOS-Dev/server-rs/src/agent/types/mission.rs))
   - OpenAPI Contract: `components.schemas.Mission` ([docs/openapi.yaml](file:///D:/TadpoleOS-Dev/docs/openapi.yaml))
   - TypeScript Type: `export interface Mission` ([src/types/mission.ts](file:///D:/TadpoleOS-Dev/src/types/mission.ts))
   - SQLite Table: `CREATE TABLE missions` ([server-rs/migrations/](file:///D:/TadpoleOS-Dev/server-rs/migrations))
2. **Explicit Bounded Contexts**: Never leak internal execution abstractions into unrelated domains (e.g. keep UI visual pulse physics decoupled from cryptographic vault state).
3. **Immutability of Value Objects**: Value objects (e.g. `MicroQuota`, `Ed25519Signature`) must be immutable and replaceable, with zero independent lifecycles.

---

## 📖 2. Glossary Governance Protocols

1. **Canonical Glossary (`docs/GLOSSARY.md`)**: The single author of record for technical specifications, state hubs (`AppState`, `CommunicationHub`, `SecurityHub`), and system constants.
2. **Lazy Local Context (`CONTEXT.md`)**: When working within isolated sandboxes or client-facing applications, maintain a concise `CONTEXT.md` at root summarizing business domain entities.
3. **Immediate In-Turn Synchronization**: When a domain term is refined or extended during architecture or implementation turns, update the glossary in the **exact same turn**.

---

## 🚫 3. Critical Domain Modeling Anti-Patterns

- ❌ **Synonym Proliferation**: Introducing fragmented synonyms for canonical terms (e.g. calling an `Agent` a "bot", a `Mission` a "subtask", or a `Runner` an "executor").
- ❌ **Leaking DB Artifacts into Domain Models**: Naming frontend domain types after physical database quirks (e.g. naming an entity `TblAgentRow_V2` instead of `Agent`).
- ❌ **Implicit Naming Mismatches**: Using camelCase `missionId` in API responses while Rust emits snake_case `mission_id` without serde field renaming.
- ❌ **Unrecorded Structural Pivots**: Renaming core aggregates without recording an ADR in `docs/adr/` and updating `docs/GLOSSARY.md`.

---

## 🛠️ 4. Verification Commands

Run deterministic domain glossary validation and terminology audit:

```powershell
# 1. Run validator self-test suite
python .agent/skills/domain-modeling/scripts/domain_glossary_validator.py --test

# 2. Audit ubiquitous domain language in docs/
python .agent/skills/domain-modeling/scripts/domain_glossary_validator.py docs

# 3. Audit entire codebase for prohibited aliases
python .agent/skills/domain-modeling/scripts/domain_glossary_validator.py .

# 4. Output structured JSON report
python .agent/skills/domain-modeling/scripts/domain_glossary_validator.py . --json
```