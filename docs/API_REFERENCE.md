# API Reference — Tadpole OS Engine

> [!NOTE]
> **Status**: In-Progress / Parity Guard Enabled
> **Version**: 1.1.4
> **Last Updated**: 2026-03-20 (Verified 100% Quality Pass - Neural Health Update)
> **Documentation Level**: Professional (Antigravity Std)

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
> **Client-Side Proxy**: The frontend utilizes a domain-specific decoupled service layer (`AgentApiService`, `MissionApiService`, `SystemApiService`) proxied through `TadpoleOSService` for clean separation of concerns.

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
| `GET /v1/capabilities/mcp-tools` | `/v1/capabilities/mcp-tools` |
| `GET /v1/benchmarks` | `/v1/benchmarks` |
| `GET /v1/infra/nodes` | `/v1/infra/nodes` |
| `GET /v1/oversight/security/quotas` | `/v1/oversight/security/quotas` |
| `GET /v1/oversight/security/audit-trail` | `/v1/oversight/security/audit-trail` |
| `GET /v1/continuity/workflows` | `/v1/continuity/workflows` |

---

## REST API — Tadpole OS Engine (Rust)

### Health & Control

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET`  | `/v1/health` | ✗ | Returns `200 OK` + realtime diagnostics. |
| `GET`  | `/v1/engine/health` | ✗ | Alias for root health check. |
| `POST` | `/v1/engine/deploy` | ✓ | Triggers a production deployment via PowerShell. |
| `POST` | `/v1/engine/kill`   | ✓ | Halts all running agents. Server remains online. |
| `POST` | `/v1/engine/transcribe` | ✓ | Transcribes audio via Groq Whisper (Cloud). |
| `POST` | `/v1/engine/speak`      | ✓ | Synthesizes text via OpenAI (Cloud). |
| `POST` | `/v1/engine/shutdown` | ✓ | Graceful server shutdown. Persists state before exit. |
| `GET`  | `/v1/engine/ws`       | ✓ | Standard WebSocket Hub for real-time telemetry. |
| `POST` | `/v1/engine/templates/install` | ✓ | Installs a pre-configured swarm template. |

### Agents

<!-- @docs-ref API_REFERENCE:GetAgents (agent.rs) -->
<!-- @docs-ref API_REFERENCE:SendTask (agent.rs) -->

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/agents` | ✓ | Lists all agents (from DashMap + DB). |
| `POST` | `/v1/agents` | ✓ | Creates or registers a new agent. Returns `201` + `Location` header. |
| `POST` | `/v1/agents/:id/tasks` | ✓ | Dispatches a task resource to the agent. Returns `202 Accepted`. |
| `PUT` | `/v1/agents/:id` | ✓ | Updates agent configuration and fields. |
| `POST` | `/v1/agents/:id/pause` | ✓ | Pauses an active agent. |
| `POST` | `/v1/agents/:id/resume` | ✓ | Resumes an idle or paused agent. |
| `POST` | `/v1/agents/:id/reset` | ✓ | Resets an agent's failure count and returns it to idle status. |
| `POST` | `/v1/agents/:id/mission` | ✓ | Synchronizes local agent mission state. |
| `GET` | `/v1/agents/:agent_id/memories` | ✓ | Lists paginated long-term vector memory entries from LanceDB. |
| `DELETE` | `/v1/agents/:agent_id/memories/:row_id` | ✓ | Deletes a specific vector memory entry. |
| `POST` | `/v1/agents/:agent_id/memories` | ✓ | Manually records a new text memory for an agent (triggers embedding). |
| `GET` | `/v1/search/memory` | ✓ | Performs global semantic search across all agent memory store vectors. |

<!-- @docs-ref API_REFERENCE:GetAgentMemory (memory.rs) -->

#### `POST /v1/agents` — Response

```
HTTP/1.1 201 Created
Location: /v1/agents/{id}
X-Request-Id: 550e8400-e29b-41d4-a716-446655440000
```

```json
{
  "status": "ok",
  "agentId": "agent-42",
  "_links": {
    "self":       { "href": "/v1/agents/agent-42", "method": "GET" },
    "update":     { "href": "/v1/agents/agent-42", "method": "PUT" },
    "tasks":      { "href": "/v1/agents/agent-42/tasks", "method": "POST" },
    "memories":   { "href": "/v1/agents/agent-42/memories", "method": "GET" },
    "pause":      { "href": "/v1/agents/agent-42/pause", "method": "POST" },
    "resume":     { "href": "/v1/agents/agent-42/resume", "method": "POST" },
    "reset":      { "href": "/v1/agents/agent-42/reset", "method": "POST" },
    "collection": { "href": "/v1/agents", "method": "GET" }
  },
  "failureCount": 0,
  "lastFailureAt": null
}
```

#### `POST /v1/agents/:id/tasks` — Request Body

