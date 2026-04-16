# 🚢 Deployment Guide

> **Status**: Active  
> **Last Verified**: 2026-04-12  
> **Classification**: Sovereign  

---

This guide covers the deployment workflow that currently exists in the repo. The shipped scripts build Linux desktop artifacts, deploy a `.deb` to a reachable Linux host, and prepare a sanitized public release workspace.

## Supported Scripts

| Script | Purpose |
| :--- | :--- |
| `scripts/build-linux-light.ps1` | Builds Linux `.deb` and `.AppImage` artifacts inside Docker. |
| `scripts/deploy-linuxlite.ps1` | Copies the built `.deb` to a Linux target over SSH and installs it with `dpkg`. |
| `scripts/publish-public.ps1` | Creates a sanitized staging directory for publishing a public GitHub mirror. |

## Pre-Flight Checks

Run these from the repo root before packaging or deployment:

```powershell
python execution/verify_all.py
python execution/parity_guard.py .
npm run build
```

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
