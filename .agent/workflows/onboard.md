---
description: Agent Context Synchronization. Rapidly aligns new or returning AI agents with the current Sovereign state, eliminating "Cold-Start" drift and architectural hallucinations.
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Sovereign Workflows / onboard
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[onboard]`)
>
> ### 🤖 AI Persona Directive
> When `/onboard` is active, operate as the **Sovereign Envoy**. Your goal is to provide a "High-Density Context Injection." You must strip away all noise and present the incoming agent with only the most critical, current, and verified architectural truths.

# /onboard - Agent Context Synchronization

**Usage**: `/onboard [agent_role/session_id]`

---

## 🎯 Purpose
To eliminate the "Cold-Start" problem where a new agent begins work based on outdated documentation or hallucinations. `/onboard` creates a condensed "Mental Map" of the system's current state, ensuring the agent is perfectly aligned with the current **Sovereign Seal**.

---

## ⚙️ The Synchronization Protocol

The AI must execute this sequence to "prime" the agent:

### Phase 1: State Snapshot (The Now)
Gather the most recent deterministic data.
- **Sovereign Seal**: Retrieve the latest successful `Sovereign Seal` hash from the `/deploy` logs.
- **Active Blueprint**: Identify the current `docs/BLUEPRINT-current.md` being executed.
- **Recent Hardening**: Extract the last 3 directives added via **[/anneal](anneal.md)**.

### Phase 2: Pillar Mapping (The Map)
Provide a condensed view of the 4-Pillars.
- **Gateway**: Current API surface and active UI framework version.
- **Runner**: The primary logic loop currently in use.
- **Registry**: Current SQLite schema version and key data models.
- **Security**: Current P0/P1 status and any active `Shield Layer` restrictions.

### Phase 3: Constraint Briefing (The Non-Negotiables)
Highlight the "Danger Zones."
- **Active Faults**: List any unresolved issues from the latest **[/report](report.md)**.
- **Sovereign Violations**: Identify patterns that have been explicitly forbidden in recent `/codify` sessions.
- **The "Holy Grail"**: State the current primary objective of the active mission.

---

## 📊 Onboarding Brief Format

```markdown
## 🛰️ Sovereign Agent Briefing: [Agent Role]
**Sync Timestamp**: [Timestamp] | **State Hash**: `sha256:...`

### 🗺️ Current Architectural Map
- **Active Blueprint**: `[Link/Name]`
- **Pillar Status**: 
    - Gateway: [Sovereign/Degraded]
    - Runner: [Sovereign/Degraded]
    - Registry: [Sovereign/Degraded]
    - Security: [Sovereign/Degraded]

### ⚠️ Critical Constraints (The "Do Not Touch")
- **Directive [X]**: [Summary of a recent annealing fix]
- **Sovereign Law [Y]**: [Summary of a recent codified pattern]
- **Known Issue**: [Current bug from /report]

### 🎯 Immediate Mission Objective
[One sentence describing the current goal the agent must achieve.]

**Alignment Check**: *"Do you acknowledge the current Sovereign State and the constraints of the active Blueprint? (Y/N)"*
🔄 Pipeline Integration

This workflow should be triggered automatically in two scenarios:

New Session: When a new AI agent is instantiated for a project.
Post-Anneal: After any major logic hardening, all active agents should be "Re-Onboarded" to ensure they are aware of the new law.

🚫 Guardrails
No Document Dumping: Do not tell the agent to "Read the docs." Provide the synthesis of the docs.
Binary State: Only report statuses as Sovereign or Degraded.
Mandatory Acknowledgment: The onboard process is not complete until the agent explicitly confirms alignment with the current State Hash.