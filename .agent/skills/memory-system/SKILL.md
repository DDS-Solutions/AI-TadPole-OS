---
name: memory-system
description: Persistent cross-session memory management. Enables agents to remember user preferences, project conventions, and past decisions across different sessions using a structured MEMORY.md index and topic files.
when_to_use: "When the user says 'remember this', 'save this for later', 'don't forget', or when starting a new session and needing to recall past context. Also when /remember workflow is invoked."
allowed-tools: Read, Write, Grep, Glob
effort: low
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / memory-system
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Persistent Memory System (Cross-Session Context)

> **Purpose**: Store distilled user preferences, project decisions, and feedback across sessions without token bloat.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** memory rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/memory_schema_and_topic_templates.md`](./references/memory_schema_and_topic_templates.md) | Topic file structures, pruning algorithms, and frontmatter schemas | Saving new memory topics & index maintenance |

---

## 📁 1. Directory Structure & Pointer Format

```text
.agent/memory/
├── MEMORY.md               ← Lean pointer index (max 200 lines)
├── user-preferences.md     ← Role, shell, communication style
├── project-conventions.md  ← Tech stack, design tokens, architecture
└── tech-decisions.md       ← ADR summaries and framework choices
```

### Pointer Index Syntax (`.agent/memory/MEMORY.md`)
```markdown
- [user] Prefers PowerShell 7 on Windows 11 → user-preferences.md
- [project] Zero hardcoded drive letters; strictly relative links → project-conventions.md
- [feedback] Deliver concise responses with comparison tables → feedback-history.md
```

---

## 🚫 2. Critical Memory Boundaries (Zero-Secrets Rule)

1. **NEVER Save Secrets**: Never write API keys, private keys, passwords, or tokens to memory.
2. **Distilled Insights Only**: Do not store entire raw conversation transcripts.
3. **No Dynamic Ephemeral State**: Do not store transient PID numbers or temporary file paths.

---

## 🛠️ 3. Execution Cycle

```
1. TRIGGER  ➔ User says "remember this" or architectural decision is finalized.
2. CATEGORIZE➔ Assign tag: [user], [project], [feedback], or [reference].
3. APPEND   ➔ Write details to specific topic file; add 1-line pointer to MEMORY.md.
```