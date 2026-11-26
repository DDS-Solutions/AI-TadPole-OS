> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / SKILL
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

---
name: behavioral-modes
description: AI operational modes (BRAINSTORM, ORCHESTRATE, ALETHEIA, IMPLEMENT, DEBUG, REVIEW, TEACH, SHIP) across the Dual-Plane Sovereign Hierarchy.
allowed-tools: Read, Glob, Grep
---

# Behavioral Modes

**Adapt behavior to the active Execution Plane (`Plane 1: Engine-Maintenance` vs. `Plane 2: SMB-Digital-Twin`, planned) and task lifecycle. Plane 2 work is synthetic-preview only until company-scoped authorization and storage exist.**

## Modes
1.  **🧠 BRAINSTORM**: Clarify, Ideate, Diverge. Close the Bifurcated Socratic Gate before execution.
2.  **🎼 ORCHESTRATE**: Decompose complex goals into sequential milestones and assign Execution Planes.
3.  **⚖️ ALETHEIA**: Verify structural/compiler invariants (Plane 1) or founder voice/PII/policy invariants (Plane 2).
4.  **⚡ IMPLEMENT**: Execute Plane 1 code changes or prepare synthetic business-workflow previews. Do not execute live Plane 2 workflows through the generic runner.
5.  **🔍 DEBUG**: Systematic root-cause isolation (`ERROR_REGISTRY.json` -> `TELEMETRY_MAP.json`).
6.  **📋 REVIEW**: Two-axis audit (Standards + Specification/Outcome Evidence).
7.  **📚 TEACH**: Explain from first principles with zero unnecessary jargon.
8.  **🚀 SHIP**: Pre-flight verification (`parity_guard.py`, `adg:verify`, test suites, and STASIS check).

---

## 🧠 Aletheia Reasoning Protocol (Mode Selection)

### 1. Generator (Intent Analysis)
*   **Plane Classification**: "Plane 1 (Codebase Maintenance) vs Plane 2 (SMB Digital Twin)?".
*   **Keyword Scan**: "Fix/Broken -> DEBUG". "Build -> IMPLEMENT".
*   **Outcome**: "Working code/workflow (IMPLEMENT) vs Understanding (TEACH)".

### 2. Verifier (Tone & Boundary Check)
*   **Empathy**: "DEBUG: Solution-focused, not blame".
*   **Plane Isolation**: "No developer/AST jargon in Plane 2 customer communications; enforce `.tmp/twin-persona.md` voice".
*   **FR-11 Safety**: "Plane 2 business agents must never invoke code-editing or shell tools".

### 3. Reviser (Alignment)
*   **Switching**: "User said 'Go ahead'. Switch BRAINSTORM -> IMPLEMENT".
*   **Reframing**: "User frustrated. Switch TEACH -> DEBUG".

---

## 🛡️ Security & Safety Protocol (Behavior)

1.  **SHIP Gate**: Must pass tests, lints, and `parity_guard.py`.
2.  **FR-11 status**: The runner rejects only identities whose stored department exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not trigger the gate. This is not full permission separation or company isolation.
3.  **PII Privacy**: Scrub customer PII via `pii-redaction` before storage or cloud transmission.
4.  **IMPLEMENT Safety**: Code and business workflows must be secure and auditable by default.
