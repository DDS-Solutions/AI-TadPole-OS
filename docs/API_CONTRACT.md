> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / API_CONTRACT
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 📄 API Contract: Tadpole OS Engine

![Status: Production](https://img.shields.io/badge/Status-Production-green)
![Version: 1.1.464](https://img.shields.io/badge/Version-1.1.464-blue)
![Auth: Bearer](https://img.shields.io/badge/Auth-Bearer_Token-emerald)

> **@docs ARCHITECTURE:ApiContract**

This is the minimalist REST contract for the Tadpole OS Engine. For high-level architecture, see the [Architecture Overview](./Architecture_Overview.md).

---

## ⚡ Quick Start (3 Steps)

1.  **Set Tokens**: `export NEURAL_TOKEN=your_operational_token` (and in production, a distinct `export NEURAL_ADMIN_TOKEN=your_admin_token` for endpoints marked **Admin**; operational tokens receive `403 Forbidden` on Admin routes).
2.  **Verify Engine**: `curl -H "Authorization: Bearer $NEURAL_TOKEN" http://localhost:8000/v1/engine/health`
3.  **List Agents**: `curl -H "Authorization: Bearer $NEURAL_TOKEN" http://localhost:8000/v1/agents`

---

## 🛰️ Standard Headers

Every response from the engine includes these observability headers:

| Header | Description |
|:--- |:--- |
| `X-Request-Id` | Unique UUID for end-to-end request tracing. |
| `X-RateLimit-Limit` | Maximum requests allowed per 60s window. |
| `X-RateLimit-Remaining` | Remaining requests in the current window. |

---

## 🔌 REST Endpoints (v1)

### Health & Control

| Method | Path | Auth | Description |
|:--- |:--- |:--- |:--- |
| `GET` | `/v1/engine/health` | Bearer | Engine health & uptime pulse. |
| `POST` | `/v1/engine/deploy` | **Admin** | Trigger pinned bunker deployment scripts (`scripts/deploy-bunker-{1,2}.{ps1,sh}`). |
| `POST` | `/v1/engine/kill` | **Admin** | Emergency halt of active agents + durable `dispatch_paused` latch. |
| `POST` | `/v1/engine/resume` | **Admin** | Clear emergency `dispatch_paused` latch and resume task dispatch. |
| `POST` | `/v1/engine/shutdown` | **Admin** | Graceful engine process shutdown. |
| `GET` | `/v1/engine/audio/capabilities` | Bearer | Runtime TTS/STT availability matrix (respects Privacy Shield, loopback local URLs, provider keys, and build features). |
| `POST` | `/v1/engine/speak` | Bearer | Text-to-Speech synthesis (`browser`, `openai` cloud, or local loopback `LOCAL_TTS_URL` / `piper`). |
| `POST` | `/v1/engine/transcribe` | Bearer | Speech-to-Text transcription (`groq` cloud or local loopback `LOCAL_WHISPER_URL` / `whisper`, MIME-preserving). |
| `GET` | `/v1/engine/ws` | WS Auth | Real-time telemetry WebSocket (`bearer.<token>` or post-connect JSON `auth`). |
| `GET` | `/v1/engine/live-voice` | WS `bearer.<token>` | Gemini Live Voice proxy (blocked `403` when Privacy Shield is active). |
| `POST` | `/v1/engine/a2a/mailbox/send` | `X-A2A-Signature` | Queue an A2A envelope from a remote agent (HMAC-SHA256 verified). |

### Agents & Swarms

| Method | Path | Auth | Description |
|:--- |:--- |:--- |:--- |
| `GET` | `/v1/agents` | Bearer | List all registered agents (Paginated). |
| `POST` | `/v1/agents` | **Admin** | Register a new agent node (with SSRF `base_url` validation). |
| `GET` | `/v1/agents/{id}`| Bearer | Retrieve a specific agent's schematic. |
| `PUT` | `/v1/agents/{id}`| **Admin** | Update agent configuration. |
| `POST` | `/v1/agents/{id}/tasks` | Bearer | **Execute**: Dispatch a task to the agent runner (supports `execution_mode`: `chat` \| `plan` \| `swarm`, legacy `safe_mode`, and `recent_findings` brainstorm handoff context). |
| `DELETE` | `/v1/agents/{id}` | **Admin** | De-register an agent node. |

### Mission Flight Recorder & Observability

| Method | Path | Auth | Description |
|:--- |:--- |:--- |:--- |
| `GET` | `/v1/missions` | Bearer | Paginated mission run catalog with status filters, search, and aggregated counts. |
| `GET` | `/v1/missions/{id}/dossier` | Bearer | Complete composite flight dossier (Merkle audit logs, pre-aligned CAS diffs, Memory Palace, TrustGraph, A2A economics, Agent 99 debrief). |
| `POST` | `/v1/missions/{id}/pin` | Bearer | Toggle retention pin flag protecting the mission from automated Reaper pruning. |
| `GET` | `/v1/cas/history` | Bearer | Retrieve Content-Addressable Store (CAS) revision history for a workspace file. |
| `GET` | `/v1/cas/diff` | Bearer | Retrieve pre-aligned side-by-side LCS diff (`aligned_lines` + `diff_summary`) for a CAS file version. |
| `POST` | `/v1/cas/restore` | **Admin** | Restore a workspace file to a specific historical CAS revision. |

### Oversight & Governance

| Method | Path | Auth | Description |
|:--- |:--- |:--- |:--- |
| `GET` | `/v1/oversight/pending` | Bearer | List all actions awaiting human approval. |
| `POST` | `/v1/oversight/{id}/decide` | **Admin** | Approve or Reject a tool execution (Ed25519 `ovs:v2` signed in production). |
| `PUT` | `/v1/oversight/settings` | **Admin** | Update oversight governance settings and security policies. |
| `PUT` | `/v1/oversight/security/quotas/{entity_id}` | **Admin** | Update agent or mission budget quotas. |
| `GET` | `/v1/oversight/ledger` | Bearer | Review historical governance decisions. |
| `GET` | `/v1/oversight/security/integrity` | Bearer | Verify the Merkle Audit Chain. |

### Institutional Knowledge Store (IKS)

| Method | Path | Description |
|:--- |:--- |:--- |
| `GET` | `/v1/knowledge` | List paginated, filtered knowledge entries (supports `limit`/`offset` and `concept_type` query parameters). |
| `POST` | `/v1/knowledge` | Write a new knowledge entry (deduplicated by hash). |
| `GET` | `/v1/knowledge/search` | Semantic k-NN search across knowledge entries. |
| `POST` | `/v1/knowledge/{id}/confirm` | Human-confirm a knowledge entry (promote to permanent memory). |
| `DELETE` | `/v1/knowledge/{id}` | Delete a knowledge entry (requires `?force=true` if confirmed). |

### Public Outward A2A Gateway

| Method | Path | Authentication | Description |
|:--- |:--- |:--- |:--- |
| `GET` | `/a2a/v1/company-agent-card.json` | Public; IP fixed-window limit | Return the published company agent card and persisted `info_cards`. |
| `GET` | `/a2a/v1/catalog/search?q={query}&limit={1..100}` | Public; IP fixed-window limit | Search the outward customer catalog. |
| `POST` | `/a2a/v1/catalog/import` | Bearer token (**Admin**) | Validate and atomically import CSV and/or QuickBooks JSON (with optional `company_key` for company-scoped offerings); each field is limited to 512 KiB. |
| `PUT` | `/a2a/v1/profile` | Bearer token (**Admin**) | Update the business name, description, model profile, advertised skills, or persisted `info_cards`. |

Malformed, empty, or oversized imports return RFC 9457 `BAD_REQUEST` and do not partially mutate catalog state. Rate-limit failures return RFC 9457 `RATE_LIMIT`.

### Template Store & Swarm Lifecycle

| Method | Path | Authentication | Description |
|:--- |:--- |:--- |:--- |
| `GET` | `/v1/engine/templates/catalog` | Bearer token | Fetch remote template catalog with guaranteed offline fallback. |
| `POST` | `/v1/engine/templates/install` | Bearer token (Admin) | Clone git repository, validate assets, stage filesystem transaction, merge MCP config, and register swarm agents. |
| `POST` | `/v1/engine/templates/import` | Bearer token (Admin) | Validate and atomically import local swarm bundle, workflows, and MCP servers. |
| `GET` | `/v1/engine/templates/installed` | Bearer token | List all installed swarm packages and asset summaries from `installed_manifest.json`. |
| `POST` | `/v1/engine/templates/uninstall` | Bearer token (Admin) | Safely unregister agents from database/registry, prune MCP servers, and archive or delete files. |

### System & Durable State Backups

| Method | Path | Authentication | Description |
|:--- |:--- |:--- |:--- |
| `GET` | `/v1/system/backup/status` | Bearer token (**Admin**) | Retrieve backup daemon health, authorized destination, passphrase status, and rolling retention metrics. |
| `POST` | `/v1/system/backup/configure` | Bearer token (**Admin**) | Update backup destination (`local` or `cloud_sync`), confined directory path, enabled status, and in-memory passphrase. |
| `POST` | `/v1/system/backup/trigger` | Bearer token (**Admin**) | Execute a synchronous non-blocking SQLite `VACUUM INTO` and `TADPOLE_SNAP_V2` (ChaCha20 + HMAC-SHA256, 210k PBKDF2-HMAC-SHA512) snapshot cycle. |
| `POST` | `/v1/system/backup/restore` | Bearer token (**Admin**) | Decrypt and verify snapshot integrity (`PRAGMA integrity_check`). Defaults to verify-only dry-run (`apply: false`); `apply: true` stages `data/state/tadpole.db.restored` + `workspaces.restored` for next restart promotion. |
| `GET` | `/v1/system/backup/history` | Bearer token (**Admin**) | List historical backup audit records from the `system_backups` ledger. |

### Additional Active Route Families (Full Schema in [`docs/openapi.yaml`](./openapi.yaml))
- **Remote Companion (`/v1/remote/*`)**: Admin-gated QR pairing token minting, device listing/revocation (`RequireAdmin`); paired Ed25519 signed requests for pending oversight, decision submission (`/v1/remote/oversight/decide`), and emergency mobile halt (`/v1/remote/agents/halt`).
- **Continuity & Workflows (`/v1/continuity/*`)**: Cron jobs & durable multi-step workflows (reads: Bearer; mutations: **Admin**).
- **Infrastructure & Models (`/v1/infra/*`, `/v1/api/pull`)**: Provider/model CRUD, probing, discovery, and pulls (reads: Bearer; mutations: **Admin**).
- **Skills & MCP (`/v1/skills/*`)**: Skill/workflow/hook/MCP registry and execution (reads: Bearer; mutations & MCP execute: **Admin**).
- **SMB Practice Sandbox (`/v1/business/*`)**: Isolated mock company workspaces (`Mock-1-Product`, `Mock-2-Service`), enquiries, approvals, fact renewal (`POST ?renew_facts=true`), and promotion gates.
- **Governance, Benchmarks & Intelligence (`/v1/governance/*`, `/v1/benchmarks/*`, `/v1/intelligence/*`, `/v1/system/{workspaces,environment,compute-profile}`)**: Role blueprints (**Admin**), mission benchmarks, symbol graph blast radius, and system inspection.

### 🧠 Mythos Engine Config (ModelConfig)

When registering or updating an agent or model node, the following parameters tune execution, quotas, pricing, and recurrent reasoning:

| Parameter | Type | Default | Description |
|:--- |:--- |:--- |:--- |
| `reasoningDepth` | `integer` | `1` | Max internal loops (1-16). |
| `actThreshold` | `float` | `0.95` | Confidence score for ACT halting. |
| `currentReasoningTurn` | `integer` | `0` | READ-ONLY: Current turn in the pulse logic. |
| `contextWindow` / `context_window` | `integer` | `128000` | Maximum token context capacity (`# / Context Field`). |
| `inputCostPerM` / `input_cost_per_m` | `float` | `0.0` | Custom cost per 1,000,000 prompt/input tokens in USD. |
| `outputCostPerM` / `output_cost_per_m` | `float` | `0.0` | Custom cost per 1,000,000 completion/output tokens in USD. |
| `rpm` | `integer` | `null` | Requests Per Minute throttle ceiling. |
| `tpm` | `integer` | `null` | Tokens Per Minute throughput ceiling. |

---

## 📄 Pagination

All list endpoints support cursor-free pagination:

| Parameter | Default | Max | Description |
|:--- |:--- |:--- |:--- |
| `page` | `1` | — | 1-indexed page number. |
| `per_page` | `25` | `100` | Items per page (Clamped 1-100). |

**Response Envelope**:
```json
{
  "data": [...],
  "total": 42,
  "_links": {
    "self": { "href": "/v1/agents?page=1", "method": "GET" },
    "next": { "href": "/v1/agents?page=2", "method": "GET" }
  }
}
```

---

## ⚠️ Error Protocol (RFC 9457)

Errors return `application/problem+json` for machine-readable remediation.

### Response Structure
```json
{
  "type": "https://tadpole.os/errors/budget-exhausted",
  "title": "Mission Budget Limit Exceeded",
  "status": 402,
  "detail": "The mission has spent $1.06, which exceeds the authorized cap of $1.00.",
  "instance": "/missions/m1/steps/14",
  "error_code": "BUDGET_EXHAUSTED"
}
```

### Registered Error Codes
| Status | Code | Meaning |
|:--- |:--- |:--- |
| `401` | `UNAUTHORIZED` | Invalid or missing `NEURAL_TOKEN` / `NEURAL_ADMIN_TOKEN`. |
| `402` | `BUDGET_EXHAUSTED` | Agent or mission budget quota reservation denied. |
| `403` | `FORBIDDEN` | Insufficient role (`NEURAL_TOKEN` used on a `RequireAdmin` route) or security policy denial (e.g., Privacy Shield blocking `/v1/engine/live-voice`). |
| `429` | `RATE_LIMITED` | Engine-level or provider-level rate limit bucket exhausted. |
| `503` | `OLLAMA_OFFLINE` | Local Ollama socket is down, prompting failover or manual restart. |
| `507` | `OOM_QUANT` | Provider VRAM OOM; suggestive of quantization fallback. |

---

> [!TIP]
> **Versioning Strategy**: All business logic is nested under `/v1/`. Root-level paths are legacy and will be deprecated. Always use the `/v1` prefix for new integrations.