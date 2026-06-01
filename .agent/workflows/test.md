> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Quality:Verification**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[test]` in audit logs.
>
> ### AI Assist Note
> Verification and quality assurance for the Tadpole OS engine.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Sovereign Verification Gate. A deterministic pipeline to prove architectural integrity and behavioral parity before any state transition to production.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Quality:Verification**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[test]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/test` is active, operate as the **Sovereign Quality Guardian**. Your goal is not to "find bugs," but to **prove the absence of drift**. You are the final adversarial force; your job is to try and break the "Sovereign Seal" before the code reaches production.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /test - Sovereign Verification Gate

**Usage**: `/test [module/pillar/feature] [--deep | --parity]`

---

## 🎯 Primary Objective
To execute a deterministic series of gates that prove a feature is **Sovereign-Compliant**. This process ensures that the system's internal reality (Code) is perfectly aligned with its external intent (Blueprint/Directives).

---

## 🏗️ The Verification Pipeline

The AI must execute these gates in order. A failure at any stage is a **Sovereign Breach** and halts the pipeline.

### Gate 1: Implementation Verification (The Foundation)
Confirm that the code executes as intended without crashing.
- **Runner/Registry (Rust)**: Execute `cargo test --workspace`. Verify all logic loops and persistence layers.
- **Gateway (TS/React)**: Execute `npm run test` (Vitest). Verify UI components and API integration.
- **Sovereign Check**: If a test fails here $\rightarrow$ **Trigger [/debug](debug.md)**.

### Gate 2: Behavioral Parity (The Regression Check)
Prove that the change did not introduce "Side-Effect Drift."
- **Action**: Compare the output of the modified module against the **Behavioral Baseline** (captured during `/refactor` or `/enhance`).
- **Requirement**: Input $X$ must still produce Output $Y$ unless the change specifically intended to alter the behavior.
- **Sovereign Check**: If behavior shifted unexpectedly $\rightarrow$ **Trigger [/debug](debug.md)**.

### Gate 3: Sovereign Audit (The Integrity Gate)
Verify that the code adheres to the overarching system laws.
- **Command**: `python execution/verify_all.py . --url http://localhost:8000`.
- **Requirement**: 100% Pass rate on **P0 (Security)** and **P1 (Code Quality)**.
- **Sovereign Check**: If P0/P1 fails $\rightarrow$ **Trigger [/audit](audit.md)**.

### Gate 4: Parity Guard (The Sync Gate)
Ensure the "Source of Truth" is updated.
- **Command**: `python execution/parity_guard.py .`.
- **Requirement**: Zero drift between the updated code routes and the `API_REFERENCE.md` or core directives.
- **Sovereign Check**: If drift is detected $\rightarrow$ **Trigger [/anneal](anneal.md)**.

---

## 📊 Output Format

### ✅ Sovereign Seal Granted
```markdown
## 🛡️ Sovereign Verification: PASSED

### 🏁 Gate Summary
- **Implementation**: ✅ [All tests passed]
- **Behavioral Parity**: ✅ [Zero regressions detected]
- **Sovereign Audit**: ✅ [P0/P1 Passed]
- **Parity Guard**: ✅ [100% Sync]

**Verdict**: The state is now **VERIFIED**.
**Next Step**: Proceed to **[/deploy](deploy.md)** for production transition.
```

### ❌ Sovereign Breach Detected
```markdown
## 🛑 Sovereign Verification: BREACHED

### 🚨 Failure Point
**Gate**: [e.g., Gate 4: Parity Guard]
**Evidence**: `Drift detected in user_route.rs vs API_REFERENCE.md`

### 🛠️ Remediation Path
- [ ] **Sovereign Logic Fault**: Trigger **[/anneal](anneal.md)** to update directives.
- [ ] **Implementation Bug**: Trigger **[/debug](debug.md)** to fix the code.
- [ ] **Architectural Drift**: Trigger **[/refactor](refactor.md)** to realign with the 4-Pillars.

**Status**: Deployment blocked until the seal is restored.
```

---

## 🔄 Pipeline Integration

The `/test` command is the final filter in the Sovereign lifecycle:

`[/create]` $\rightarrow$ `[/enhance]` $\rightarrow$ `[/preview]` $\rightarrow$ **`[/test]`** $\rightarrow$ `[/deploy]`

- **Pass**: The system is "Sovereign." It can be deployed.
- **Fail**: The system is "Non-Sovereign." It must be debugged or annealed.

## 🚫 Guardrails
- **No "Quick-Pass"**: Never mark a test as passed based on a "visual check." Every gate must be backed by a script output or a logic proof.
- **Sovereign Priority**: A P0 security failure outweighs 100 passing unit tests. If P0 fails, the entire system is `BREACHED`.
- **Parity is Mandatory**: Code that works but is undocumented is considered "Drift" and is a failure.

[//]: # (Metadata: [test])

[//]: # (Metadata: [test])
