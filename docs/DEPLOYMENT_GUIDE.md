> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / DEPLOYMENT_GUIDE
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🚢 Deployment Guide

> **Status**: Active  
> **Last Verified**: 2026-10-01 (vs tip `bbcf0d4` / v1.1.463)  
> **Classification**: Sovereign  

---

This guide covers the **supported** deployment paths in-repo today:

1. **GHCR + `docker-compose.yml`** (primary server/node path)
2. **Linux desktop `.deb` / `.AppImage`** via PowerShell packaging helpers

> [!WARNING]
> There is **no** maintained `scripts/publish-public.ps1` (or root `deploy.ps1`) on tip. Public release staging is owned by GitHub Actions / `docs/RELEASE_PROCESS.md`, not a local publish script.

## Primary Path: GHCR Compose

`docker-compose.yml` pulls the prebuilt image (no local `build:` step):

```yaml
image: ghcr.io/dds-solutions/ai-tadpole-os:latest
```

### Quick start

```bash
cp .env.example .env
# Set NEURAL_TOKEN (required). For production also set ADMIN_TOKEN and CAPABILITY_KEY_CURR.
docker compose up -d
```

Engine health: `curl -sf http://localhost:8000/v1/engine/health`

| Compose fact | Operator note |
| :--- | :--- |
| Bind | Publishes **`0.0.0.0:8000`** and sets `BIND_ADDRESS=0.0.0.0` — reachable on LAN; harden with firewall/Tailscale or prefer loopback for local-only. |
| Required | `NEURAL_TOKEN` — compose fails fast if unset (`${NEURAL_TOKEN:?...}`). |
| Optional inject | `ADMIN_TOKEN` / `NEURAL_ADMIN_TOKEN`, `CAPABILITY_KEY_CURR` / `CAPABILITY_KEY_PREV`, `PRIVACY_MODE` — compose passes `${VAR:-}` so demos still boot; set them in `.env` for production. Production **requires** admin token ≠ `NEURAL_TOKEN`. |
| CORS | `ALLOWED_ORIGINS` may be empty in compose; engine still allows built-in localhost/Tauri origins when unset (see GETTING_STARTED). |
| Observability | Prometheus scrapes `/v1/engine/metrics` (bearer). Grafana binds `127.0.0.1:3000` (default password `admin`). |

### Production secrets checklist

| Variable | Why |
| :--- | :--- |
| `NEURAL_TOKEN` | API / WS bearer |
| `ADMIN_TOKEN` or `NEURAL_ADMIN_TOKEN` | Required when `TADPOLE_ENV`/`NODE_ENV=production`; must differ from `NEURAL_TOKEN` |
| `CAPABILITY_KEY_CURR` | 64-char hex (`openssl rand -hex 32`) or empty for ephemeral key; malformed → **panic at boot** |
| `PRIVACY_MODE` | `true` → local Ollama only, **≤15B** models; no safe local model → `NullProvider` / degraded missions |
| Provider keys | As needed (`GOOGLE_API_KEY`, `GROQ_API_KEY`, …) |

See `.env.example` and `docs/OPERATIONS_MANUAL.md` for the full matrix.

## Desktop Packaging Scripts

| Script | Purpose |
| :--- | :--- |
| `scripts/build-linux-light.ps1` | Builds Linux `.deb` and `.AppImage` artifacts inside Docker. |
| `scripts/deploy-linuxlite.ps1` | Copies the built `.deb` to a Linux target over SSH and installs it with `dpkg`. |

### Pre-Flight Checks

```powershell
python execution/verify_all.py
python execution/parity_guard.py .
npm run build
```

> [!IMPORTANT]
> **STASIS Check** *(IDENTITY.md Directive #7)*: Verify the system is **NOT** in `STASIS` mode before deployment. If `STASIS` is active, deployment is **blocked** until Entity 0 issues an explicit `RESUME` command.

Use `cargo build --release --manifest-path server-rs/Cargo.toml` when you also want an explicit backend release build locally.

### Build Linux Artifacts

Requirements: Docker daemon + PowerShell 7+.

```powershell
./scripts/build-linux-light.ps1
```

Artifacts: `dist/linux-light/appimage/`, `dist/linux-light/deb/`.

### Deploy To a Linux Host

1. Find the first `.deb` inside `dist/linux-light/`
2. Copy with `scp`, install with `sudo dpkg -i` + `apt-get install -f -y`
3. Update `TargetIP` / `TargetUser` in `scripts/deploy-linuxlite.ps1` first

```powershell
./scripts/deploy-linuxlite.ps1
```

## Monitoring And Verification

After deployment:

- Verify the engine on the expected host/port (`/v1/engine/health`).
- Confirm `NEURAL_TOKEN`, `DATABASE_URL`, admin/capability secrets, and provider keys on the target.
- Smoke-test `/v1/agents`, telemetry WS `/v1/engine/ws`, and (if used) `/v1/engine/live-voice`.
- Check dashboard + engine logs for WebSocket connectivity and telemetry health.
