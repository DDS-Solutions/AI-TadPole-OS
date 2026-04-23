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
name: intelligent-routing
description: Automatic agent selection and intelligent task routing.
allowed-tools: Read, Glob, Grep
version: 1.0.0
---

# Intelligent Agent Routing

**AI Project Manager: Analyze -> Select -> Invoke.**

## Core Logic
1.  **Analyze**: Classify User Request (Keywords, Domain, Complexity).
2.  **Select**:
    *   **Simple (1 Domain)**: Direct Agent (e.g., "Fix CSS" -> `frontend-specialist`).
    *   **Moderate (2 Domains)**: Sequential Agents (e.g., "Add API & DB" -> `backend` + `db`).
    *   **Complex**: `orchestrator` (e.g., "Build new app").
3.  **Invoke**: Silent analysis, explicit announcement ("🤖 Applying knowledge of...").

## Domain Matrix
- **Security**: `security-auditor` (auth, jwt, hash).
- **Frontend**: `frontend-specialist` (css, react, ui).
- **Backend**: `backend-specialist` (api, express, node).
- **Mobile**: `mobile-developer` (ios, android, flutter).
- **Database**: `database-architect` (sql, schema).
- **DevOps**: `devops-engineer` (docker, ci/cd).
- **Test**: `test-engineer` (jest, coverage).
- **Debug**: `debugger` (error, fix, crash).

---

## 🧠 Aletheia Reasoning Protocol (Meta-Cognition)

### 1. Generator (Capability Mapping)
*   **Intent**: "Code (Implement) vs Concepts (Plan)?".
*   **Overlap**: "Firewalls? Security or Network agent? -> Context determines.".
*   **Mode**: "Debug mode? -> Use Debugger.".

### 2. Verifier (Routing Check)
*   **Overkill**: "System Architect for a typo?".
*   **Underkill**: "Junior Dev for DB redesign?".
*   **Loop**: "Avoid agent ping-pong.".

### 3. Reviser (Optimization)
*   **Directness**: "Skip Orchestrator for simple tasks.".
*   **Clarity**: "Tell user *why* agent was chosen.".

---

## 🛡️ Security & Safety Protocol (Routing)

1.  **Impersonation**: Verify agent definition source.
2.  **Context Leak**: Limit history sharing to need-to-know.
3.  **Injection**: Prevent prompt injection in routing logic.
4.  **Loops**: Prevent infinite self-delegation.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
