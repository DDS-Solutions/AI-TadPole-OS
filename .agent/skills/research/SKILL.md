---
name: research
description: Executes isolated research and investigation subagents on throwaway git branches to prevent cluttering main working state.
when_to_use: "Use when investigating third-party APIs, reading external docs, or building throwaway research spikes."
allowed-tools: Read, Glob, Grep, Bash, Write
disable-model-invocation: true
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / research
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

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