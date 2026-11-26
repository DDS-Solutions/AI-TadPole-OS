# Cross-Session Memory Schema & Topic Templates (L3)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / memory-system / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Memory index bloat, broken topic pointers, or accidental credential leakage.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[MEMORY_SYSTEM]`)

---

## 1. The 4-Category Sovereign Memory Taxonomy

All persistent cross-session knowledge recorded under `.agent/memory/` must be classified under one of 4 standardized categories:

| Category Tag | Domain Scope | Primary Topic File | Example Entry |
|---|---|---|---|
| `[user]` | User preferences, working style, terminal environment | `user-preferences.md` | `[user] Prefers PowerShell 7 on Windows 11 → user-preferences.md` |
| `[project]` | Core architectural constraints, repo idioms, invariants | `project-conventions.md` | `[project] Zero hardcoded drive letters; relative paths only → project-conventions.md` |
| `[tech]` | Architectural Decision Records (ADRs), blocked upgrades | `tech-decisions.md` | `[tech] TypeScript 7.0 upgrade blocked due to eslint AST parser → tech-decisions.md` |
| `[feedback]` | Behavioral corrections, communication formatting guidance | `feedback-history.md` | `[feedback] Deliver concise responses with comparison tables → feedback-history.md` |

---

## 2. Topic File Markdown Schemas

Every topic file created under `.agent/memory/` must maintain structured YAML frontmatter:

### `user-preferences.md`
```markdown
---
type: user
created: 2026-08-19
updated: 2026-10-17
---

# User Preferences
- **Operating System**: Windows 11 / PowerShell 7
- **Communication Style**: Concise, table-driven, action-first
- **Preferred Patterns**: Guard clauses, early returns, cognitive load minimization
```

### `project-conventions.md`
```markdown
---
type: project
created: 2026-08-19
updated: 2026-10-17
---

# Project Conventions
- **Backend Hubs**: Rust Axum (Port 8000), Tokio async channels, SQLite WAL mode
- **Frontend Stack**: React 19 + TypeScript + Tailwind CSS v4
- **Path Portability**: Strictly repository-relative or canonical `file:///` URLs
- **Documentation Guard**: ADG v2 with zero hand-written witness claims
```

### `tech-decisions.md`
```markdown
---
type: project
created: 2026-07-26
updated: 2026-10-17
---

# Technical Decisions & Compatibility Notes
- **TypeScript Version**: Maintained at TypeScript `5.9.3` until `@typescript-eslint` supports TS >=7.1.
- **Database Architecture**: SQLite with sqlx runtime queries; LanceDB for vector embeddings.
```

---

## 3. Zero-Secrets & PII Quarantine Policy

Under NO circumstances may confidential credentials be written to `.agent/memory/`:
- **Prohibited Tokens**: API keys (`sk-...`, `ghp_...`, `AIza...`), Bearer tokens, passwords, private keys, database connection strings with embedded passwords.
- **Quarantine Action**: If an agent detects a secret in `.agent/memory/`, it must immediately redact or erase it and alert the user.
- **Automated Guard**: Run [`scripts/memory_manager.py --audit`](../scripts/memory_manager.py) to scan for potential secret leaks.

---

## 4. Index Pruning & Compaction Protocol

To prevent `MEMORY.md` from consuming excessive context tokens:
1. **Line Ceiling**: `MEMORY.md` must not exceed **200 lines**.
2. **Pruning Algorithm**:
   - Merge duplicate or overlapping entries into consolidated bullet points.
   - Move detailed context from `MEMORY.md` into dedicated topic files (`user-preferences.md`, `tech-decisions.md`).
   - Maintain only 1-line pointers in `MEMORY.md` using the canonical syntax:
     ```markdown
     - [<category>] <Concise distilled insight> → <topic-file>.md
     ```
   - Prune obsolete or superseded decisions.

---

## 5. Verification Tools

Audit pointer integrity and scan for credential leaks using:
- [`scripts/memory_manager.py`](../scripts/memory_manager.py)