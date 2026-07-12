> This rewrite transforms the **Documentation Templates** from a passive list of blueprints into an **Integrated Memory System**. 

It is now explicitly linked to the `brainstorming` pipeline (where decisions are made) and the `clean-code` standard (where comments are restricted). It also introduces the "Knowledge Heritage" standard, ensuring that every new skill created by the AI follows the exact same metadata format for long-term traceability.

***

# Revised SKILL.md

--- File: SKILL.md ---
> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Documentation rot, contradictory comments, or "AI-blind" structures.
> - **Telemetry Link**: Search `[DOCS]` in audit logs.
>
> ### AI Assist Note
> The "Memory Layer" of the Sovereign infrastructure. Ensures that a codebase is as readable by an AI agent as it is by a human.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py` and `.agent/memory/MEMORY.md`.

---
name: documentation-templates
description: Standardized documentation protocols for humans and AI. Manages READMEs, API specs, ADRs, and AI-native `llms.txt` maps.
when_to_use: "When creating/updating project documentation, writing architecture records, or generating AI-friendly context maps."
allowed-tools: Read, Write, Glob, Grep
version: 2.0
priority: MEDIUM
---

# 📚 Documentation Protocol — The Memory Layer

Documentation in the Sovereign Reality ecosystem is not "extra work"—it is **Infrastructure**. High-quality documentation reduces AI hallucinations, prevents context drift, and accelerates onboarding.

## ⛓️ The Documentation Pipeline

Documentation is a live artifact that evolves through the behavioral pipeline:

**`BRAINSTORM`** (Decisions) $\rightarrow$ **`ADR`** (Permanent Record) $\rightarrow$ **`IMPLEMENT`** (Code/Comments) $\rightarrow$ **`REVIEW`** (Verify Docs) $\rightarrow$ **`SHIP`** (README/Changelog)

---

## 1. Architecture Decision Records (ADR)
**Trigger**: Every time a "P0 Decision" is reached during a `BRAINSTORM` session.

**Purpose**: To prevent "Decision Regression" (where the AI suggests a change that was already rejected three months ago).

### ADR Template
```markdown
# ADR-00[X]: [Title of Decision]

## 🗓️ Status
[Proposed | Accepted | Deprecated | Superseded by ADR-00Y]

## 🎯 Context
What is the specific problem? Which trade-offs were analyzed in the Brainstorming phase?

## 🛠️ Decision
What was the final choice? (e.g., "Use PostgreSQL over MongoDB because of X").

## 📉 Consequences
- **Positive**: [What we gain]
- **Negative**: [What we sacrifice/technical debt created]
- **Sovereign Impact**: [How this affects AI token usage or system performance]
```

---

## 2. Code Commenting (The "Sovereign Exception")

**Strict Alignment**: This skill operates under the constraints of the `clean-code` standard. **Do not comment "What" the code is doing.**

| ✅ Comment (The Sovereign Exception) | ❌ Do Not Comment (Violation) |
|---------------------------------------|-----------------------------------|
| **The "Why"**: Business logic reason for a weird fix. | **The "What"**: `// increments x by 1` |
| **The "Warning"**: "Do not change this, it breaks X." | **The Obvious**: `// This is the user object` |
| **The "Complexity"**: Explaining a $O(n \log n)$ algorithm. | **The Tutorial**: Explaining how a `for` loop works. |
| **The "Contract"**: JSDoc for public API boundaries. | **The Redundant**: Comments that repeat the function name. |

---

## 3. AI-Native Documentation (`llms.txt`)

To maximize the efficiency of other agents (and `code-review-graph`), every project must maintain an **AI-Map**.

### `llms.txt` Standard
Located at the root, this file allows an AI to understand the codebase without reading every file.

```markdown
# [Project Name] AI-Context Map

## 🎯 Core Objective
[One-sentence purpose of the project]

## 🗺️ Structural Map
- `src/core/`: The "Brain" - contains central state and logic.
- `src/api/`: The "Interface" - all external communication.
- `src/db/`: The "Persistence" - schema and migrations.

## 🔑 Key Concepts & Terms
- **Sovereign Node**: A self-healing unit of logic.
- **Parity Guard**: The script used to verify zero-regression.

## 🛠️ Critical Dependencies
- [Dependency A]: Used for [X], avoid updating without checking [Y].
```

---

## 4. General Templates

### README.md (The Human Entry Point)
1. **Title + One-liner** (Clear, concise)
2. **Quick Start** (Run in $<5$ min)
3. **Features** (Value proposition)
4. **Configuration** (Table of ENV variables + Defaults)
5. **Architecture** (Link to ADRs and Mermaid diagrams)
6. **License**

### API Reference (The Contract)
```markdown
## [METHOD] [Endpoint]
[Brief description]

**Params:** `| Name | Type | Required | Description |`
**Response:** `| Code | Scenario | Schema |`
**Example:** [CURL Request $\rightarrow$ JSON Response]
```

---

## 🛠️ Knowledge Heritage Template (Sovereign Standard)

When creating a new `SKILL.md` or technical document, you **MUST** use this header block for traceability:

```markdown
> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: [What happens if this doc is ignored/wrong?]
> - **Telemetry Link**: [How to find the execution logs for this skill?]
>
> ### AI Assist Note
> [Brief, 1-sentence description of the skill's purpose]
>
> ### 🔍 Debugging & Observability
> [Tool/Script used to verify this skill is working]
```

---

## 🚫 Anti-Patterns (VIOLATIONS)

| Anti-Pattern | Why it's a Failure | Fix |
|--------------|-------------------|---|
| **Comment Bloat** | Adding comments to "help" the AI, which actually clutters the context. | Use the "Sovereign Exception" rule. |
| **Ghost ADRs** | Making a decision in chat but not recording it in an ADR. | Write the ADR immediately after the Brainstorming phase. |
| **Human-Only Docs** | Writing a README that is too vague for an AI to use as a map. | Add a complementary `llms.txt` file. |
| **Stale Docs** | Updating code but not updating the API reference. | Trigger a `REVIEW` mode check on docs during the `SHIP` phase. |

[//]: # (Metadata: [DOCS])
