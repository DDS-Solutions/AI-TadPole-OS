> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / API_CONTRACT
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 📄 API Contract: Tadpole OS Engine

![Status: Production](https://img.shields.io/badge/Status-Production-green)
![Version: 1.1.462](https://img.shields.io/badge/Version-1.1.462-blue)
![Auth: Bearer](https://img.shields.io/badge/Auth-Bearer_Token-emerald)

> **@docs ARCHITECTURE:ApiContract**

This is the minimalist REST contract for the Tadpole OS Engine. For high-level architecture, see the [Architecture Overview](./Architecture_Overview.md).

---

## ⚡ Quick Start (3 Steps)

1.  **Set Token**: `export NEURAL_TOKEN=your_secure_token`
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

| Method | Path | Description |
|:--- |:--- |:--- |
| `GET` | `/v1/engine/health` | Engine health & uptime pulse. |
| `POST` | `/v1/engine/deploy` | Trigger external deployment logic. |
| `POST` | `/v1/engine/kill` | Emergency shutdown of all active agent tasks. |
| `POST` | `/v1/engine/speak` | Text-to-Speech (TTS) synthesis. |
| `POST` | `/v1/engine/a2a/mailbox/send` | Queue an A2A envelope from a remote agent (verifies `X-A2A-Signature`). |

### Agents & Swarms

| Method | Path | Description |
|:--- |:--- |:--- |
| `GET` | `/v1/agents` | List all registered agents (Paginated). |
| `POST` | `/v1/agents` | Register a new agent node. |
| `GET` | `/v1/agents/{id}`| Retrieve a specific agent's schematic. |
| `PUT` | `/v1/agents/{id}`| Update agent configuration. |
| `POST` | `/v1/agents/{id}/tasks` | **Execute**: Dispatch a task to the agent runner. |
| `DELETE` | `/v1/agents/{id}` | De-register an agent node. |

### Oversight & Governance

| Method | Path | Description |
|:--- |:--- |:--- |
| `GET` | `/v1/oversight/pending` | List all actions awaiting human approval. |
| `POST` | `/v1/oversight/{id}/decide` | Approve or Reject a tool execution. |
| `GET` | `/v1/oversight/ledger` | Review historical governance decisions. |
| `GET` | `/v1/oversight/security/integrity` | Verify the Merkle Audit Chain. |

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
| `GET` | `/a2a/v1/company-agent-card.json` | Public; IP fixed-window limit | Return the published company agent card. `/a2a/v1/agent-card.json` is an alias. |
| `GET` | `/a2a/v1/catalog/search?q={query}&limit={1..100}` | Public; IP fixed-window limit | Search the outward customer catalog. |
| `POST` | `/a2a/v1/catalog/import` | Bearer token | Validate and atomically import CSV and/or QuickBooks JSON; each field is limited to 512 KiB. |
| `PUT` | `/a2a/v1/profile` | Bearer token | Update the business name, description, model profile, or advertised skills. |

Malformed, empty, or oversized imports return RFC 9457 `BAD_REQUEST` and do not partially mutate catalog state. Rate-limit failures return RFC 9457 `RATE_LIMIT`.

### Template Store & Swarm Lifecycle

| Method | Path | Authentication | Description |
|:--- |:--- |:--- |:--- |
| `GET` | `/v1/engine/templates/catalog` | Bearer token | Fetch remote template catalog with guaranteed offline fallback. |
| `POST` | `/v1/engine/templates/install` | Bearer token (Admin) | Clone git repository, validate assets, stage filesystem transaction, merge MCP config, and register swarm agents. |
| `POST` | `/v1/engine/templates/import` | Bearer token (Admin) | Validate and atomically import local swarm bundle, workflows, and MCP servers. |
| `GET` | `/v1/engine/templates/installed` | Bearer token | List all installed swarm packages and asset summaries from `installed_manifest.json`. |
| `POST` | `/v1/engine/templates/uninstall` | Bearer token (Admin) | Safely unregister agents from database/registry, prune MCP servers, and archive or delete files. |

### 🧠 Mythos Engine Config (ModelConfig)

When registering or updating an agent, the following parameters tune the recurrent reasoning behavior:

| Parameter | Type | Default | Description |
|:--- |:--- |:--- |:--- |
| `reasoningDepth` | `integer` | `1` | Max internal loops (1-16). |
| `actThreshold` | `float` | `0.95` | Confidence score for ACT halting. |
| `currentReasoningTurn` | `integer` | `0` | READ-ONLY: Current turn in the pulse logic. |

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
  "type": "https://tadpoleos.dev/errors/budget-exhausted",
  "title": "Mission Budget Limit Exceeded",
  "status": 403,
  "detail": "The mission has spent $1.06, which exceeds the authorized cap of $1.00.",
  "instance": "/missions/m1/steps/14",
  "error_code": "BUDGET_LIMIT_EXCEEDED"
}
```

### Registered Error Codes
| Status | Code | Meaning |
|:--- |:--- |:--- |
| `401` | `UNAUTHORIZED` | Invalid or missing `NEURAL_TOKEN`. |
| `403` | `BUDGET_LIMIT_EXCEEDED` | Mission budget allocation cap exceeded (PAUSED status). |
| `429` | `RATE_LIMITED` | Engine-level or provider-level rate limit bucket exhausted. |
| `503` | `OLLAMA_OFFLINE` | Local Ollama socket is down, prompting failover or manual restart. |
| `507` | `OOM_QUANT` | Provider VRAM OOM; suggestive of quantization fallback. |

---

> [!TIP]
> **Versioning Strategy**: All business logic is nested under `/v1/`. Root-level paths are legacy and will be deprecated. Always use the `/v1` prefix for new integrations.