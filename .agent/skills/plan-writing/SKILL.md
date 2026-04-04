---
name: plan-writing
description: Structured task planning. Breakdowns, dependencies, verification.
allowed-tools: Read, Glob, Grep
---

# Plan Writing

**Small tasks. Clear verification.**

## Principles
1.  **Small**: 2-5 mins per task.
2.  **Verifiable**: "How do I know it's done?".
3.  **Specific**: "Add `auth` middleware" > "Add security".
4.  **Dynamic**: No fixed templates.
5.  **Location**: `{task-slug}.md` in Project Root.

## Structure
```markdown
# [Task Name]

## Goal
One sentence.

## Tasks
- [ ] Action → Verify: [Check]
- [ ] Action → Verify: [Check]

## Done When
- [ ] Criteria 1
```

---

## 🧠 Aletheia Reasoning Protocol (Planning)

### 1. Generator (Decomposition)
*   **Atomic**: "< 10 mins?".
*   **Dependency**: "API key needed before client?".
*   **Verification**: "How to prove it works?".

### 2. Verifier (Reality Check)
*   **Complexity**: "'Refactor App' is not a step. Break it down.".
*   **Risks**: "Flag unknowns.".
*   **Rollback**: "Can I undo?".

### 3. Reviser (Clarity)
*   **Actionable**: "Handle NullPointer" > "Fix bug".
*   **Context**: "Add file paths.".

---

## 🛡️ Security & Safety Protocol (Planning)

1.  **No Secrets**: Plans are artifacts (Text).
2.  **Destructive**: Mark deletions with ⚠️.
3.  **Auth**: Verify intent for infra changes.
4.  **Backup**: Plan backups before migrations.
