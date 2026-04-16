# 🚀 Release Process

> **Status**: Active  
> **Last Verified**: 2026-04-12  
> **Classification**: Sovereign  

---

This is the current release workflow reflected by the repository scripts and build layout.

## Versioning

Tadpole OS follows Semantic Versioning:
- **MAJOR**: breaking runtime, API, or packaging changes
- **MINOR**: new product capabilities
- **PATCH**: fixes, hardening, and documentation alignment

## Required Verification

Run these before cutting a release:

```powershell
python execution/verify_all.py
python execution/parity_guard.py .
npm run build
npm run test
```

Recommended backend validation:

```powershell
cargo build --release --manifest-path server-rs/Cargo.toml
```

## Release Paths

### 1. Linux Desktop Artifacts

Use the current Docker-backed builder:

```powershell
./scripts/build-linux-light.ps1
```

This produces `.deb` and `.AppImage` bundles in `dist/linux-light/`.

### 2. Targeted Linux Host Deployment

For a direct SSH deployment of the generated `.deb`:

```powershell
./scripts/deploy-linuxlite.ps1
```

Make sure the script's `TargetIP` and `TargetUser` values are configured first.

### 3. Public GitHub Mirror / Sanitized Release

For a public-safe staging copy:

```powershell
./scripts/publish-public.ps1
```

This creates `.tmp/public-release/` with tracked files, sanitized hostnames/IPs, and a clean git history root for publishing.

## Release Notes Checklist

- Summarize security and trust changes first.
- Call out any API, deployment, or starter-kit behavior changes.
- Note frontend bundle-size regressions or improvements.
- Include documentation parity status and verification results.

## Post-Release Checks

- Verify the dashboard can authenticate with a real `NEURAL_TOKEN`.
- Confirm WebSocket connectivity on `/engine/ws`.
- Smoke-test a task dispatch, an oversight flow, and one starter-kit/template install path if templates changed.
