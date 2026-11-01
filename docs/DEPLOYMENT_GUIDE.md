> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / DEPLOYMENT_GUIDE
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🚢 Deployment Guide

> **Status**: Active  
> **Last Verified**: 2026-07-13  
> **Classification**: Sovereign  

---

This guide covers the deployment workflow that currently exists in the repo. The shipped scripts build Linux desktop artifacts, deploy a `.deb` to a reachable Linux host, and prepare a sanitized public release workspace.

## Supported Scripts

| Script | Platform | Purpose |
| :--- | :--- | :--- |
| `scripts/build-linux-light.ps1` | Linux | Builds `.deb` and `.AppImage` artifacts inside Docker. |
| `scripts/build-macos-universal.sh` | macOS | Compiles dual-slice sidecar, lipo-splices, and packages universal `.dmg`. |
| `scripts/deploy-linuxlite.ps1` | Linux | Copies built `.deb` to a Linux target over SSH and installs it with `dpkg`. |
| `scripts/publish-public.ps1` | All | Creates a sanitized staging directory for publishing a public GitHub mirror. |

## Pre-Flight Checks

Run these from the repo root before packaging or deployment:

```powershell
python execution/verify_all.py
python execution/parity_guard.py .
npm run build
```

> [!IMPORTANT]
> **Dispatch Check** *(IDENTITY.md Directive #7)*: Check the engine's actual `dispatch_paused` state and active work before deployment. Budget exhaustion denies an affected provider call but does not create a global STASIS state or deployment lockout.

Use `cargo build --release --manifest-path server-rs/Cargo.toml` when you also want an explicit backend release build locally.

## Build Linux Artifacts

`scripts/build-linux-light.ps1` is the current packaging entry point for Linux desktop output.

Requirements:
- Docker Desktop or another working Docker daemon
- PowerShell 7+

Run:

```powershell
./scripts/build-linux-light.ps1
```

Artifacts are extracted to:

- `dist/linux-light/appimage/`
- `dist/linux-light/deb/`

---

## Build macOS Universal Bundle (.dmg)

`scripts/build-macos-universal.sh` produces a Universal 2 Mach-O fat binary containing native slices for both **Apple Silicon (`aarch64-apple-darwin`)** and **Intel Macs (`x86_64-apple-darwin`)**.

Requirements:
- macOS host (or macOS-14 GitHub Actions runner)
- Xcode Command Line Tools (`xcode-select --install`)
- Rust stable with both targets: `rustup target add aarch64-apple-darwin x86_64-apple-darwin`
- Node.js 24+

Run:

```bash
npm run tauri:build:macos
# or directly:
bash scripts/build-macos-universal.sh
```

Artifacts are produced in:

- `src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg`

The local helper and release workflow share this build path. Packaging requires both `arm64` and `x86_64` slices in the shell and bundled engine, valid code signatures, and a nonempty disk image. Recheck an existing bundle with `bash scripts/verify-macos-bundle.sh`. The main executable is read from `Info.plist`. These artifact checks do not replace a native launch, installation, and shutdown test on each supported processor architecture.

> [!NOTE]
> **Persistent Data on macOS**: The shell places its database, neural token, logs and PID locks under `~/Library/Application Support/com.tadpole.os/`. Set an absolute `TADPOLE_DATA_DIR` environment variable before launching to override it. Missing `HOME`, relative paths, and directory creation/permission errors stop startup before launching the engine. macOS data directories use owner-only permissions. Configuration loaded from `.env` is read after directory selection, so `.env` cannot relocate that directory.

> [!IMPORTANT]
> **Ad-hoc Signed Only**: `bundle.macOS.signingIdentity` is `"-"`; the app is ad-hoc signed, without Apple notarization. This follows [Tauri's ad-hoc signing configuration](https://v2.tauri.app/distribute/sign/macos/#ad-hoc-signing). Gatekeeper may require approval in System Settings → Privacy & Security. For a trusted local build, the documented quarantine command is:
> ```bash
> xattr -cr /Applications/AI-Tadpole-OS.app
> ```

## Deploy To a Linux Host

`scripts/deploy-linuxlite.ps1` is the current SSH-based deployment helper.

What it does:
1. Finds the first `.deb` inside `dist/linux-light/`
2. Copies it to the target machine with `scp`
3. Installs it remotely with `sudo dpkg -i` and `apt-get install -f -y`

Before running it:
- Update `TargetIP` and `TargetUser` in `scripts/deploy-linuxlite.ps1`
- Ensure SSH key-based access works
- Build the Linux artifacts first

Run:

```powershell
./scripts/deploy-linuxlite.ps1
```

## Public Release Staging

`scripts/publish-public.ps1` prepares a sanitized `.tmp/public-release/` directory from tracked files only.

It currently:
- Copies tracked repo files into a clean staging directory
- Rewrites known hostnames and non-placeholder IP addresses
- Removes internal deployment and scratch artifacts
- Initializes a clean git repo in the staged directory for push/publish workflows

Run:

```powershell
./scripts/publish-public.ps1
```

## Monitoring And Verification

After deployment:
- Verify the engine is reachable on the expected host/port.
- Confirm `NEURAL_TOKEN`, `DATABASE_URL`, and provider secrets are set correctly on the target host.
- Run a smoke test against `/v1/engine/live-voice`, `/v1/agents`, or another expected route based on your deployment profile.
- Check the dashboard and the engine log stream for WebSocket connectivity and telemetry health.
