> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / TODO
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[TODO]`)

# Tadpole OS Technical Debt & Backlog

## Maintenance & Hygiene [COMPLETE]
- [x] Prune unused imports across the `server-rs` project.
- [x] Modularize `AppState` hubs into `src/state/hubs/`.
- [x] Simplify `router.rs` with extracted middleware and sub-routers.
- [x] Standardize `execution/` scripts with `[OK]` / `[FAIL]` reporting.
- [x] Clean root directory (moved `.log`, `.txt`, `.py`, `.ps1` to appropriate subdirs).

## Upcoming Features
- [x] Implement advanced RAG weighting for mission clusters.
- [ ] Add support for local ONNX embeddings in the engine.

## Market Readiness & Hardening
- [x] **TrustGraph & Semantic Entity Linking (GraphRAG):** Bridge the limits of vector-only LanceDB RAG to support cross-mission entity-relation tracking and deep structural reasoning.
- [x] **Swarm Memory Deduplication:** Track and deduplicate recursive operational context patterns to dramatically prune context window bloat and reduce token cost/latency.
- [x] **Interactive Oversight Gate (HITL):** Implement a WebSocket-backed approval/authorization ledger requiring signed user confirmation before running sensitive tools (e.g., shell sidecars, filesystem mutation, heavy spending).
- [x] **Agent-Specific Role-Based Access Control (RBAC):** Move beyond static blacklists to define granular security policies isolated per agent (e.g., individual filesystem, API, or shell tool access boundaries).
- [x] **Dynamic Provider Failover & Hot-Swapping:** Implement dynamic slot routing to seamlessly fall back to secondary local/cloud models when a target provider (like a local Ollama socket) fails or rate-limits.
- [x] **Distributed OpenTelemetry (OTel) Tracing:** Inject request-lifespan trace IDs (`X-Request-Id`) across asynchronous boundary steps for visualization in Jaeger or Datadog.
- [x] **RFC 9457 UI Remediation Pathways:** Map granular backend `ProblemDetails` error codes in React stores to trigger interactive self-healing instructions directly on the dashboard.
- [x] **Strict TypeScript Type Hardening:** Push frontend type-safety from ~50% to 95%+ strict type compliance to eliminate silent state-management errors.

## Nexus Architectural Refactoring [COMPLETE]
- [x] **Command Processor Refactor** (`src/logic/command_processor.ts`): Transition from monolithic dispatcher to Strategy Pattern / Plug-in Architecture.
- [x] **State Management Hardening** (`src/stores/workspace_store.ts` & `provider_store.ts`): Extract side-effects and API logic into dedicated Service Layers.
- [x] **Backend Orchestration Modularization** (`server-rs/src/agent/runner/synthesis.rs`): Decompose the synthesis cycle into discrete, trait-based modules.
- [x] **Layout Logic Extraction** (`src/layouts/Dashboard_Layout.tsx`): Move global state glue and event listeners into HOCs or specialized hooks.
- [x] **Documentation Engine Migration** (`src/pages/Docs.tsx`): Transition to a Markdown-driven content management system.

## 🏢 Sovereign Process Digital Twin — Feature Initiative

> **Summary**: Tadpole OS is architecturally well-suited to mirror an existing operational
> system or enterprise as a living digital twin — shadowing roles, ingesting telemetry,
> and running autonomous processes in a verified runtime. The swarm model, oversight
> system, continuity scheduler, and governance blueprints already cover the core
> primitives. The missing layer is **data connectivity** and **observation-first (mirror) mode**
> blueprints.

### ✅ What Already Works (No Build Required)
- Agent `role` + `department` fields map 1:1 to an org chart
- `RoleBlueprint` governance templates = reusable role definitions per business type
- Oversight gate = human approval chains (mirrors real approval flows)
- Continuity Scheduler (`ScheduledJob`) = recurring business processes (daily reports, weekly syncs)
- Budget tracking per agent = cost-center modeling
- `docs/knowledge/` = business knowledge base / SOPs
- Merkle audit trail = non-repudiation for business decisions
- Community template ecosystem = pre-built industry archetypes (Marketing, Legal, Finance)

### 🔨 What Needs to Be Built

#### Phase 1 — Remote MCP Server Blueprints (Data Connector Templates)
> **Architecture Pivot**: These blueprints will be developed in the remote `Tadpole-OS-Industry-Templates` repository, not baked into the core Tadpole OS engine.
- [x] **Database & CRM/SaaS Blueprint** — Template MCP server for querying relational/document databases, lightweight CRMs, and SMB accounting tools
- [x] **REST / Webhook Ingestion Blueprint** — Template MCP server for real-time web services, sync operations, and messaging event buses
- [x] **Local Filesystem & Log Scanner Blueprint** — Template MCP server for parsing flat-file repositories, operational logs, and system outputs
- [x] **Hardware & Edge Device (RS-232/USB/TCP) Blueprint** — Template MCP server for local serial interfaces, Modbus registers, and physical telemetry data streams
  > Each connector blueprint = one MCP server template that agents query as a tool. Plugs into the existing
  > `mcp_config.json` + `McpConfig` system. No Rust changes needed — pure Python/Node MCP servers.

#### Phase 2 — Mirror Mode (Passive Observation)
- [x] Add `mirror_mode: bool` flag to `AppConfig` / `.env`
- [x] When `mirror_mode = true`, agents consume connector data but **do not act** unless
  drift from expected state is detected
- [x] Drift detection: compare real connector data against agent's expected state, emit
  `oversight:drift` WebSocket event for human review
- [x] Add `GET /v1/engine/mirror/status` route returning current drift alerts

#### Phase 3 — Scenario Branching ("What-If Fork")
- [ ] SQLite state snapshot: `POST /v1/engine/snapshot` → dumps current DB + DashMap to
  a named checkpoint file
- [ ] Fork endpoint: `POST /v1/engine/snapshot/{id}/fork` → restores snapshot into an
  isolated in-memory state for simulation
- [ ] Scenario runs are sandboxed — no real tool calls (mock mode enforced)
- [ ] `GET /v1/engine/scenarios` → compare KPIs between live state and scenario state

#### Phase 4 — KPI Dashboard Layer
- [ ] Define a `KpiDefinition` schema: `{ id, name, formula, source_tool, target }` 
- [ ] Extend `continuity` scheduler to run KPI evaluations on a cron cadence
- [ ] Emit KPI snapshots to the WebSocket telemetry channel (`0x03` header prefix)
- [ ] Frontend: new **📊 Process Intelligence** page with trend lines per KPI

#### Phase 5 — Human-to-Agent Identity Mapping
- [x] Add `shadows_human_id: Option<String>` to `EngineAgent`
- [x] Extend agent config panel with "Shadows Real Person" field
- [ ] Store historical decisions per human ID — agent learns from actual human patterns
  via working memory accumulation

#### Phase 6 — General Compliance & Validation Framework
- [ ] **Edge & Instrumentation Audit Integration** — Connect device/sensor telemetry streams to the cryptographically sealed Merkle audit trail (ALCOA+).
- [ ] **Compliance Directive Templates & Validation Packages** — Pre-written compliance templates, risk models (FMEA), and OQ/PQ test scripts to package as a pre-built Software Validation Package (SVP) for rapid regulatory auditor accreditation sign-off (ISO 9001 / ISO 17025 / GxP).

### 📋 Notes & Design Decisions
- **Start with Phase 1** — connector blueprints unlock everything else. Even without mirror mode,
  agents with custom database and service access become immediately useful for developers and system managers.
- **MCP-first approach** is correct — avoids Rust recompile per integration. Third-party
  APIs change frequently; MCP servers are independently deployable.
- **Template repo** (`AI-Tadpole-OS-Industry-Templates`) should eventually ship
  pre-built MCP configs for common setups to accelerate custom deployment.
- **Scenario fork should be read-only by default** — the real production swarm must
  never be contaminated by simulation state.
- **Compliance angle**: Digital twin audit trail (Merkle chain) doubles as a compliance
  record for regulated industries (finance, healthcare, manufacturing, testing).