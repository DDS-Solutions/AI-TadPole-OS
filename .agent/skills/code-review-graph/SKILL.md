---
name: code-review-graph
description: Token-efficient code review using Tree-sitter AST graphs and MCP. Reduces AI assistant token usage by 6.8–49x by computing blast radius of changes instead of reading entire codebases. Uses SQLite graph database for structural analysis.
when_to_use: "When reviewing code in large codebases (500+ files), when token costs are high, when making multi-file changes with cross-module dependencies, or when working with monorepos. Also for dead code detection, architecture visualization, and refactoring previews. NOT for small projects under 200 files with isolated single-file changes."
allowed-tools: Read, Grep, Glob, Bash
effort: medium
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / code-review-graph
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Code Review Graph (AST Blast Radius Protocol)

> **Purpose**: Reduce AI token consumption by up to **49x** by loading precise structural AST dependency slices instead of reading entire folders.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core logic below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/graph_cli_and_benchmarks.md`](./references/graph_cli_and_benchmarks.md) | External MCP configuration, token reduction benchmarks, SQLite schema | External repo setup & MCP server config |

---

## 1. Native Tadpole OS Graph Commands

In this repository, graph intelligence is natively powered by `server-rs/src/bin/graph_query.rs`:

```powershell
# 1. Inspect Blast Radius of a file before refactoring (MANDATORY GUARD)
npm run graph:blast:guard -- --path server-rs/src/routes/system.rs

# 2. Look up symbol definition, callers, and callees
npm run graph:lookup -- --name AppState

# 3. Export complete symbol graph to JSON
npm run graph:export
```

---

## 2. Mandatory Pre-Refactor Protocol

Before making non-trivial code modifications across Rust, TypeScript, or Python files:
1. **Query Blast Radius**: Run `npm run graph:blast:guard -- --path <target_file>`.
2. **Ingest Targeted Slice**: Review only the returned callers and callees (~1k–3k tokens) instead of reading every directory file.
3. **Verify Constraints**: Ensure edits preserve all required call signatures and return types.

---

## 3. Decision Matrix: When to Query the Graph

```
Task Scope:
├── Single Isolated File / Pure Doc ➔ Direct edit (Skip graph)
├── Multi-File / Shared Contract     ➔ Run graph:blast:guard (Mandatory)
└── Unfamiliar Function Symbol       ➔ Run graph:lookup (Instant caller mapping)
```