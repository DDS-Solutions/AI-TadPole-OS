---
name: tdd-workflow
description: TDD workflow. Red-Green-Refactor.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# TDD Workflow

**Write tests first.**

## The Cycle
1.  🔴 **RED**: Write failing test.
2.  🟢 **GREEN**: Minimal code to pass.
3.  🔵 **REFACTOR**: Improve without breaking.

## Principles
- **Laws**: No code without failing test. Just enough to fail. Just enough to pass.
- **Green**: YAGNI. Simplest thing.
- **Refactor**: Remove duplication. Improve naming.

## AAA Pattern
1.  **Arrange**: Setup.
2.  **Act**: Execute.
3.  **Assert**: Verify.

## Prioritization
1.  Happy path.
2.  Error cases.
3.  Edge cases.

---

## 🧠 Aletheia Reasoning Protocol (Test-First)

### 1. Generator (Spec)
*   **Input**: "Null? Empty?".
*   **Output**: "Throw or return null?".
*   **State**: "Side effects?".

### 2. Verifier (Check)
*   **Fail**: "Did it fail first?".
*   **Pass**: "Was it simple?".
*   **Coverage**: "Missed catch block?".

### 3. Reviser (Refactor)
*   **Clean**: "Remove duplication.".
*   **Name**: "Descriptive test names.".

---

## 🛡️ Security & Safety Protocol (TDD)

1.  **Secrets**: No real keys in tests.
2.  **Boundaries**: Test auth explicitly.
3.  **DoS**: Test loop limits/large inputs.
4.  **Mocking**: Don't mock security verifiers.
