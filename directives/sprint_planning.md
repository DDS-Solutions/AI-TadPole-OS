> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[sprint_planning]` in audit logs.
>
> ### AI Assist Note
> 🗓️ Directive: Sprint Planning (SOP-OPS-03)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🗓️ Directive: Sprint Planning (SOP-OPS-03)

## 🎯 Primary Objective
Govern the 2-week development and orchestration cycle of the Tadpole OS cluster. This directive ensures that resource allocation is optimized for "Highest Value" outcomes.

---

## 🏗️ Planning Phases

### 1. Velocity Analysis
- **Tool**: `engagement_report.md`.
- **Action**: Review the previous sprint's mission success rate and token efficiency.

### 2. Goal Synthesis (The North Star)
- **Focus**: Identify 3 "Must-Haves" for the cycle (e.g., "Add local Whisper support," "Optimize LanceDB index").
- **Constraint**: Goals must align with the `feature_roadmap.md`.

### 3. Resource Allocation
- **Mechanism**: Assign specific specialist agents to each goal.
- **Budget**: Set `budget_usd` limits per agent-sprint.

---

## 🛠️ Execution SOP

### 1. Backlog Grooming
- **Action**: Use `user_feedback_analysis.md` to identify top 5 bugs or feature requests.

### 2. Timeline Mapping
- **Output**: `SPRINT_PLAN_[ID].md` in the current workspace. Include clear "Definition of Done" (DoD) for each task.

---

## 📊 Status Verification
Progress is tracked daily via the `ops_review.md` pulse. Any goal at risk of slippage must trigger a "Strategic Pivot" oversight request.
[//]: # (Metadata: [sprint_planning])
