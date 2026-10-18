# Sovereign Agent Roster & Domain Routing Map (L3)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / intelligent-routing / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Routing ambiguity, agent mismatch, or uncoordinated multi-domain execution.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[INTELLIGENT_ROUTING]`)

---

## 1. Domain-to-Specialist Mapping Matrix

The Sovereign Swarm routes prompts to specialized agent personas based on domain keywords and architectural scope:

| Domain Category | Primary Trigger Keywords | Primary Agent Specialist | Secondary / Co-Agent | Bound Skills |
|---|---|---|---|---|
| **System Architecture & Blast Radius** | `architecture`, `blast radius`, `audit`, `refactor`, `deep module`, `god object`, `hub` | `nexus-engineer` | `elon` (CTO) | [`improve-codebase-architecture`](file:///D:/TadpoleOS-Dev/.agent/skills/improve-codebase-architecture/SKILL.md), [`code-review-graph`](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-graph/SKILL.md) |
| **Security, Auth & Zero Trust** | `security`, `auth`, `jwt`, `token`, `vault`, `cwrf`, `sanitize`, `vulnerability`, `pbkdf2` | `security-auditor` | `nexus-engineer` | [`vulnerability-scanner`](file:///D:/TadpoleOS-Dev/.agent/skills/vulnerability-scanner/SKILL.md), [`red-team-tactics`](file:///D:/TadpoleOS-Dev/.agent/skills/red-team-tactics/SKILL.md), [`pii-redaction`](file:///D:/TadpoleOS-Dev/.agent/skills/pii-redaction/SKILL.md) |
| **Rust Backend & Server Hubs** | `rust`, `axum`, `tokio`, `server-rs`, `route`, `endpoint`, `channel`, `ipc`, `tauri` | `tadpole-backend-specialist` | `nexus-engineer` | [`rust-pro`](file:///D:/TadpoleOS-Dev/.agent/skills/rust-pro/SKILL.md), [`clean-code`](file:///D:/TadpoleOS-Dev/.agent/skills/clean-code/SKILL.md), [`server-management`](file:///D:/TadpoleOS-Dev/.agent/skills/server-management/SKILL.md) |
| **Frontend UI/UX & Design Systems** | `react`, `ui`, `ux`, `tailwind`, `component`, `css`, `theme`, `animation`, `hick's law` | `tadpole-frontend-specialist` | `web-designer` | [`frontend-design`](file:///D:/TadpoleOS-Dev/.agent/skills/frontend-design/SKILL.md), [`tailwind-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/tailwind-patterns/SKILL.md), [`web-design-guidelines`](file:///D:/TadpoleOS-Dev/.agent/skills/web-design-guidelines/SKILL.md) |
| **Database, SQLite & Vector Store** | `database`, `sqlite`, `sqlx`, `lancedb`, `vector`, `migration`, `wal`, `schema`, `index` | `database-architect` | `tadpole-backend-specialist` | [`database-design`](file:///D:/TadpoleOS-Dev/.agent/skills/database-design/SKILL.md), [`domain-modeling`](file:///D:/TadpoleOS-Dev/.agent/skills/domain-modeling/SKILL.md) |
| **Testing, Invariants & QA** | `test`, `vitest`, `invariant`, `witness`, `coverage`, `qa`, `assert`, `adg` | `qa-99` (Quality Auditor) | `test-engineer` | [`testing-patterns`](file:///D:/TadpoleOS-Dev/.agent/skills/testing-patterns/SKILL.md), [`tdd-workflow`](file:///D:/TadpoleOS-Dev/.agent/skills/tdd-workflow/SKILL.md), [`verify-changes`](file:///D:/TadpoleOS-Dev/.agent/skills/verify-changes/SKILL.md) |
| **Debugging, Forensics & RCA** | `bug`, `crash`, `panic`, `error`, `memory leak`, `hang`, `regression`, `stack trace` | `debugger` | `nexus-engineer` | [`systematic-debugging`](file:///D:/TadpoleOS-Dev/.agent/skills/systematic-debugging/SKILL.md), [`lint-and-validate`](file:///D:/TadpoleOS-Dev/.agent/skills/lint-and-validate/SKILL.md) |
| **Internationalization (i18n)** | `i18n`, `l10n`, `translate`, `locale`, `rtl`, `bidi`, `icu`, `intl` | `utility-writer` | `tadpole-frontend-specialist` | [`i18n-localization`](file:///D:/TadpoleOS-Dev/.agent/skills/i18n-localization/SKILL.md) |
| **Documentation & ADR Synthesis** | `documentation`, `readme`, `adr`, `sop`, `wiki`, `api spec`, `markdown` | `utility-writer` | `mission-analyst` | [`documentation-templates`](file:///D:/TadpoleOS-Dev/.agent/skills/documentation-templates/SKILL.md), [`digital-twin-voice`](file:///D:/TadpoleOS-Dev/.agent/skills/digital-twin-voice/SKILL.md) |
| **Mobile & Touch Ecosystem** | `android`, `ios`, `compose`, `swiftui`, `mobile`, `touch`, `material 3` | `mobile-developer` | `tadpole-frontend-specialist` | [`mobile-design-android`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-android/SKILL.md), [`mobile-design-ios`](file:///D:/TadpoleOS-Dev/.agent/skills/mobile-design-ios/SKILL.md) |
| **Research, Scouting & Web Intelligence** | `scout`, `github`, `crawl`, `scrape`, `benchmarks`, `competitor`, `papers` | `explorer-scout` | `mission-analyst` | [`explorer-scout`](file:///D:/TadpoleOS-Dev/.agent/skills/explorer-scout/SKILL.md), [`world-model-synthesis`](file:///D:/TadpoleOS-Dev/.agent/skills/world-model-synthesis/SKILL.md), [`seo-fundamentals`](file:///D:/TadpoleOS-Dev/.agent/skills/seo-fundamentals/SKILL.md) |
| **Multi-Agent Coordination & Strategy** | `orchestrate`, `plan`, `swarm`, `coordination`, `handoff`, `session state`, `roadmap` | `tadpole-alpha` (COO) | `agent-of-nine` (CEO) | [`coordinator-mode`](file:///D:/TadpoleOS-Dev/.agent/skills/coordinator-mode/SKILL.md), [`parallel-agents`](file:///D:/TadpoleOS-Dev/.agent/skills/parallel-agents/SKILL.md), [`plan-writing`](file:///D:/TadpoleOS-Dev/.agent/skills/plan-writing/SKILL.md), [`handoff`](file:///D:/TadpoleOS-Dev/.agent/skills/handoff/SKILL.md) |

---

## 2. Multi-Domain Routing Rules

1. **2+ Cross-Layer Domains**: When a task crosses layer boundaries (e.g. *"Create a new Rust endpoint and bind it to a React Zustand store with a test"*), escalate to **Coordinator / Orchestrator Mode**. Propose an implementation plan before writing code.
2. **Specialist Over Generalist**: If keywords match both a general domain (e.g. "code") and a specific domain (e.g. "security vulnerability"), the specific specialist (`security-auditor`) takes precedence over generalist personas.
3. **Explicit Mentions Override**: Explicit mentions in prompt (e.g. `@nexus-engineer` or `@qa-99`) immediately override automatic keyword routing.
4. **Transparent Indication**: Always announce the applied specialists cleanly at the start of complex responses.