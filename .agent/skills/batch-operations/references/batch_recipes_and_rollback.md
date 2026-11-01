# Batch Operations Recipes & Safe Rollback (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / batch-operations / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Partial multi-file corruption, catastrophic unintended regex replacements.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document provides recipes for multi-file refactoring, batch search-and-replace, and safety rollbacks.

---

## 1. Safety Invariants

1. **Pre-flight Git Status**: Never execute a batch operation with uncommitted or unstaged changes without checking `git status`.
2. **Deterministic Dry-Run**: Always preview matches using `--dry-run` or count target instances before modifying disk files.
3. **Rollback Command**: Keep an instant rollback strategy ready:
   ```powershell
   git checkout -- <path>
   ```

---

## 2. Cross-Platform Safe Regex Recipes

| Task | PowerShell Recipe | Python Script Recipe |
|---|---|---|
| Bump Version in `package.json` | `(Get-Content package.json) -replace '"version": ".*"', '"version": "2.1.0"' \| Set-Content package.json` | `python .agent/skills/batch-operations/scripts/batch_runner.py --find '"version": ".*"' --replace '"version": "2.1.0"' --glob "package.json"` |
| Update Deprecated Imports | `Get-ChildItem -Recurse -Filter *.tsx \| ForEach-Object { ... }` | `python .agent/skills/batch-operations/scripts/batch_runner.py --find "from '@/old'" --replace "from '@/new'" --glob "src/**/*.tsx"` |
