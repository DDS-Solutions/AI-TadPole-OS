# Session Checkpoint & Context Compaction Templates (L3 Reference)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / context-compression / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Context degradation, attention dilution, or loss of critical architectural decisions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document provides standardized compaction templates, preservation matrices, and handoff protocols to prevent context degradation in long-running agent workflows.

---

## 1. The 3-Tier Compression Model

```
                    ┌────────────────────────┐
                    │  LEVEL 1: MICRO-COMPACT│ (Tool output truncation)
                    └───────────┬────────────┘
                                │
                    ┌───────────▼────────────┐
                    │ LEVEL 2: PHASE SUMMARY │ (Lifecycle transition)
                    └───────────┬────────────┘
                                │
                    ┌───────────▼────────────┐
                    │LEVEL 3: SESSION COMPACT│ (Full handoff / 20+ turns)
                    └────────────────────────┘
```

---

## 2. Level 1: Micro-Compact Template

Triggered immediately when ripgrep, search, or directory walks exceed 100 lines. Replace the raw dump with:

```markdown
### 🔍 Search Compaction Summary
- **Query / Pattern**: `[Query Pattern]` in `[Directory]`
- **Total Matches**: `[N]` occurrences across `[M]` files
- **Key Target Files**:
  1. `src/services/auth.rs` (Lines 42-65): Token validation handler
  2. `src/routes/login.rs` (Line 112): Route binding
- *Raw output discarded to conserve token budget.*
```

---

## 3. Level 2: Phase Transition Summary

Triggered when transitioning between development phases (e.g., `BRAINSTORM` $\rightarrow$ `ORCHESTRATE` or `ORCHESTRATE` $\rightarrow$ `IMPLEMENT`):

```markdown
### 🔄 Phase Transition Checkpoint: [Completed Phase] ➔ [Next Phase]

**Key Decisions Confirmed**:
1. Selected Option A (Axum WebSocket) over Option B (Long-Polling) for real-time telemetry.
2. Invariant: PBKDF2 iteration floor pinned at 600,000 for encrypted credential vaults.

**Files Inspected & Stored**:
- Inspected `src/config.rs` and `src/vault.rs`; API contracts confirmed.

**Execution Directive**:
Purging exploratory file dumps and proceeding directly with surgical implementation.
```

---

## 4. Level 3: Full Session Compaction & Handoff Schema

Triggered at 20+ turns, upon user request ("Summarize what we've done"), or when preparing a cross-session handoff:

```markdown
# Session Checkpoint (Turn [X])

## 1. User Objective
[Brief 1-2 sentence statement of the overarching goal]

## 2. Completed Milestones
- [x] Phase 1: Mapped codebase symbol dependencies (6,541 nodes / 33,071 edges).
- [x] Phase 2: Refactored top heavy skills to L1/L2/L3 progressive disclosure architecture.
- [x] Phase 3: Added deterministic validation scripts for API, modes, and clean-code.

## 3. Active Context & Next Actions
- [ ] In Progress: Validate code-review and context-compression skill test suites.
- [ ] Next Action: Commit all aligned skill files and verify Active Documentation Guard.

## 4. Architectural Decisions & Invariants
1. **L1/L2 Spec Sizing**: Frontmatter on line 1, L2 body strictly $\le 5,000$ words.
2. **Zero Forbidden Claims**: Never hand-write `Witness Tests:` or `Invariants:` in file headers (ADG v2 compliance).
3. **Black-Box Ergonomics**: Every skill must provide deterministic tooling in `scripts/`.

## 5. Modified Files Manifest
- `.agent/skills/clean-code/SKILL.md`
- `.agent/skills/clean-code/scripts/clean_code_linter.py`
- `.agent/skills/code-review/SKILL.md`
- `.agent/skills/code-review-graph/SKILL.md`
```

---

## 5. Multi-Session Handoff Storage

When ending a session or compacting before tool reset:
- Save the structured Level 3 checkpoint to `.tmp/handoffs/session_checkpoint.md`.
- Returning agents can immediately ingest this single artifact (~500–1,000 tokens) to achieve 100% state synchronization without re-reading conversation transcripts.