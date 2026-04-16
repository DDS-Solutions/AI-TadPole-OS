# /test - Sovereign Verification Gate

$ARGUMENTS

---

## 🎯 Primary Objective
Verify the integrity of a mission or feature through automated testing gates. This workflow ensures that no "Broken Logic" ever enters the Sovereign Registry.

## 🏗️ Verification Gates

### 1. Unit & Integration (The Foundation)
- **Backend (Rust)**: `cargo test --workspace`.
- **Frontend (TS/React)**: `npm run test` (Vitest).

### 2. Full System Audit (The Sovereign Gate)
- **Command**: `python execution/verify_all.py . --url http://localhost:8000`.
- **Requirement**: Must pass P0 (Security) and P1 (Code Quality/Parity) categories.

### 3. Documentation Parity
- **Command**: `python execution/parity_guard.py .`.
- **Constraint**: Zero drift between code routes and instruction sets.

---

## 🛠️ Execution Protocol

1. **Detect Changes**: Identify which hubs (Gateway/Runner/Registry/Security) are affected.
2. **Run Targeted Tests**: Execute specific unit tests for the modified modules.
3. **Run Full Audit**: Execute `verify_all.py` before final submission/shipment.

---

## Output Format

### Test Execution (Success)

```markdown
## ✅ Sovereign Verification Passed

### Gates Summary
- **Unit/Integration**: PASSED
- **Security (P0)**: PASSED
- **Parity (P1)**: PASSED
- **Performance**: [Target %]

✨ System ready for Sovereign Registry intake.
```

### Test Execution (Failure)

```markdown
## ❌ Verification Gate Breach

### 🚨 Failures
1. **[Gate Name]**: [Reason]

### Resolution
- Apply remediation plan
- Rerun `verify_all.py`
```
