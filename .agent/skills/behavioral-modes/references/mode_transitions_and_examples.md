# Behavioral Modes: State Machine Transitions & Formats (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / behavioral-modes / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Mode bleed, unverified assumptions, conversational drift.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document provides explicit output schemas, transition rules, and concrete examples for each of the 8 Sovereign operational modes.

---

## 1. Lifecycle State Machine

```
                   ┌─────────────┐
                   │ BRAINSTORM  │ (Divergent Ideation & Socratic Gate)
                   └──────┬──────┘
                          │ (Goal & Scope Confirmed)
                          ▼
                   ┌─────────────┐
                   │ ORCHESTRATE │ (Task Decomposition & Roadmap)
                   └──────┬──────┘
                          │ (Step Contract Formulated)
                          ▼
                   ┌─────────────┐
                   │  ALETHEIA   │ (Constraint Proof & Invariant Checking)
                   └──────┬──────┘
                          │ (Logic Verified)
                          ▼
                   ┌─────────────┐
                   │  IMPLEMENT  │ (Convergent Execution & Clean Code)
                   └──────┬──────┘
                          │ (Code Written)
                          ▼
                   ┌─────────────┐
         ┌─────────┤   REVIEW    │ (Audit, Security & Spec Match)
         │ (Fault) └──────┬──────┘
         ▼                │ (Pass)
  ┌─────────────┐         ▼
  │    DEBUG    │  ┌─────────────┐
  └──────┬──────┘  │    SHIP     │ (Release Gates & Final Deployment)
         │         └─────────────┘
         └─────────► (Fix Applied $\rightarrow$ Re-Review)
```

---

## 2. Mode Output Schemas & Concrete Examples

### Mode 1: 🧠 BRAINSTORM
* **Objective**: Divergent exploration without touching code. Socratic Gate applies.
* **Schema**:
  ```markdown
  ### Context & Problem Statement
  [Brief summary of goal and constraints]

  ### Options & Trade-Offs
  - **Option A ([Name])**: [Description]
    - *Pros*: [...]
    - *Cons / Risks*: [...]
  - **Option B ([Name])**: [Description]
    - *Pros*: [...]
    - *Cons / Risks*: [...]

  ### Socratic Clarification
  [1-2 targeted questions to narrow the decision before implementation]
  ```

### Mode 2: 🎼 ORCHESTRATE
* **Objective**: Decompose complex multi-file tasks into deterministic steps.
* **Schema**:
  ```markdown
  ### Mission Roadmap
  1. [ ] **Step 1: [Action]** `[MODE: ALETHEIA]` — [Criteria]
  2. [ ] **Step 2: [Action]** `[MODE: IMPLEMENT]` — [Target files]
  3. [ ] **Step 3: [Action]** `[MODE: REVIEW]` — [Audit & tests]
  ```

### Mode 3: ⚖️ ALETHEIA
* **Objective**: Proof of correctness, invariant checking, and edge-case discovery.
* **Schema**:
  ```markdown
  ### Invariants & Assumptions
  - `[Invariant 1]`: [Guaranteed property]
  - `[Assumption 1]`: [Condition verified]

  ### Edge-Case Matrix
  | Edge Case | Failure Mode | Mitigation |
  |---|---|---|
  | [Scenario] | [Impact] | [Defense] |
  ```

### Mode 4: ⚡ IMPLEMENT
* **Objective**: Zero-chatter, surgical execution. Production-grade code only.
* **Schema**:
  ```markdown
  [Code Block / Patch]

  **Changes Made**:
  - [File 1]: [1-sentence summary of non-obvious rationale]
  - [File 2]: [1-sentence summary of non-obvious rationale]
  ```

### Mode 5: 🔍 DEBUG
* **Objective**: Systematic root cause isolation before proposing any fix.
* **Schema**:
  ```markdown
  - **🔍 Symptom**: [Observed failure, error message, or unexpected output]
  - **🧪 Hypothesis**: [Falsifiable proposition of root cause]
  - **🔬 Evidence**: [Telemetry trace, log line, or reproduction output proving hypothesis]
  - **🎯 Root Cause**: [Definitive point of divergence]
  - **✅ Solution**: [Targeted minimal fix]
  - **🛡️ Prevention**: [Test or invariant added to block recurrence]
  ```

### Mode 6: 📋 REVIEW
* **Objective**: Two-axis audit (Repo standards & Spec coverage).
* **Schema**:
  ```markdown
  ### Audit Findings
  | Severity | Finding | Impact | Recommended Fix |
  |---|---|---|---|
  | Critical / High / Med / Low | [Description] | [Risk] | [Code change] |
  ```

### Mode 7: 📚 TEACH
* **Objective**: Architectural education from first principles without condescension.
* **Schema**:
  ```markdown
  ### Core Concept (First Principles)
  [Intuitive definition]

  ### System Analogy
  [Real-world comparison]

  ### Codebase Implementation
  [Walkthrough of concrete implementation in Tadpole OS]
  ```

### Mode 8: 🚀 SHIP
* **Objective**: Final release readiness and verification gates.
* **Schema**:
  ```markdown
  ### Release Verification Matrix
  - [ ] Lint & Formatting: PASS
  - [ ] Type Checking (`tsc -b`): PASS
  - [ ] Invariant Tests (`npm run test:invariants:fast`): PASS
  - [ ] Documentation & Parity Guard: PASS
  - [ ] Working Tree Clean: READY
  ```

---

## 3. Plan-Execute-Critic (PEC) Handover Contracts

When operating across complex tasks:
1. **PLAN Handover**: Must produce a discrete task list with declared target files before transitioning to `IMPLEMENT`.
2. **EXECUTE Handover**: Must run a compilation, lint, or local test command before declaring execution complete.
3. **CRITIC Handover**: If errors occur, must transition to `DEBUG` with explicit reproduction evidence rather than blindly attempting blind retries.

---

## 4. Dual-Plane Mode Adaptation (`IDENTITY.md`)

Plane 2 is a planned runtime, not an active operating mode. Until company-scoped authorization and storage exist, these mode examples are limited to synthetic previews and implementation planning; do not load live business data, persist persona artifacts, or execute customer, invoice, or connector workflows through the generic runner. A mode or plane label does not enforce `FR-11`.
