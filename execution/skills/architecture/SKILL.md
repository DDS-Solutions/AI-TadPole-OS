> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Infrastructure:Execution**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: architecture
description: Architectural decision-making framework. Requirements analysis, trade-off evaluation, ADR documentation.
allowed-tools: Read, Glob, Grep
---

# Architecture Decision Framework

**"Simplicity is the ultimate sophistication."**

- Start simple.
- Add complexity ONLY when proven necessary.
- Removing complexity is harder than adding it.

---

## 🧠 Aletheia Reasoning Protocol (System Design)

**Complexity is a budget. Spend it wisely.**

### 1. Generator (Pattern Divergence)
*   **Architecture Styles**: "Monolith vs Microservices vs Serverless".
*   **Data Flows**: "Event-driven vs Request-Response".
*   **State Location**: "Client-side vs Server-side state".

### 2. Verifier (Trade-off Analysis)
*   **CAP Theorem**: "Consistency vs Availability - what can we lose?".
*   **Cost Estimate**: "Is this cloud bill going to surprise us?".
*   **Team Fit**: "Does the team know how to debug distributed traces?".

### 3. Reviser (Decision Record)
*   **Write ADR**: Document *why* we chose X over Y.
*   **Simplify**: "Merge these two services into one module".

---

## 🛡️ Security & Safety Protocol (Architecture)

1.  **Secure by Design**: Security is an architectural layer.
2.  **Attack Surface**: Minimize entry points.
3.  **Data Sovereignty**: Design data flows that respect where data lives.
4.  **Fail Safe**: Architecture must handle component failure gracefully.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
