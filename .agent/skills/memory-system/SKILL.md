---
name: memory-system
description: "Persistent cross-session memory management. Enables agents to remember user preferences, project conventions, and past decisions across different sessions using a structured MEMORY.md index and topic files."
when_to_use: "When the user says 'remember this', 'save this for later', 'don't forget', or when starting a new session and needing to recall past context. Also when /remember workflow is invoked."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / memory-system
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Memory index bloat, broken topic pointers, or accidental credential leakage.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Persistent Cross-Session Memory System

> **Purpose**: Store distilled user preferences, project conventions, and architectural decisions across sessions without context token bloat, ensuring instant cold-start alignment.  
> **Workflow Bindings**: Integrated into [`/onboard`](../../workflows/onboard.md) and [`/codify`](../../workflows/codify.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign memory rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/memory_schema_and_topic_templates.md`](./references/memory_schema_and_topic_templates.md) | Topic file schemas, 4-category taxonomy, and pruning algorithms | Creating new topic files & maintaining line ceilings |
| [`scripts/memory_manager.py`](./scripts/memory_manager.py) | Automated pointer auditor, credential leak scanner, and entry recorder | Auditing memory health, verifying pointers, and adding entries |
| [`MEMORY.md`](../../memory/MEMORY.md) | Active live Sovereign memory pointer index | Cold-start state restoration at session inception |

---

## 📁 1. Directory Structure & Pointer Architecture

```text
.agent/memory/
├── MEMORY.md               ← Lean pointer index (strictly capped at 200 lines)
├── user-preferences.md     ← Role, shell, output style, and IDE preferences
├── project-conventions.md  ← Tech stack, design tokens, architecture idioms
└── tech-decisions.md       ← Compatibility status and Architectural Decision Records
```

### Pointer Index Syntax (`.agent/memory/MEMORY.md`)
Every pointer in `MEMORY.md` must be a concise, single-line reference matching:
```markdown
- [<category>] <Distilled high-signal insight> → <topic-file>.md
```

### The 4 Standard Categories:
- **`[user]`**: User habits, operating system, terminal preferences, feedback style.
- **`[project]`**: Non-negotiable repository idioms, coding conventions, architectural rules.
- **`[tech]`**: Architectural Decision Records (ADRs), blocked dependencies, framework choices.
- **`[feedback]`**: Behavioral calibrations, prompt corrections, format preferences.

---

## 🔒 2. Zero-Secrets & Confidentiality Rules

1. **NEVER Store Credentials**: Under NO circumstances may API keys (`sk-...`, `ghp_...`, `AIza...`), tokens, passwords, private keys, or sensitive connection strings be recorded into memory.
2. **Distilled Insights Only**: Never dump raw chat transcripts, tool dumps, or stack traces into memory files.
3. **No Transient Ephemeral State**: Do not record temporary PIDs, transient filenames, or ephemeral debug logs.

---

## 🔄 3. Memory Lifecycle: The 4-Phase Cycle

```
[1. TRIGGER] ───> [2. CATEGORIZE] ───> [3. RECORD TOPIC] ───> [4. INDEX POINTER]
  User command       Select category      Append detailed       Add 1-line pointer
  or settled ADR     tag: [user], etc.    markdown entry        to MEMORY.md
```

1. **Trigger**: User explicitly requests *"remember this"* or a significant architectural trade-off is finalized.
2. **Categorize**: Classify into `[user]`, `[project]`, `[tech]`, or `[feedback]`.
3. **Record Topic**: Append detailed markdown notes with timestamps to the designated topic file.
4. **Index Pointer**: Add a single-line summary pointer to `.agent/memory/MEMORY.md`.
5. **Verify**: Run `python .agent/skills/memory-system/scripts/memory_manager.py` to ensure pointers resolve and zero secrets leaked.

---

## 🧠 4. Socratic Gate for Memory Persistence

Before adding or modifying memory, evaluate these 3 gates:
- **Gate 1 (Zero-Secrets)**: Does the entry contain ANY tokens, passwords, keys, or proprietary private data? (If yes, abort immediately).
- **Gate 2 (Signal vs. Transient Noise)**: Will this insight remain relevant across multiple future sessions, or is it ephemeral debugging noise?
- **Gate 3 (Pointer Resolution)**: Is the topic file created on disk AND indexed with a valid pointer in `MEMORY.md`?

---

## 🚫 5. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Credential Leakage** | Writing API tokens or credentials into memory files. | Quarantined immediately; run `memory_manager.py --scan-secrets`. |
| **Raw Transcript Flooding** | Pasting dozens of chat turns into topic files. | Distill to 2–3 actionable bullets with rationale. |
| **Orphaned Topic Files** | Creating a `.md` file in `.agent/memory/` without linking in `MEMORY.md`. | Always add a corresponding pointer line in `MEMORY.md`. |
| **Broken Pointers** | Listing `→ missing-topic.md` in `MEMORY.md`. | Validate pointers with `memory_manager.py --audit`. |
| **Index Bloat** | `MEMORY.md` grows beyond 200 lines, polluting context window. | Move context into topic files and prune superseded pointers. |

---

## 🛠️ 6. Deterministic Verification Gate

Validate memory system integrity and scan for credential leaks:

```powershell
# 1. Run built-in self-tests
python .agent/skills/memory-system/scripts/memory_manager.py --test

# 2. Audit active workspace memory
python .agent/skills/memory-system/scripts/memory_manager.py .agent/memory
```