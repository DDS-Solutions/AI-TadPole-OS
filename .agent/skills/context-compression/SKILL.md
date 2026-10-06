---
name: context-compression
description: Manage and compress conversation context in long sessions. Detect when context is growing large, summarize completed work phases, archive old findings while preserving key decisions. Prevents context degradation.
when_to_use: "When a session has 20+ turns, when context feels repetitive, when the agent is losing track of earlier work, or when the user says 'summarize what we've done'. NOT for short sessions."
allowed-tools: Read, Write, Grep
effort: low
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / context-compression
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Context Compression & Session Management

> **Purpose**: Prevent context degradation across long multi-turn sessions (20+ turns) by distilling completed work into high-signal checkpoints.
> **Workflow Binding**: Used directly during [`/orchestrate`](../../workflows/orchestrate.md) and [`/onboard`](../../workflows/onboard.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** compression rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/checkpoint_and_handoff_templates.md`](./references/checkpoint_and_handoff_templates.md) | Level 1/2/3 checkpoint formats and multi-session handoff files | Session checkpointing & multi-turn handoffs |

---

## 🚦 1. Compression Triggers & Levels

| Level | Scope | When to Trigger | Result / Benefit |
|---|---|---|---|
| **Level 1: Micro-Compact** | Large Tool Output | Grep/Find outputs > 100 lines | Summarize key files & match counts |
| **Level 2: Phase Summary** | Completed Phase | Transitioning from Research to Code | Preserve decisions & discard raw reads |
| **Level 3: Session Checkpoint**| Full Session | 20+ turns or user asks for summary | Emit structured progress checkpoint |

---

## 🧭 2. Preservation vs Discard Directives

- **Preserve**: Architecture decisions, file paths/line numbers touched, test verification results.
- **Discard**: Raw file dumps, verbose tool invocation syntax, transient intermediate logs.