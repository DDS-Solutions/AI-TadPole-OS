---
name: batch-operations
description: Apply operations across multiple files simultaneously. Pattern-based bulk modifications, search-and-replace across codebases, consistent changes to many files at once.
when_to_use: "When the user needs to change multiple files with the same pattern, rename across a codebase, add imports to many files, update versions, or apply consistent modifications. NOT for single-file edits."
allowed-tools: Read, Write, Edit, Grep, Glob, Bash, RunCommand
version: "2.1.0"
priority: medium
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / batch-operations
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Batch Operations — Multi-File Changes Protocol

> **Purpose**: Apply consistent, verifiable changes across multiple files simultaneously with dry-run safety and rollback readiness.
> **Workflow Binding**: Used during [`/refactor`](../../workflows/refactor.md) and [`/enhance`](../../workflows/enhance.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/batch_recipes_and_rollback.md`](./references/batch_recipes_and_rollback.md) | Multi-file regex patterns, cross-platform batch recipes, and git rollback procedures | Designing bulk string/AST replacements |
| [`scripts/batch_runner.py`](./scripts/batch_runner.py) | Deterministic batch runner tool with preview dry-run and `--test` suite | Automated multi-file search and replace operations |

---

## 🔄 Batch Operation Protocol

```
1. PREVIEW (DRY RUN) ➔ Count matches and preview file list without modifying disk.
2. GIT SANITY CHECK  ➔ Ensure working directory is clean (`git status`).
3. EXECUTE BATCH     ➔ Apply replacement using batch_runner or atomic edits.
4. TEST & VERIFY     ➔ Run test suite (`npm test` / `cargo test`) to ensure zero regressions.
```

---

## 🛡️ Socratic Gate: Multi-File Blast Radius

Before executing batch operations:
1. **Scope Specificity**: Is the glob pattern sufficiently narrow to avoid matching build artifacts (`target/`, `node_modules/`, `dist/`)?
2. **Ambiguity Check**: Could the pattern accidentally match unintended substrings or variable names?
3. **Rollback Plan**: Is `git checkout -- .` ready if unexpected side effects occur?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Modifying Without Dry-Run | Corrupts dozens of files silently | Always execute with `--dry-run` or preview first |
| Dirty Working Directory | Inability to separate batch changes from prior work | Stash or commit existing changes before bulk edits |
| Broad Grep Replacements | Matches words inside comments, licenses, or tests | Use targeted AST tools or strict word boundary regex |

---

## 🛠️ Verification & Execution Commands

```powershell
# Preview changes (Dry Run)
python .agent/skills/batch-operations/scripts/batch_runner.py --find "old_pattern" --replace "new_pattern" --glob "src/**/*.tsx"

# Execute batch changes
python .agent/skills/batch-operations/scripts/batch_runner.py --find "old_pattern" --replace "new_pattern" --glob "src/**/*.tsx" --execute

# Run self-test suite
python .agent/skills/batch-operations/scripts/batch_runner.py --test
```