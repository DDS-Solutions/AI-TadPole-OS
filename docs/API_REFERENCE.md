# 🛰️ API Reference: Tadpole OS Engine

> **Intelligence Level**: High-Fidelity (ECC Optimized)  
> **Status**: Verified Production-Ready  
> **Version**: 1.7.0  
> **Last Hardened**: 2026-04-10  
> **Classification**: Sovereign  

---

> [!IMPORTANT]
> **AI Assist Note (Tool-Use Guidance)**:
> - **Tracing**: All requests MUST propagate `traceparent` headers.
> - **Auth**: Use `Bearer <NEURAL_TOKEN>`.
> - **RFC 9457**: All errors follow the Problem Details specification.
> - **Tool Flow**: `POST /v1/agents/:id/tasks` -> `agent_runner` -> `tool_invocation` -> `Yield (Oversight)` -> `Result`.
> - **HATEOAS**: Browse resources via `_links` in response envelopes.

---

## 🏗️ 2026-04-10 Technical Hardening
The agent API contract has been hardened for **Level 5 AI Awakening**:
- **Swarm Pulse (Binary)**: 10Hz binary telemetry pipeline (`/ws`) for the Force-Graph visualizer.
- **Problem Details**: 100% RFC 9457 compliance across all error surfaces.
- **HATEOAS Discovery**: Navigable resource envelopes for Agents and Missions.
- **Oversight Persistence**: Transactional decision logging with Merkle verification.

---

---

## Table of Contents

