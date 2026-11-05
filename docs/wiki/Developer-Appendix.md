---
title: "Developer Appendix"
tier: "dev"
status: "verified"
version: "1.1.464"
last-verified: "2026-11-02"
commit: "3eef5c2d"
network-badge: "none"
risk-tags: ["RISK: MEDIUM"]
---

# 🛠️ Developer Appendix

This appendix is for **core developers and contributors** working on the Tadpole OS codebase (**Plane 1: Codebase Self-Governance** under the target hierarchy in `directives/IDENTITY.md`). It covers architecture, extension points, testing infrastructure, and the current `FR-11` status: the generic runner has an exact department-marker rejection gate, while company-scoped Plane 2 authorization and storage remain pending.

> [!NOTE]
> This page is a summary reference. For the full architectural specification, see [`ARCHITECTURE.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/ARCHITECTURE.md) in the main repository.

---

## 1. Architecture Overview

Tadpole OS follows a **4-Pillar** architecture:

| Pillar | Responsibility | Key Components |
|--------|---------------|----------------|
| **Gateway** | HTTP/WebSocket API surface | `server-rs/src/router.rs`, Axum framework |
| **Registry** | Agent, provider, and skill management | `RegistryHub`, `GovernanceHub`, `ResourceHub` |
| **Engine** | Autonomous mission execution | `AgentRunner`, `RunContext`, `ToolOrchestrator` |
| **Security Root** | Audit, budget, and access control | `SecurityHub`, `MerkleAuditTrail`, `BudgetGuard` |

### 1.1 State Architecture (`AppState`)

The [[Glossary#appstate|AppState]] struct (`server-rs/src/state/mod.rs`) is the central coordination hub, containing five sub-hubs and global session context:

```
AppState
├── comms: Arc<CommunicationHub>         (real-time events, oversight, WebSocket)
├── governance: Arc<GovernanceHub>       (limits, policies, privacy mode)
├── registry: Arc<RegistryHub>           (agents, providers, skills, MCP)
├── security: Arc<SecurityHub>           (audit, budget, shell scanner)
├── resources: Arc<ResourceHub>          (DB pool, HTTP client, audio, graphs)
├── mirror_mode: bool                    (read from environment / runtime flag)
├── drift_alerts: Arc<DashMap<String, Value>> (active drift observed in mirror mode)
├── session_id: String                   (unique boot session correlation UUID)
└── base_dir: PathBuf                    (root workspace & data directory)
```

---

## 2. Backend (Rust / Axum)

### 2.1 Source Structure

```
server-rs/
├── src/
│   ├── adapter/           # External environment adapters & Git worktree sandboxing
│   │   └── worktree.rs    # Ephemeral Git worktree lifecycle isolation
│   ├── router.rs          # Axum route definitions (Gateway Pillar, 140+ endpoints)
│   ├── state/             # AppState and Hub definitions (hubs/, init/, mock.rs)
│   ├── routes/            # HTTP & WebSocket handler implementations
│   │   ├── business.rs    # Mock SMB Digital Twin endpoints
│   │   ├── missions.rs    # Mission Flight Recorder & Run History
│   │   └── ...
│   ├── agent/             # Core agent execution & swarm orchestration
│   │   ├── runner/        # Execution loop, tool dispatcher, prompt synthesis
│   │   │   ├── a2a_ledger.rs   # Agent payment ledger (SQLite transaction_ledger)
│   │   │   ├── a2a_mailbox.rs  # Inter-agent communication mailboxes
│   │   │   ├── a2a_router.rs   # Financial router for agent requests
│   │   │   ├── conductor.rs    # DAG plan generator & topological scheduler
│   │   │   └── tools/          # Tool dispatching & filesystem sandboxes
│   │   ├── business/      # Mock SMB Digital Twin service, connectors & SOPs
│   │   ├── continuity/    # Workflow job scheduler & 5-stage step executor
│   │   ├── context_manager.rs # Rolling dialogue compaction & scratchpad
│   │   ├── reaper.rs      # Background actor process supervision & cleanup
│   │   ├── mcp/           # Native Model Context Protocol (MCP) host
│   │   ├── capability_map.rs # Swarm tiered capability recruitment index
│   │   ├── mission.rs     # Swarm mission lifecycle & conductor state
│   │   ├── blackboard.rs  # Shared blackboard multi-agent coordination
│   │   ├── anthropic.rs   # Anthropic Claude provider
│   │   ├── gemini.rs      # Google Gemini provider
│   │   ├── groq.rs        # Groq LPU provider
│   │   └── openai.rs      # OpenAI & local Ollama / Gemma 4 / Qwen provider
│   ├── security/          # Audit trails, Merkle trees, budget guards
│   └── telemetry/         # OpenTelemetry tracing, pulse types, binary codecs
├── data/                  # Runtime templates, skills, and local DBs
└── migrations/            # SQLx migration scripts
```

### 2.2 Route Registration

All routes are nested under `/v1` in `create_router()`. The Parity Guard (`execution/parity_guard.py`) validates that every `/v1` route in `router.rs` has a corresponding documented entry in `docs/openapi.yaml` and `docs/API_REFERENCE.md`.

### 2.3 Error Handling

The engine uses RFC 9457 compliant [[Glossary#problemdetails|ProblemDetails]] for all API error responses, with unified error types via [[Glossary#apperror|AppError]] mapped in `docs/ERROR_REGISTRY.json`.

---

## 3. Frontend (React / Vite / Tailwind)

### 3.1 Key Hooks & Stores

| Hook/Store | Location | Purpose |
|-----------|----------|---------|
| `useEngineStatus` | `src/hooks/use_engine_status.ts` | Real-time engine health, ping, and connection status |
| `use_vault_store` | `src/stores/vault_store.ts` | Client-side credential vault & PBKDF2 key derivation |
| `use_workspace_store` | `src/stores/workspace_store.ts` | Workspace clustering, sandboxes, and CAS snapshot retention |
| `use_agent_store` | `src/stores/agent_store.ts` | Agent definitions, active mission runs, and prompt tuning |
| `use_department_store` | `src/stores/department_store.ts` | Swarm department hierarchies & `FR-11` permission boundaries |
| `use_security_store` | `src/stores/security_store.ts` | Zero-Trust policy enforcement, kill-switches, and audit log |
| `use_trace_store` | `src/stores/trace_store.ts` | Mission Flight Recorder waterfall traces & pulse telemetry |
| `useLayoutNavigation` | `src/hooks/useLayoutNavigation.ts` | Multi-tab navigation and global keyboard shortcuts |
| `useSwarmTelemetry` | `src/components/useSwarmTelemetry.ts` | Decoupled 10Hz socket telemetry, sliding window buffer, and simulation reheat |
| `useCognitionCapabilities` | `src/components/agent-config/CognitionSection.tsx` | Isolated capability store subscriptions for agent cognitive configuration |
| `useAgentConfig` | `src/components/agent-config/useAgentConfig.ts` | Consolidated agent configuration form state and mutation dispatchers |

### 3.2 Build Configuration

- **Framework**: Vite + React 19 + TypeScript
- **Styling**: Tailwind CSS with canonical glassmorphic theme tokens
- **Dev Server**: `npm run dev` → `http://localhost:5173`
- **Desktop Bundle**: Tauri v2 desktop shell (`src-tauri/`)

---

## 4. Extension Points

### 4.1 Agent Skills & Open Standards

Skills follow the **`agentskills.io`** open specification:
- **Location**: `.agent/skills/<skill-name>/SKILL.md` (progressive disclosure: L1 YAML frontmatter, L2 instructions, L3 references).
- **Execution**: Deterministic scripts live in `execution/` or `scripts/`.
- Manifest definitions in `data/skills/` are validated against their script targets by [[Parity-Guard|Parity Guard]].

### 4.2 Model Context Protocol (MCP) Integration

The engine hosts an MCP client and server (`server-rs/src/agent/mcp/`) for external tool aggregation via `RegistryHub::mcp_host`.

### 4.3 Lifecycle Hooks

Pre/post tool execution hooks are managed by `RegistryHub::hooks` (`HooksManager`), supporting audit inspection, parameter validation, and conditional approvals.

### 4.4 Provider SDK & Multi-Model Slots

Developers can implement the `LlmProvider` trait defined in [`provider_trait.rs`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/server-rs/src/agent/provider_trait.rs) to integrate new LLM backends (Anthropic, Gemini, OpenAI, Groq, Ollama, local Gemma 4 / Qwen 2.5). Register new providers in `server-rs/src/agent/runner/provider/mod.rs`.

### 4.5 Custom Tool Adapters

Custom tools run through the central dispatch system in `server-rs/src/agent/runner/tools/dispatcher.rs`. Tools enforce workspace sandboxing and input validation before execution.

### 4.6 Workflow Engine Extensions (Continuity)

The long-running workflow scheduling engine, Continuity, executes asynchronous tasks. Extensions to scheduling logic can be made in the Continuity scheduler (`server-rs/src/agent/continuity/scheduler.rs`) and step executor (`server-rs/src/agent/continuity/executor.rs`). The executor follows a 5-stage linearized lifecycle:
1. `claim_or_skip_job`: Atomic pending job acquisition.
2. `spawn_heartbeat_keeper`: Non-blocking heartbeat keeper preventing timeout reaping.
3. `parse_job_execution_config`: Validates mission payload and env settings.
4. `execute_job_mission`: Dispatches to runner within worktree sandbox.
5. `finalize_job_execution`: Status persistence, metric emission, and lease release.

### 4.7 Mock SMB Digital Twin Practice Workspaces

Developers can extend isolated business practice simulations:
- **Company Profiles**: Register JSON specifications under `starter_kits/mock_smb/companies/` (e.g. `Mock-1-Product.json`, `Mock-2-Service.json`).
- **SOP Procedures**: Define customer enquiry workflows under `starter_kits/mock_smb/sops/`.
- **Connector Handlers**: Extend service logic in `server-rs/src/agent/business/`.

### 4.8 Client-Side Cryptography

Client-side cryptography (PBKDF2 key stretching and AES-256-GCM encryption) is offloaded to a dedicated Web Worker in `src/workers/crypto.worker.ts`.

### 4.9 Agent-to-Agent (A2A) Protocol & Settlement

Tadpole OS enables agents to pay peer agents for invoking sub-tasks:
- **`A2aLedger`**: Interacts with the local SQLite `transaction_ledger` database to create, commit, or roll back pending allocations.
- **`A2aMailbox`**: Standardized buffer for inter-agent messages and payment challenges.
- **`A2aRouter`**: Coordinates the routing of requests to target agent providers and handles payment gateway mappings.
- **Two-Phase Commit (2PC)**: Wraps payment transactions in `Prepare` $\rightarrow$ `Commit` / `Rollback` to ensure zero financial leakage on task failure.

---

## 5. Testing & Verification

### 5.1 Verification Commands

```powershell
# Active Documentation Guard (ADG v2) integrity verification
npm run adg:verify

# Machine-maintained witness test suite
npm run test:witness

# Executable safety and security invariants
npx vitest run tests/invariants/

# Product and target version parity check
node scripts/bump_version.cjs --check

# Documentation parity & link guard (drift detection)
python execution/parity_guard.py

# Frontend unit and component tests
npm test

# Backend Rust test suite
cd server-rs; cargo test

# Cargo formatting check
cargo fmt --manifest-path server-rs/Cargo.toml --check
```

### 5.2 Null Provider Mode

For deterministic CI and offline testing without live LLM calls:

```
TADPOLE_NULL_PROVIDERS=true
```

This routes all LLM calls through a `NullProvider` returning structured mock responses.

---

## 6. Contributing

- See [`CONTRIBUTING.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/CONTRIBUTING.md) for contribution guidelines.
- See [`DEVELOPMENT.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/DEVELOPMENT.md) for development setup details.
- See [`CODE_OF_CONDUCT.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/CODE_OF_CONDUCT.md) for community standards.

---

## 7. Symbol Graph Intelligence

For non-trivial code changes, use the scriptable Tree-sitter symbol graph to inspect blast radius before refactoring:

```powershell
# Pre-refactor blast radius check
npm run graph:blast:guard -- --path server-rs/src/routes/missions.rs

# Lookup symbol callers and callees
npm run graph:lookup -- --name SymbolName

# Analyze all symbols within a specific file
npm run graph:file -- --path src/pages/Missions.tsx

# Export symbol connectivity graph
npm run graph:export

# Generate scoped audit context
npm run graph:audit
```

To enforce documentation parity and auto-fix drifted docstring symbols:

```powershell
# Validate all files (warn-only)
cargo run --manifest-path server-rs/Cargo.toml --bin graph_query -- validate

# Validate only git-modified files (fast pre-commit)
cargo run --manifest-path server-rs/Cargo.toml --bin graph_query -- validate --diff

# Auto-fix drifted symbol names via Jaro-Winkler similarity
cargo run --manifest-path server-rs/Cargo.toml --bin graph_query -- validate --fix

# Strict mode: exit code 1 on any failure (CI gate)
cargo run --manifest-path server-rs/Cargo.toml --bin graph_query -- validate --strict
```

See [`CLI_TOOLS.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/CLI_TOOLS.md#graph-intelligence-cli-graph_query) for the full subcommand reference.

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Developer-Appendix)


[//]: # (Metadata: [wiki])
