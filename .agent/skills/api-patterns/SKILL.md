---
name: api-patterns
description: API design principles and decision-making. REST vs GraphQL vs tRPC selection, response formats, versioning, pagination.
when_to_use: "When designing REST/GraphQL/tRPC APIs, defining response formats, versioning, pagination, or API authentication. NOT for UI/frontend work."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: high
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / api-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# API Patterns — Architectural Engineering Standards

> API design principles and decision-making.
> **Learn to THINK: select the right architectural paradigm for the specific deployment topology.**

---

## 🏛️ Tadpole OS Native API Architecture

In this codebase, API communication follows strict sovereign principles:
- **Core Engine Gateway**: High-performance Axum Rust router ([server-rs/src/router.rs](file:///d:/TadpoleOS-Dev/server-rs/src/router.rs)) handling `/v1/*` routes.
- **Contract as Source of Truth**: OpenAPI 3.1 specification maintained at [docs/openapi.yaml](file:///d:/TadpoleOS-Dev/docs/openapi.yaml) verified by `execution/parity_guard.py`.
- **Frontend Clients**: Strictly typed services located in `src/services/*_api*.ts` consuming typed contracts.
- **Envelope Standard**: Predictable JSON response envelopes (`success`, `data`, `error`).
- **Auth Model**: Bearer tokens (`ADMIN_TOKEN`, `NEURAL_TOKEN`, `OVERSIGHT_PUBLIC_KEY`).

---

## 🎯 Progressive Disclosure & L3 Topic Index

Read **REQUIRED** core standards below; consult deep **L3 Topic Guides** on demand:

| Guide / Tool | Topic & Scope | Trigger / When to Read |
|---|---|---|
| [`references/api-style.md`](./references/api-style.md) | REST vs GraphQL vs tRPC decision tree | Choosing API communication style |
| [`references/rest.md`](./references/rest.md) | Resource naming, HTTP verbs, status code selection | Designing REST endpoints |
| [`references/response.md`](./references/response.md) | Envelope pattern, error schema, pagination strategies | Structuring payload contracts |
| [`references/auth.md`](./references/auth.md) | JWT, Bearer token, Passkey/WebAuthn, API keys | Authentication & session security |
| [`references/versioning.md`](./references/versioning.md) | URI (`/v1`), Header, Query, sunset headers | Planning API lifecycle & evolution |
| [`references/rate-limiting.md`](./references/rate-limiting.md) | Token bucket, sliding window, 429 Retry-After headers | Protecting APIs from abuse/overload |
| [`references/documentation.md`](./references/documentation.md) | OpenAPI/Swagger best practices, schema components | Writing API specs & docs |
| [`references/security-testing.md`](./references/security-testing.md) | OWASP API Top 10, BOLA/IDOR, authz bypass checks | Pre-release security auditing |
| [`references/graphql.md`](./references/graphql.md) | Schema design, depth/complexity limiting | When evaluating GraphQL queries |
| [`references/trpc.md`](./references/trpc.md) | End-to-end TS monorepo type inference | When evaluating tRPC architectures |
| [`scripts/api_validator.py`](./scripts/api_validator.py) | Deterministic Axum and OpenAPI endpoint validator with `--test` suite | Automated API audit or self-test verification |

---

## 🔗 Related Sovereign Skills

| Need | Specialist Skill |
|---|---|
| Native Rust/Axum Implementation | [`rust-pro`](../rust-pro/SKILL.md) |
| Database Schemas & Migrations | [`database-design`](../database-design/SKILL.md) |
| API Vulnerability & Pen-Testing | [`vulnerability-scanner`](../vulnerability-scanner/SKILL.md) |
| Red Team & Auth Bypass Testing | [`red-team-tactics`](../red-team-tactics/SKILL.md) |

---

## ✅ API Design Decision Checklist

Before declaring an API contract complete:
- [ ] **Consumer Profile Identified**: Internal IPC, web client, subagent swarm, or external public integration?
- [ ] **Architecture Selected**: REST (Axum/OpenAPI for public/polyglot), WebSocket (telemetry/streaming)?
- [ ] **Resource Nouns Used**: Plural lowercase nouns (`/v1/agents`), no verbs in endpoint paths?
- [ ] **Response Envelope Standardized**: Consistent `{ success, data, error }` schema?
- [ ] **HTTP Status Code Precision**: 200 (read/update), 201 (created), 204 (deleted), 400 (bad request), 401/403 (auth/authz), 422 (validation), 429 (rate limit)?
- [ ] **Input Validation Enforced**: Strongly-typed extractors (`Json<T>`, `Query<T>`, Zod schemas)?
- [ ] **Observability Instrumented**: Structured logs with trace IDs, OpenTelemetry integration?

---

## 🚫 Critical Anti-Patterns to Avoid

- **No Verbs in URIs**: Never use `/api/getAgents` or `/api/deleteUser`. Use `GET /api/agents` and `DELETE /api/users/{id}`.
- **No Leaked Stack Traces**: Never return raw database errors or stack traces in HTTP responses. Mask internal errors behind structured error codes (`docs/ERROR_REGISTRY.json`).
- **No Unbounded Endpoints**: Every collection query MUST enforce a default `limit` and `offset`/cursor to prevent memory exhaustion.
- **No Raw Wildcard CORS in Production**: Enforce strict origin validation.

---

## 🛠️ Verification Gate

Run the optimized API validator script to audit endpoint hygiene across OpenAPI specs and Axum backend routes:

```powershell
# Run API endpoint best-practice audit
python .agent/skills/api-patterns/scripts/api_validator.py .

# Output machine-readable JSON report
python .agent/skills/api-patterns/scripts/api_validator.py . --json
```