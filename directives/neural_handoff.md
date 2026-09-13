> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / neural_handoff
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[neural_handoff]`)

# 🛰️ Directive: Neural Handoff (SOP-SWARM-01)

## 🎯 Primary Objective
Ensure 100% strategic alignment and context continuity during recursive agent recruitment. This directive governs how "Strategic Intent" is injected from a parent agent to a specialist sub-agent.

---

## 🧠 Handoff Protocol

### 1. Intent Extraction
- **Action**: Before using the `recruit_specialist` tool, the parent agent must synthesize its "Current Strategic Goal" into a 2-paragraph summary.
- **Requirement**: Focus on "Why this task matters" (Strategic) vs "How to do it" (Tactical).

### 2. Neural Lineage Schema — Mandatory Handoff Format *(IDENTITY.md Directive #2)*
- **Mechanism**: The parent's intent is injected into the sub-agent's system prompt under `--- STRATEGIC INTENT FROM OVERSEER ---`.
- **Constraint**: Avoid "Context Bloat." Prune redundant filesystem and log data to prioritize the mission objective.
- **Schema**: Every handoff file MUST include all 5 fields below — no partial handoffs are permitted:

  | Field | Description |
  | :--- | :--- |
  | `[Task-Slug]` | Unique identifier for the active task |
  | `[Current State]` | Exact execution phase at time of handoff |
  | `[Verified Facts]` | Confirmed truths established by the Aletheia Protocol |
  | `[Pending Blockers]` | Unresolved dependencies or open contradictions |
  | `[Next Actionable Step]` | Single, concrete first action for the receiving node |

  > The `{task-slug}.md` is the **Sovereign State Unit** — the primary recovery key if the session restarts. Write to it before any `/compact`.

### 3. Verification
- **Check**: The sub-agent must acknowledge the strategic intent in its first reasoning cycle (`working_memory` entry 1).

---

## 🚦 Governance Gates
- **Depth Guard**: Recruitment depth must not exceed 5 (enforced by `AppState`).
- **Lineage Guard**: The `swarm_lineage` hook must block any circular recruitment (e.g., A -> B -> A).

## 📝 Reporting Protocol
Parent agents must log "Handoff Efficiency" in the mission summary. Did the sub-agent require clarification? If yes, refine the Intent Extraction logic in the next turn.