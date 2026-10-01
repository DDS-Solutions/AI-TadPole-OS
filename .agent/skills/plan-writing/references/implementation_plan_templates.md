> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / plan-writing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Scope creep or unverified plan execution.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[PLAN_WRITING]`)

# Implementation Plan Blueprints & Slicing Reference (L3)

---

## 1. Feature Addition Blueprint Template

```markdown
# Feature: [Feature Name]

## 1. Goal
One-sentence summary of the capability being added.

## 2. Blast Radius & Affected Symbols
- Modified files: `src/services/agent_service.ts`, `server-rs/src/routes/agents.rs`
- Ingested symbol graph slice: 1.2k tokens

## 3. Atomic Tasks (Vertical Slices)
- [ ] Task 1: Add backend route & state hub handler. Verify: `cargo test`.
- [ ] Task 2: Bind frontend client service & Zustand store. Verify: `npm test`.
- [ ] Task 3: Render UI component with dark mode styles. Verify: Visual check.

## 4. Verification Gate
- Automated tests: `npm run test && cargo test`
- Security scan: `python .agent/skills/vulnerability-scanner/scripts/security_scan.py .`
```

---

## 2. Expand-Contract Refactoring Slicing

1. **Expand Phase**: Introduce the new function/interface alongside the legacy code.
2. **Migrate Phase**: Update callers incrementally in small atomic batches with green tests.
3. **Contract Phase**: Deprecate and remove the legacy function once all callers migrate.