---
name: handoff
description: Compacts current session state into a clean handoff document for a fresh agent session.
when_to_use: "Use when ending a context-heavy session, transferring state to a new agent, or preparing for multi-session handoff."
allowed-tools: Read, Glob, Grep, Write
disable-model-invocation: true
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SKILL]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for compacting session state into inter-agent handoff documents.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

# Session Handoff Compaction

> Source: mattpocock/skills (handoff)

Write a structured handoff document summarizing the current conversation state so a fresh agent session can resume work immediately with zero context token bloat.

Save file to: `.tmp/handoffs/handoff-<timestamp>.md`.

---

## Handoff Document Template

```markdown
# Session Handoff: <Task Title>

## 1. Goal & Context
<Brief summary of the primary objective and current status>

## 2. Completed Work & Key Decisions
- <Decision / implementation completed>
- <File modified / added>

## 3. Pending Work & Active Blockers
- [ ] <Next step for incoming agent>
- [ ] <Known blocker or unresolved question>

## 4. Suggested Next Skills
- `@[skills/...]`
- `@[skills/...]`
```
