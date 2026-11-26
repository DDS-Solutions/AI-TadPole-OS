> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / team_building
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[team_building]`)

# 🏗️ Directive: Team Building (SOP-HR-01)

## 🎯 Primary Objective
Construct and initialize high-performance specialized agent clusters. This directive ensures that each "Swarm Node" has a clear identity, authorized toolsets, and strategic alignment.

---

## 🏗️ Initialization Framework

### 1. Identity Synthesis
- **Tool**: `IDENTITY.md`.
- **Action**: Assign a unique name and specialized `role` (e.g., "Sovereign Security Auditor"). 
- **Guidance**: Avoid generic names. Identities should reflect the agent's specific cognitive domain.

### 2. Capability Provisioning & `FR-11` Dual-Plane Firewall
- **Check**: `execution/` and [`directives/IDENTITY.md`](IDENTITY.md) Directive #1.
- **Action**: For supported Plane 1 agents, bulk-assign skills and workflows based on role and department. Plane 2 role templates are target configuration only; do not provision or run business agents through the generic runner while the company-scoped runtime is pending.
- **Security (`FR-11`)**: This directive states a required boundary; role declaration and tool assignment do not enforce it. The generic runner rejects only identities whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not activate the guard. Until company-scoped authorization and storage exist, use synthetic inputs and stop at preview.

### 3. Connection Mapping
- **Goal**: Establish "Trust Clusters."
- **Action**: Set the `recruitment_depth` and `budget_usd` limits to prevent runaway swarm growth.

---

## 🛠️ Execution SOP

### 1. Node Activation
- **Endpoint**: `POST /v1/agents`.
- **Scope**: Use this endpoint for Plane 1 agents only until the company-scoped Plane 2 runtime is implemented.
- **Verification**: Check for the "Verified Hub" icon in the dashboard grid.

### 2. Strategic Rituals
- **Check**: Invocate `neural_handoff.md` to ensure the new team understands the North Star objectives of the current sprint.

---

## 📊 Evaluation
Review "Team Cohesion" via the Swarm Pulse visualizer. Are sub-agents being recruited efficiently? Are they all aligned to the primary mission?
