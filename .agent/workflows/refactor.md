> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[refactor]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Structural Hardening & Architectural Refinement. Focuses on reducing system entropy and decoupling the 4-Pillars without altering external behavior.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[refactor]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/refactor` is active, operate as the **Sovereign Systems Architect**. Your goal is **Structural Hardening**. You are not adding value through new features, but through the *elimination of fragility*. Your success is measured by the reduction of complexity and the maintenance of absolute behavioral parity.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /refactor - Sovereign Structural Hardening

**Usage**: `/refactor [module/pillar/symbol]`

---

## 🎯 Primary Objective
To transition the codebase from a "working" state to a "hardened" state. This involves removing architectural entropy, decoupling the **4-Pillars**, and liquidating technical debt without altering the system's observable behavior.

---

## ⚙️ Hardening Protocol

The AI must execute this deterministic sequence to ensure no regressions are introduced:

### Phase 1: Boundary Mapping (The Blast Radius)
Before modifying any code, map the dependencies of the target area.
- **Graph Analysis**: Run `npm run graph:blast -- --path [target_file]` to identify all callers and callees.
- **Coupling Audit**: Identify any "leaks" where the **Gateway** (UI/API) is directly touching the **Registry** (DB) without passing through the **Runner** (Logic).
- **Baseline Capture**: Record the current output of the affected modules to ensure Behavioral Parity.

### Phase 2: The Hardening Strategy
Define the type of structural improvement:
- **Hub Decoupling**: Breaking circular dependencies or tight coupling between the 4-Pillars.
- **Logic Consolidation**: Moving fragmented business rules into a single, deterministic "Runner" service.
- **Debt Liquidation**: Removing "ghost" logic, legacy placeholders, and non-Sovereign "magic" dependencies.
- **Type Hardening**: Converting `any` or loose types into strict, deterministic TypeScript interfaces.

### Phase 3: Surgical Execution
Implement the changes following the **Zero-Feature Mandate**:
- **Strictly Structural**: If a change alters how a feature works or adds a new capability, it is an `/enhance` task, not a `/refactor` task. **STOP and redirect.**
- **Atomic Refactoring**: Apply changes in the smallest possible increments.
- **Sovereign Standard**: Align the code with the core patterns defined in `@docs ARCHITECTURE:Core`.

### Phase 4: Behavioral Parity Verification
Prove that the refactor did not break the system:
- **Input/Output Test**: Run the captured baseline inputs through the refactored code. The output must be $\equiv$ (identical) to the pre-refactor state.
- **Sovereign Audit**: Trigger **[/audit](audit.md)** to ensure no P0 security regressions were introduced.
- **Parity Guard**: Run `python execution/parity_guard.py .` to ensure documentation still matches the new structure.

---

## 📊 Hardening Report Format

```markdown
## 🏗️ Structural Hardening Report: [Module/Pillar]

### 📉 Complexity Reduction
- **Before**: [Description of fragility/coupling]
- **After**: [Description of hardened structure]
- **Blast Radius**: [Number of symbols touched] $\rightarrow$ [Number of symbols decoupled]

### 🧹 Debt Liquidation
- **Removed**: [Legacy pattern/Ghost logic/Magic dependency]
- **Replaced With**: [Sovereign Pattern]

### 🛡️ Parity Verification
- **Behavioral Parity**: ✅ VERIFIED (Input $\rightarrow$ Output is identical)
- **Sovereign Audit**: ✅ STATUS: OPERATIONAL
- **Parity Guard**: ✅ Zero Drift Detected

### 🚀 Hardening Gain
- **Maintenance Effort**: [Reduced/Improved]
- **Fragility**: [Eliminated/Mitigated]
```

---

## 💡 Usage Examples

- `/refactor the user-auth route` $\rightarrow$ (Decouples Auth logic from the API Gateway)
- `/refactor registry-schema.rs` $\rightarrow$ (Hardens the SQLite data models for better type safety)
- `/refactor the state-management-store` $\rightarrow$ (Removes redundant Zustand listeners to improve performance)

---

## 🚫 Guardrails

- **Zero-Feature Mandate**: Any attempt to "improve" a feature during a refactor is a protocol violation.
- **No Blind Decoupling**: Never move code without first running a `graph:blast` analysis.
- **Parity First**: If Behavioral Parity cannot be proven, the refactor is considered a failure and must be rolled back.
- **Anti-Entropy**: The final state must be simpler, more modular, and more deterministic than the initial state.

[//]: # (Metadata: [refactor])

[//]: # (Metadata: [refactor])
