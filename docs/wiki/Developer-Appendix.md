---
title: "Developer Appendix"
tier: "dev"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "none"
risk-tags: ["RISK: MEDIUM"]
---

# 🛠️ Developer Appendix

This appendix is for **core developers and contributors** working on the Tadpole OS codebase. It provides an overview of the architecture, extension points, and testing infrastructure.

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

The [[Glossary#appstate|AppState]] struct is the central coordination hub, containing five sub-hubs:

```
AppState
├── comms: CommunicationHub     (real-time events, oversight, WebSocket)
├── governance: GovernanceHub   (limits, policies, privacy mode)
├── registry: RegistryHub      (agents, providers, skills, MCP)
├── security: SecurityHub      (audit, budget, shell scanner)
├── resources: ResourceHub     (DB pool, HTTP client, audio, graphs)
└── base_dir: PathBuf          (root data directory)
```

---

## 2. Backend (Rust / Axum)

### 2.1 Source Structure

```
server-rs/
├── src/
│   ├── router.rs          # Axum route definitions (Gateway Pillar)
│   ├── state/             # AppState and Hub definitions
│   │   └── hubs/          # comms.rs, gov.rs, reg.rs, res.rs
│   ├── routes/            # HTTP handler implementations
│   ├── agent/             # AgentRunner, provider integrations
│   │   ├── runner/        # Core execution loop & swarm orchestration
│   │   │   ├── a2a_ledger.rs   # Agent payment ledger
│   │   │   ├── a2a_mailbox.rs  # Agent communication mailboxes
│   │   │   ├── a2a_router.rs   # Financial router for agent requests
│   │   │   ├── conductor.rs    # DAG plan generator & topological scheduler
│   │   │   └── mod.rs
│   │   ├── anthropic.rs   # Anthropic provider
│   │   ├── gemini.rs      # Google Gemini provider
│   │   └── groq.rs        # Groq provider
│   ├── security/          # Audit trails, budget guards
│   └── telemetry/         # OTel tracing, pulse types
├── data/                  # Runtime data (skills, workflows, DB)
└── migrations/            # SQL migration files
```

### 2.2 Route Registration

All routes are nested under `/v1` in `create_router()`. The Parity Guard validates that every `/v1` route has a corresponding entry in `docs/openapi.yaml`.

### 2.3 Error Handling

The engine uses RFC 9457 compliant [[Glossary#problemdetails|ProblemDetails]] for all API error responses, with unified error types via [[Glossary#apperror|AppError]].

---

## 3. Frontend (React / Vite)

### 3.1 Key Hooks & Stores

| Hook/Store | Purpose |
|-----------|---------|
| `useEngineStatus` | Real-time engine connection status |
| `use_vault_store` | Client-side credential vault management |
| `useLayoutNavigation` | Tab navigation and keyboard shortcuts |

### 3.2 Build Configuration

- **Framework**: Vite + React
- **Config**: `vite.config.ts` in project root
- **Dev Server**: `npm run dev` → `http://localhost:5173`

---

## 4. Extension Points

### 4.1 Custom Skills

Create a JSON manifest in `data/skills/` with the following structure:

```json
{
  "name": "my-custom-skill",
  "description": "Description of the skill",
  "execution_command": "python execution/my_script.py"
}
```

The [[Parity-Guard|Parity Guard]] validates that the referenced execution script exists.

### 4.2 MCP Integration

The engine hosts a Model Context Protocol (MCP) server for external tool aggregation via `RegistryHub::mcp_host`.

### 4.3 Lifecycle Hooks

Pre/post tool execution hooks are managed by `RegistryHub::hooks` (`HooksManager`).

### 4.4 Provider SDK

Developers can implement the `LlmProvider` trait defined in [`provider_trait.rs`](../../server-rs/src/agent/provider_trait.rs) to integrate new LLM backends (e.g. Anthropic, Gemini, OpenAI). The new provider must be registered in [`provider/mod.rs`](../../server-rs/src/agent/runner/provider/mod.rs).

### 4.5 Custom Tool Adapters

Custom tools run through the central dispatch system in [`dispatcher.rs`](../../server-rs/src/agent/runner/tools/dispatcher.rs). New static or dynamic tools can be mapped here.

### 4.6 Workflow Engine Extensions

The long-running workflow scheduling engine, Continuity, executes asynchronous tasks. Extensions to scheduling logic can be made in the Continuity scheduler [`scheduler.rs`](../../server-rs/src/agent/continuity/scheduler.rs) and the executor [`executor.rs`](../../server-rs/src/agent/continuity/executor.rs).

### 4.7 Authentication Plugins `[UI-MISSING]`

Future extensions for multi-tenant and OAuth-based authentication will hook into the Axum request lifecycle. Currently restricted to local single-tenant vault authentication.

### 4.8 Vault & Crypto Extensions

Client-side cryptography (key derivation and encryption) is offloaded to a Web Worker in [`crypto.worker.ts`](../../src/workers/crypto.worker.ts). Custom encryption formats or key-stretching parameters can be configured there.

### 4.9 Agent-to-Agent (A2A) Payments & Mailboxes

Tadpole OS enables agents to pay peer agents for invoking sub-tasks. Developers can extend payment behaviors by working with:
- **`A2aLedger`**: Interacts with the local SQLite `transaction_ledger` database to create, commit, or roll back pending allocations.
- **`A2aMailbox`**: Standardized buffer for inter-agent messages and payment challenges.
- **`A2aRouter`**: Coordinates the routing of requests to target agent providers and handles payment gateway mappings.

For transaction safety, developers should wrap payments in the Two-Phase Commit (2PC) pattern (`Prepare` $\rightarrow$ `Commit` / `Rollback`) to ensure zero balance leakage on failure.

---

## 5. Testing

### 5.1 Running Tests

```powershell
# Frontend tests
npm test

# Backend tests
cd server-rs
cargo test

# Parity Guard (drift detection)
python execution/parity_guard.py .

# End-to-end (Playwright)
npx playwright test
```

### 5.2 Null Provider Mode

For CI/testing without real LLM calls:

```
TADPOLE_NULL_PROVIDERS=true
```

This routes all LLM calls through a `NullProvider` that returns mock responses.

---

## 6. Contributing

- See [`CONTRIBUTING.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/CONTRIBUTING.md) for contribution guidelines.
- See [`DEVELOPMENT.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/DEVELOPMENT.md) for development setup details.
- See [`CODE_OF_CONDUCT.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/CODE_OF_CONDUCT.md) for community standards.

---

## 7. Symbol Graph Intelligence

For non-trivial code changes, use the scriptable symbol graph to inspect blast radius:

```powershell
npm run graph:lookup -- --name SymbolName
npm run graph:file -- --path src/pages/Neural_Map.tsx
npm run graph:blast -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius
npm run graph:export
```

To enforce documentation parity and auto-fix drifted docstring symbols:

```powershell
# Validate all files (warn-only)
cargo run --bin graph_query -- validate

# Validate only git-modified files (fast pre-commit)
cargo run --bin graph_query -- validate --diff

# Auto-fix drifted symbol names via Jaro-Winkler similarity
cargo run --bin graph_query -- validate --fix

# Strict mode: exit code 1 on any failure (CI gate)
cargo run --bin graph_query -- validate --strict

# Export blast radius as Mermaid or interactive HTML
cargo run --bin graph_query -- blast --name SymbolName --format mermaid
cargo run --bin graph_query -- blast --name SymbolName --format html
```

See [`CLI_TOOLS.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/CLI_TOOLS.md#graph-intelligence-cli-graph_query) for the full subcommand reference.

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit d549a5a8 on 2026-07-11 -->
[//]: # (wiki-page: Developer-Appendix)


[//]: # (Metadata: [wiki])
