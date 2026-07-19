---
title: "Configuration"
tier: "2"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
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
| `NEURAL_TOKEN` | [RISK: HIGH] | string | `""` | Secret API token required to authenticate all client-dashboard requests. |
| `NEURAL_ENGINE_ACCESS_TOKEN` | [RISK: HIGH] | string | `""` | Core sidecar access token. |
| `AUDIT_PRIVATE_KEY` | [RISK: HIGH] | string | `""` | Private key seed for cryptographically signing transaction logs. |
| `WORKFLOW_ENCRYPTION_KEY` | [RISK: HIGH] | string | `""` | Encryption key for securing workflow states. |
| `CAPABILITY_KEY_CURR` | [RISK: HIGH] | string | `""` | Current signing key for client tokens. |
| `CAPABILITY_KEY_PREV` | [RISK: HIGH] | string | `""` | Previous signing key for client tokens. |
| `TEST_PROVIDER_KEY` | [RISK: HIGH] | string | `""` | Used in integration tests to bypass API keys. |

→ *See [[Vault-&-Credentials|§1]] for how API keys are encrypted and stored client-side.*

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
| `DISCORD_WEBHOOK` | [RISK: HIGH] | string | `""` | Discord notification webhook URL |

> [!WARNING]
> Cloud provider API keys are decrypted only in memory during active runs. Plaintext keys are never written to disk or transmitted to the Tadpole OS backend. See [[Vault-&-Credentials|§1]] for the full security model.

→ *See [[LLM-Providers|§2]] for provider-specific configuration details.*

---

