---
title: "Configuration"
tier: "2"
status: "verified"
version: "1.1.464"
last-verified: "2026-11-20"
commit: "3eef5c2d"
network-badge: "optional"
risk-tags:
  - "RISK: HIGH"
---

# 🔧 Configuration

⚡ OPTIONAL NETWORK

This page is the comprehensive reference for all environment variables, feature flags, and configuration settings used by Tadpole OS. Every entry is tagged with an inline risk level.

### 1.1 Config file editing anchors

- **[Input] .env file**
  - *Default state*: Populated from `.env.example`.
  - *Visible location*: Main project root directory.
  - *Observable side effect*: Defines active ports, keys, paths, and flags.
- **[Button] Save env**
  - *Default state*: N/A (Manual file save).
  - *Visible location*: Text editor/IDE.
  - *Observable side effect*: Prepares environment configuration for the next backend engine boot.

---

## 1. Secrets & Authentication

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `NEURAL_TOKEN` | [RISK: HIGH] | string | `""` | Operational API token (`AuthenticatedRole::Deploy`) required to authenticate client-dashboard requests. |
| `NEURAL_ADMIN_TOKEN` / `ADMIN_TOKEN` | [RISK: HIGH] | string | `""` | Administrative API token (`AuthenticatedRole::Admin`) required for all `RequireAdmin` routes (`403 Forbidden` for `NEURAL_TOKEN`). **Mandatory and distinct from `NEURAL_TOKEN` in production (`TADPOLE_ENV=production`).** |
| `OVERSIGHT_PUBLIC_KEY` | [RISK: HIGH] | string | `""` | 64-hex Ed25519 verifying key pinned for signed oversight decisions (**mandatory in production**). |
| `TADPOLE_BACKUP_PASSPHRASE` | [RISK: HIGH] | string | `""` | Passphrase (≥12 chars) for `TADPOLE_SNAP_V2` automated backups (never written to `system_backup.json`). |
| `TADPOLE_BACKUP_ROOT` | [RISK: MEDIUM] | string | `""` | Optional extra authorized backup destination root (outside default `backups/` and `data/backups/`). |
| `GEV_MCP_AUTHORIZATION` | [RISK: HIGH] | string | `""` | Complete `Bearer <test-token>` header value injected into the local GEV Port 3000 MCP client; never store a literal value in `.agent/mcp_config.json`. |
| `NEURAL_ENGINE_ACCESS_TOKEN` | [RISK: HIGH] | string | `""` | Core sidecar access token (takes precedence over `NEURAL_TOKEN` when set). |
| `AUDIT_PRIVATE_KEY` | [RISK: HIGH] | string | `""` | Private key seed for cryptographically signing transaction logs. |
| `WORKFLOW_ENCRYPTION_KEY` | [RISK: HIGH] | string | `""` | Encryption key for securing workflow states. |
| `CAPABILITY_KEY_CURR` | [RISK: HIGH] | string | `""` | Current signing key for client tokens. |
| `CAPABILITY_KEY_PREV` | [RISK: HIGH] | string | `""` | Previous signing key for client tokens. |
| `TEST_PROVIDER_KEY` | [RISK: HIGH] | string | `""` | Used in integration tests to bypass API keys. |

→ *See [[Vault-&-Credentials]] for how API keys are encrypted and stored client-side.*

---

## 2. Cloud Provider API Keys

☁️ REQUIRES NETWORK

| Variable | Risk | Type | Default | Endpoint |
|----------|------|------|---------|----------|
| `OPENAI_API_KEY` | [RISK: HIGH] | string | `""` | `api.openai.com` |
| `GROQ_API_KEY` | [RISK: HIGH] | string | `""` | `api.groq.com` |
| `ANTHROPIC_API_KEY` | [RISK: HIGH] | string | `""` | `api.anthropic.com` |
| `GOOGLE_API_KEY` | [RISK: HIGH] | string | `""` | `generativelanguage.googleapis.com` |
| `INCEPTION_API_KEY` | [RISK: HIGH] | string | `""` | `api.inceptionlabs.ai` |
| `DEEPSEEK_API_KEY` | [RISK: HIGH] | string | `""` | `api.deepseek.com` |
| `OPENROUTER_API_KEY` | [RISK: HIGH] | string | `""` | `openrouter.ai` |
| `REPLICATE_API_KEY` | [RISK: HIGH] | string | `""` | `api.replicate.com` |
| `DISCORD_WEBHOOK` | [RISK: HIGH] | string | `""` | Discord notification webhook URL |

