> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / intelligent-routing
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Routing ambiguity or agent mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[INTELLIGENT_ROUTING]`)

# Intelligent Routing Roster & Domain Map (L3)

---

## 1. Domain-to-Specialist Mapping Matrix

| Domain Category | Trigger Keywords | Primary Agent Specialist | Fallback / Co-Agent |
|---|---|---|---|
| **System Architecture / Audit** | audit, blast radius, AST graph, zero trust, CWRF | `nexus-engineer` | `security-auditor` |
| **Security & Auth** | authentication, JWT, token, vault, secret, sanitize | `security-auditor` | `tadpole-backend-specialist` |
| **Backend & Routes** | API, endpoint, Axum, Rust, SQLite, Tokio, channel | `tadpole-backend-specialist` | `nexus-engineer` |
| **Customer Integrations** | PII, migration, client compatibility, sync | `customer-backend-specialist` | `tadpole-backend-specialist` |
| **Frontend & UI/UX** | React, CSS, Tailwind, Zustand, component, theme | `tadpole-frontend-specialist` | `web-designer` |
| **Mobile & Touch** | Jetpack Compose, Android, iOS, touch, M3 | `mobile-developer` | `tadpole-frontend-specialist` |
| **Database & Schema** | SQLite, schema, migration, WAL, index, query | `database-architect` | `tadpole-backend-specialist` |
| **Testing & Verification** | unit test, vitest, cargo test, e2e, coverage | `test-engineer` | `debugger` |
| **Root Cause Debugging** | bug, crash, panic, 500 error, memory leak | `debugger` | `nexus-engineer` |
| **Performance Profiling** | slow, latency, flamegraph, throughput, jank | `performance-optimizer` | `nexus-engineer` |

---

## 2. Multi-Domain Routing Rules

1. **2+ Cross-Layer Domains**: Escalate to `orchestrator` with an implementation plan.
2. **Ambiguity Resolution**: Specific domain specialists take precedence over generalists (`security-auditor` > `backend-specialist`).
3. **Explicit Mentions**: Direct user mentions (e.g. `@debugger`) override automatic heuristics.