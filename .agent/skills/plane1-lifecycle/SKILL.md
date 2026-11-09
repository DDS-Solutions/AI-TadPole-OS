---
name: plane1-lifecycle
description: "Sovereign Plane 1 Codebase Lifecycle Engine SOP. Enforces Git worktree sandboxing, AST blast radius inspection, TDD RED/GREEN witness proofs, and unified fast-gate verification."
when_to_use: "When executing Plane 1 codebase self-maintenance, bug fixing, refactoring, feature engineering, or dependency upgrades in the AI-Tadpole-OS repository."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "1.0.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Sovereign Governance / plane1-lifecycle
> - **Architecture**: `@docs ARCHITECTURE:Core`, `@docs ARCHITECTURE:Sandboxing`
> - **Failure Path**: Direct in-place editing of root working tree during refactors, unproven bug fixes, skipped verification gates, uncommitted debris on failure.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`) and `reports/plane1/fast_gate_results.json`

# 🐸 Plane 1 Lifecycle: Sovereign Codebase Maintenance Protocol

> **Philosophy**: Plane 1 is the Sovereign Codebase Lifecycle Engine. It recursively knows, operates, diagnoses, fixes, maintains, and improves the `AI-Tadpole-OS` codebase (`server-rs/`, `src/`, `execution/`, `.agent/skills/`).  
> **Non-Negotiables**: Never mutate dirty working trees directly during multi-file refactors; always isolate in git worktrees; never claim a bug is fixed without a failing assertion witness (RED); verify with `npm run check:plane1` before any merge.

---

## 🎯 Dual-Plane Boundary Guard (`FR-11`)

Every autonomous mission must check its plane classification:
- **Plane 1 (Engine Maintenance)**: Allowed full access to codebase tools, AST graphs, compilers, worktrees, and test runners. Must keep customer PII and company records out of source control.
- **Plane 2 (SMB Digital Twin)**: Fail-closed; must never touch code editing or repository mutations.

---

## 🔄 The 6-Phase Sovereign Execution Loop

```
[1. AST Blast Guard] ──> [2. Worktree Sandbox] ──> [3. RED Witness]
 Inspect caller/callee    Checkout isolated branch    Prove failure before
 blast radius via graph   in .tmp/worktrees/<task>    modifying codebase
         │
         ▼
[4. Clean-Code Edit] ──> [5. Fast-Gate Verify] ──> [6. Sandbox Merge/Abort]
 Minimal diff, zero       Run check:plane1 in         Squash-merge if green;
 over-engineering         sandbox (all gates green)   clean abort on failure
```

---

### Phase 1: AST Blast Radius Inspection
Before touching any source file in `server-rs/` or `src/`, inspect callers, callees, and impacted symbols:
```powershell
npm run graph:blast:guard -- --path <file_path>
npm run graph:lookup -- --name <SymbolName>
```
*Rule*: Ingest the scoped symbol slice to understand upstream dependencies and downstream callers before editing.

---

### Phase 2: Worktree Sandboxing Mandate
- **Hybrid Enforcement**:
  - **Multi-file modifications, core engine logic, or Rust routes**: **MUST** be executed within an isolated Git worktree.
  - **Single-file documentation, markdown typos, or minor comments**: Permitted to edit directly in-place.
- **Initialize Sandbox**:
  ```powershell
  python execution/plane1_worktree.py create <task-slug>
  ```
- **Constraint**: Perform all modifications, file creations, and tests exclusively inside `.tmp/worktrees/<task-slug>`.

---

### Phase 3: TDD RED Phase (Failing Test Witness Proof)
For any bug fix or behavioral modification:
1. Write a minimal, expressive test asserting the desired behavior.
2. Execute the test and capture the **failing assertion witness** (the test must fail with the expected assertion error, not a syntax error or compilation crash).
3. Record the failing evidence in the mission state or Sovereign State Unit (`{task-slug}.md`).

---

### Phase 4: Implementation (Minimal Clean-Code Patch)
Apply the smallest, most direct implementation patch:
- Minimizing cognitive load and cyclomatic complexity.
- No dead abstractions, speculative generality, or gratuitous wrappers.
- Adhere to existing patterns in the codebase (`clean-code` and `simplify-code` skills).

---

### Phase 5: Fast-Gate Pre-Flight Verification
Execute the unified Plane 1 Fast-Gate inside the worktree (or root for single-file docs):
```powershell
npm run check:plane1
```
*Fast path for frontend-only changes*:
```powershell
npm run check:plane1:fast
```
*Verification Invariant*: **All 5 Gates MUST PASS** (TypeScript + ADG v2 Manifest + Executable Invariants + Parity Guard + Rust Fast Check).

---

### Phase 6: Clean Sandbox Merge or Abort
1. **If Fast-Gate Passes (GREEN)**:
   Merge the verified sandbox back into the base branch and clean up:
   ```powershell
   python execution/plane1_worktree.py merge <task-slug>
   ```
2. **If Fast-Gate Fails after 3 attempts (Aletheia Circuit Breaker)**:
   Halt immediately. Purge the sandbox to leave zero uncommitted debris:
   ```powershell
   python execution/plane1_worktree.py abort <task-slug>
   ```
   Escalate to Entity 0 with the exact failure stack trace.

---

## 🔇 Attention Economics & Communication Protocol

1. **Initial Confirmation**: Confirm a human request once in $\le 100$ words naming the concrete object and expected outcome.
2. **Silent Execution**: Stay silent while compiling, testing, and running sandboxes. Do not stream noisy play-by-play progress logs to user chat channels.
3. **Structured Blocker Escalation**: If blocked or in need of a decision, surface a single concrete question (`human_action`). Never dump ambiguous, open-ended paragraphs.
4. **Completion Report**: Deliver the result with the verified commit SHA and test witness status in 1-2 concise sentences.

