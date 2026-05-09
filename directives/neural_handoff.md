> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[neural_handoff]` in audit logs.
>
> ### AI Assist Note
> 🛰️ Directive: Neural Handoff (SOP-SWARM-01)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🛰️ Directive: Neural Handoff (SOP-SWARM-01)

## 🎯 Primary Objective
Ensure 100% strategic alignment and context continuity during recursive agent recruitment. This directive governs how "Strategic Intent" is injected from a parent agent to a specialist sub-agent.

---

## 🧠 Handoff Protocol

### 1. Intent Extraction
- **Action**: Before using the `recruit_specialist` tool, the parent agent must synthesize its "Current Strategic Goal" into a 2-paragraph summary.
- **Requirement**: Focus on "Why this task matters" (Strategic) vs "How to do it" (Tactical).

### 2. Context Injection (SEC-04)
- **Mechanism**: The parent's intent is injected into the sub-agent's system prompt under `--- STRATEGIC INTENT FROM OVERSEER ---`.
- **Constraint**: Avoid "Context Bloat." Prune redundant filesystem and log data to prioritize the mission objective.

### 3. Verification
- **Check**: The sub-agent must acknowledge the strategic intent in its first reasoning cycle (`working_memory` entry 1).

---

## 🚦 Governance Gates
- **Depth Guard**: Recruitment depth must not exceed 5 (enforced by `AppState`).
- **Lineage Guard**: The `swarm_lineage` hook must block any circular recruitment (e.g., A -> B -> A).

## 📝 Reporting Protocol
Parent agents must log "Handoff Efficiency" in the mission summary. Did the sub-agent require clarification? If yes, refine the Intent Extraction logic in the next turn.
[//]: # (Metadata: [neural_handoff])