```json
{
  "message": "Analyze security posture",
  "clusterId": null,
  "department": "Security",
  "provider": "google",
  "modelId": "gemini-2.0-flash",
  "apiKey": null,
  "baseUrl": null,
  "rpm": 15,
  "tpm": 1000000,
  "budgetUsd": 5.0,
  "swarmDepth": 0,
  "swarmLineage": [],
  "externalId": null,
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
| Hierarchy Level | Determined by `swarmDepth`: `0` = OVERLORD, `1` = ALPHA NODE, `2` = CLUSTER ALPHA, `3+` = AGENT |
| Mission Context | Shared findings from previous swarm steps (persisted in SQLite via `mission_steps`) |
| Recruitment Lineage | The full agent chain (e.g., `agent-1 -> agent-3 -> agent-7`) for context awareness |
| Skills & Workflows | The agent's registered capabilities, prioritized by the active **Neural Slot**'s configuration, used to scope behavior |
| Swarm Protocol | Anti-recursion rules, redundancy checks, hierarchy compliance, Aletheia deep analysis |
| `IDENTITY.md` | Global OS identity loaded from root `directives/IDENTITY.md` |
| `LONG_TERM_MEMORY.md` | Persistent swarm memory loaded from root `directives/LONG_TERM_MEMORY.md` |
| Safe Mode Suffix | If `safeMode: true`, appends instructions disabling all execution tools |

> [!NOTE]
> The per-agent `systemPrompt` field in `modelConfig` is stored in the registry and available via `PUT /v1/agents/:id`, but it is **not** appended to the generated prompt. The engine's synthesized prompt already provides role alignment through the agent's `role`, `department`, `description`, and the global `IDENTITY.md` context. The `systemPrompt` field is reserved for future use or custom integrations.

> [!TIP]
> To customize agent behavior, update the agent's `role`, `department`, and `description` fields via `PUT /v1/agents/:id`. For global behavioral changes, edit `directives/IDENTITY.md` on the server. For persistent memory across missions, use `directives/LONG_TERM_MEMORY.md`.


### Oversight

<!-- @docs-ref API_REFERENCE:GetPendingOversight (oversight.rs) -->
<!-- @docs-ref API_REFERENCE:GetOversightLedger (oversight.rs) -->
<!-- @docs-ref API_REFERENCE:DecideOversight (oversight.rs) -->

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET`  | `/v1/oversight/pending` | ✓ | Lists pending oversight entries awaiting approval. |
| `GET`  | `/v1/oversight/ledger`  | ✓ | Lists recently decided oversight entries (bounded). |
| `POST` | `/v1/oversight/:id/decide` | ✓ | Approves or rejects a pending entry. |
| `PUT`  | `/v1/oversight/settings` | ✓ | Updates global governance settings (e.g. `autoApproveSafeSkills`). |
| `GET`  | `/v1/oversight/security/integrity` | ✓ | Returns the global Merkle integrity score and verification status. |
| `GET`  | `/v1/oversight/security/quotas` | ✓ | Lists active resource quotas, budget usage, and system defense metrics. |
| `GET`  | `/v1/oversight/security/audit-trail` | ✓ | Retrieves the tamper-evident Merkle hash-chain logs with per-record verification status. |
| `PUT`  | `/v1/oversight/security/quotas/:id` | ✓ | Updates budget quotas for a specific agent. |
| `POST` | `/v1/oversight/security/verify` | ✓ | Triggers a cryptographic verification of the entire audit chain. |

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
  "systemDefense": {
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
  "status": "Verified",
  "score": 100.0,
  "chain_length": 1450,
  "verified_at": "2026-03-03T16:12:44Z"
}
```

### Infrastructure

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/infra/providers` | ✓ | Lists all LLM providers. |
| `PUT` | `/v1/infra/providers/:id` | ✓ | Updates a provider (key, URL, protocol). |
| `POST` | `/v1/infra/providers/:id/test` | ✓ | Performs a real-time connectivity handcheck (Test Trace). Requires **Secure API Key** and **Hybrid Protocol** to be configured. |
| `GET` | `/v1/infra/models` | ✓ | Lists all registered models. |
| `PUT` | `/v1/infra/models/:id` | ✓ | Updates a model entry. |
| `GET` | `/v1/infra/nodes` | ✓ | Lists all registered Bunker nodes. |
| `POST` | `/v1/infra/nodes/discover` | ✓ | Triggers a local network discovery scan for new Bunkers. |

### Skills & Workflows

