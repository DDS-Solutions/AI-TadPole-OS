# Code Review Graph CLI & Benchmarks Deep Reference (L3)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / code-review-graph / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Missing symbol graph index, uninspected high blast radius, or outdated Tree-sitter binaries.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document provides deep specifications for the AST graph query engine, blast radius calculation heuristics, and token reduction benchmarks in Tadpole OS.

---

## 1. Token Reduction & Performance Benchmarks

Loading an entire codebase or whole directories into an AI prompt consumes tens of thousands of tokens and causes attention dilution. By slicing only direct callers and callees from the Tree-sitter symbol graph, context is scoped to the minimal blast radius:

| Codebase Type | Naive Context (Tokens) | Graph-Assisted (Tokens) | Efficiency Gain |
|---|---|---|---|
| **Monorepo (27k+ files)** | 739,352 | 15,049 | **49.1x reduction** |
| **API Framework (3k+ files)** | 138,585 | 37,217 | **3.7x reduction** |
| **HTTP Client Library** | 64,666 | 14,090 | **4.6x reduction** |
| **Tadpole OS Core (~6.5k nodes)** | ~450,000 | ~2,500 | **~180x reduction per edit slice** |

---

## 2. Graph Intelligence CLI Reference

Tadpole OS provides native symbol graph tooling powered by Tree-sitter and `server-rs`:

| Command | Purpose | When to Use |
|---|---|---|
| `npm run graph:blast:guard -- --path <file>` | Inspect blast radius of callers & callees | **Mandatory** before refactoring shared routes or contracts |
| `npm run graph:lookup -- --name <Symbol>` | Retrieve exact definition, signature, and usages | When analyzing an unfamiliar type, function, or struct |
| `npm run graph:file -- --path <file>` | List all symbols exported and consumed by a file | When auditing file cohesion or preparing PR reviews |
| `npm run graph:blast -- --path <file> --name <sym>` | Compute deep multi-hop blast radius | When altering core database or IPC schemas |
| `npm run graph:export` | Export updated symbol graph to JSON | After completing non-trivial multi-file refactors |

---

## 3. Graph Schema & Audit Context Structure

The symbol graph exports structured node and edge telemetry to `reports/intelligence/audit_context.json`:

```json
{
  "generated_at": "2026-10-11T16:16:28.547986400+00:00",
  "root": "\\\\?\\D:\\TadpoleOS-Dev",
  "stats": {
    "node_count": 6541,
    "edge_count": 33071,
    "anomaly_count": 22
  },
  "top_connected": [
    {
      "id": "5dd9c30dba44b7f4/client_pool.rs:len",
      "node": {
        "name": "len",
        "path": "5dd9c30dba44b7f4/client_pool.rs",
        "kind": "func",
        "signature": "pub async fn len(&self) -> usize {",
        "start_line": 145,
        "end_line": 147
      },
      "callers": [
        { "name": "prune_large_strings", "path": "df54141fdf5374d2/serialization.rs" }
      ],
      "callees": []
    }
  ]
}
```

### Blast Radius Risk Thresholds:
- **Low Risk ($\le 3$ callers)**: Safe for targeted local edits with standard unit tests.
- **Medium Risk (4–10 callers)**: Requires integration test execution across affected caller modules.
- **High Risk ($> 10$ callers)**: Requires explicit architectural review and regression verification.