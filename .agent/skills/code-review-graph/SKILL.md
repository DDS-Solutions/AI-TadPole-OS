---
name: code-review-graph
description: "Token-efficient code review using Tree-sitter AST graphs and MCP. Reduces AI assistant token usage by up to 49x by computing blast radius instead of reading entire folders."
when_to_use: "When reviewing code in large repositories, multi-file refactoring, tracing cross-module dependencies, or auditing blast radius before code edits."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / code-review-graph
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Missing symbol graph index, uninspected high blast radius, or outdated Tree-sitter binaries.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Code Review Graph (AST Blast Radius Protocol)

> **Purpose**: Reduce AI token consumption by up to **49x** by loading precise structural AST dependency slices instead of reading entire folders or massive directories into context.

---

## 📚 Progressive Disclosure & L3 Reference Index

Consult deep **L3 Reference Guides** and verification tools on demand:
- **Graph CLI & Benchmarks Deep Reference**: [graph_cli_and_benchmarks.md](./references/graph_cli_and_benchmarks.md)
- **Deterministic Blast Radius Validator**: [graph_blast_validator.py](./scripts/graph_blast_validator.py)

---

## 1. Native Tadpole OS Graph Commands

In this repository, graph intelligence is natively powered by `server-rs` and Tree-sitter AST symbol extraction:

```powershell
# 1. Inspect Blast Radius of a file before refactoring (MANDATORY GUARD)
npm run graph:blast:guard -- --path server-rs/src/routes/system.rs

# 2. Look up symbol definition, callers, and callees
npm run graph:lookup -- --name AppState

# 3. View all symbols exported and consumed by a file
npm run graph:file -- --path src/pages/Neural_Map.tsx

# 4. Compute deep multi-hop blast radius for a symbol
npm run graph:blast -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius

# 5. Export complete symbol graph to JSON
npm run graph:export
```

Audit commands generate high-fidelity graph context at:
`reports/intelligence/audit_context.json`

---

## 2. Mandatory Pre-Refactor & Subagent Guard

Before making non-trivial code modifications across Rust, TypeScript, or Python files:
1. **Query Blast Radius**: Run `npm run graph:blast:guard -- --path <target_file>`.
2. **Ingest Targeted Slice**: Review only the returned callers and callees (~1k–3k tokens) instead of reading every directory file.
3. **Verify Constraints**: Ensure edits preserve all required call signatures, parameter types, and return values.

---

## 3. Decision Matrix: When to Query the Graph

```
Task Scope:
├── Single Isolated File / Pure Docs  ➔ Direct edit (Skip graph query)
├── Multi-File / Shared Contract      ➔ Run graph:blast:guard (Mandatory)
├── Core System / Rust Routes         ➔ Run graph:blast:guard + graph:lookup
└── Unfamiliar Function Symbol        ➔ Run graph:lookup (Instant caller mapping)
```

---

## 🛠️ 4. Deterministic Verification Gate

Validate blast radius telemetry and audit context using the deterministic validator:

```powershell
# Run built-in self tests
python .agent/skills/code-review-graph/scripts/graph_blast_validator.py --test

# Audit current graph context report
python .agent/skills/code-review-graph/scripts/graph_blast_validator.py --file reports/intelligence/audit_context.json
```

[//]: # (Metadata: [CODE_REVIEW_GRAPH])