> [!TIP]
> **Agent-Written Skills (Sapphire Phase 2)**: Agents can autonomously propose new capabilities to the system using the `propose_capability` schema tool. Once approved by a human through the oversight dashboard, the JSON manifest is saved to root `execution/` and immediately available to the entire swarm via the `CapabilitiesRegistry`.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET`  | `/v1/capabilities`     | ✓ | Lists all dynamic capabilities from `SkillRegistry`. |
| `GET`  | `/v1/capabilities/skills/:name`     | ✓ | Retrieves a specific skill manifest. |
| `PUT`  | `/v1/capabilities/skills/:name` | ✓ | Creates or updates a skill (Manifest validation). |
| `DELETE`| `/v1/capabilities/skills/:name`| ✓ | Deletes a skill by name. |
| `GET`  | `/v1/capabilities/workflows/:name`     | ✓ | Retrieves a specific workflow manifest. |
| `PUT`  | `/v1/capabilities/workflows/:name`| ✓ | Creates or updates a workflow by name. |
| `DELETE`| `/v1/capabilities/workflows/:name`| ✓ | Deletes a workflow by name. |
| `GET` | `/v1/skills` | ✓ | Lists all specialized agent skills. |
| `GET` | `/v1/skills/:name` | ✓ | Retrieves a specific agent skill. |

### Continuity Scheduler (Autonomous AI)
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET`  | `/v1/continuity/jobs` | ✓ | Lists all scheduled background tasks. |
| `POST` | `/v1/continuity/jobs` | ✓ | Creates a new scheduled job (Cron pattern, `budget_usd` required). |
| `GET`  | `/v1/continuity/jobs/:id` | ✓ | Retrieves a specific scheduled job by ID. |
| `PUT` | `/v1/continuity/jobs/:id` | ✓ | Updates an existing scheduled job. |
| `DELETE`| `/v1/continuity/jobs/:id`| ✓ | Deletes a scheduled job. |
| `GET`| `/v1/continuity/jobs/:id/runs`| ✓ | Lists Historical execution runs for a specified scheduled job. |
| `POST`| `/v1/continuity/jobs/:id/enable`| ✓ | Enables an inactive scheduled job. |
| `POST`| `/v1/continuity/jobs/:id/disable`| ✓ | Disables an active scheduled job without deleting it. |
| `GET`  | `/v1/continuity/workflows` | ✓ | Lists all registered autonomous workflows. |
| `PUT`  | `/v1/continuity/workflows/:name` | ✓ | Creates or updates a multi-step workflow. |
| `POST` | `/v1/continuity/workflows/:name/steps` | ✓ | Appends or updates a step in an existing workflow. |
| `DELETE`| `/v1/continuity/workflows/:name`| ✓ | Deletes a workflow definition. |

### Benchmarks

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/v1/benchmarks` | ✓ | Lists all historical benchmark results. |
| `GET` | `/v1/benchmarks/:test_id` | ✓ | Returns history for a specific test ID (for comparison). |
| `POST` | `/v1/benchmarks` | ✓ | Records a new benchmark result manually. |
| `POST` | `/v1/benchmarks/run/:test_id` | ✓ | Triggers a real-time benchmark execution. |

### System & Knowledge
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET`  | `/v1/docs/knowledge` | ✓ | Lists all available system knowledge items (KIs). |
| `GET`  | `/v1/env-schema`     | ✓ | Returns the environment variable validation schema. |

### MCP (Model Context Protocol)

The engine provides a unified protocol for discovering and executing capabilities, including system tools, agent skills, and external services.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET`  | `/v1/capabilities/mcp-tools` | ✓ | Lists all available tools from the `McpHost` registry. Aggregates system, legacy, and native capabilities. Includes real-time telemetry `stats`. |
| `POST` | `/v1/capabilities/mcp-tools/:name/execute` | ✓ | Directly executes an MCP tool by name. Used by the interactive **Tool Lab**. |

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
  "isError": false,
  "metadata": {
    "latency_ms": 45,
    "governance_audit_id": "audit-123"
  }
}
```

#### System Tools (A2A)

| Tool Name | Arguments | Description |
|-----------|-----------|-------------|
| `recruit_specialist` | `{ "agentId": "...", "message": "..." }` | Standardized agent-to-agent recruitment. Delegates to the engine's `spawn_subagent` logic. |

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
| `engine:health` | `{ uptime, activeAgents, maxDepth, tpm, recruitCount, timestamp }` | Heartbeat (every 5s) |
| `engine:mcp_pulse` | `{ tool_name, status, latency_ms, stats }` | Real-time tool telemetry pulse |
| `agent:status` | `{ agentId, status }` | Agent status change (thinking, idle, etc.) |
| `agent:message` | `{ agentId, text }` | Agent output text |
| `agent:update` | `{ agentId, data: EngineAgent }` | Full agent state sync |
| `agent:health` | `{ agentId, status, failureRate, throttled }` | Real-time health monitoring & throttling status |
| `oversight:new` | `{ entry: OversightEntry }` | New pending oversight request |
| `oversight:decision` | `{ id, decision }` | Oversight decision broadcast |
| `system:message` | `{ text, level }` | System-level notifications (info, warning, error, success) |

### Client → Server

WebSocket messages should be JSON with a `type` field:

```json
{
  "type": "agent:send",
  "agentId": "2",
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
