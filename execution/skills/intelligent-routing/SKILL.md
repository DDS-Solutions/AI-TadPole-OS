> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / SKILL
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

---
name: intelligent-routing
description: Automatic agent selection and intelligent task routing across the Dual-Plane Sovereign Hierarchy (Plane 1: Engine-Maintenance | Plane 2: SMB-Digital-Twin).
allowed-tools: Read, Glob, Grep
version: 1.1.0
---

# Intelligent Agent Routing

**AI Project Manager: Classify Plane -> Analyze -> Select -> Invoke.**

## Core Logic
1.  **Classify Execution Plane (`IDENTITY.md`)**:
    *   **Plane 1 (`Engine-Maintenance`)**: Codebase architecture, Rust/TS development, AST blast-radius checks, and test suites.
    *   **Plane 2 (`SMB-Digital-Twin`)**: Company-agnostic SMB workflows, founder voice calibration, data ingestion, customer support, and invoicing (enforces `FR-11` Permission Firewall — zero code-edit or shell privileges).
2.  **Analyze**: Classify User Request (Keywords, Domain, Complexity).
3.  **Select**:
    *   **Simple (1 Domain)**: Direct Agent (e.g., "Fix CSS" -> `tadpole-frontend-specialist`, "Check return policy" -> `founder-twin`).
    *   **Moderate (2 Domains)**: Sequential Agents (e.g., "Add API & DB" -> `tadpole-backend-specialist` + `database-architect`).
    *   **Complex**: `tadpole-alpha` / `orchestrator` (e.g., "Build new full-stack feature" or "Onboard new SMB workspace").
4.  **Invoke**: Silent analysis, explicit announcement ("🤖 Applying knowledge of...").

## Domain Matrix
- **Architecture**: `nexus-engineer` (architecture, blast radius, audit, refactor).
- **Security**: `security-auditor` (auth, jwt, hash, vault, pii).
- **Frontend**: `tadpole-frontend-specialist` (css, react, tailwind, ui).
- **Backend**: `tadpole-backend-specialist` (rust, axum, tokio, server-rs, tauri).
- **Mobile**: `mobile-developer` (ios, android, compose, swiftui).
- **Database**: `database-architect` (sqlite, sqlx, lancedb, schema).
- **SMB Business & Digital Twin**: `founder-twin` / `support-analyst` / `bookkeeper` (smb, business, digital twin, founder voice, invoice, quickbooks, catalog).
- **Test & QA**: `qa-99` / `test-engineer` (vitest, invariant, witness, coverage).
- **Debug**: `debugger` (error, fix, crash, panic).

---

## 🧠 Aletheia Reasoning Protocol (Meta-Cognition)

### 1. Generator (Skill Mapping)
*   **Plane & Intent**: "Plane 1 Codebase Maintenance vs Plane 2 SMB Digital Twin?".
*   **Overlap**: "Firewalls? Security or Network agent? -> Context determines.".
*   **Mode**: "Debug mode? -> Use Debugger.".

### 2. Verifier (Routing Check)
*   **Overkill**: "System Architect for a typo?".
*   **Underkill**: "Junior Dev for DB redesign?".
*   **FR-11 Firewall**: "Did a Plane 2 business prompt request code/shell tools? Block and sandbox.".
*   **Loop**: "Avoid agent ping-pong.".

### 3. Reviser (Optimization)
*   **Directness**: "Skip Orchestrator for simple tasks.".
*   **Clarity**: "Tell user *why* agent was chosen.".

---

## 🛡️ Security & Safety Protocol (Routing)

1.  **Impersonation**: Verify agent definition source.
2.  **Context Leak**: Limit history sharing to need-to-know. Company workspace isolation (`NFR-1`) is planned and not enforced by the current runner; use synthetic business inputs and stop at preview.
3.  **FR-11 Gate Status**: The runner rejects only agents whose stored department exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive). Other labels do not activate the gate; company-scoped authorization is pending.
4.  **Injection**: Prevent prompt injection in routing logic.
5.  **Loops**: Prevent infinite self-delegation.
