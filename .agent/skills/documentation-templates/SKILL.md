---
name: documentation-templates
description: Standardized documentation protocols for humans and AI. Manages READMEs, API specs, ADRs, and AI-native context maps.
when_to_use: "When creating or updating project documentation, writing Architecture Decision Records (ADRs), or generating AI context maps."
allowed-tools: Read, Write, Edit, Glob, Grep
version: 2.1
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / documentation-templates
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Documentation Protocols & Knowledge Heritage

> **Philosophy**: Documentation is Infrastructure. It prevents decision regressions and accelerates agent reasoning.
> **Workflow Binding**: Used directly during [`/codify`](../../workflows/codify.md), [`/report`](../../workflows/report.md), and [`/audit`](../../workflows/audit.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** documentation rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/adr_and_llms_templates.md`](./references/adr_and_llms_templates.md) | Standard ADR format, `llms.txt` schema, API contract templates | Writing ADRs and updating root documentation |

---

## 📚 1. The Documentation Lifecycle

```
BRAINSTORM (/brainstorm) ➔ ADR (docs/adr/) ➔ IMPLEMENT (/create) ➔ VERIFY (/test) ➔ CODIFY (/codify)
```

---

## ✍️ 2. The Code Commenting Rule ("Comment the Why, Not the What")

- **Comment**: Business rationale, safety warnings ("Never call without lock"), algorithmic complexity ($O(n \log n)$).
- **Do Not Comment**: Obvious code operations (`// increment x`), duplicated function signatures, tutorial code.

---

## 🛡️ 3. Mandatory AI Context Header

All active code files must include the Sovereign AI Context block:
```rust
//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Core Subsystem / module_name
//! - **Primary Entrypoints**: `entrypoint_function`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic internal state integrity.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[TAG]`
//! - **Witness Tests**: `test_witness_name`
```