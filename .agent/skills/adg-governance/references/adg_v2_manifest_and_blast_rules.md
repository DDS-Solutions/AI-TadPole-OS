# Active Documentation Guard (ADG v2) Governance & Blast Rules (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / adg-governance / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: ADG v1 anti-patterns, hand-written test witness claims, unverified doc drift.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document details the Active Documentation Guard v2 rules and AST blast radius tooling in Tadpole OS.

---

## 1. Forbidden ADG v1 Anti-Patterns

Per `@docs docs/ADG_V2.md`, the following are strictly forbidden in file headers:
- ❌ **`Witness Tests: ...`**: Never hand-write test witness claims in headers. All test witnesses are machine-derived by `npm run adg:generate` into `adg.manifest.json`.
- ❌ **`Invariants: ...`**: Never write loose claims. Safety guarantees belong in executable invariant tests in `tests/invariants/`.

---

## 2. Mandatory File Header Format

Every production code file must start with:

```typescript
/**
 * @docs ARCHITECTURE:Subsystem_Name
 *
 * ### AI Context Alignment
 * - **Subsystem**: [Subsystem / Component]
 * - **Primary Entrypoints**: `entrypoint_a`, `entrypoint_b`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[TAG_NAME]`
 */
```

---

## 3. Pre-Commit Verification Commands

```powershell
# 1. Inspect affected callers and callees before edits
npm run graph:blast:guard -- --path server-rs/src/routes/system.rs

# 2. Verify documentation parity & check for forbidden v1 claims
npm run adg:verify
npm run adg:lint
```
