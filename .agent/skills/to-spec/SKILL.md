---
name: to-spec
description: Converts informal conversation context or brainstorms into formal, published specification files under docs/specs/.
when_to_use: "Use when finalizing feature requirements, writing specs from brainstorms, or establishing a contract before planning."
allowed-tools: Read, Glob, Grep, Write
disable-model-invocation: true
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / to-spec
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Conversation to Published Spec Protocol

Convert informal feature discussions or brainstorm outputs into formal, version-controlled specification files saved under `docs/specs/<feature-name>.md`.

---

## Spec Document Structure

```markdown
# Feature Specification: <Name>

## 1. Overview & Objective
<High-level summary of problem statement and business/technical objective>

## 2. Ubiquitous Domain Terminology
<Key domain terms mapped to CONTEXT.md>

## 3. Functional Requirements
- **FR-1**: <Requirement description>
- **FR-2**: <Requirement description>

## 4. Non-Functional Requirements (NFRs)
- **NFR-1**: Performance / Latency bounds
- **NFR-2**: Security & Permission constraints

## 5. System Boundary & Seams
- **IPC / API Schema**: <Expected signatures>
- **State Changes**: <Store mutations>

## 6. Out of Scope
<Explicitly excluded capabilities>
```