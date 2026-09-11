---
name: mcp-builder
description: MCP (Model Context Protocol) server building principles. Tool design, resource patterns, best practices.
when_to_use: "When building MCP (Model Context Protocol) servers, designing MCP tools, or implementing MCP resource patterns."
allowed-tools: Read, Write, Edit, Glob, Grep
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mcp-builder
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Model Context Protocol (MCP) Server Architecture

> **Purpose**: Standardized protocol connecting AI agents to external tool endpoints and data resources.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core MCP rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/mcp_server_specs.md`](./references/mcp_server_specs.md) | Full JSON-RPC 2.0 specs, stdio/SSE/WebSocket transport configurations | MCP server creation & custom protocol debugging |

---

## 🛠️ 1. Active Workspace MCP Registry

| MCP Server | Server Purpose | Key Bound Tools | Fallback Mode |
|---|---|---|---|
| **`github`** | Issues, PRs, Git commits, branch creation | `create_issue`, `create_pull_request`, `push_files` | Local git commands |
| **`context7`** | Framework documentation & library resolution | `query-docs`, `resolve-library-id` | Offline markdown files |
| **`google-developer-knowledge`**| Google developer knowledgebase | `search_documents`, `answer_query` | Web search API |
| **`shadcn`** | Component registry & stubs | `search_items_in_registries` | Local React components |

---

## 📐 2. Tool & Resource Design Principles

1. **Explicit Schemas**: Define JSON Schema types for every tool argument with unambiguous descriptions.
2. **Predictable URIs**: Use structured URI templates for resources (`docs://readme`, `db://tables/{name}`).
3. **Structured Errors**: Return JSON error details rather than raw runtime stack traces.

---

## 🚫 3. Anti-Patterns to Avoid

- **No Ambiguous Tool Names**: Use active verb-noun naming (`create_user`, `fetch_metrics`).
- **No Unbounded Output**: Always paginate or cap resource payloads to avoid saturating agent context.