---
name: parallel-agents
description: Multi-agent orchestration patterns. Use when multiple independent tasks can run with different domain expertise or when comprehensive analysis requires multiple perspectives.
when_to_use: "When a task requires 2+ specialist agents, comprehensive multi-domain analysis, or coordinated parallel execution. Use with the /orchestrate workflow. NOT for single-domain tasks where one agent suffices."
allowed-tools: Read, Glob, Grep
effort: medium
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / parallel-agents
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Native Parallel Agent Orchestration

> **Purpose**: Coordinate multiple specialized agents for multi-domain initiatives and comprehensive code analysis.
> **Workflow Binding**: Used directly during the [`/orchestrate`](../../workflows/orchestrate.md) workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** orchestration rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/orchestration_patterns.md`](./references/orchestration_patterns.md) | Full 26-agent directory, multi-agent fan-out trees, context handoff schemas | Multi-agent coordination & domain routing |

---

## 🧭 1. Orchestration Decision Flow

```
User Task Analysis:
├── Single Domain / Local Fix ➔ Direct single specialist (No orchestration overhead)
└── Multi-Domain Initiative    ➔ Parallel Agent Fan-Out (Explore ➔ Implement ➔ Verify)
```

---

## ⚡ 2. Core Execution Topologies

1. **Comprehensive Analysis**: Read-only parallel exploration (`security-auditor` + `performance-optimizer`) followed by a central synthesis turn.
2. **Feature Implementation**: Sequential execution where backend schema changes precede frontend UI binding.
3. **Verification Fan-Out**: Independent parallel validation (`test-engineer` runs Vitest while `security-auditor` validates policies).

---

## 🛠️ 3. Execution Directives

- **Clean Handoffs**: Pass exact file paths, line ranges, and concise findings between agent turns.
- **Synthesize First**: Always review worker findings before committing multi-file modifications.