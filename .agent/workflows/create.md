---
description: Orchestrates the end-to-end creation of new applications. Integrates discovery, brainstorming, blueprinting, and multi-agent execution.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[create]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[create]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/create` is triggered, operate as the **Sovereign Orchestrator**. Your goal is not just to generate code, but to manage a pipeline of specialized agents to ensure the output is architecturally sound, secure, and maintainable.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /create - Sovereign Application Orchestration

**Usage**: `/create [project_description/concept]`

---

## 🎯 Objective
To move a concept from a vague user request to a fully deployed, audited, and verified application by orchestrating a sequence of specialized cognitive workflows.

---

## ⚙️ The Orchestration Pipeline

The `/create` command must follow this deterministic sequence:

### Phase 1: Discovery & Vector Analysis
- **Analyze Request**: Deconstruct the user's prompt into a **Problem Vector**.
- **Gap Analysis**: If the request is ambiguous, use the `conversation-manager` to resolve:
    - Core functionality (MVP features).
    - Target audience/User personas.
    - Critical constraints (Performance, Security, Budget).

### Phase 2: Architectural Ideation (The Brainstorm)
- **Trigger Workflow**: Immediately transition to the **[/brainstorm](brainstorm.md)** workflow.
- **Objective**: Evaluate 3 divergent architectural paths.
- **Output**: A selected "Sovereign-Lead" path that optimizes for the project's specific goals.

### Phase 3: The Sovereign Blueprint
Before execution, generate a **Blueprint Document** for user approval. This is the "Contract of Truth."
- **Tech Stack**: Default to **Rust (Backend)**, **TypeScript/React/Tailwind (Frontend)** unless specified otherwise.
- **Schema Design**: Preliminary data models and relationship maps.
- **Route Map**: Defined API endpoints and frontend page hierarchy.
- **File Tree**: A complete projected directory structure.

### Phase 4: Multi-Agent Execution
Once the Blueprint is approved, orchestrate the following specialists:
1. **`database-architect`**: Implements the schema and migrations.
2. **`tadpole-backend-specialist`**: Develops the API, logic layers, and Rust routes. Pair with `customer-backend-specialist` for customer-facing behavior.
3. **`frontend-specialist`**: Constructs the UI/UX using the defined design system.
4. **`frontend-specialist` + `tadpole-backend-specialist`**: Hook the frontend to the backend and verify connectivity.

### Phase 5: Verification & Hardening
- **Automated Preview**: Execute `auto_preview.py` to launch the application.
- **Parity Check**: Run `python execution/parity_guard.py .` to ensure the generated code matches the Blueprint.
- **Audit**: Run a baseline **[/audit](audit.md)** to ensure no P0 security vulnerabilities were introduced during generation.

---

## 📊 Execution Flow Summary

`User Request` $\rightarrow$ `Vector Analysis` $\rightarrow$ `Brainstorm` $\rightarrow$ `Blueprint` $\rightarrow$ `Multi-Agent Build` $\rightarrow$ `Audit/Parity` $\rightarrow$ `Deployment`

---

## 💡 Usage Examples

- `/create sovereign-wallet` $\rightarrow$ (Triggers Discovery $\rightarrow$ Brainstorm for security $\rightarrow$ Blueprint for Rust/TS $\rightarrow$ Execution)
- `/create real-time-analytics-dashboard` $\rightarrow$ (Triggers Discovery $\rightarrow$ Brainstorm for data throughput $\rightarrow$ Blueprint $\rightarrow$ Execution)

## 🚫 Guardrails
- **No Stealth Building**: Never start coding before the **Blueprint** is approved.
- **Sovereign Standard**: All created apps must adhere to the project's global architectural standards defined in `@docs ARCHITECTURE:Core`.
- **Determinism**: If a specialist agent deviates from the Blueprint, the Orchestrator must force a correction.

[//]: # (Metadata: [create])
