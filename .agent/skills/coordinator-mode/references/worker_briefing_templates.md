# Multi-Agent Worker Briefing Templates & Orchestration Architecture (L3 Reference)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / coordinator-mode / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Lazy subagent prompt delegation, concurrent write collisions, unhandled worker failures.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document provides standardized worker briefing schemas, multi-agent coordination patterns (Fork-Join, Pipeline, Critical Section), and worker output synthesis checklists.

---

## 1. Concrete Worker Briefing Schemas

Every worker subagent briefing must provide bounded scope, explicit file targets, concrete constraints, and verification requirements.

### A. 🔍 Research & Investigation Worker Briefing
```text
Role: Exploratory Research Subagent (Read-Only)
Task: Investigate symbol dependencies and caller hierarchy for [Target Symbol] in [File Path].
Context: We are resolving [Defect/Goal] in [Subsystem].
Scope Boundaries:
- Target File: [File Path]
- Callees & Callers: Inspect only direct 1-hop connections.
Constraints:
- READ-ONLY. Do not modify any code files.
- Keep total output under 200 words.
Deliverable: Return structured markdown:
1. Exact line numbers of definition and usages.
2. Potential side effects or edge cases identified.
```

### B. ⚡ Surgical Implementation Worker Briefing
```text
Role: Implementation Subagent (Write-Scoped)
Task: Implement [Change Description] in [Target File].
Context: Approved architectural blueprint [Link/Reference].
Scope Boundaries:
- Target File: [Target File] (Lines [Start Line] to [End Line])
Constraints:
- Zero chatter. Surgical implementation only.
- Adhere strictly to clean-code standards (Guard clauses, max 3 indent levels).
- Do NOT modify public API signatures without coordinator authorization.
Verification Requirement:
- Run `cargo check` or `npx vitest run [test_path]` to prove zero syntax or type regressions.
Deliverable: Single code patch with 1-sentence non-obvious rationale.
```

### C. 🛡️ Verification & Security Audit Worker Briefing
```text
Role: QA / Security Auditor Subagent (Adversarial Review)
Task: Audit diff between [Base Commit] and HEAD for [Target Subsystem].
Scope Boundaries:
- Files: [List of modified files]
Rubric Focus:
- Correctness, input validation, SQL/Command injection hazards, unhandled error drops.
Deliverable: Tabulated Two-Axis Review report (Standards vs Spec) with severity tags (BLOCKING, SUGGESTION, NIT).
```

---

## 2. Multi-Agent Coordination Topologies

### 1. Fork-Join (Map-Reduce) Pattern
- **When to Use**: Broad exploratory research, parallel file audits, independent module scans.
- **Workflow**:
  ```
                         ┌─────────────┐
                         │ COORDINATOR │
                         └──────┬──────┘
             ┌──────────────────┼──────────────────┐
             ▼                  ▼                  ▼
      ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
      │  Worker A   │    │  Worker B   │    │  Worker C   │ (Read-Only)
      └──────┬──────┘    └──────┬──────┘    └──────┬──────┘
             └──────────────────┼──────────────────┘
                                ▼
                         ┌─────────────┐
                         │  SYNTHESIS  │ (Coordinator distills & filters)
                         └─────────────┘
  ```

### 2. Sequential Pipeline Pattern
- **When to Use**: Staged transitions where output of phase N is the contract for phase N+1.
- **Rule**: `RESEARCH` $\rightarrow$ `ARCHITECT` $\rightarrow$ `IMPLEMENT` $\rightarrow$ `VERIFY`.

### 3. Critical Section / File Lock Pattern
- **Non-Negotiable Invariant**: **Never allow two workers to edit the same file concurrently.**
- If two subtasks touch `src/routes/system.rs`, sequence them serially: Worker 1 completes and verifies $\rightarrow$ Worker 2 begins.

---

## 3. Coordinator Synthesis Checklist

When workers return outputs, the Coordinator must **never blindly paste** their responses. Execute this checklist:
- [ ] **Completeness**: Did the worker answer the exact scope requested without evading constraints?
- [ ] **Hallucination Check**: Do cited line numbers, function names, and file paths exist in the workspace?
- [ ] **Cross-Worker Consistency**: Do findings from Worker A contradict findings from Worker B? If so, resolve divergence.
- [ ] **Synthesis Compression**: Distill worker findings into a concise, high-signal briefing for the user or subsequent worker.