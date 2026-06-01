> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[ROADMAP]` in audit logs.
>
> ### AI Assist Note
> 🗺️ AI-Tadpole-OS Roadmap
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🗺️ Tadpole OS Roadmap

AI-Tadpole-OS is evolving from a high-performance local engine into a fully autonomous, sovereign multi-agent swarm platform. This roadmap outlines our journey through strategic hardening and advanced capability expansion.

## 🟢 Phase 1: Core Engine & Sovereign Intelligence (Current)
- [x] **High-Performance Rust Backend**: Axum-based engine with lock-step phase transitions.
- [x] **Local-First Memory**: LanceDB vector store with Gemini `text-embedding-004`.
- [x] **Reactive Dashboard**: Real-time telemetry and Agent Management UI.
- [x] **Modular Skills (MCP)**: Native Support for Model Context Protocol.
- [x] **Voice Synthesis/Recognition**: Piper STT/TTS with Bunker caching.

## 🟡 Phase 2: Autonomous Continuity & Swarm Governance
- [x] **Intelligent Model Registry & Capability Sync** `IMR-01`: Dynamic provider model discovery (Ollama, OpenAI, Groq, Google, etc.) with automatic capability enrichment (tool support, vision, context window, embedding detection). Enables UI-level enforcement and intelligent swarm routing. Full local/self-hosted model support. → [Spec](directives/features/intelligent-model-registry.md)
- [x] **Continuity Scheduler**: Background swarm execution with budget-capped cron jobs. Cron-driven `start_scheduler` polls every 60s with atomic agent claim, workflow branching, budget enforcement, and auto-disable on max-failure threshold.
- [x] **Unified Oversight Gate**: WebSocket-backed HITL approval/authorization ledger with signed user confirmation for sensitive tools (Shell, Budget, filesystem). Includes Agent-Specific RBAC with per-agent security policy isolation.
- [x] **GraphRAG Integration**: TrustGraph-inspired entity-relation retrieval bridging vector-only RAG limits for cross-mission structural reasoning.
- [ ] **Flow-based Ingestion**: Asynchronous data processing pipelines for high-volume context.
- [ ] **Procedural Template Ecosystem**: "One-Click" deployment templates for specialized agent clusters.
- [ ] **Sovereign STT Improvements**: Higher precision Whisper integration.
- [ ] **Unified Communication Bridge**: Real-time notifications and HITL via Slack/Discord MCP.
- [x] **Swarm Memory Deduplication**: Cross-mission pattern recognition to prune recursive context bloat and reduce token cost/latency.
- [x] **Dynamic Provider Failover & Hot-Swapping**: Dynamic slot routing with seamless fallback to secondary local/cloud models on failure or rate-limit.

## 🟡 Phase 3: High-Fidelity Observability & Distributed Swarms
- [x] **Advanced Swarm Observability**: Industry-standard telemetry (Observe-Call-Audit) for mission auditing.
- [x] **Swarm Collaboration Infrastructure**: Persistent directives, hierarchical delegation, and peer-led audit loops.
- [x] **Global Swarm Memory (Vault)**: Cross-mission intelligence sharing via persistent vector store.
- [x] **Autonomous Skill Synthesis**: Agent-driven tool creation and dynamic registry enrollment.
- [ ] **P2P Swarm Network**: Multi-node coordination across local networks (Bunker Mesh).
  > **Foundation already built** — mDNS-SD peer discovery (`SwarmDiscoveryManager`) is running at boot, auto-registering peers via `_tadpole._tcp.local.` into `registry.nodes`. `AddressResolver` handles local/Docker/gateway routing. What remains:
  > - **`POST /v1/infra/nodes/discover`**: Replace hardcoded stub with real mDNS-triggered scan + health check against discovered peer addresses.
  > - **Knowledge Sync Endpoint**: `GET /v1/knowledge/sync?since=<timestamp>` — passive pull-based replication using `registry.nodes` address map. Requires IKS to exist first.
  > - **Gossip Relay Layer**: Active push of `gossip:knowledge_update` events to known peer addresses when a new KS entry is written. Uses `content_hash` for conflict-free dedup across nodes.
  > - **Peer Health Monitor**: Periodic liveness check of all nodes in `registry.nodes`; mark stale after missed heartbeats; emit `swarm:node_lost` WebSocket event.
  > - **Mission Load Routing**: `AgentRunner` consults `registry.nodes` to optionally delegate sub-missions to less-loaded peer Bunkers via their existing `/v1/agent/:id/run` REST surface.
- [ ] **Context Cores**: Portable, versioned knowledge artifacts for cross-mission portability.
- [ ] **Graceful Degradation Protocols**: Amber/Red badging for provider failover.
- [x] **Self-Annealing Operations**: Autonomous recovery from failed tool calls.
## 🟢 Phase 4: Autonomous Evolution & Swarm Mesh (Complete)
- [x] **Global Intelligence Loop (RAG)**: Automated context retrieval from the Global Swarm Vault.
- [x] **Autonomous Tool Refinement**: `refactor_synthesized_skill` for self-patching capabilities.
- [x] **Dynamic Registry Refresh**: Hot-reloading of agent-generated skills without restart.
- [x] **Evolution Telemetry**: Real-time tracking of skill synthesis and refinement history.

## 🔴 Phase 5: Resilient Infrastructure & Security Hardening
- [x] **Secret-Aware Telemetry**: Automated redaction of API keys and bearer tokens in logs.
- [x] **Granular Backend Errors**: RFC 9457 `ProblemDetails` compliance with machine-readable `ErrorCode` mapped in React stores to trigger interactive self-healing instructions on the dashboard.
- [x] **Automated Trace Correlation**: `X-Request-Id` injection across async boundary steps for full request-lifespan tracking (Jaeger/Datadog compatible).
- [x] **Strict TypeScript Hardening**: Frontend type-safety pushed to 95%+ strict compliance, eliminating silent state-management errors.
- [ ] **Institutional Knowledge Store**: Cross-cluster semantic persistent memory. LanceDB-backed (same pattern as `VectorMemory`) with an extended schema designed for P2P mesh compatibility from day one:
  > **Schema must include at creation** (to avoid migration later when Bunker Mesh lands):
  > - `source_node_id: Option<String>` — which Bunker wrote the entry; used for gossip provenance.
  > - `content_hash: String` — MD5/Blake3 of `text`; dedup key for both local deduplication and cross-node gossip conflict resolution.
  > - `cluster_id: Option<String>` — scopes entries to a cluster; allows cross-cluster queries with optional isolation.
  > - `topic: String` — semantic category tag (e.g., `"finance"`, `"agent_pattern"`) for fast pre-filter before vector search.
  > - `confidence: f32` — decay-weighted relevance score; allows TTL-based knowledge aging.
  > - `ttl: Option<i64>` — Unix expiry timestamp; entries past TTL excluded from retrieval without deletion overhead.

> [!IMPORTANT]
> **Phase 6 Prerequisites** — The following open items are direct dependencies of Phase 6 and should be completed first:
> - **Institutional Knowledge Store** (Phase 5) — Blocker for Phase 6.5 human pattern learning and persistent drift state in Phase 6.2. Without it, mirror mode state is ephemeral across restarts.
> - **Self-Annealing Operations** (Phase 3) — Turns Phase 6.2 drift alerts into autonomous recovery actions rather than dead-end notifications.
> - **Flow-based Ingestion** (Phase 2) — Needed before Phase 6.1 high-volume connectors go live at scale.

## 🟡 Phase 6: Sovereign Process Digital Twin

> Mirror an operational enterprise or system as a living digital twin — shadowing roles, ingesting telemetry, and running autonomous processes in a verified runtime. The swarm model, oversight system, continuity scheduler, and governance blueprints already cover the core primitives.

#### Phase 6.1 — Developer MCP Server Blueprints (Data Connector Templates)
- [ ] **Database & ERP Blueprint**: Template MCP server for querying relational/document databases and enterprise resource planning systems.
- [ ] **REST / Webhook Ingestion Blueprint**: Template MCP server for real-time web services, sync operations, and messaging event buses.
- [ ] **Local Filesystem & Log Scanner Blueprint**: Template MCP server for parsing flat-file repositories, operational logs, and system outputs.
- [ ] **Hardware & Edge Device (RS-232/USB/TCP) Blueprint**: Template MCP server for local serial interfaces, Modbus registers, and physical telemetry data streams.

#### Phase 6.2 — Mirror Mode (Passive Observation)
- [ ] **Mirror Mode Flag**: `mirror_mode: bool` in `AppConfig` / `.env`.
- [ ] **Drift Detection**: Compare real connector data against agent expected state; emit `oversight:drift` WebSocket events.
- [ ] **Mirror Status Route**: `GET /v1/engine/mirror/status` returning current drift alerts.

#### Phase 6.3 — Scenario Branching ("What-If Fork")
- [ ] **State Snapshots**: `POST /v1/engine/snapshot` → named checkpoint of DB + DashMap state.
- [ ] **Fork Endpoint**: Restore snapshot into isolated in-memory state for sandboxed simulation (mock mode enforced, no real tool calls).
- [ ] **Scenario Comparison**: `GET /v1/engine/scenarios` → KPI delta between live and scenario state.

#### Phase 6.4 — KPI Dashboard Layer
- [ ] **KPI Definition Schema**: `{ id, name, formula, source_tool, target }` structure.
- [ ] **Continuity Scheduler Integration**: Cron-driven KPI evaluations.
- [ ] **WebSocket KPI Telemetry**: Emit KPI snapshots via `0x03` header prefix.
- [ ] **Business Intelligence UI**: New 📊 BI dashboard page with trend lines per KPI.

#### Phase 6.5 — Human-to-Agent Identity Mapping
- [ ] **Shadow Identity Field**: `shadows_human_id: Option<String>` on `EngineAgent`.
- [ ] **Agent Config Panel**: "Shadows Real Person" field in UI.
- [ ] **Pattern Learning**: Historical decisions per human ID feed into working memory accumulation.

#### Phase 6.6 — General Compliance & Validation Framework
- [ ] **Edge & Instrumentation Audit Integration**: Connect device and sensor telemetry streams directly to the cryptographically sealed Merkle audit trail, satisfying international data integrity guidelines (such as ALCOA+).
- [ ] **Compliance Directive Templates & Validation Packages**: Pre-written compliance templates, risk models (FMEA), and OQ/PQ test scripts to package as a pre-built Software Validation Package (SVP) for rapid regulatory auditor accreditation sign-off (ISO 9001 / ISO 17025 / GxP).

## 🚀 Future Vision
- **Hardware-Agnostic Acceleration**: Optimized for NVIDIA, ARM (Mac), and RISC-V.
- **Tadpole Hub**: Decentralized registry for public agent templates.
- **Deep Web Integration**: Native SERP and browsing clusters for truly autonomous research.
- **Local ONNX Embeddings**: Native ONNX-based embedding engine for fully air-gapped, GPU-accelerated vector operations without cloud dependency.
- **Universal Sovereign Ingestion**: Native multi-modal document ingestion (PDFs, Office files, Images) utilizing MarkItDown to convert unstructured corporate assets into agent-native Markdown for enhanced RAG and Swarm context.

---

## 📋 Feature Specs
Detailed feature specifications live in [`directives/features/`](directives/features/). Each spec is linked from its roadmap item and contains full architecture, implementation plan, acceptance criteria, and files affected.

| Feature ID | Name | Phase | Status |
|---|---|---|---|
| `IMR-01` | [Intelligent Model Registry & Capability Sync](directives/features/intelligent-model-registry.md) | Phase 2 | ✅ Completed |
| `IMR-02` | Dynamic Capability Overrides | Phase 2 | 🗓 Planned |
| `GFO-01` | Dynamic Provider Failover & Hot-Swapping | Phase 2 | ✅ Completed |
| `OVR-01` | Unified Oversight Gate (HITL + RBAC) | Phase 2 | ✅ Completed |
| `GRP-01` | GraphRAG / TrustGraph Integration | Phase 2 | ✅ Completed |
| `MDD-01` | Swarm Memory Deduplication | Phase 2 | ✅ Completed |
| `OTL-01` | Distributed OpenTelemetry Trace Correlation | Phase 5 | ✅ Completed |
| `RFC-01` | RFC 9457 Granular Backend Errors | Phase 5 | ✅ Completed |
| `TSH-01` | Strict TypeScript Hardening (95%+) | Phase 5 | ✅ Completed |
| `DTW-01` | Digital Twin — MCP Data Connectors | Phase 6 | 🗓 Planned |
| `DTW-02` | Digital Twin — Mirror Mode | Phase 6 | 🗓 Planned |
| `DTW-03` | Digital Twin — Scenario Branching | Phase 6 | 🗓 Planned |
| `DTW-04` | Digital Twin — KPI Dashboard | Phase 6 | 🗓 Planned |
| `DTW-05` | Digital Twin — Human-Agent Identity Mapping | Phase 6 | 🗓 Planned |
| `DTW-06` | Digital Twin — Compliance Framework | Phase 6 | 🗓 Planned |

---
*Last Updated: May 30, 2026*

[//]: # (Metadata: [ROADMAP])
