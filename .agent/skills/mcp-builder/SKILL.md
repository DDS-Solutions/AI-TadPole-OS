---
name: mcp-builder
description: "MCP (Model Context Protocol) server building principles. Tool design, resource patterns, JSON-RPC 2.0 schemas, and transport standards."
when_to_use: "When building MCP servers, designing MCP tools, configuring workspace MCP integrations, or debugging JSON-RPC 2.0 transport issues."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mcp-builder
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Malformed JSON-RPC payloads, stdout log contamination, or transport disconnects.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Model Context Protocol (MCP) Server Architecture

> **Purpose**: Standardize the design, schema validation, transport configuration, and security sandbox boundaries for Model Context Protocol (MCP) servers and tools across the Sovereign workspace.  
> **Workflow Bindings**: Integrated into [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md) and [`/enhance`](file:///D:/TadpoleOS-Dev/.agent/workflows/enhance.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/mcp_server_specs.md`](file:///D:/TadpoleOS-Dev/.agent/skills/mcp-builder/references/mcp_server_specs.md) | Full JSON-RPC 2.0 schemas, standard error codes, stdio/SSE/WebSocket transports | Building MCP servers, designing tools, or debugging RPC disconnects |
| [`scripts/mcp_validator.py`](file:///D:/TadpoleOS-Dev/.agent/skills/mcp-builder/scripts/mcp_validator.py) | Automated tool schema validator, config auditor, and stdio server scaffolder | Validating tool parameters and generating production-ready server stubs |
| [`docs/ARCHITECTURE.md`](file:///D:/TadpoleOS-Dev/docs/ARCHITECTURE.md) | Sovereign System Architecture and Core Subsystems | Integrating external MCP tools with Tadpole OS state hubs |

---

## 🛠️ 1. Active Workspace MCP Registry

| MCP Server | Server Purpose | Key Bound Tools | Transport / Fallback |
|---|---|---|---|
| **`github`** | Issues, PRs, Git commits, branch creation | `create_issue`, `create_pull_request`, `push_files`, `list_commits` | Local git commands |
| **`context7`** | Official library documentation & framework resolution | `query-docs`, `resolve-library-id` | Offline markdown files |
| **`google-developer-knowledge`** | Official Google developer knowledgebase | `search_documents`, `answer_query` | Web search API |
| **`shadcn`** | Modern UI component registry & example templates | `search_items_in_registries`, `view_items_in_registries` | Local React components |
| **`brave-search`** | High-precision web and local search | `brave_web_search`, `brave_local_search` | Native search engine |
| **`stitch`** | Multi-file synchronization & layout integration | Multi-agent coordination endpoints | Direct file edits |

---

## 📐 2. Tool & Resource Design Principles

### Principle 1: Explicit JSON Schemas
Every MCP tool must provide an explicit `inputSchema` with `type: "object"`. Every property must have a declared `type` (e.g. `string`, `integer`, `boolean`), a clear functional `description`, and be listed in `required` if mandatory.

### Principle 2: Strict Stdio Transport Hygiene
In `stdio` servers, **NEVER write log messages or diagnostics to `stdout`**. Writing non-JSON data to `stdout` corrupts the JSON-RPC message framing and crashes the client connection. **All logging must be directed to `stderr`** (e.g. `sys.stderr.write()` or `console.error()`).

### Principle 3: Standard Error Codes Over Crashes
Servers must never exit on invalid client input. Return standard JSON-RPC error codes (`-32602` for invalid params, `-32601` for method not found, `-32603` for internal execution errors) with concise, actionable error messages.

### Principle 4: Bounded Output Payloads
Always truncate or paginate large outputs (e.g. capped at 50,000 characters or 100 array items) to avoid saturating LLM context windows.

---

## 🧠 3. Socratic Gate for MCP Tool Design

Before finalizing a new MCP tool or server, answer these 4 gates:
- **Gate 1 (Single Responsibility)**: Does the tool perform a single deterministic action rather than a multi-step workflow?
- **Gate 2 (Schema Explicitness)**: Are all arguments typed and annotated with clear descriptions indicating valid formats?
- **Gate 3 (Stdio Hygiene)**: Does the server route 100% of logging and debugging information to `stderr`?
- **Gate 4 (Secret Safety)**: Does the configuration avoid hardcoding plaintext secrets or sensitive tokens?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Stdout Log Pollution** | `print("Server started")` breaks JSON-RPC framing on stdio. | Direct all logs to `sys.stderr` or `console.error()`. |
| **Ambiguous Schema** | Declaring `"properties": {}` or omitting parameter types. | Fully define property types, descriptions, and required arrays. |
| **Unbounded Payload** | Returning a 200,000-line database dump that overflows LLM tokens. | Enforce pagination parameters (`limit`, `offset`) and payload caps. |
| **Hardcoded Secrets** | Storing raw API tokens in `mcp_config.json`. | Use environment variable expansion (e.g. `"${GITHUB_TOKEN}"`). |
| **Crash on Bad Input** | Throwing unhandled exceptions that terminate the stdio process. | Catch exceptions and return a `{ "code": -32602, "message": "..." }` error object. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate schemas and scaffold production MCP servers:

```powershell
# 1. Run built-in self-tests
python .agent/skills/mcp-builder/scripts/mcp_validator.py --test

# 2. Validate a tool schema JSON
python .agent/skills/mcp-builder/scripts/mcp_validator.py --tool-schema path/to/tool.json

# 3. Scaffold a new Python stdio MCP server template
python .agent/skills/mcp-builder/scripts/mcp_validator.py --scaffold-py --name database_reader --output .tmp/db_server.py
```