- [🛰️ Standard Headers](#standard-headers)
- [📄 Pagination](#pagination)
- [🔌 REST API Endpoints](#rest-api--tadpole-os-engine-rust)
  - [Health & Control](#health--control)
  - [Agents](#agents)
  - [Oversight](#oversight)
  - [Infrastructure](#infrastructure)
  - [Skills & Workflows](#skills--workflows)
  - [Continuity Scheduler](#continuity-scheduler-autonomous-ai)
  - [Benchmarks](#benchmarks)
- [📡 WebSocket Events](#websocket-events)
- [⚠️ Error Handling](#error-format-rfc-9457)
- [🔑 Authentication](#authentication)

---

> Base URL: `http://localhost:8000` (Engine Control)
> API Base: `http://localhost:8000/v1` (Business Logic)

All endpoints (except health) require `Authorization: Bearer <NEURAL_TOKEN>`.

> [!TIP]
> **API Versioning**: All business endpoints are available under the `/v1/` prefix (e.g., `/v1/agents`). Root-level paths (e.g., `/agents`) are maintained for backwards compatibility but will be deprecated in a future release. New integrations should always use the `/v1/` prefix.
> **Engine Control Propagation**: Routes under `/engine/` (health, deploy, etc.) are also available under `/v1/engine/` for consistent client routing.

> **Hybrid REST Philosophy**: Tadpole OS utilizes a **Hybrid Level 2/3** maturity model. Every response adheres to RFC 9457 Problem Details. Core business resources (Agents, Missions) implement the **HATEOAS** (Hypermedia as the Engine of Application State) pattern, providing navigability via `_links` in response envelopes.
> **Backend Orchestration**: The engine initialization is decoupled into `startup.rs` (Background Tasks) and `router.rs` (API Table), ensuring maximum availability and maintainability.
> **Client-Side Proxy**: The frontend utilizes a domain-specific decoupled service layer (`agent_api_service`, `mission_api_service`, `system_api_service`) proxied through `tadpole_os_service` for clean separation of concerns.

---

## Standard Headers

Every response from the engine includes the following headers:

| Header | Description |
|--------|-------------|
| `X-Request-Id` | Unique UUID for end-to-end request tracing. Echoes the client-sent value if provided, otherwise generates a new one. |
| `X-RateLimit-Limit` | Maximum requests allowed per window. |
| `X-RateLimit-Remaining` | Remaining requests in the current window. |
| `X-RateLimit-Reset` | Unix timestamp when the rate limit window resets. |
| `Content-Type` | `application/json` for all API responses. |

---

## Pagination

All list endpoints support cursor-free pagination via query parameters:

| Parameter | Default | Max | Description |
|-----------|---------|-----|-------------|
| `page` | `1` | — | Page number (1-indexed). |
| `per_page` | `25` | `100` | Items per page. Clamped to 1–100. |

### Paginated Response Envelope

```json
{
  "data": [...],
  "page": 1,
  "per_page": 25,
  "total": 42,
  "total_pages": 2,
  "_links": {
    "self":  { "href": "/v1/agents?page=1&per_page=25", "method": "GET" },
    "first": { "href": "/v1/agents?page=1&per_page=25", "method": "GET" },
    "next":  { "href": "/v1/agents?page=2&per_page=25", "method": "GET" },
    "last":  { "href": "/v1/agents?page=2&per_page=25", "method": "GET" }
  }
}
```

> [!NOTE]
> `prev` link is included on pages > 1. `next` link is omitted on the last page.

### Paginated Endpoints

| Endpoint | Base Path |
|----------|----------|
| `GET /v1/agents` | `/v1/agents` |
| `GET /v1/oversight/pending` | `/v1/oversight/pending` |
| `GET /v1/oversight/ledger` | `/v1/oversight/ledger` |
| `GET /v1/skills/mcp-tools` | `/v1/skills/mcp-tools` |
| `GET /v1/benchmarks` | `/v1/benchmarks` |
| `GET /v1/infra/nodes` | `/v1/infra/nodes` |
| `GET /v1/infra/model-store/catalog` | `/v1/infra/model-store/catalog` |
| `GET /v1/oversight/security/quotas` | `/v1/oversight/security/quotas` |
| `GET /v1/oversight/security/audit-trail` | `/v1/oversight/security/audit-trail` |
| `GET /v1/continuity/workflows` | `/v1/continuity/workflows` |

---

## REST API — Tadpole OS Engine (Rust)

### Health & Control

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/engine/live-voice` | ✓ | Gemini Multimodal Live WebSocket |
| `POST` | `/v1/engine/deploy` | ✓ | Trigger Deployment |
| `POST` | `/v1/engine/kill` | ✓ | Kill All Agents |
| `POST` | `/v1/engine/shutdown` | ✓ | Graceful Shutdown |
| `POST` | `/v1/engine/speak` | ✓ | Text-to-Speech Synthesis |
| `POST` | `/v1/engine/transcribe` | ✓ | Audio Transcription |

### Agents

<!-- @docs-ref API_REFERENCE:GetAgents (agent.rs) -->
<!-- @docs-ref API_REFERENCE:SendTask (agent.rs) -->

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/agents` | ✓ | List all agents |
| `GET` | `/v1/agents/graph` | ✓ | Get Swarm Knowledge Graph |
| `GET` | `/v1/agents/{agent_id}/memories` | ✓ | Get Agent Memories |
| `POST` | `/v1/agents` | ✓ | Create a new agent |
| `POST` | `/v1/agents/{agent_id}/memories` | ✓ | Record New Memory |
| `POST` | `/v1/agents/{id}/mission` | ✓ | Sync Mission State |
| `POST` | `/v1/agents/{id}/pause` | ✓ | Pause Agent |
| `POST` | `/v1/agents/{id}/reset` | ✓ | Reset Agent Failure Count |
| `POST` | `/v1/agents/{id}/resume` | ✓ | Resume Agent |
| `POST` | `/v1/agents/{id}/tasks` | ✓ | Send a task to an agent |
| `PUT` | `/v1/agents/{id}` | ✓ | Update an agent |
| `DELETE` | `/v1/agents/{agent_id}/memories/{row_id}` | ✓ | Delete Memory Entry |
| `DELETE` | `/v1/agents/{id}` | ✓ | De-register an agent |

<!-- @docs-ref API_REFERENCE:GetAgentMemory (memory.rs) -->

> [!NOTE]
> **Memory Entry Contract**: Agent memory responses use `entries[].{id,text,mission_id,timestamp}` where `timestamp` is a Unix epoch number (seconds).

#### `POST /v1/agents` — Response

```
HTTP/1.1 201 Created
Location: /v1/agents/{id}
X-Request-Id: 550e8400-e29b-41d4-a716-446655440000
```

```json
{
  "status": "ok",
  "agent_id": "agent-42",
  "_links": {
    "self":       { "href": "/v1/agents/agent-42", "method": "GET" },
    "tasks":      { "href": "/v1/agents/agent-42/tasks", "method": "POST" },
    "collection": { "href": "/v1/agents", "method": "GET" }
  }
}
```

#### `POST /v1/agents/:id/tasks` — Request Body

```json
{
  "message": "Analyze security posture",
  "cluster_id": null,
  "department": "Security",
  "provider": "google",
  "model_id": "gemini-2.0-flash",
  "api_key": null,
  "base_url": null,
  "rpm": 15,
  "tpm": 1000000,
  "budget_usd": 5.0,
  "swarm_depth": 0,
  "swarm_lineage": [],
  "external_id": null,
  "analysis": true
}
```

> [!TIP]
> **Mission Analysis (Agent 99)**: Setting `analysis: true` triggers a specialized **Success Auditor** upon mission completion. The auditor uses LanceDB vector memory for **Cross-Mission Pattern Recognition**, **Semantic Pruning**, **Memory Deduplication**, and **Behavioral Drift** detection to provide top-tier insights and optimization suggestions.

#### System Prompt Architecture

When a task is dispatched to an agent via `/tasks`, the Rust engine **dynamically constructs** a system prompt server-side. The frontend does **not** send a system prompt in the payload — the engine builds one from the agent's registry data, mission state, and shared context files.

**Data Sources (in order of assembly):**

| Source | Description |
|--------|-------------|
| Agent Identity | Name, ID, Role, Department, Description from the agent registry |
| Hierarchy Level | Determined by `swarm_depth`: `0` = OVERLORD, `1` = ALPHA NODE, `2` = CLUSTER ALPHA, `3+` = AGENT |
| Mission Context | Shared findings from previous swarm steps (persisted in SQLite via `mission_steps`) |
| Recruitment Lineage | The full agent chain (e.g., `agent-1 -> agent-3 -> agent-7`) for context awareness |
| Skills & Workflows | The agent's registered skills, prioritized by the active **Neural Slot**'s configuration, used to scope behavior |
| Working Memory | Persistent reasoning scratchpad (Chain of Thought/Milestones) injected as `--- CURRENT WORKING CONTEXT ---` |
| Swarm Protocol | Anti-recursion rules, redundancy checks, hierarchy compliance, Aletheia deep analysis |
| `IDENTITY.md` | Global OS identity loaded from root `directives/IDENTITY.md` |
| `LONG_TERM_MEMORY.md` | Persistent swarm memory loaded from root `directives/LONG_TERM_MEMORY.md` |
| Safe Mode Suffix | If `safe_mode: true`, appends instructions disabling all execution tools |

> [!NOTE]
> The per-agent `system_prompt` field in `model_config` is stored in the registry and available via `PUT /v1/agents/:id`, but it is **not** appended to the generated prompt. The engine's synthesized prompt already provides role alignment through the agent's `role`, `department`, `description`, and the global `IDENTITY.md` context. The `system_prompt` field is reserved for future use or custom integrations.

> [!TIP]
> To customize agent behavior, update the agent's `role`, `department`, and `description` fields via `PUT /v1/agents/:id`. For global behavioral changes, edit `directives/IDENTITY.md` on the server. For persistent memory across missions, use `directives/LONG_TERM_MEMORY.md`.

#### 4.1 Security Governance Layer
Governance data is now managed by the **Budget Guard** with **Debounced Persistence**.

- **Endpoints**:
    - `GET /v1/security/quota`: Returns aggregate USD usage.
    - `GET /v1/security/budget`: Returns cluster-level budget status.

> [!NOTE]
> **Real-time Accuracy**: Quota reports aggregate both database-persisted costs and in-memory buffered usage from the debounced sync layer.

### Oversight

<!-- @docs-ref API_REFERENCE:GetPendingOversight (oversight.rs) -->
<!-- @docs-ref API_REFERENCE:GetOversightLedger (oversight.rs) -->
<!-- @docs-ref API_REFERENCE:DecideOversight (oversight.rs) -->

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/oversight/ledger` | ✓ | List recently decided oversight entries |
| `GET` | `/v1/oversight/pending` | ✓ | List pending oversight entries |
| `GET` | `/v1/oversight/security/audit-trail` | ✓ | Get Merkle audit logs |
| `GET` | `/v1/oversight/security/health` | ✗ | Security health pulse |
| `GET` | `/v1/oversight/security/integrity` | ✓ | Get Merkle integrity status |
| `GET` | `/v1/oversight/security/missions/quotas` | ✓ | List all mission quotas |
| `GET` | `/v1/oversight/security/policies` | ✓ | List dynamic tool permission policies |
| `GET` | `/v1/oversight/security/quotas` | ✓ | Get active resource quotas and system defense |
| `POST` | `/v1/oversight/{id}/decide` | ✓ | Approve or reject an oversight entry |
| `PUT` | `/v1/oversight/security/missions/{id}/quota` | ✓ | Update specific mission quota |
| `PUT` | `/v1/oversight/security/policies` | ✓ | Update dynamic tool permission policy |
| `PUT` | `/v1/oversight/settings` | ✓ | Update global oversight settings |

<!-- @docs-ref API_REFERENCE:GetIntegrityStatus (oversight.rs) -->
<!-- @docs-ref API_REFERENCE:GetQuotas (oversight.rs) -->
<!-- @docs-ref API_REFERENCE:GetAuditTrail (oversight.rs) -->
<!-- @docs-ref API_REFERENCE:UpdateAgentQuota (oversight.rs) -->

#### `GET /v1/oversight/security/quotas` — Response
<!-- @docs-ref API_REFERENCE:GetQuotas (oversight.rs) -->
Returns budget and hardware-level telemetry.

```json
{
  "budget": {
    "total_usd": 100.0,
    "remaining_usd": 85.5,
    "reset_at": "2026-04-01T00:00:00Z"
  },
  "system_defense": {
    "ram_pressure": 45.2,
    "cpu_load": 12.5,
    "sandbox_type": "Docker",
    "is_sandboxed": true
  }
}
```

#### `GET /v1/oversight/security/integrity` — Response
Returns the Result of the Merkle Chain health check.

```json
{
  "status": "SECURE",
  "integrity_score": 1.0,
  "verified_count": 50,
  "total_count": 50
}
```

#### `GET /v1/oversight/security/policies` — Response
Returns the collection of active tool permission policies from the SQLite database.

```json
[
  { "tool_name": "list_dir", "mode": "allow" },
  { "tool_name": "run_command", "mode": "prompt" }
]
```

#### `PUT /v1/oversight/security/policies` — Request Body
Updates or inserts a tool permission policy. Mode must be one of `allow`, `deny`, or `prompt`.

```json
{
  "tool_name": "run_command",
  "mode": "prompt"
}
```

### Infrastructure

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/infra/model-store/catalog` | ✓ | Get Model Catalog |
| `GET` | `/v1/infra/models` | ✓ | List Models |
| `GET` | `/v1/infra/nodes` | ✓ | List Nodes |
| `GET` | `/v1/infra/providers` | ✓ | List Providers |
| `POST` | `/v1/api/pull` | ✓ | Proxy Model Pull |
| `POST` | `/v1/infra/model-store/pull` | ✓ | Pull Model to Swarm Node |
| `POST` | `/v1/infra/nodes/discover` | ✓ | Discover Network Nodes |
| `POST` | `/v1/infra/providers/{id}/test` | ✓ | Test Provider Connectivity |
| `PUT` | `/v1/infra/models/{id}` | ✓ | Update a model entry |
| `PUT` | `/v1/infra/providers/{id}` | ✓ | Update a provider |
| `DELETE` | `/v1/infra/models/{id}` | ✓ | Delete a model entry |
| `DELETE` | `/v1/infra/providers/{id}` | ✓ | Delete a provider |

#### `GET /v1/infra/nodes` — Response
Returns a list of discovered and registered nodes in the swarm.

```json
{
  "data": [
    {
      "id": "bunker-alpha",
      "name": "Bunker Alpha",
      "address": "10.0.0.1:8000",
      "status": "online",
      "last_seen": "2026-03-21T14:30:00Z",
      "metadata": {
        "vram_gb": "24",
        "cpu_cores": "12"
      },
      "_links": {
        "self": { "href": "/v1/infra/nodes/bunker-alpha", "method": "GET" },
        "pull": { "href": "/v1/infra/model-store/pull", "method": "POST" }
      }
    }
  ]
}
```

#### `POST /v1/infra/model-store/pull` — Request Body
Proxies the pull command to the target node's local inference engine.

```json
{
  "node_id": "bunker-alpha",
  "model_name": "llama3:8b",
  "insecure": false
}
```

### Skills & Workflows

> [!TIP]
> **Agent-Written Skills (Sapphire Phase 2)**: Agents can autonomously propose new skills to the system using the `propose_skill` schema tool. Once approved by a human through the oversight dashboard, the JSON manifest is saved to root `execution/` and immediately available to the entire swarm via the `ScriptSkillsRegistry`.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/search/memory` | ✓ | Semantic Memory Search |
| `GET` | `/v1/skills` | ✓ | List all system skills |
| `DELETE` | `/v1/skills/hooks/{name}` | ✓ | Delete a system hook |

### Continuity Scheduler (Autonomous AI)
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/continuity/jobs` | ✓ | List all scheduled background tasks |
| `GET` | `/v1/continuity/jobs/{id}/runs` | ✓ | List job execution runs |
| `POST` | `/v1/continuity/jobs` | ✓ | Create a new scheduled job |
| `POST` | `/v1/continuity/jobs/{id}/disable` | ✓ | Disable an active scheduled job |
| `POST` | `/v1/continuity/jobs/{id}/enable` | ✓ | Enable an inactive scheduled job |
| `PUT` | `/v1/continuity/jobs/{id}` | ✓ | Update a scheduled job |
| `DELETE` | `/v1/continuity/jobs/{id}` | ✓ | Delete a scheduled job |

### Benchmarks

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/benchmarks` | ✓ | List all benchmark results |
| `GET` | `/v1/benchmarks/{test_id}` | ✓ | Get history for a specific benchmark test |
| `POST` | `/v1/benchmarks` | ✓ | Record a new benchmark result |
| `POST` | `/v1/benchmarks/{test_id}` | ✓ | Trigger a real-time benchmark |

### System & Knowledge
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET`  | `/v1/docs/knowledge` | ✓ | Lists all available system knowledge items (KIs). |
| `GET`  | `/v1/env-schema`     | ✓ | Returns the environment variable validation schema. |

### MCP (Model Context Protocol)

The engine provides a unified protocol for discovering and executing skills, including system tools, agent skills, and external services.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/skills/mcp-tools` | ✓ | List all available MCP tools |
| `POST` | `/v1/skills/mcp-tools/{name}/execute` | ✓ | Execute an MCP tool |

#### MCP Tool Response Envelope

All MCP tools return a standardized structure with integrated telemetry:

```json
{
  "name": "list_file_symbols",
  "description": "Lists all identifiers in a file",
  "input_schema": { ... },
  "source": "system",
  "stats": {
    "total_invocations": 42,
    "successful_invocations": 40,
    "failed_invocations": 2,
    "avg_latency_ms": 124.5,
    "last_executed_at": "2026-03-03T16:12:44Z"
  }
}
```

```json
{
  "content": [
    {
      "type": "text",
      "text": "Success: Result of tool execution..."
    }
  ],
  "is_error": false,
  "metadata": {
    "latency_ms": 45,
    "governance_audit_id": "audit-123"
  }
}
```

#### System Tools (A2A)

| Tool Name | Arguments | Description |
|-----------|-----------|-------------|
| `update_working_memory(content: string)` | `{ "memory": { ... } }` | Directly update the agent's persistent reasoning scratchpad. Useful for managing self-assigned milestones and preserving "Chain of Thought" across turns. |
| `recruit_specialist(agent_id: string, message: string)` | `{ "agent_id": "...", "message": "..." }` | Request assistance from a specialized agent within the same cluster. Delegates to the engine's `spawn_subagent` logic. |

### 2.2 Native Code Intelligence Tools (Hydra-RS)

These tools are built directly into the engine for zero-latency, token-efficient code analysis.

#### `list_file_symbols`
Lists all identifiers (functions, classes, structs) within a specific file.
- **Input**: `{ "path": "src/main.rs" }`
- **Output**: A formatted string containing `kind name -> signature`.

#### `get_symbol_body`
Retrieves the full source code body of a specific symbol.
- **Input**: `{ "path": "src/main.rs", "symbol_name": "AppState" }`
- **Output**: Raw source code of the symbol.

#### Lifecycle Hooks Governance

The engine periodically scans `server-rs/data/hooks` for subdirectories named `pre-tool` and `post-tool`. Any executable scripts within these directories are automatically executed before or after a tool call.


---

## WebSocket Events

Connect to `ws://localhost:8000/engine/ws` (Rust Engine WebSocket Hub).

### Server → Client

| Event Type | Payload | Description |
|------------|---------|-------------|
| `engine:health` | `{ uptime, active_agents, max_depth, tpm, recruit_count, timestamp }` | Heartbeat (every 5s) |
| `engine:mcp_pulse` | `{ tool_name, status, latency_ms, stats }` | Real-time tool telemetry pulse |
| `swarm:pulse` | `[Binary MessagePack]` | **The Pulse**: High-speed (10Hz) binary updates for the Swarm Visualizer (Engine Dashboard). Prefixed with header `0x02`. |
| `agent:create` | `{ agent_id, data: engine_agent }` | New agent registered in the live registry |
| `agent:status` | `{ agent_id, status, mission_id, current_task }` | Agent status change (thinking, idle, etc.) |
| `agent:message` | `{ agent_id, text, mission_id, message_id }` | Agent output text |
| `agent:update` | `{ agent_id, data: engine_agent }` | Full agent state sync |
| `agent:health` | `{ agent_id, status, failure_rate, throttled }` | Real-time health monitoring & throttling status |
| `oversight:new` | `{ entry: oversight_entry }` | New pending oversight request |
| `oversight:decision` | `{ id, decision }` | Oversight decision broadcast |
| `system:message` | `{ text, level }` | System-level notifications (info, warning, error, success) |
| `trace:span` | `{ id, name, parent_id, start_time, mission_id, attributes }` | New OTel-compliant span initialized |
| `trace:span_update` | `{ id, end_time, status, attributes }` | Span completion or update (set result/error) |

### Client → Server

WebSocket messages should be JSON with a `type` field:

```json
{
  "type": "agent:send",
  "agent_id": "2",
  "message": "Hello"
}
```

---

## Error Format (RFC 9457)

All errors follow the **Problem Details for HTTP APIs** format, providing top-tier observability and machine-readability:

```json
{
  "type": "https://tadpole.os/errors/404",
  "title": "Agent Not Found",
  "status": 404,
  "detail": "The agent with ID 'xyz' does not exist in the active swarm registry.",
  "instance": null
}
```

> [!NOTE]
> All errors are now returned as `(StatusCode, Json<ProblemDetails>)` ensuring strict compliance with high-end REST standards. The `instance` field is reserved for specific resource pointers.

---

## Authentication

All protected endpoints require a Bearer token:

```
Authorization: Bearer <NEURAL_TOKEN>
```

The token is configured via the `NEURAL_TOKEN` environment variable.

> [!IMPORTANT]
> **Production requirement**: `NEURAL_TOKEN` must be explicitly set. The engine panics at startup in release builds if the variable is missing — there is no insecure fallback. Development builds will log a loud warning and use a temporary placeholder, but this is not safe for any externally-accessible deployment.

---

## Static File Serving (Option B)

In production, the Rust engine serves the built React dashboard directly from the `dist/` directory on port `8000`. This eliminates the need for a separate static file server (Node.js `serve`, Nginx, etc.).

- **SPA Routing**: All unmatched paths fall back to `dist/index.html` for client-side routing.
- **Configuration**: Set `STATIC_DIR=/app/dist` in the environment (default: `./dist`).
- **Dev Mode**: If no `dist/` directory exists, static serving is disabled — use Vite's dev server on `:5173`.

---

## 5. CLI Operations

### 5.1 Mission Debriefing Tool (`execution/debrief_mission.py`)
Deterministic script for post-mission synthesis and institutional memory updates.

| Parameter | Type | Description | Default |
| :--- | :--- | :--- | :--- |
| `--mission_id` | `INTEGER` | Optional mission ID to debrief. | Latest mission |
| `--commit` | `FLAG` | If set, appends key learnings to `LONG_TERM_MEMORY.md`. | `false` |
| `--verbose` | `FLAG` | Enables detailed logging of LLM interactions. | `false` |

**Example Usage**:
```bash
python execution/debrief_mission.py --commit --verbose
```
