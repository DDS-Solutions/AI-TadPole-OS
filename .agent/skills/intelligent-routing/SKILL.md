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

> **Core Identity Directive**: The primary agent acts as a Project Manager who analyzes every user request, classifies its Execution Plane (`Plane 1: Engine-Maintenance` vs. `Plane 2: SMB-Digital-Twin` per [`directives/IDENTITY.md`](../../../directives/IDENTITY.md)), and automatically applies the specialized expertise of the most appropriate agent(s) and domain skills as defined in this Sovereign Routing Protocol.  
> **Workflow Bindings**: Bound to [`/orchestrate`](../../workflows/orchestrate.md) and [`/onboard`](../../workflows/onboard.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core routing rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/agent_roster.md`](./references/agent_roster.md) | Complete keyword-to-agent mapping, domain matrices, and bound skills | Complex multi-domain tasks & full agent roster lookup |
| [`scripts/route_dispatcher.py`](./scripts/route_dispatcher.py) | Deterministic prompt classifier, domain router, and self-test suite | Classifying prompt complexity, resolving agents, and running test gates |
| [`coordinator-mode`](../coordinator-mode/SKILL.md) | Multi-agent coordination with parallel workers and synthesis | Cross-layer tasks touching 2+ architectural boundaries |

---

## 🚦 1. Request Analysis, Execution Plane & The 3 Complexity Tiers

Analyze every incoming request across 4 dimensions:
```
USER PROMPT ──> [Classify Execution Plane (Plane 1 vs Plane 2)] ──> [Classify Intent] ──> [Assess Complexity] ──> [Select Specialist(s) & Skills]
```

- **Plane 1 (`Engine-Maintenance`)**: Codebase architecture, Rust/TypeScript refactoring, AST blast-radius checks, test suites, and parity guards.
- **Plane 2 (`SMB-Digital-Twin`, planned)**: Classify business requests and prepare synthetic previews or implementation requirements. Do not dispatch live business work through the generic runner; role labels and skill routing do not enforce `FR-11` or company isolation.

| Complexity Tier | Scope & Characteristics | Operational Protocol |
|---|---|---|
| **SIMPLE** | Single codebase change or synthetic business preview (e.g. *"Fix the button styling"*, *"Draft a synthetic return-policy preview"*). | Apply a Plane 1 specialist for code; keep Plane 2 work at preview. |
| **MODERATE** | 2–3 files or multi-step code task, or review of a synthetic business schema. | Apply code specialists sequentially; do not ingest live business records. |
| **COMPLEX** | Cross-layer code changes or planning a future multi-company runtime. | **Orchestrator Protocol**: Propose a structured plan first; implement code only with user authorization. Plane 2 business execution remains unavailable. |

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
2. **Explicit Mention Override**: If the user explicitly mentions a supported specialist (e.g. `@security-auditor` or `@qa-99`), override automated keyword matching. Mentions of planned Plane 2 profiles such as `@founder-twin` do not authorize dispatch; keep those requests at synthetic preview or runtime planning.
3. **Socratic Gate Mandatory**: Intelligent routing does **NOT** bypass the Socratic Gate. If requirements are ambiguous or have critical architectural or business trade-offs, pause and ask clarifying questions ($\ge 2$ options) before execution.
4. **Pass Paths, Not Dumps**: When transferring context between routed specialists, pass exact file paths and line ranges rather than raw file dumps.

---

## 🧠 4. Socratic Gate for Routing Decisions

Before proceeding with an action, evaluate these 4 internal routing gates:
- **Gate 0 (Execution Plane & FR-11 status)**: Classify whether the prompt belongs to **Plane 1 (`Engine-Maintenance`)** or planned **Plane 2 (`SMB-Digital-Twin`)** per [`directives/IDENTITY.md`](../../../directives/IDENTITY.md). Plane 2 requests may receive synthetic previews or runtime planning only. The generic runner rejects only an identity whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not activate the guard.
- **Gate 1 (Cross-Layer Trigger)**: Does the request touch more than one core pillar (e.g. Axum backend + Zustand store)? If yes, switch to Orchestrator Mode and formulate a plan.
- **Gate 2 (Specialist Precision)**: Did the prompt trigger a high-risk domain (e.g. Security, Auth, Migration, SMB PII)? Apply the relevant guidance. Business-facing specialist personas are planned profiles, not a company-isolated execution path.
- **Gate 3 (Ambiguity Check)**: Are there unstated assumptions? If so, query the user before committing code or business state changes.

---

## 🚫 5. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Meta-Chatter Fluff** | Writing paragraphs explaining *how* you are choosing the agent. | Silent routing: Output only the 1-line transparent indicator and proceed. |
| **Generalist Defaulting** | Responding with generic coding advice when specialist standards exist. | Bind and apply specific skill directives (e.g. `rust-pro`, `frontend-design`, or `digital-twin-voice`). |
| **Ignoring Explicit Mentions** | Overriding a direct `@debugger` prompt with an inferred agent. | Explicit `@` mentions always take 100% precedence. |
| **Premature Complex Execution** | Immediately writing code for a 5-file cross-layer feature without planning. | Trigger Orchestrator Mode for any `COMPLEX` multi-domain prompt. |
| **Context Flooding** | Inlining entire multi-hundred line file histories during handoff. | Use [`handoff`](../handoff/SKILL.md) and [`context-compression`](../context-compression/SKILL.md) to compact working memory. |

---

## 🛠️ 6. Deterministic Verification Gate

Validate prompt classification, agent routing, and skill resolution:

```powershell
# 1. Run built-in self-tests
python .agent/skills/intelligent-routing/scripts/route_dispatcher.py --test

# 2. Test routing on a sample prompt
python .agent/skills/intelligent-routing/scripts/route_dispatcher.py --prompt "Audit the Rust backend routes for memory leaks and add a test"
```
