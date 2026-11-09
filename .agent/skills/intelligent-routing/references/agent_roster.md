# Sovereign Agent Roster & Domain Routing Map (L3)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / intelligent-routing / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Routing ambiguity, agent mismatch, or uncoordinated multi-domain execution.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[INTELLIGENT_ROUTING]`)

---

## 1. Domain-to-Specialist Mapping Matrix

The routing catalog classifies prompts by domain and scope. Plane 1 specialists are supported by the generic runner; Plane 2 business personas below are planned profiles and must not be dispatched for live work:

| Domain Category | Primary Trigger Keywords | Primary Agent Specialist | Secondary / Co-Agent | Bound Skills |
|---|---|---|---|---|
| **System Architecture & Blast Radius** | `architecture`, `blast radius`, `audit`, `refactor`, `deep module`, `god object`, `hub` | `nexus-engineer` | `elon` (CTO) | [`improve-codebase-architecture`](../../improve-codebase-architecture/SKILL.md), [`code-review-graph`](../../code-review-graph/SKILL.md) |
| **Security, Auth & Zero Trust** | `security`, `auth`, `jwt`, `token`, `vault`, `cwrf`, `sanitize`, `vulnerability`, `pbkdf2` | `security-auditor` | `nexus-engineer` | [`vulnerability-scanner`](../../vulnerability-scanner/SKILL.md), [`red-team-tactics`](../../red-team-tactics/SKILL.md), [`pii-redaction`](../../pii-redaction/SKILL.md) |
| **Rust Backend & Server Hubs** | `rust`, `axum`, `tokio`, `server-rs`, `route`, `endpoint`, `channel`, `ipc`, `tauri` | `tadpole-backend-specialist` | `nexus-engineer` | [`rust-pro`](../../rust-pro/SKILL.md), [`clean-code`](../../clean-code/SKILL.md), [`server-management`](../../server-management/SKILL.md) |
| **Frontend UI/UX & Design Systems** | `react`, `ui`, `ux`, `tailwind`, `component`, `css`, `theme`, `animation`, `hick's law` | `tadpole-frontend-specialist` | `web-designer` | [`frontend-design`](../../frontend-design/SKILL.md), [`tailwind-patterns`](../../tailwind-patterns/SKILL.md), [`web-design-guidelines`](../../web-design-guidelines/SKILL.md) |
| **Database, SQLite & Vector Store** | `database`, `sqlite`, `sqlx`, `lancedb`, `vector`, `migration`, `wal`, `schema`, `index` | `database-architect` | `tadpole-backend-specialist` | [`database-design`](../../database-design/SKILL.md), [`domain-modeling`](../../domain-modeling/SKILL.md) |
| **Testing, Invariants & QA** | `test`, `vitest`, `invariant`, `witness`, `coverage`, `qa`, `assert`, `adg` | `qa-99` (Quality Auditor) | `test-engineer` | [`testing-patterns`](../../testing-patterns/SKILL.md), [`tdd-workflow`](../../tdd-workflow/SKILL.md), [`verify-changes`](../../verify-changes/SKILL.md) |
| **Debugging, Forensics & RCA** | `bug`, `crash`, `panic`, `error`, `memory leak`, `hang`, `regression`, `stack trace` | `debugger` | `nexus-engineer` | [`systematic-debugging`](../../systematic-debugging/SKILL.md), [`lint-and-validate`](../../lint-and-validate/SKILL.md) |
| **Internationalization (i18n)** | `i18n`, `l10n`, `translate`, `locale`, `rtl`, `bidi`, `icu`, `intl` | `utility-writer` | `tadpole-frontend-specialist` | [`i18n-localization`](../../i18n-localization/SKILL.md) |
| **Documentation & ADR Synthesis** | `documentation`, `readme`, `adr`, `sop`, `wiki`, `api spec`, `markdown` | `utility-writer` | `mission-analyst` | [`documentation-templates`](../../documentation-templates/SKILL.md) |
| **SMB Business Operations & Digital Twin (planned)** | `smb`, `business`, `digital twin`, `founder voice`, `onboarding`, `invoice`, `customer support`, `quickbooks`, `client`, `bookkeeper`, `catalog` | Preview classification only | No live business dispatch | [`digital-twin-voice`](../../digital-twin-voice/SKILL.md), [`smb-data-ingestion`](../../smb-data-ingestion/SKILL.md), [`pii-redaction`](../../pii-redaction/SKILL.md), [`domain-modeling`](../../domain-modeling/SKILL.md) |
| **Mobile & Touch Ecosystem** | `android`, `ios`, `compose`, `swiftui`, `mobile`, `touch`, `material 3` | `mobile-developer` | `tadpole-frontend-specialist` | [`mobile-design-android`](../../mobile-design-android/SKILL.md), [`mobile-design-ios`](../../mobile-design-ios/SKILL.md) |
| **Research, Scouting & Web Intelligence** | `scout`, `github`, `crawl`, `scrape`, `benchmarks`, `competitor`, `papers` | `explorer-scout` | `mission-analyst` | [`explorer-scout`](../../explorer-scout/SKILL.md), [`world-model-synthesis`](../../world-model-synthesis/SKILL.md), [`seo-fundamentals`](../../seo-fundamentals/SKILL.md) |
| **Multi-Agent Coordination & Strategy** | `orchestrate`, `plan`, `swarm`, `coordination`, `handoff`, `session state`, `roadmap` | `tadpole-alpha` (COO) | `agent-of-nine` (CEO) | [`coordinator-mode`](../../coordinator-mode/SKILL.md), [`parallel-agents`](../../parallel-agents/SKILL.md), [`plan-writing`](../../plan-writing/SKILL.md), [`handoff`](../../handoff/SKILL.md) |

---

## 2. Multi-Domain Routing Rules

1. **2+ Cross-Layer Domains**: When a task crosses layer boundaries (e.g. *"Create a new Rust endpoint and bind it to a React Zustand store with a test"*), escalate to **Coordinator / Orchestrator Mode**. Propose an implementation plan before writing code.
2. **Specialist Over Generalist**: If keywords match both a general domain (e.g. "code") and a specific domain (e.g. "security vulnerability"), the specific specialist (`security-auditor`) takes precedence over generalist personas.
3. **Explicit Mentions Override**: Explicit mentions of supported specialists (e.g. `@nexus-engineer` or `@qa-99`) override automatic keyword routing. Mentioning planned Plane 2 profiles (e.g. `@founder-twin`) does not authorize live business dispatch.
4. **Transparent Indication**: Always announce the applied specialists cleanly at the start of complex responses.
5. **FR-11 status**: A roster entry or plane label grants no permissions. The generic runner rejects only identities whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not activate the guard. Until company-scoped authorization and storage exist, use synthetic inputs and stop at preview.