## 3. Security Posture

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `PRIVACY_MODE` | [RISK: HIGH] | boolean | `false` | When `true`, strictly blocks outbound cloud AI connections. |
| `TADPOLE_ALLOW_LOCAL_HTTP` | [RISK: HIGH] | boolean | `false` | Bypasses secure HTTPS checks for local development loops. |
| `AUTO_APPROVE_SAFE_SKILLS` | [RISK: HIGH] | boolean | `true` | Allows safe read-only operations to bypass the [[Approvals-&-Quotas|Oversight Queue]]. |

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
| `VITE_SYSTEM_BREAKER_FAILURE_THRESHOLD` | [RISK: HIGH] | number | `5` | Maximum consecutive failures before tripping [[Glossary#circuit-breaker|Circuit Breaker]]. |
| `VITE_SYSTEM_BREAKER_COOLDOWN_MS` | [RISK: HIGH] | number | `10000` | Cooldown time (ms) before attempting Half-Open probe. |

→ *See [[Network-Boundaries|§2]] for CORS, proxy, and boundary configuration details.*

---

## 5. Persistence & Storage

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `DATABASE_URL` | [RISK: HIGH] | string | `sqlite://data/tadpole.db` | Database connection file path. |
| `DATA_DIR` | [RISK: MEDIUM] | string | `./data` | Root data storage path. |
| `WORKSPACE_ROOT` | [RISK: MEDIUM] | string | (Dynamic) | Base sandboxing root path. Resolves to current directory or parent if in `server-rs` (code: [state/mod.rs:L412-421](../../../server-rs/src/state/mod.rs#L412-L421)). |
| `RESOURCE_ROOT` | [RISK: LOW] | string | `./resources` | Storage path for system instructions and graphics. |
| `STATIC_DIR` | [RISK: MEDIUM] | string | `./static` | Directory path for static frontend assets. |

→ *See [[Updates-&-Backups|§1]] for backup path recommendations.*

---

## 6. Engine Limits

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `MAX_AGENTS` | [RISK: MEDIUM] | number | `50` | Registry limit for maximum concurrent agent identities. Code default is `50` (defined in [state/mod.rs:L547-551](../../../server-rs/src/state/mod.rs#L547-L551)). `.env.example` ships `100`. |
| `MAX_CLUSTERS` | [RISK: MEDIUM] | number | `10` | Caps active project directories. |
| `MAX_SWARM_DEPTH` | [RISK: MEDIUM] | number | `5` | Hard limits recursive agent-spawning depth. Recommended: 3 for standard office hardware. |
| `MAX_TASK_LENGTH` | [RISK: MEDIUM] | number | `32768` | Caps token consumption boundaries per prompt. |
| `ENGINE_RATE_LIMIT` | [RISK: MEDIUM] | number | `2000` | Maximum HTTP requests per minute per IP. |
| `DEFAULT_AGENT_BUDGET_USD` | [RISK: MEDIUM] | number | `1.0` | Spending budget limit for new sub-agents (USD). |
| `WORKFLOW_CONCURRENCY_LIMIT` | [RISK: MEDIUM] | number | `5` | Limits simultaneous SOP workflow execution loops. |
| `WORKFLOW_AGENT_TIMEOUT_SECS` | [RISK: MEDIUM] | number | `600` | Time limit for sub-agent executions in workflows. |
| `MAX_CONCURRENT_RUNNERS` | [RISK: MEDIUM] | number | `20` | Upper limit on concurrently running agent worker tasks. |

---

## 7. Provider Endpoints

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `OLLAMA_HOST` | [RISK: MEDIUM] | string | `http://localhost:11434` | Endpoint for local Ollama model runs. |
| `LANCEDB_DEDUPE_THRESHOLD` | [RISK: MEDIUM] | number | `0.9` | Similarity score threshold for RAG deduplication. |
| `LANCEDB_DRIFT_THRESHOLD` | [RISK: MEDIUM] | number | `0.8` | Similarity score threshold for re-indexing triggers. |
| `SME_SYNC_INTERVAL_MINS` | [RISK: MEDIUM] | number | `30` | Re-indexing interval for background data sync workers. |
| `FAILOVER_AMBER_THRESHOLD` | [RISK: MEDIUM] | number | `3` | Failure count leading to provider health warning. |
| `FAILOVER_RED_THRESHOLD` | [RISK: MEDIUM] | number | `5` | Failure count leading to provider offline state. |
| `FAILOVER_MAX_ATTEMPTS` | [RISK: MEDIUM] | number | `3` | Maximum retry attempts to alternative providers. |
| `PROVIDER_TIMEOUT_SECS` | [RISK: MEDIUM] | number | `60` | Timeout limit on active model generations. |
| `PIPER_MODEL_PATH` | [RISK: MEDIUM] | string | `""` | Local path to Piper text-to-speech weights. |
| `VAD_MODEL_PATH` | [RISK: MEDIUM] | string | `""` | Local path to Silero VAD voice model weights. |
| `WHISPER_MODEL_PATH` | [RISK: MEDIUM] | string | `""` | Local path to Whisper speech recognition weights. |

→ *See [[LLM-Providers|§3]] for provider configuration and failover logic.*
→ *See [[RAG-&-Memory|§1.2]] for LanceDB and vector memory tuning.*

---

## 8. Observability & Telemetry

| Variable | Risk | Type | Default | Description |
|----------|------|------|---------|-------------|
| `RUST_LOG` | [RISK: SAFE] | string | `server_rs=info,tower_http=debug` | Diagnostic logging output levels. |
| `DISABLE_TELEMETRY` | [RISK: LOW] | boolean | `false` | Stops logging OTel telemetry. |
| `HEARTBEAT_INTERVAL_SECS` | [RISK: LOW] | number | `3` | Heartbeat update interval in seconds. |

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
| `TOKIO_THREAD_STACK_SIZE` | [RISK: LOW] | number | `2097152` | Tokio worker thread stack size in bytes (default 2 MB). |

---

## 10. Build Feature Flags

These compile-time flags control which optional subsystems are included in the binary:

| Flag | Risk | Description |
|------|------|-------------|
| `vector-memory` | [RISK: MEDIUM] | Compiles LanceDB + Apache Arrow for persistent semantic memory indexing. Stores vectors in `data/memory.lance/`. Disabled by default for simplified Windows installation. |
| `neural-audio` | [RISK: MEDIUM] | Integrates ONNX runtime (`ort`), Whisper STT, and audio utilities for real-time voice streaming. Disabled by default on legacy CPUs. |

→ *See [[Installation|§3]] for compilation commands.*

---

## 11. Tool Execution Environment

The CLI scan and drift verification script `parity_guard.py` is invoked directly via the terminal:

**[Command] Run Parity Guard**
```powershell
python execution/parity_guard.py .
```
This utility relies on command-line arguments and does not read any `PARITY_GUARD` environment variables.

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](../GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Configuration)


[//]: # (Metadata: [wiki-admin])
