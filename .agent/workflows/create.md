---
description: Orchestrates the end-to-end creation of new applications. Integrates discovery, brainstorming, blueprinting, and multi-agent execution.
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Sovereign Workflows / create
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[create]`)
>
> ### 🤖 AI Persona Directive
> When `/create` is triggered, operate as the **Sovereign Orchestrator**. Your goal is not just to generate code, but to manage a pipeline of specialized agents to ensure the output is architecturally sound, secure, and maintainable.

# /create - Sovereign Application Orchestration

**Usage**: `/create [project_description/concept]`

---

## 🎯 Objective
To move a concept from a vague user request to a fully deployed, audited, and verified application by orchestrating a sequence of specialized cognitive workflows.

> 💡 **Bound Skills & References**:
> - Use [`app-builder`](../skills/app-builder/SKILL.md) as the main application building orchestrator across full-stack layers.
> - Use [`brainstorming`](../skills/brainstorming/SKILL.md) and [`plan-writing`](../skills/plan-writing/SKILL.md) for architectural blueprinting.
> - Use [`ui-ux-pro-max`](ui-ux-pro-max.md) for generating design systems and aesthetic interfaces.

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
1. **[`database-architect`](../agents/database-architect.md)**: Implements the schema and migrations.
2. **[`tadpole-backend-specialist`](../agents/tadpole-backend-specialist.md)**: Develops the API, logic layers, and Rust routes. Pair with [`customer-backend-specialist`](../agents/customer-backend-specialist.md) for customer-facing behavior.
3. **[`frontend-specialist`](../agents/frontend-specialist.md)**: Constructs the UI/UX using the defined design system.
4. **[`frontend-specialist`](../agents/frontend-specialist.md) + [`tadpole-backend-specialist`](../agents/tadpole-backend-specialist.md)**: Hook the frontend to the backend and verify connectivity.

### Phase 5: Verification & Hardening
- **Automated Preview**: Execute `python execution/auto_preview.py start` to launch the application.
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

---

## 🕸️ Graph Intelligence
Before initializing code scaffolds or registering new routes, check existing symbols and endpoints:
```powershell
npm run graph:lookup -- --name Router
npm run graph:file -- --path src/App.tsx
```