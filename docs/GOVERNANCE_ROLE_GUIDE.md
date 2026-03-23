# 👔 Swarm Governance: Role & Blueprint Guide

## Overview

Tadpole OS uses a **Blueprint-First** architecture for agent management. Roles are not just labels; they are templates that define an agent's technical skills (Skills) and operational protocols (Workflows).

## Role Types

### 1. System Roles (Red)
*Examples: CEO (Agent of Nine), Security Auditor, Emergency Ops.*
- **Governance**: High-level oversight and destructive skill.
- **CEO Sovereignty**: The Agent of Nine (ID 1) is the only node with system-level permission to use the `issue_alpha_directive` tool for neural handoffs.
- **Success Auditor (ID 99)**: A specialized system node used for **Mission Analysis**. It has read-only access to mission logs and is optimized for prescriptive optimization tasks.
- **Concurrent Recruitment**: In the **Tadpole OS** architecture, system roles are optimized for **Parallel Swarming**, allowing them to recruit multiple specialists simultaneously without sequential latency.
- **Typical Models**: Claude 3.5 Sonnet, GPT-4o.

#### [NEW] Alpha Strategic Command
Alpha Nodes are the tactical anchors of their mission clusters. Beyond task delegation, they are responsible for:
- **Optimization Gatekeeping**: Alpha Nodes with active proposals are marked by a pulsing **"Brain" icon** in the UI. Clicking this reveals the "Neural Oversight" dashboard.
- **Strategic Authorization**: The operator (Overlord) interacts with the Alpha's oversight UI to authorize cluster-wide reconfigurations.
- **Intent Alignment**: Ensuring that specialists remain aligned with the mission's `Strategic Intent` during complex swarming cycles.

### 2. Departmental Roles (Blue/Emerald)
*Examples: Backend Dev, Growth Hacker, UX Designer.*
- **Governance**: Specialized toolsets for domain-specific tasks.
- **Typical Models**: Llama 3.3, Gemini 1.5 Pro.

---

## 🛠️ Managing Blueprint Definitions

### 1. The Reactive Role Store
All roles are managed via the `useRoleStore` (Zustand). Any change to a role definition is reactively propagated across the entire Swarm Intelligence View.

### 2. Creating New Roles ("Promote to Role")
Instead of defining roles in config files, you can build them through **Configuration-by-Example**:
1. Open any agent's **Neural Node Configuration**.
2. Manually toggle the **Skills** and **Workflows** you want the role to have. (Note: The available list is dynamically loaded from the **Skills & Workflows Manager**.)
3. Scroll to the bottom and click **"Promote to Role"**.
4. Enter a name for the new role (e.g., "DevOps Lead").
5. The system captures that exact configuration and saves it as a new selectable **Blueprint**.

### 3. Blueprint Persistence
Role Blueprints are persisted in LocalStorage under the key `tadpole-roles-storage`. This ensures your organizational templates survive browser refreshes while remaining local to your neural sector.

---

## 🛡️ Best Practices

- **Swarm Protocol Compliance**: Role-based agents are bound by the engine's linear pathing protocol. They will automatically block circular recursion (A -> B -> A) regardless of their specific blueprint instructions to ensure swarm stability.
- **Minimal Privilege**: Assign only the skills required for a role to reduce tokens-per-mission and improve security.
- **Concurrent Capacity**: Roles designed for heavy recruitment should be assigned to providers with high RPM/TPM tiers to leverage **Parallel Swarming** effectively.
- **Tiered Overrides**: Remember that an agent's individual configuration (Secondary/Tertiary slots) can extend a role's base blueprint without modifying the template itself.
- **Blueprint Naming**: Use clear, protocol-based names (e.g., "Financial Auditor v2") to ensure consistency across large swarms.
