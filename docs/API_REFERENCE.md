# Tadpole OS — API Reference

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / API_REFERENCE
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

**Version**: 1.1.462

**Generated**: 2026-09-12 15:08:55
Welcome to the official API reference for the Tadpole OS Sovereign Engine. Protected endpoints require a valid `NEURAL_TOKEN` provided via the `Authorization: Bearer <token>` header. The public outward agent-card and catalog-search endpoints are token-free and enforce a 60-request-per-minute IP fixed window.

## Endpoints

### ImportOutwardCatalog

- **Endpoint**: `POST /a2a/v1/catalog/import`
- **Handler**: `pub async fn import_catalog_handler` in `outward_routes.rs`

Validates and atomically imports CSV or QuickBooks JSON payloads up to 512 KiB.

---

### SearchOutwardCatalog

- **Endpoint**: `GET /a2a/v1/catalog/search`
- **Handler**: `pub async fn search_catalog_handler` in `outward_routes.rs`

Searches the public customer catalog with a bounded result limit. No bearer token is required.

---

### GetOutwardAgentCard

- **Endpoint**: `GET /a2a/v1/company-agent-card.json`
- **Handler**: `pub async fn get_agent_card_handler` in `outward_routes.rs`

Returns the public A2A agent card. No bearer token is required; requests are IP rate limited.

---

### UpdateOutwardProfile

- **Endpoint**: `PUT /a2a/v1/profile`
- **Handler**: `pub async fn update_profile_handler` in `outward_routes.rs`

Updates the outward business profile, model profile, or advertised skills.

---

### GetAgents

- **Endpoint**: `GET /v1/agents`
- **Handler**: `pub async fn get_agents` in `agent.rs`

Retrieves the list of all registered agents in the swarm. Implements
HATEOAS-compliant pagination to allow for efficient UI rendering and discovery.

### 🛰️ Registry Introspection
This handler pulls directly from the engine's memory-mapped `AgentResponse`.
It maps raw back-end models into a clean, RESTful representation for
dashboard consumption.

---

### GetAgentMemory

- **Endpoint**: `GET /v1/agents/:agent_id/memories`
- **Handler**: `pub async fn get_agent_memory` in `memory.rs`

Retrieves semantic memories for a specific agent by scanning its local
workspace's LanceDB vector store.

---

### GetAgent

- **Endpoint**: `GET /v1/agents/:id`
- **Handler**: `pub async fn get_agent` in `agent.rs`

Retrieves the detailed state of a specific agent by its unique identifier.
Provides O(1) discovery for high-density swarms.

---

### ResetAgent

- **Endpoint**: `POST /v1/agents/:id/reset`
- **Handler**: `pub async fn reset_agent` in `agent.rs`

Resets an agent's failure count and returns it to idle status.
Used to clear "Self-heal cooldowns" after configuration fixes.

---

### SendTask

- **Endpoint**: `POST /v1/agents/:id/tasks`
- **Handler**: `pub async fn send_task` in `agent.rs`

Dispatches a high-level text task to a specific autonomous agent.
Automatically handles distributed trace propagation (via W3C `traceparent`)
and validates agent existence before dispatch.
Identical payloads for the same agent and `X-Request-Id` are accepted once
per 15-second window, including concurrent deliveries. Duplicates return
HTTP 202 with `duplicate: true` and do not start another runner.

### 🔦 Distributed Tracing (AGNT-01)
If a `traceparent` header is present in the UI request, it is parsed
and injected into the mission payload. This ensures that the engine's
background `AgentRunner` spans are correctly linked to the front-end
session in our Jaeger/OTel traces.

---

### GetTemplatesCatalog

- **Endpoint**: `GET /v1/engine/templates/catalog`
- **Handler**: `pub async fn get_templates_catalog` in `templates/mod.rs`

Fetches available templates from the remote repository index or falls back to offline catalog.

---

### ImportTemplate

- **Endpoint**: `POST /v1/engine/templates/import`
- **Handler**: `pub async fn import_template` in `templates/mod.rs`

Imports a locally staged swarm bundle with validation, namespacing, and atomic configuration merging.

---

### InstallTemplate

- **Endpoint**: `POST /v1/engine/templates/install`
- **Handler**: `pub async fn install_template` in `templates/mod.rs`

Clones a remote template repository and installs member agents, workflows, skills, and MCP configuration.

---

### GetInstalledTemplates

- **Endpoint**: `GET /v1/engine/templates/installed`
- **Handler**: `pub async fn get_installed_templates` in `templates/mod.rs`

Lists all currently installed swarms along with their member agents, workflows, and skills.

---

### UninstallTemplate

- **Endpoint**: `POST /v1/engine/templates/uninstall`
- **Handler**: `pub async fn uninstall_template` in `templates/mod.rs`

Safely deactivates agents, unregisters them from DB & state, archives or deletes files, and prunes MCP config.

---

### GetBlastRadius

- **Endpoint**: `GET /v1/intelligence/blast-radius`
- **Handler**: `pub async fn get_blast_radius` in `intelligence.rs`

Calculates the downstream impact of changing a specific symbol.

---

### GetCodeGraph

- **Endpoint**: `GET /v1/intelligence/graph`
- **Handler**: `pub async fn get_code_graph` in `intelligence.rs`

Returns the full high-fidelity symbol graph for visualization.

---

### DecideOversight

- **Endpoint**: `POST /v1/oversight/:id/decide`
- **Handler**: `pub async fn decide_oversight` in `oversight/ledger.rs`

Commits a human decision (Approve/Reject) for a specific oversight request.
Triggers the internal resolution channel to unblock or kill the agent task.

---

### GetOversightLedger

- **Endpoint**: `GET /v1/oversight/ledger`
- **Handler**: `pub async fn get_ledger` in `oversight/ledger.rs`

Provides a historical audit trail of all previous oversight decisions.
Directly queries the SQLite persistence layer with support for pagination.

---

### GetPendingOversight

- **Endpoint**: `GET /v1/oversight/pending`
- **Handler**: `pub async fn get_pending` in `oversight/ledger.rs`

Returns a collection of all actions (file edits, network requests, etc.)
currently paused and awaiting human verification.

---

### GetAuditTrail

- **Endpoint**: `GET /v1/oversight/security/audit-trail`
- **Handler**: `pub async fn get_audit_trail` in `oversight/security.rs`

Retrieves the tamper-evident Merkle hash-chain logs with accurate total count pagination.

---

### GetIntegrityStatus

- **Endpoint**: `GET /v1/oversight/security/integrity`
- **Handler**: `pub async fn get_integrity_status` in `oversight/security.rs`

Verifies the last N records in the Merkle chain and returns an integrity score.
Propagates database infrastructure errors cleanly instead of triggering false-positive tamper alarms.

---

### GetQuotas

- **Endpoint**: `GET /v1/oversight/security/quotas`
- **Handler**: `pub async fn get_security_quotas` in `oversight/quotas.rs`

Returns global budget telemetry, including total spent, remaining,
and system defense metrics.

---

### UpdateAgentQuota

- **Endpoint**: `PUT /v1/oversight/security/quotas/:entity_id`
- **Handler**: `pub async fn update_agent_quota` in `oversight/quotas.rs`

Updates the budget quota and reset period for a specific agent.

---

