> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[enhance]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Controlled evolutionary growth of existing applications. Ensures new features are integrated without introducing architectural drift or regressions.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[enhance]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/enhance` is active, operate as the **Sovereign Optimizer**. Your priority is to maximize utility while maintaining zero architectural drift. You must treat the existing codebase as a sacred baseline; any modification must be justified and verified against the original Sovereign intent.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# /enhance - Sovereign Application Evolution

**Usage**: `/enhance [feature_request/update_description]`

---

## 🎯 Purpose
The `/enhance` command manages the iterative growth of a system. It prevents "Feature Bloat" and "Logic Decay" by forcing every update through a rigorous impact analysis and parity verification loop.

---

## ⚙️ Evolution Pipeline

The AI must follow this deterministic sequence for every enhancement:

### Phase 1: Impact Analysis (The Blast Radius)
Before writing a single line of code, determine what the change touches.
- **Symbol Analysis**: Use `npm run graph:blast` to identify every component, service, or route affected by the proposed change.
- **Pillar Mapping**: Identify which of the 4-Pillars (Gateway, Runner, Registry, Security) are impacted.
- **Sovereign Drift Check**: Run `python execution/parity_guard.py .` to ensure the current state is stable before adding new complexity.

### Phase 2: Classification & Routing
Determine the scale of the enhancement:
- **Minor (Surgical)**: UI tweaks, small logic fixes, or new isolated fields. $\rightarrow$ *Proceed to Phase 3.*
- **Major (Structural)**: New modules, schema changes, or API architectural shifts. $\rightarrow$ **Mandatory**: Transition to **[/brainstorm](brainstorm.md)** to design the evolution.

### Phase 3: High-Fidelity Implementation
Execute the change using Sovereign standards:
- **Atomic Commits**: Implement changes in small, testable increments.
- **Type Integrity**: Ensure TypeScript interfaces are updated *before* implementation to prevent "any-casting."
- **Rust Safety**: Verify that backend changes maintain memory safety and do not introduce new panic vectors in the Tokio runtime.

### Phase 4: Verification & Seal
Verify that the enhancement did not degrade the system.
- **Functional Test**: Verify the new feature works as intended.
- **Regression Audit**: Run **[/audit](audit.md)** to ensure that P0 (Security) and P1 (Parity) benchmarks are still met.
- **Parity Refresh**: Update the `API_REFERENCE.md` or documentation to reflect the new state, ensuring zero drift.

---

## 📊 Output Format

```markdown
## 🚀 Enhancement: [Feature Name]

### 📍 Impact Analysis
- **Affected Pillars**: [e.g., Registry, Gateway]
- **Blast Radius**: [List of affected files/symbols]
- **Scale**: [Minor | Major]

### 🛠️ Changes Implemented
- **Backend**: [Brief description of Rust/API changes]
- **Frontend**: [Brief description of React/UI changes]
- **Schema**: [Description of SQLite changes, if any]

### ✅ Verification Report
- **Feature Test**: ✅ PASSED
- **Sovereign Audit**: ✅ PASSED (No regressions introduced)
- **Parity Guard**: ✅ Documentation synchronized with code

### 🩺 System Health
- **Engine Heartbeat**: 200 OK
- **Frontend Performance**: Stable
```

---

## 💡 Usage Examples

- `/enhance add dark mode` $\rightarrow$ (Minor: Analysis $\rightarrow$ Implement $\rightarrow$ Audit)
- `/enhance integrate stripe payments` $\rightarrow$ (Major: Analysis $\rightarrow$ **Brainstorm** $\rightarrow$ Implement $\rightarrow$ Audit)
- `/enhance refactor user-auth flow` $\rightarrow$ (Major: Analysis $\rightarrow$ **Brainstorm** $\rightarrow$ Implement $\rightarrow$ Audit)

---

## 🚫 Guardrails

- **No "Quick Fixes"**: Never bypass the Impact Analysis phase. "Just changing one line" is how architectural drift begins.
- **Documentation First**: If the enhancement changes an API endpoint, the documentation must be updated *before* the code is finalized.
- **Constraint Adherence**: If a request conflicts with the core architecture (e.g., introducing a heavy dependency into a lean Rust backend), the AI must flag this as a **Sovereign Violation** and propose an alternative.

[//]: # (Metadata: [enhance])
```

[//]: # (Metadata: [enhance])
