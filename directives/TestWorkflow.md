> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / TestWorkflow
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[MISMATCH]`)

> Traceability via `execution/parity_guard.py`.

# 🧪 Directive: Test Workflow & Validation SOP

## 🎯 Primary Objective
Execute a deterministic verification workflow for the supported engine and track planned Plane 2 acceptance separately ([`IDENTITY.md`](IDENTITY.md)):
- **Plane 1 (`Engine-Maintenance`)**: Rust/TypeScript unit tests, ADG v2 invariant witnesses, and symbol blast-radius verification.
- **Plane 2 (`SMB-Digital-Twin`, planned)**: No live business workflow tests are supported until company-scoped authorization, storage, and connectors exist. Use synthetic fixtures for future acceptance tests; the current runner only rejects the exact stored department marker described below.

---

## 🔄 5-Phase Verification Lifecycle

1. **Phase 1 — Static & Graph Guard (`Plane 1`)**: Run `npm run graph:blast:guard -- --path <target>` and `npm run adg:verify` to confirm symbol and header integrity.
2. **Phase 2 — Unit & Invariant Suite (`Plane 1`)**: Execute `npx vitest run tests/invariants/` and targeted `cargo test` suites.
3. **Phase 3 — Dual-Plane Status Check (`FR-11` / `NFR-1`)**: Inspect the runner's exact `Plane 2: SMB-Digital-Twin` department rejection and record that other labels do not activate it. Do not infer zero code-editing or shell privileges from roles, skill metadata, or handoff labels. Record company-scoped storage and live workflow acceptance as pending; do not use live company data.
4. **Phase 4 — Documentation & Version Parity**: Run `node scripts/bump_version.cjs --check` and `python execution/parity_guard.py`.
5. **Phase 5 — Evidence Recording**: Record pass/fail counts and execution receipts in the active `{task-slug}.md` plan before marking any milestone `[x]`.

> [!NOTE]
> **Compliance Heartbeat** *(IDENTITY.md Directive #8)*: Every 10th swarm operation must include an automated self-audit checksum verifying directive alignment (`IDENTITY.md v1.1.464`), the current `FR-11` gate and its exact-marker limitation, and Merkle Audit Trail continuity (`[COMPLIANCE:OK]` / `[COMPLIANCE:DRIFT]`).
