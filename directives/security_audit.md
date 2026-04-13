# 🛡️ Directive: Security Audit (SOP-SEC-01)

## 🎯 Primary Objective
Perform a comprehensive, non-bypassable security audit of the Tadpole OS engine and its supporting infrastructure. Your goal is to identify vulnerabilities, secret leakages, or architectural drifts that compromise the integrity of the Sovereign Swarm.

---

## 🏗️ Audit Layers

### 1. Engine Integrity (Rust Backend)
- **Tool**: `cargo clippy --workspace --all-targets`
- **Focus**: 
  - Ensure all `Result` and `Option` types are handled without unsafe `unwrap()`.
  - Verify that `parking_lot` is used for all sync mutexes to prevent poisoning.
  - Audit `server-rs/src/security/scanner.rs` to ensure secret signatures are up to date.

### 2. Sandbox Verification (Filesystem)
- **SOP**: Manually inspect `server-rs/src/adapter/filesystem.rs`.
- **Constraint**: Verify that all paths are canonicalized using `std::fs::canonicalize` BEFORE any `starts_with(workspace_root)` check (SEC-03).
- **Test**: Attempt to create a mock symlink in a scratch directory to verify the sandbox blocks escape attempts.

### 3. Financial Gating (Budget Guard)
- **Goal**: Zero unauthorized token spend.
- **Audit**: Verify `server-rs/src/security/metering.rs`. Ensure that `check_budget` aggregates BOTH the SQLite persistence layer AND the in-memory `DashMap` buffers.

### 4. Merkle Audit Trail
- **Goal**: Chain of custody non-repudiation.
- **Action**: Run `verify_chain()` on the `audit.rs` module. Check for any broken cryptographic links in the `tadpole.db` audit ledger.

---

## 🚦 Triage Levels
- [ ] **CRITICAL**: Immediate sandbox escape, unredacted API keys in logs, or bypass of Budget Guard.
- [ ] **HIGH**: Unsafe `unwrap()` in core engine pathways, or missing `security:new` oversight gates.
- [ ] **MEDIUM**: Standard linting errors or minor documentation drift.

## 📝 Reporting Protocol
All findings must be logged to `reports/security_audit_[TIMESTAMP].md` in the current workspace. Each entry must provide a "Remediation Path" and a specific file-link to the offending code block.