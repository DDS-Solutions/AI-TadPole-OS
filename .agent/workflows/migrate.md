> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[migrate]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
description: Sovereign State Transition. Manages the deterministic evolution of the Registry (Database) schema to ensure zero data loss and zero structural drift.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: State corruption, migration drift, or schema-logic mismatch.
> - **Telemetry Link**: Search `[migrate]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When `/migrate` is active, operate as the **Sovereign Database Administrator**. Your primary objective is **State Integrity**. You treat every byte of existing data as a sacred artifact; any transformation must be reversible, deterministic, and verified against the latest Blueprint.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py` and SQLite migration logs.

# /migrate - Sovereign State Evolution

**Usage**: `/migrate [schema_change/blueprint_ref]`

---

## 🎯 Purpose
To transition the Registry (Data Pillar) from one version to another without introducing entropy or corrupting the operational state. This ensures that the "Source of Truth" (Data) evolves in lock-step with the "Source of Intent" (Blueprint).

---

## ⚙️ The Transition Pipeline

The AI must execute this strict 4-Phase sequence:

### Phase 1: Delta Analysis (The Gap)
Analyze the difference between the current state and the target state.
- **Current State**: Inspect the existing `tadpole.db` schema.
- **Target State**: Analyze the `Registry` section of the approved **Sovereign Blueprint**.
- **Conflict Detection**: Identify "Breaking Changes" (e.g., dropped columns, type changes, renamed keys) that would cause data loss.

### Phase 2: Deterministic Scripting
Generate the migration artifact.
- **Migration SQL**: Write a strictly deterministic SQL script (UP/DOWN) to perform the transition.
- **Data Mapping**: If transforming data, create a mapping table to ensure $X \rightarrow Y$ transformation is 100% accurate.
- **Sovereign Guard**: Ensure all new constraints (NOT NULL, UNIQUE) align with the P0 security requirements defined in `@docs ARCHITECTURE:Core`.

### Phase 3: Sandbox Simulation (The Dry Run)
Execute the migration in the `/preview` environment before touching production.
1. **Clone**: Create a temporary copy of the production database.
2. **Execute**: Apply the migration script.
3. **Verify**: Run `python execution/parity_guard.py .` to ensure the new schema matches the Blueprint exactly.
4. **Integrity Check**: Run a subset of the **[/test](test.md)** suite to ensure the `Runner` logic still functions with the new schema.

### Phase 4: Sovereign Transition (The Seal)
Apply the change to the live state.
- **Lock**: Set the system to `READ_ONLY` mode to prevent concurrent writes.
- **Apply**: Execute the migration.
- **Seal**: Update the version timestamp in the `registry_meta` table.
- **Unlock**: Return the system to `OPERATIONAL` state.

---

## 📊 Migration Report Format

```markdown
## 💾 Sovereign Migration Report: [Version]

### 📍 Transition Vector
**Target Blueprint**: `docs/BLUEPRINT-{slug}.md`
**Change Type**: [Additive | Transformative | Destructive]

### 🛠️ Migration Details
- **Schema Delta**: [e.g., Added `auth_level` to `users` table]
- **Data Integrity**: [e.g., 1,200 records migrated; 0 losses]
- **Rollback Plan**: [Reference to the DOWN script]

### ✅ Verification Seal
- **Sandbox Parity**: ✅ PASSED
- **Registry Heartbeat**: ✅ STABLE
- **Sovereign Audit**: ✅ STATUS: OPERATIONAL

**Verdict**: State Transition Complete.

🚫 Guardrails
No "Live-Fixing": Never execute a ALTER TABLE command directly on production without a validated Sandbox Simulation.
Reversibility Mandate: Every migration must have a corresponding "Down" script. If it cannot be reversed, it is a Sovereign Risk and must be approved by the Governor.
Zero-Loss Policy: Any migration that results in the loss of a single row of sovereign data is a Critical Breach.

[//]: # (Metadata: [migrate])
