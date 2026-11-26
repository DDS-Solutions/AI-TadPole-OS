---
name: parallel-agents
description: "Multi-agent orchestration patterns. Use when multiple independent tasks can run with different domain expertise or when comprehensive analysis requires multiple perspectives."
when_to_use: "When a task requires 2+ specialist agents, comprehensive multi-domain analysis, or coordinated parallel execution. Use with the /orchestrate workflow. NOT for single-domain tasks where one agent suffices."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / parallel-agents
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Cyclic worker dependencies, payload bloat across agent handoffs, or uncoordinated file clobbering.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Native Parallel Agent Orchestration

> **Purpose**: Coordinate multiple specialized agents for multi-domain initiatives, comprehensive system audits, and fan-out/fan-in architectural workflows.  
> **Workflow Bindings**: Integrated directly into [`/orchestrate`](../../workflows/orchestrate.md) and [`/plan`](../../workflows/plan.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/orchestration_patterns.md`](./references/orchestration_patterns.md) | Full 26-agent directory, multi-agent fan-out trees, context handoff schemas | Multi-agent coordination & domain routing |
| [`scripts/orchestration_guard.py`](./scripts/orchestration_guard.py) | Automated briefing auditor and dependency cycle detector | Validating worker task packets and dependency topologies |

---

## 🧭 1. Orchestration Decision Flow

```
User Task Analysis:
├── Single Domain / Local Fix ➔ Direct single specialist (No orchestration overhead)
└── Multi-Domain Initiative    ➔ Parallel Agent Fan-Out (Explore ➔ Implement ➔ Verify)
```

---

## ⚡ 2. Core Execution Topologies

1. **Comprehensive Analysis (Fan-Out/Fan-In)**: Read-only parallel exploration (`security-auditor` + `performance-optimizer`) followed by a central synthesis turn.
2. **Feature Implementation (Pipelined)**: Sequential execution where database migrations and Rust backend services precede React frontend UI wiring.
3. **Verification Fan-Out (Parallel Gates)**: Independent parallel validation (`test-engineer` runs Vitest while `lint-runner` validates types and invariants).

---

## 🧠 3. Socratic Gate for Multi-Agent Spawning

Before spawning parallel subagents, verify these 3 gates:
- **Gate 1 (Domain Isolation)**: Do the subagents operate on strictly disjoint files or read-only resources to avoid race conditions and file clobbering?
- **Gate 2 (Briefing Quality)**: Does each worker briefing explicitly state the exact target files, line ranges, and exit condition?
- **Gate 3 (Token Economy)**: Is worker briefing payload kept lean ($\le 1,500$ words) without dumping raw conversation transcripts?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Cyclic Dependencies** | Agent A waits on Agent B, which waits on Agent A. | Model tasks as a Directed Acyclic Graph (DAG) verified by `orchestration_guard.py`. |
| **Overlapping File Writes** | Two concurrent workers write to the same file simultaneously. | Strict separation: exactly one worker writes to a given file. |
| **Context Flooding** | Passing 10,000-word chat dumps to subagents. | Extract concise briefs ($\le 1,500$ words) with target paths. |
| **Premature Fan-Out** | Spawning 5 agents for a 2-line single-file typo fix. | Use single-agent direct execution when one specialist suffices. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate worker briefing packets and detect DAG cycles:

```powershell
# 1. Run built-in self-tests
python .agent/skills/parallel-agents/scripts/orchestration_guard.py --test

# 2. Check an agent briefing packet
python .agent/skills/parallel-agents/scripts/orchestration_guard.py --check-briefing "Target: src/components/Map.tsx Task: Memoize render loop"
```