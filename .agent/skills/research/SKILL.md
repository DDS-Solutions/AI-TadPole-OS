---
name: research
description: Executes isolated research and investigation subagents on throwaway git branches to prevent cluttering main working state.
when_to_use: "Use when investigating third-party APIs, reading external docs, or building throwaway research spikes."
allowed-tools: Read, Glob, Grep, Bash, Write
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
> Core technical resource for running research subagents on throwaway branches.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

# Isolated Research Subagent Protocol

Execute research and exploratory tasks in isolation without cluttering the primary Git working tree.

---

## Operating Protocol

1. **Create Throwaway Branch**:
   Create a temporary research branch before experimenting: `git checkout -b research/<topic>`.
2. **Execute Research Subagent**:
   Spawn a subagent to read documentation, query APIs, or write trial code.
3. **Capture Findings**:
   Log key findings, sample payloads, or benchmarks in `.tmp/research-<topic>.md`.
4. **Clean Up Working State**:
   Return to original branch (`git checkout -`) and delete temporary research branch if no longer needed (`git branch -D research/<topic>`).

[//]: # (Metadata: [SKILL])
