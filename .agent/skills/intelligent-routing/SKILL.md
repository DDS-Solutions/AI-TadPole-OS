---
name: intelligent-routing
description: "Automatic agent selection and intelligent task routing. Analyzes user requests, classifies intent and complexity, and automatically selects the best specialist agent(s) and skills without requiring explicit user mentions."
when_to_use: "Always active. Automatically selects the best specialist agent and domain skills for each user request without requiring explicit user mentions."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / intelligent-routing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Routing ambiguity, agent mismatch, or uncoordinated multi-domain execution.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Intelligent Agent Routing Protocol

> **Core Identity Directive**: The primary agent acts as a Project Manager who analyzes every user request and automatically applies the specialized expertise of the most appropriate agent(s) and domain skills as defined in this Sovereign Routing Protocol.  
> **Workflow Bindings**: Bound to [`/orchestrate`](file:///D:/TadpoleOS-Dev/.agent/workflows/orchestrate.md) and [`/onboard`](file:///D:/TadpoleOS-Dev/.agent/workflows/onboard.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core routing rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/agent_roster.md`](file:///D:/TadpoleOS-Dev/.agent/skills/intelligent-routing/references/agent_roster.md) | Complete keyword-to-agent mapping, domain matrices, and bound skills | Complex multi-domain tasks & full agent roster lookup |
| [`scripts/route_dispatcher.py`](file:///D:/TadpoleOS-Dev/.agent/skills/intelligent-routing/scripts/route_dispatcher.py) | Deterministic prompt classifier, domain router, and self-test suite | Classifying prompt complexity, resolving agents, and running test gates |
| [`coordinator-mode`](file:///D:/TadpoleOS-Dev/.agent/skills/coordinator-mode/SKILL.md) | Multi-agent coordination with parallel workers and synthesis | Cross-layer tasks touching 2+ architectural boundaries |

---

## 🚦 1. Request Analysis & The 3 Complexity Tiers

Analyze every incoming request across 3 dimensions:
```
USER PROMPT ──> [Classify Intent] ──> [Assess Complexity] ──> [Select Specialist(s) & Skills]
```

| Complexity Tier | Scope & Characteristics | Operational Protocol |
|---|---|---|
| **SIMPLE** | Single file, single domain (e.g. *"Fix the button styling"*, *"Add a field to struct"*). | Apply specialist directly with transparent indicator. Execute immediately. |
| **MODERATE** | 2–3 files, single architectural layer (e.g. *"Add a new REST route and unit test"*). | Apply primary and co-agent sequentially with transparent indicator. |
| **COMPLEX** | Cross-layer, multi-domain, full-stack, or schema transitions (e.g. *"Add user auth with Rust route, SQLite migration, and React UI"*). | **Orchestrator Protocol**: Propose structured implementation plan first; await user confirmation before modifying code. |

---

## 🤖 2. Transparent Specialist Indication

When applying a specialist agent's domain persona and expertise, announce it cleanly at the beginning of the turn:

```markdown
🤖 **Applying knowledge of `@nexus-engineer` + `@tadpole-backend-specialist`...**

[Proceed directly with the specialized response or tool actions]
```

---

## ⚡ 3. Core Operational Directives

1. **Zero Meta-Chatter**: Perform domain classification instantly and silently. Never output preliminary chatter (*"Analyzing your request...", "Determining the best agent..."*).
2. **Explicit Mention Override**: If the user explicitly mentions an agent (e.g. `@security-auditor` or `@qa-99`), override automated keyword matching and route directly to the specified specialist.
3. **Socratic Gate Mandatory**: Intelligent routing does **NOT** bypass the Socratic Gate. If requirements are ambiguous or have critical architectural trade-offs, pause and ask clarifying questions ($\ge 2$ options) before code generation.
4. **Pass Paths, Not Dumps**: When transferring context between routed specialists, pass exact file paths and line ranges rather than raw file dumps.

---

## 🧠 4. Socratic Gate for Routing Decisions

Before proceeding with an action, evaluate these 3 internal routing gates:
- **Gate 1 (Cross-Layer Trigger)**: Does the request touch more than one core pillar (e.g. Axum backend + Zustand store)? If yes, switch to Orchestrator Mode and formulate a plan.
- **Gate 2 (Specialist Precision)**: Did the prompt trigger a high-risk domain (e.g. Security, Auth, Migration)? Ensure the specific specialist persona (`security-auditor`, `database-architect`) is actively applied.
- **Gate 3 (Ambiguity Check)**: Are there unstated assumptions? If so, query the user before committing code changes.

---

## 🚫 5. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Meta-Chatter Fluff** | Writing paragraphs explaining *how* you are choosing the agent. | Silent routing: Output only the 1-line transparent indicator and proceed. |
| **Generalist Defaulting** | Responding with generic coding advice when specialist standards exist. | Bind and apply specific skill directives (e.g. `rust-pro` or `frontend-design`). |
| **Ignoring Explicit Mentions** | Overriding a direct `@debugger` prompt with an inferred agent. | Explicit `@` mentions always take 100% precedence. |
| **Premature Complex Execution** | Immediately writing code for a 5-file cross-layer feature without planning. | Trigger Orchestrator Mode for any `COMPLEX` multi-domain prompt. |
| **Context Flooding** | Inlining entire multi-hundred line file histories during handoff. | Use [`handoff`](file:///D:/TadpoleOS-Dev/.agent/skills/handoff/SKILL.md) and [`context-compression`](file:///D:/TadpoleOS-Dev/.agent/skills/context-compression/SKILL.md) to compact working memory. |

---

## 🛠️ 6. Deterministic Verification Gate

Validate prompt classification, agent routing, and skill resolution:

```powershell
# 1. Run built-in self-tests
python .agent/skills/intelligent-routing/scripts/route_dispatcher.py --test

# 2. Test routing on a sample prompt
python .agent/skills/intelligent-routing/scripts/route_dispatcher.py --prompt "Audit the Rust backend routes for memory leaks and add a test"
```