> [!WARNING]
> Provider API keys are stored encrypted in the browser vault. During task dispatch, the browser sends the decrypted key in the request body to the configured Tadpole OS engine, which uses it for the provider call. Use a trusted engine endpoint and HTTPS when remote; see [[Vault-&-Credentials]] for details.

→ *See [[LLM-Providers]] for provider-specific configuration details.*

---

## 3. Security Posture

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `TADPOLE_ENV` | [RISK: HIGH] | string | `development` | Set to `production` to enforce fail-closed startup (`NEURAL_ADMIN_TOKEN`, `OVERSIGHT_PUBLIC_KEY`, `ALLOWED_ORIGINS`). |
| `PRIVACY_MODE` | [RISK: HIGH] | boolean | `false` | Routes runner LLM calls to local endpoints and blocks cloud voice endpoints (`/v1/engine/live-voice`, cloud `/v1/engine/speak`, cloud `/v1/engine/transcribe` — `403 Forbidden`), while permitting strict loopback `LOCAL_WHISPER_URL` / `LOCAL_TTS_URL`; not a host-level egress firewall. |
| `TADPOLE_ALLOW_LOCAL_HTTP` | [RISK: HIGH] | boolean | `false` | Allows backend Rust SSRF guard to reach local HTTP LLM providers (does not affect browser `SubtleCrypto`). |
| `AUTO_APPROVE_SAFE_SKILLS` | [RISK: HIGH] | boolean | `false` | Fail-closed by default (`server-rs/src/state/mod.rs`); when explicitly enabled, allows safe read-only operations to bypass the Oversight Queue. |
| `TADPOLE_TRUST_INTERPRETERS` | [RISK: HIGH] | boolean | `false` | Opt-in allowing agent shell execution of `python`, `node`, `cargo`, `npm`, `pnpm`, `rustc`. |

---

