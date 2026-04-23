> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: parallel-agents
description: Multi-agent orchestration patterns. Use when multiple independent tasks can run with different domain expertise.
allowed-tools: Read, Glob, Grep
---

# Parallel Agents

**Coordinate, don't just broadcast.**

## Use Cases
- **Good**: Complex multi-domain tasks, comprehensive reviews (Sec + Perf + Test).
- **Bad**: Simple single-domain tasks.

## Invocation Patterns
1.  **Single**: "Use `security-auditor` to review auth".
2.  **Chain**: "Explore -> Backend -> Test".
3.  **Context**: "Frontend finding -> Test generation".

## Available Agents
- `orchestrator`: Coordination.
- `security-auditor`: Security/Auth.
- `penetration-tester`: Red team/Exploit.
- `backend-specialist`: API/Node.
- `frontend-specialist`: React/UI.
- `test-engineer`: Testing/TDD.
- `devops-engineer`: CI/CD/Docker.
- `database-architect`: SQL/Schema.
- `mobile-developer`: RN/Flutter.
- `debugger`: Fixes.
- `performance-optimizer`: Speed.

---

## 🧠 Aletheia Reasoning Protocol (Orchestration)

### 1. Generator (Team Assembly)
*   **Roles**: "Who is the 'Reviewer'? Who is the 'Doer'?".
*   **Dependencies**: "Backend needs API spec first?".
*   **Synthesis**: "How to merge conflicting advice?".

### 2. Verifier (Loop Check)
*   **Deadlock**: "Agent A waiting for B, B waiting for A?".
*   **Noise**: "50 pages for a 1-line fix?".
*   **Cost**: "Worth the token cost?".

### 3. Reviser (Streamlining)
*   **Handoff**: "Pass file path, not content.".
*   **Focus**: "Tell Security Agent to ONLY look at auth.".

---

## 🛡️ Security & Safety Protocol (Multi-Agent)

1.  **Isolation**: Context separation where possible.
2.  **Hallucination**: Verify "Agent A said X" is true.
3.  **Command Authority**: Only Orchestrator/User authorizes destruction.
4.  **Context Limit**: Monitor token usage.

## Synthesis Protocol
**One unified report, not separate outputs.**
1.  **Summary**: What happened?
2.  **Contributions**: "Sec found X, Backend found Y".
3.  **Actions**: Critical fixes first.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
