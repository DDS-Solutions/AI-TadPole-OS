# 🚀 Release Process & Lifecycle

> **Intelligence Level**: Architectural (Level 4)  
> **Status**: Verified Production-Ready  
> **Version**: 1.0.0  
> **Last Hardened**: 2026-04-10  
> **Classification**: Sovereign  

---

## 📦 Versioning Strategy
Tadpole OS adheres to **Semantic Versioning (SemVer) 2.0.0**.
- **MAJOR**: Breaking architectural changes (e.g., Engine Protocol overhaul).
- **MINOR**: New features (e.g., New Provider, Visualizer upgrade).
- **PATCH**: Bug fixes and security hardening.

---

## 🛠️ Pre-Flight Verification (Mandatory)

Before ANY release, the following integrity checks MUST pass:

### 1. Integrity Audit
Run the master verification suite:
```powershell
python execution/verify_all.py
```
**Required State**: `[OK] Engine Connection`, `[OK] DB Integrity`, `[OK] Tool Discovery`.

### 2. Parity Check
Ensure the documentation suite is synchronized with the compiled backend routes:
```powershell
python execution/parity_guard.py
```
**Required State**: Zero drift between `docs/CODEBASE_MAP.md` and Axum router state.

### 3. Build Validation
Perform clean builds for both layers:
- **Backend**: `cd server-rs; cargo clean; cargo build --release`
- **Frontend**: `npm run build` (Audit the `/dist` bundle for size regressions).

---

## ⛴️ Distribution Channels

### 1. Sovereign Bunker (Local Deployment)
For private, air-gapped instances:
1.  **Tag**: `git tag -a v1.6.x -m "Release description"`
2.  **Archive**: Create a tarball of the project root (excluding `node_modules`, `target`, and `.env`).
3.  **Deploy**: Transfer to the Bunker node and run `npm run setup`.

### 2. GitHub Release (Marketing & Public)
For public releases (handled via `scripts/deploy/deploy_linux.ps1`):
1.  **Build**: Execute the multi-stage Docker build.
2.  **Push**: Push the container image to the secure registry.
3.  **Tag**: Create a GitHub Release with high-fidelity release notes (using the `walkthrough.md` template).

---

## 🛰️ Post-Release Monitoring

Once live, monitor the **Engine Dashboard** for the first 100 missions:
- **Error Rate**: Should remain below 1% (RFC 9457 compliant).
- **Swarm Density**: Verify no memory leaks in `FuturesUnordered`.
- **Audit Integrity**: Periodically run `verify_chain()` on the production database.

---

> [!CAUTION]
> **Data Migration**: All releases involving SQLite schema changes MUST include a SQL migration script in `server-rs/migrations/`. Do not perform manual `ALTER TABLE` operations in production environments.
