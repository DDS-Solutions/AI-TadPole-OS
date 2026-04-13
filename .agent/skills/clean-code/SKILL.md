---
name: clean-code
description: Pragmatic coding standards - concise, direct, no over-engineering, no unnecessary comments
allowed-tools: Read, Write, Edit
version: 2.1
priority: CRITICAL
---

# Clean Code - Pragmatic Standards

**Rule: Be concise, direct, and solution-focused.**

## Core Principles
- **SRP**: Single Responsibility.
- **DRY**: Don't Repeat Yourself.
- **KISS**: Keep It Simple.
- **Sovereign Standards**:
  - **Rust**: No `unwrap()` in core paths; favor `?` or `expect("context")`. Use `Arc<Mutex<T>>` for shared state.
  - **TypeScript**: Strict typing (no `any`). Use Zustand for global state, components for UI.
  - **CSS**: Tailwind v4 (CSS-first config). Use tokens over ad-hoc offsets.
- **Naming**: `userCount` (Intent) > `n`. `getUser()` (Action). `isActive` (Bool).
- **Functions**: Small (5-10 lines), Pure (No side effects), Flattened (Guard clauses).

## 🧠 Aletheia Reasoning Protocol (Refactoring)

**Code is read 10x more than it is written.**

### 1. Generator (Expression)
*   **Draft**: Write the ugly version first. Get it working.
*   **Alternatives**: "Can this be a map? A reduce? A class?".
*   **Naming**: Generate 3 names for this function. Pick the best one.

### 2. Verifier (Cognitive Load)
*   **The "3 AM Test"**: "If I read this at 3 AM during an outage, will I understand it?".
*   **Complexity**: "Is this one-liner clever, or just confusing?".
*   **Dependencies**: "Do I really need Lodash for this?".

### 3. Reviser (Polish)
*   **Extract**: Move that 5-line block to a named function.
*   **Guard**: Invert if-checks to reduce nesting.

---

## 🛡️ Security & Safety Protocol (Code)

1.  **Input Validation**: "Clean" input is validated input.
2.  **Output Encoding**: "Clean" output is escaped output (XSS prevention).
3.  **No Eval**: Clean code never uses `eval()` or `new Function()`.
4.  **Secrets**: Clean code retrieves secrets from env, never hardcoded.

---

## 🔴 Pre-Work Checklist (Mental)
1.  **Imports**: Who imports this? Will I break them?
2.  **Deps**: What does this import?
3.  **Scope**: Edit file + dependencies in SAME task.

## 🔴 Completion Checklist
- [ ] **Goal Met?** Exactly what user asked.
- [ ] **Code Works?** Tested/Verified.
- [ ] **Lint/Types?** No red squiggles.
- [ ] **Cleanup?** No console.logs or dead comments.

## 🔴 Verification Scripts
**Run the core audit matching your scope:**
- **System Audit**: `python execution/verify_all.py . --url http://localhost:8000`
- **Parity Check**: `python execution/parity_guard.py .` (Mandatory for documentation)
- **Frontend Audit**: `npm run build && npm run lint`
- **Security Check**: `python .agent/skills/vulnerability-scanner/scripts/security_scan.py`

**Process**: Run -> Read Output -> Summarize to User -> Ask -> Fix.
