# Verification Tiers & Evidence Matrix

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / verify-changes / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. The 4 Verification Tiers

Never declare a change "done" based solely on visual inspection. Progress through each verification tier:

```
[TIER 1: COMPILATION] ───> [TIER 2: STATIC LINT] ───> [TIER 3: AUTOMATED TEST] ───> [TIER 4: RUNTIME PROOF]
 tsc / cargo check          eslint / clippy             vitest / cargo test          CLI run / HTTP check
```

| Tier | Required Command | Evidence Required |
|---|---|---|
| **Tier 1: Compilation** | `npx tsc --noEmit` or `cargo check` | Zero compiler errors |
| **Tier 2: Static Analysis** | `npm run lint` or `cargo clippy` | Zero lint warnings / violations |
| **Tier 3: Test Witness** | `npm test` or `cargo test` | Passed test asserting changed behavior |
| **Tier 4: Runtime Execution** | Run CLI or invoke endpoint | Verified stdout response or HTTP 200 payload |

---

## 2. Evidence Presentation Format

When presenting proof of verification to the user, include:
1. **Command Executed**: The exact shell command invoked.
2. **Execution Result**: Exit code and execution duration.
3. **Output Evidence**: The relevant output lines proving the behavior changed.
