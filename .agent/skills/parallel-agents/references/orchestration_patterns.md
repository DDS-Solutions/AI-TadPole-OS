> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / parallel-agents
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Inter-agent context truncation or uncoordinated parallel writes.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[PARALLEL_AGENTS]`)

# Multi-Agent Orchestration Patterns & Directory (L3)

---

## 1. Specialist Agent Directory & Triggers

| Agent Name | Primary Specialty | Key Trigger Context |
|---|---|---|
| `orchestrator` | Swarm conductor & synthesis | Multi-domain initiatives & complex apps |
| `security-auditor` | Security analysis & threat modeling | Auth, tokens, policies, Zero Trust |
| `tadpole-backend-specialist` | Axum & Rust infrastructure | Routes, SQLite, channels, state hubs |
| `customer-backend-specialist`| Customer data & migration safety | PII, compatibility, client endpoints |
| `tadpole-frontend-specialist`| React & Tailwind UI | Components, themes, visual hierarchy |
| `test-engineer` | Verification & test suites | Vitest, unit tests, coverage gates |
| `debugger` | Root cause analysis (RCA) | Panic traces, 500 errors, state desyncs |

---

## 2. Advanced Multi-Agent Fan-Out Pattern

```text
Orchestrator
  ├── Fork Subagent A (Security Auditor) ➔ Scan routes for Zero Trust (Read-only)
  ├── Fork Subagent B (Performance Lead) ➔ Inspect hot database queries (Read-only)
  └── Coordinator Synthesis ➔ Merge findings into unified refactoring plan
```