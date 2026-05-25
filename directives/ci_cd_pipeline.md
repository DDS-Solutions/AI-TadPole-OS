> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[ci_cd_pipeline]` in audit logs.
>
> ### AI Assist Note
> 🚀 Directive: CI/CD Pipeline (SOP-DEV-03)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🚀 Directive: CI/CD Pipeline (SOP-DEV-03)

## 🎯 Primary Objective
Automate the path from code-commit to production-ready artifact. This directive ensures that no code reaches the "Sovereign" branch without passing 100% of the verification gates.

---

## 🏗️ Pipeline Gates

### 1. Build & Lint (P0)
- **Rust**: `cargo check` and `cargo fmt --check`.
- **Frontend**: `npm run build` (includes TS compilation) and `npm run lint`.

### 2. Unit & Integration Testing (P1)
- **Engine**: `cargo test --all-features`.
- **Frontend**: `npm run test` (Vitest).
- **Parity**: `python execution/parity_guard.py .`

### 3. Security Scan (P1)
- **Tool**: `vulnerability-scanner`.
- **Check**: No CRITICAL or HIGH vulnerabilities in the `Cargo.lock` or `package-lock.json`.

### 4. Deployment Verification (P2)
- **Action**: Deploy to `local-staging` or `dev-cluster-1`.
- **Smoke Test**: `python execution/verify_all.py . --url <STAGING_URL>`.

---

## 🚦 Release Condition
Final "Ship" approval requires a successful run of the **Master Audit** and a manually signed-off `walkthrough.md` from the lead architect or overseer.

## 📊 Deployment Tracking
Update the `RELEASE_PROCESS.md` in the docs folder with the new version tag and any "Breaking Changes" identified during the pipeline run.
[//]: # (Metadata: [ci_cd_pipeline])
