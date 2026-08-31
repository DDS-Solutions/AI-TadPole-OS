> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / code-review-graph
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Missing symbol graph index or outdated Tree-sitter binaries.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[CODE_REVIEW_GRAPH]`)

# Code Review Graph CLI & Benchmarks Deep Reference (L3)

---

## 1. Token Reduction & Performance Benchmarks

| Codebase Type | Naive Context (Tokens) | Graph-Assisted (Tokens) | Efficiency Gain |
|---|---|---|---|
| **Monorepo (27k+ files)** | 739,352 | 15,049 | **49.1x reduction** |
| **API Framework (3k+ files)** | 138,585 | 37,217 | **3.7x reduction** |
| **HTTP Client Library** | 64,666 | 14,090 | **4.6x reduction** |

---

## 2. Standalone MCP Configuration Options

```bash
# Global installation via pipx or uv
pipx install code-review-graph
code-review-graph install --platform claude-code
code-review-graph install --platform cursor

# Build local SQLite graph index
code-review-graph build
```

---

## 3. SQLite Graph Schema Representation

```sql
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL, -- function, class, struct, route
    file_path TEXT NOT NULL,
    start_line INTEGER,
    end_line INTEGER
);

CREATE TABLE edges (
    source_id TEXT,
    target_id TEXT,
    relation TEXT, -- calls, imports, implements, tests
    FOREIGN KEY(source_id) REFERENCES nodes(id),
    FOREIGN KEY(target_id) REFERENCES nodes(id)
);
```