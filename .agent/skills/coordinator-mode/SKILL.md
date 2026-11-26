---
name: coordinator-mode
description: "Advanced multi-agent orchestration with parallel workers, synthesis protocols, and coordinator lifecycle. Use when complex tasks require multiple agents working in parallel with intelligent result synthesis."
when_to_use: "When the user needs multi-agent coordination, parallel task execution, complex multi-domain work, or when /coordinate or /orchestrate is invoked. NOT for single-domain tasks."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / coordinator-mode
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Lazy prompt delegation, concurrent write collisions, missing result synthesis.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Coordinator Mode — Multi-Agent Conductor Protocol

> **Role Definition**: You are the **Conductor**. You decompose, dispatch, monitor, and synthesize. You do NOT perform raw code edits directly when coordinating multi-worker swarms; you formulate precise subtasks and integrate returned deliverables.
> **Workflow Binding**: Used directly during the [`/orchestrate`](../../workflows/orchestrate.md) workflow.

---

## 📚 Progressive Disclosure & L3 Reference Index

Consult deep **L3 Reference Guides** and verification tools on demand:
- **Worker Briefing Templates & Topologies**: [worker_briefing_templates.md](./references/worker_briefing_templates.md)
- **Deterministic Coordinator Validator**: [coordinator_validator.py](./scripts/coordinator_validator.py)

---

## 🧭 1. The 5-Stage Conductor Lifecycle

```
1. DECOMPOSE ➔ Break complex user intent into atomic, bounded subtasks.
2. DISPATCH  ➔ Launch parallel workers for read-only exploration; serialize write operations.
3. MONITOR   ➔ Track worker completion notifications reactively without polling loops.
4. SYNTHESIZE➔ Critically audit and distill worker outputs (Never blindly paste!).
5. VERIFY    ➔ Run deterministic verification gates before reporting completion to the user.
```

---

## ⚡ 2. Concurrency Safety Matrix

| Operation Category | Execution Strategy | Rationale & Guard |
|---|---|---|
| **Codebase Exploration** | Parallel / Concurrent | Read-only; workers cannot create write conflicts or state corruption. |
| **Security & Spec Audits** | Parallel / Concurrent | Multi-axis review benefits from diverse worker perspectives. |
| **File Modifications** | **Serialized / Mutex** | **Never allow two workers to edit the same file concurrently.** |
| **Database Migrations** | **Strictly Serialized** | Schema must stabilize and reconcile before route handlers execute. |

---

## 🚫 3. Non-Negotiable Worker Briefing Directives

> 🔴 **Never Delegate Understanding**: Never write prompts like *"Based on your findings, fix the code"*. Brief workers with exact target files, line numbers, and explicit constraints.

Every dispatched worker prompt **MUST** specify:
1. **Bounded Scope**: Target file paths, line ranges, and symbols.
2. **Action Category**: Explicitly declared as `READ-ONLY` exploration or `WRITE-SCOPED` implementation.
3. **Verification Criteria**: The exact test command or check (`cargo check`, `npx vitest`, `npm run adg:verify`) to prove correctness.

---

## ⚖️ 4. Anti-Patterns & Failure Modes

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **The Lazy Delegator** | Briefing a worker with "fix whatever is broken". | Provide exact file paths, line ranges, and reproduction evidence. |
| **The Fire-and-Forget** | Skipping synthesis and pasting raw worker dumps to the user. | Execute the Coordinator Synthesis Checklist; distill findings. |
| **The File Collision** | Spawning multiple concurrent write agents on the same file. | Enforce Critical Section file locking; sequence writes serially. |
| **The Polling Spammer** | Querying task status in a loop instead of waiting for event wakeup. | Rely on reactive system notifications; end turns gracefully. |
| **The Zombie Subagent** | Failing to inspect logs or recover when a subagent times out. | Inspect failure logs, refine constraints, and re-dispatch with surgical scope. |

---

## 🛠️ 5. Deterministic Verification Gate

Audit worker briefing prompts and dispatch safety:

```powershell
# Run built-in self tests
python .agent/skills/coordinator-mode/scripts/coordinator_validator.py --test

# Validate a worker briefing prompt or dispatch plan
python .agent/skills/coordinator-mode/scripts/coordinator_validator.py --file path/to/briefing_or_plan.json
```

[//]: # (Metadata: [COORDINATOR_MODE])