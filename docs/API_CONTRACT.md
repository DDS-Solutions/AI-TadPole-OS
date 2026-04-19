> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 📄 API Contract: Tadpole OS Engine

![Status: Production](https://img.shields.io/badge/Status-Production-green)
![Version: 1.1.13](https://img.shields.io/badge/Version-1.1.13-blue)
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

| Status | Code | Meaning |
|:--- |:--- |:--- |
| `401` | `UNAUTHORIZED` | Invalid or missing `NEURAL_TOKEN`. |
| `429` | `RATE_LIMITED` | Engine-level token bucket exhausted. |
| `507` | `OOM_QUANT` | Provider VRAM OOM; suggestive of quantization fallback. |

---

> [!TIP]
> **Versioning Strategy**: All business logic is nested under `/v1/`. Root-level paths are legacy and will be deprecated. Always use the `/v1` prefix for new integrations.
