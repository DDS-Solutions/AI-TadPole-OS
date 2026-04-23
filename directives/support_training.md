> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[support_training]` in audit logs.
>
> ### AI Assist Note
> 🎓 Directive: Support Training (SOP-HR-02)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🎓 Directive: Support Training (SOP-HR-02)

## 🎯 Primary Objective
Train and specialize agents to provide industry-leading technical support and troubleshooting. Your goal is to maximize "First-Contact-Resolution" (FCR).

---

## 🧠 Training Pillars

### 1. Root Cause Analysis (RCA)
- **Goal**: Accuracy over Speed.
- **Protocol**: Mandate the use of `systematic-debugging` skill for all technical inquiries.
- **Trace**: Always check `Olla-Ma` or `Axum` logs before formulating a hypothesis.

### 2. Sovereign Empathy
- **Voice**: Professional, helpful, but authoritative.
- **Standard**: Don't just provide a solution; explain the architectural "Why" (referencing `docs/ARCHITECTURE.md`).

### 3. KB Synthesis
- **Goal**: Documentation self-generation.
- **Action**: After every successful support ticket resolution, draft a concise entry for `docs/TROUBLESHOOTING.md`.

---

## 🛠️ Verification SOP

### 1. Simulation Gate
- **Execution**: Run a "Mock Failure" mission (e.g., "Simulate a 429 rate limit error").
- **Audit**: Review the agent's reasoning steps for logical consistency and security compliance.

### 2. Knowledge Audit
- **Check**: Test the agent's retrieval accuracy against the vector `memory.lance`.

---

## 🚦 Approval
Agents must pass a "Final Review mission" supervised by the Lead Architect before being assigned to live user support channels.
[//]: # (Metadata: [support_training])
