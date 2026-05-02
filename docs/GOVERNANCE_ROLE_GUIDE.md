> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 👔 Swarm Governance: Role & Blueprint Guide
**Intelligence Level**: High (ECC Optimized)
**Source of Truth**: `server-rs/src/state/mod.rs` (GovernanceHub), `server-rs/src/agent/types.rs` (RoleBlueprint)
**Last Hardened**: 2026-04-20
**Standard Compliance**: ECC-GOV (Enhanced Contextual Clarity - Governance Standards)

> [!IMPORTANT]
> **AI Assist Note (Governance Logic)**:
> This guide defines the safety boundaries of the Tadpole Engine.
> - **Oversight Gate**: Mandatory for tools requesting `ShellExecute`, `DeleteFile`, or `BudgetExpenditure`. Enforced via `requires_oversight = true`.
> - **Privacy Shield**: When active, external cloud resolver blocks take priority over all agent configurations.
> - **Identity**: Agent 1 (Sovereign) occupies the highest governance tier. Alpha Agents (ID 2 and ID "alpha") manage tactical mission clusters.
> - **Missions**: All missions are recorded in the Merkle Audit Trail.

---

## 👔 Governance Enforcement Flow

```mermaid
graph TD
    Agent["Agent Runner"]
    Gate["Governance Hub"]
    Audit["Merkle Audit Trail"]
    User["Human Overlord"]

    Agent -- "Request Tool Call" --> Gate
    Gate -- "Check Policy (Safety Gate)" --> Gate
    alt Is High Risk (Shell/Delete/Spend)
        Gate -- "requires_oversight = true" --> User
        User -- "Approve" --> Audit
        User -- "Reject" --> Agent
    end
    Audit -- "Commit Signed Hash" --> Audit
    Audit -- "Allow Execution" --> Agent
```

---

> **Status**: Stable  
> **Version**: 1.3.1  
> **Last Updated**: 2026-04-20  
> **Classification**: Sovereign  

---

## Overview

Tadpole OS uses a **Blueprint-First** architecture for agent management. Roles are not just labels; they are persistent templates that define an agent's technical skills (Skills) and operational protocols (Workflows).

## Role Types

### 1. System Roles (Red)
*Examples: CEO (Sovereign), Security Auditor, Emergency Ops.*
- **Governance**: High-level oversight and destructive skill.
- **CEO Sovereignty**: The Sovereign (ID 1) is the only node with system-level permission to use the `issue_alpha_directive` tool for neural handoffs.
- **Success Auditor (ID 99)**: A specialized system node used for **Mission Analysis**. It has read-only access to mission logs and is optimized for prescriptive optimization tasks.
- **Concurrent Recruitment**: System roles are optimized for **Parallel Swarming**, allowing them to recruit multiple specialists simultaneously without sequential latency.
- **Typical Models**: Claude 3.5 Sonnet, GPT-4o.
- **Swarm Insight**: The CEO role has unique telepresence capabilities within the **Swarm_Visualizer**, allowing for interactive viewport focus and mission log extraction.

### 2. Alpha Strategic Command (Emerald)
Alpha Nodes (ID 2, "alpha") are the tactical anchors of their mission clusters. Beyond task delegation, they are responsible for:
- **Optimization Gatekeeping**: Alpha Nodes with active proposals are marked by a pulsing **"Brain" icon** in the UI.
- **Strategic Authorization**: The operator (Overlord) interacts with the Alpha's oversight UI to authorize cluster-wide reconfigurations.
- **Goal Alignment**: Ensuring that specialists remain aligned with the mission's **Primary Goal** during complex swarming cycles.

### 3. Departmental Roles (Blue)
*Examples: Backend Dev, Growth Hacker, UX Designer.*
- **Governance**: Specialized toolsets for domain-specific tasks.
- **Requires Oversight Toggle**: Departmental roles can be configured with mandatory oversight. This is recommended for junior nodes or those accessing sensitive production data.
- **Typical Models**: Llama 3.3, Gemini 1.5 Pro.

---

## 🛠️ Managing Blueprint Definitions

### 1. The Reactive Role Store
All roles are managed via the `use_role_store` (Zustand). Any change to a role definition is reactively propagated across the entire Swarm Intelligence View.

### 2. Creating New Roles ("Promote to Role")
Instead of defining roles manually, you can build them through **Configuration-by-Example**:
1. Open any agent's **Neural Node Configuration**.
2. Manually toggle the **Skills** and **Workflows** you want the role to have.
3. Click **"Promote to Role"**.
4. The system captures that configuration and saves it as a new selectable **Blueprint**.

### 3. Blueprint Persistence
Role Blueprints are persisted in the backend **SQLite** database (`tadpole.db`) under the `role_blueprints` table. This ensures your organizational templates are sovereign, shareable, and resilient across different deployment sectors.

#### Schema Manifest (ECC-DB-01)
| Field | Type | Description |
| :--- | :--- | :--- |
| `id` | TEXT (PK) | Unique blueprint slug (e.g., `researcher-v1`) |
| `name` | TEXT | Friendly display name |
| `department`| TEXT | Target organizational unit |
| `skills` | JSON | Stringified array of skill slugs |
| `workflows` | JSON | Stringified array of workflow slugs |
| `requires_oversight` | BOOL | Default safety gate status |

---

## 🛡️ Best Practices

- **Swarm Protocol Compliance**: Role-based agents are bound by the engine's linear pathing protocol. They will automatically block circular recursion (A -> B -> A).
- **Minimal Privilege**: Assign only the skills required for a role to reduce tokens-per-mission and improve security.
- **Concurrent Capacity**: Roles designed for heavy recruitment should be leverage **Parallel Swarming** effectively by using tier-one models.
- **Tiered Overrides**: An agent's individual configuration can extend a role's base blueprint without modifying the template itself.
- **Mission Quota Alignment**: Ensure that role-level budgets are consistent with the **Mission-Level Quotas** assigned to their clusters.

[//]: # (Metadata: [GOVERNANCE_ROLE_GUIDE])

[//]: # (Metadata: [GOVERNANCE_ROLE_GUIDE])