## 4. Network & Binding

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `PORT` | [RISK: LOW] | number | `8000` | The HTTP server listen port. |
| `BIND_ADDRESS` | [RISK: HIGH] | string | `127.0.0.1` | The local loopback IP address of the engine. |
| `ALLOWED_ORIGINS` | [RISK: HIGH] | string | `http://localhost:5173` | List of permitted browser CORS locations. |
| `TRUST_PRIVATE_NETWORKS` | [RISK: HIGH] | boolean | `false` | Bypasses proxy verification on local subnet cards. |
| `ALLOW_UNSAFE_CORS` | [RISK: HIGH] | boolean | `false` | Enables unsafe open CORS profiles. |
| `TRUSTED_PROXIES` | [RISK: HIGH] | string | `""` | Permitted proxy IP lists. |
| `VITE_SYSTEM_BREAKER_FAILURE_THRESHOLD` | [RISK: HIGH] | number | `5` | Maximum consecutive failures before tripping [[Circuit Breaker|Glossary#circuit-breaker]]. |
| `VITE_SYSTEM_BREAKER_COOLDOWN_MS` | [RISK: HIGH] | number | `10000` | Cooldown time (ms) before attempting Half-Open probe. |

→ *See [[Network-Boundaries]] for CORS, proxy, and boundary configuration details.*

---

## 5. Persistence & Storage

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `DATABASE_URL` | [RISK: HIGH] | string | Unset: `data/state/tadpole.db` | Database connection path. If unset, an existing legacy `data/tadpole.db` is copied to the canonical path when needed. |
| `DATA_DIR` | [RISK: MEDIUM] | string | `./data` | Root data storage path. |
| `WORKSPACE_ROOT` | [RISK: MEDIUM] | string | (Dynamic) | Base sandboxing root path. Resolves to current directory or parent if in `server-rs` (code: [state/mod.rs:L79-88](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/server-rs/src/state/mod.rs#L79-L88)). |
| `RESOURCE_ROOT` | [RISK: LOW] | string | `./resources` | Storage path for system instructions and graphics. |
| `STATIC_DIR` | [RISK: MEDIUM] | string | `dist` | Directory path for static frontend assets (`server-rs/src/router.rs:72`). |

→ *See [[Updates-&-Backups]] for backup path recommendations.*

---

## 6. Engine Limits

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `MAX_AGENTS` | [RISK: MEDIUM] | number | `50` | Registry limit for maximum concurrent agent identities (code default `50` in [state/mod.rs:L267-271](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/server-rs/src/state/mod.rs#L267-L271); `.env.example` ships `50`). |
| `MAX_CLUSTERS` | [RISK: MEDIUM] | number | `10` | Caps active project directories. |
| `MAX_SWARM_DEPTH` | [RISK: MEDIUM] | number | `5` | Hard limits recursive agent-spawning depth. Recommended: 3 for standard office hardware. |
| `MAX_TASK_LENGTH` | [RISK: MEDIUM] | number | `32768` | Caps token consumption boundaries per prompt. |
| `ENGINE_RATE_LIMIT` | [RISK: MEDIUM] | number | `2000` | Maximum HTTP requests per minute per IP. |
| `DEFAULT_AGENT_BUDGET_USD` | [RISK: MEDIUM] | number | `1.0` | Spending budget limit for new sub-agents (USD). |
| `DEFAULT_MISSION_BUDGET_USD` | [RISK: MEDIUM] | number | `5.0` | Default spending budget limit for new missions (USD). |
| `WORKFLOW_CONCURRENCY_LIMIT` | [RISK: MEDIUM] | number | `5` | Limits simultaneous SOP workflow execution loops. |
| `WORKFLOW_AGENT_TIMEOUT_SECS` | [RISK: MEDIUM] | number | `600` | Time limit for sub-agent executions in workflows. |
| `MAX_CONCURRENT_RUNNERS` | [RISK: MEDIUM] | number | `10` | Upper limit on concurrently running agent worker tasks (`server-rs/src/state/mod.rs:231-235`). |

---

## 7. Provider Endpoints

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `OLLAMA_HOST` | [RISK: MEDIUM] | string | `http://localhost:11434` | Endpoint for local Ollama model runs. |
| `LANCEDB_DEDUPE_THRESHOLD` | [RISK: MEDIUM] | number | `0.2` (`.env.example`: `0.85`) | Cosine distance/similarity threshold for RAG deduplication. |
| `LANCEDB_DRIFT_THRESHOLD` | [RISK: MEDIUM] | number | `0.85` (`.env.example`: `0.6`) | Threshold for semantic drift alerts and re-indexing triggers. |
| `SME_SYNC_INTERVAL_MINS` | [RISK: MEDIUM] | number | `30` | Re-indexing interval for background data sync workers. |
| `FAILOVER_AMBER_THRESHOLD` | [RISK: MEDIUM] | number | `3` | Failure count leading to provider health warning. |
| `FAILOVER_RED_THRESHOLD` | [RISK: MEDIUM] | number | `5` | Failure count leading to provider offline state. |
| `FAILOVER_MAX_ATTEMPTS` | [RISK: MEDIUM] | number | `3` | Maximum retry attempts to alternative providers. |
| `PROVIDER_TIMEOUT_SECS` | [RISK: MEDIUM] | number | `60` | Timeout limit on active model generations. |
| `PIPER_MODEL_PATH` | [RISK: MEDIUM] | string | `""` | Local path to Piper text-to-speech weights (`--features neural-audio`). |
| `VAD_MODEL_PATH` | [RISK: MEDIUM] | string | `""` | Local path to Silero VAD voice model weights (`--features neural-audio`). |
| `WHISPER_MODEL_PATH` | [RISK: MEDIUM] | string | `""` | Local path to Whisper speech recognition weights (`--features neural-audio`). |
| `LOCAL_WHISPER_URL` | [RISK: MEDIUM] | string | `""` | Strict loopback-only (`localhost`/`127.0.0.1`/`::1`) OpenAI-compatible Whisper STT server URL (e.g., `http://127.0.0.1:8001`). Allowed under `PRIVACY_MODE=true`. |
| `LOCAL_WHISPER_MODEL` | [RISK: LOW] | string | `whisper-1` | Model name sent to `LOCAL_WHISPER_URL`. |
| `LOCAL_TTS_URL` | [RISK: MEDIUM] | string | `""` | Strict loopback-only (`localhost`/`127.0.0.1`/`::1`) OpenAI-compatible TTS server URL (e.g., Kokoro-FastAPI or Piper HTTP at `http://127.0.0.1:8880`). Allowed under `PRIVACY_MODE=true`. |
| `LOCAL_TTS_MODEL` | [RISK: LOW] | string | `kokoro` | Model name sent to `LOCAL_TTS_URL`. |

→ *See [[LLM-Providers]] for provider configuration and failover logic.*
→ *See [[RAG-&-Memory]] for LanceDB and vector memory tuning.*

---

## 8. Observability & Telemetry

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `RUST_LOG` | [RISK: SAFE] | string | `server_rs=info,tower_http=debug` | Diagnostic logging output levels. |
| `DISABLE_TELEMETRY` | [RISK: LOW] | boolean | `false` | Stops logging OTel telemetry. |
| `HEARTBEAT_INTERVAL_SECS` | [RISK: LOW] | number | `3` | Heartbeat update interval in seconds. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | [RISK: LOW] | string | `""` | OpenTelemetry OTLP collector endpoint (e.g., Jaeger). |

---

## 9. Dev / Test / CI

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `BUNKER_NODES` | [RISK: HIGH] | string | `bunker-1:Swarm Bunker 1:localhost` | Node identification list. Example: `node1:Main Node:localhost,node2:Backup:10.0.0.1` |
| `TADPOLE_NULL_PROVIDERS` | [RISK: MEDIUM] | boolean | `false` | Mocks LLM responses for CI/testing. |
| `LEGACY_JSON_BACKUP` | [RISK: MEDIUM] | boolean | `false` | Keeps old JSON backup files active. |
| `SKIP_DB_SEED` | [RISK: MEDIUM] | boolean | `false` | Skip database default seeding. |
| `CLUSTER_ID` | [RISK: LOW] | string | `local-1` | Unique identifier tag for the local node. |
| `KUBERNETES_SERVICE_HOST` | [RISK: LOW] | string | `""` | Auto-loaded host address in cluster environments. |
| `TOKIO_THREAD_STACK_SIZE` | [RISK: LOW] | number | `16777216` | Tokio worker thread stack size in bytes (defaults to 16 MB in debug mode, 4 MB in release mode to prevent `STATUS_STACK_OVERFLOW` on Windows). |

---

## 10. Build Feature Flags

These compile-time flags control which optional subsystems are included in the binary:

| Flag | Risk | Description |
|------|------|-------------|
| `vector-memory` | [RISK: MEDIUM] | Compiles LanceDB + Apache Arrow for persistent semantic memory indexing. Stores agent and mission vectors under `data/workspaces/<cluster-id>/`. Disabled by default (`default = []` in `server-rs/Cargo.toml`). |
| `neural-audio` | [RISK: MEDIUM] | Integrates ONNX runtime (`ort`), Whisper STT, and audio utilities for real-time voice streaming. Disabled by default on legacy CPUs. |

→ *See [[Installation]] for compilation commands.*

---

## 11. Tool Execution Environment

The CLI scan and drift verification script `parity_guard.py` is invoked directly via the terminal:

**[Command] Run Parity Guard**
```powershell
python execution/parity_guard.py .
python execution/parity_guard.py --fix   # or FIX=1 to sync docs/API_REFERENCE.md
```
Supports `--fix`, positional `FIX=1`, and `FIX=1` environment variable to regenerate `docs/API_REFERENCE.md` from `docs/openapi.yaml`.

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Configuration)


[//]: # (Metadata: [wiki-admin])
