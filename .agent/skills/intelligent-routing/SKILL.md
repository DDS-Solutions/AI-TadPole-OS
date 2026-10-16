---
name: intelligent-routing
description: Automatic agent selection and intelligent task routing. Analyzes user requests and automatically selects the best specialist agent(s) without requiring explicit user mentions.
when_to_use: "Always active. Automatically selects the best specialist agent for each user request without explicit user mentions."
version: 1.1.0
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / intelligent-routing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Intelligent Agent Routing Protocol

> **Purpose**: Automatically analyze incoming user requests and route them to the most appropriate specialist agent(s) without requiring explicit user mentions.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core routing rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/agent_roster.md`](./references/agent_roster.md) | Complete keyword-to-agent mapping, multi-domain resolution | Complex task decomposition & agent roster |

---

## 1. Request Analysis & Complexity Tiers

Analyze every incoming request across 3 dimensions:
```
USER PROMPT ➔ Classify Intent ➔ Assess Complexity ➔ Select Specialist(s)
```

| Complexity Tier | Characteristics | Action / Protocol |
|---|---|---|
| **SIMPLE** | Single file, single domain (e.g. *"Fix the button styling"*) | Auto-apply specialist directly with transparent indicator. |
| **MODERATE** | 2–3 files, single layer (e.g. *"Add a new REST endpoint"*) | Auto-apply relevant specialist(s) sequentially. |
| **COMPLEX** | Multi-layer, architectural, new app (e.g. *"Add login with dark mode UI"*) | **Orchestrator Protocol**: Propose implementation plan; wait for confirmation. |

---

## 2. Transparent Specialist Indication

When applying a specialist agent's domain knowledge, indicate it cleanly to the user:
```markdown
🤖 **Applying knowledge of `@nexus-engineer` + `@tadpole-backend-specialist`...**

[Proceed directly with the specialized response or action]
```

---

## 3. Core Operational Directives

1. **Silent Analysis**: Perform domain matching instantly without meta-chatter (*"I am analyzing your prompt..."*).
2. **Explicit Mention Override**: If the user explicitly mentions `@agent-name`, override automated routing.
3. **Socratic Gate Integration**: Routing does NOT bypass the Socratic gate. If requirements are ambiguous, clarify trade-offs first before generating code.