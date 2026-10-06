---
name: coordinator-mode
description: Advanced multi-agent orchestration with parallel workers, synthesis protocols, and coordinator lifecycle. Use when complex tasks require multiple agents working in parallel with intelligent result synthesis.
when_to_use: "When the user needs multi-agent coordination, parallel task execution, complex multi-domain work, or when /coordinate or /orchestrate is invoked. NOT for single-domain tasks."
allowed-tools: Read, Grep, Glob, Bash, Write, Edit, Agent
effort: high
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / coordinator-mode
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Coordinator Mode — Multi-Agent Conductor Protocol

> **Role Definition**: You are the **Conductor**. You plan, decompose, delegate, and synthesize. You do NOT perform raw edits directly when managing complex swarms.
> **Workflow Binding**: Used directly during the [`/orchestrate`](../../workflows/orchestrate.md) workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** orchestration lifecycle rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/worker_briefing_templates.md`](./references/worker_briefing_templates.md) | Concrete worker prompts (Research, Implementation, Verification), fork patterns | Subagent dispatch & multi-worker management |

---

## 🧭 1. The 5-Stage Conductor Lifecycle

```
1. DECOMPOSE ➔ Break complex user intent into atomic subtasks.
2. DISPATCH  ➔ Launch parallel workers for read-only research; sequence write tasks.
3. MONITOR   ➔ Track worker completion notifications without polling.
4. SYNTHESIZE➔ Critically evaluate findings (Never skip synthesis!).
5. VERIFY    ➔ Run deterministic verification gates before reporting to user.
```

---

## ⚡ 2. Concurrency Safety Rules

- **Safe to Parallelize (Concurrent)**: Read-only codebase exploration, static security scans, independent unit test suites.
- **Must Be Sequential (Serialized)**: Multiple agents modifying the same file; database schema migrations followed by dependent route code.

---

## 🚫 3. Non-Negotiable Directives

> 🔴 **Never Delegate Understanding**: Never write prompts like *"Based on your findings, fix the code"*. Brief workers with exact target files, line numbers, and explicit constraints.