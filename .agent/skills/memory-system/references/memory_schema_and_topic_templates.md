> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / memory-system
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Memory index bloat or accidental credential storage.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[MEMORY_SYSTEM]`)

# Cross-Session Memory Schema & Topic Templates (L3)

---

## 1. Topic File Templates

### `user-preferences.md`
```markdown
---
type: user
created: 2026-08-19
updated: 2026-08-19
---

# User Preferences
- OS: Windows 11 / PowerShell 7
- Output Style: Concise, table-driven, action-first
- Architecture: 3-Layer (Directives -> Orchestration -> Execution)
```

### `project-conventions.md`
```markdown
---
type: project
created: 2026-08-19
updated: 2026-08-19
---

# Project Conventions
- Backend: Rust Axum (Port 8000), SQLite WAL mode, Zero-Trust CBS
- Frontend: React + TypeScript + Tailwind CSS v4
- Portability: Strictly repository-relative paths (No hardcoded drive letters)
```

---

## 2. Index Pruning Protocol

When `MEMORY.md` reaches 200 lines:
1. Merge duplicate or redundant entries.
2. Archive completed, historical entries to topic files.
3. Keep only active, high-priority context pointers in `MEMORY.md`.