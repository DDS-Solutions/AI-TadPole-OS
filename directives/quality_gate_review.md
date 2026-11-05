> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / quality_gate_review
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[quality_gate_review]`)

# 🛡️ Directive: Quality Gate Review (SOP-DEV-06)

## 🎯 Primary Objective
Serve as the final "Sovereign Gatekeeper" before code is merged into the production branch. This directive ensures that all quality, security, and performance standards are met with zero compromise.

> 💡 **Bound Skills**:
> - Use `@[skills/code-review]` to execute Two-Axis Parallel Reviews (`Standards` vs `Spec`) and Fowler code smell checks.
> - Use `@[skills/systematic-debugging]` to enforce the Phase 1 Reproduction Gate (<2s red-capable script) for any bug fix.

---

## 🏗️ The 4-Gate Protocol

### 1. Gate 1: Technical Integrity
- **Build**: `cargo build --workspace` and `npm run build` MUST pass without warnings.
- **Lint**: Zero linting errors in Rust or TypeScript.

### 2. Gate 2: Security & Privacy
- **Audit**: `vulnerability-scanner` must return zero CRITICAL or HIGH results.
- **Redaction**: Verify that no new code introduces unredacted API key logging.
- **Privacy**: Ensure `PRIVACY_MODE` logic is correctly integrated for any new system call.

### 3. Gate 3: Documentation Parity
- **Audit**: `parity_guard.py` must return 0 drift.
- **Reference**: New API endpoints must be documented in `API_REFERENCE.md`.

### 4. Gate 4: Performance Benchmarking
- **Requirement**: `verify_all.py` must be executed against the local staging environment.
- **KPI**: Latency must stay within the thresholds defined in `Benchmark_Spec.md`.

---

## 🛠️ Decision SOP
- **Pass**: All 4 gates are cleared. Feature is ready for `deploy_to_prod.md`.
- **Conditional Pass**: Minor documentation gaps (must be fixed within 24 hours).
- **Fail**: Any P0 Security, Build, or Linting error. Immediate return to the development phase.

## 📊 Sign-Off Protocol
The final `QUALITY_GATE_SIGN_OFF_[DATE].md` must be archived in the cluster's permanent audit ledger.