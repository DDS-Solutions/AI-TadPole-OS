> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / context-compression
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Context degradation or loss of critical architectural decisions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[CONTEXT_COMPRESSION]`)

# Session Checkpoint & Context Compaction Templates (L3)

---

## 1. Level 3 Session Checkpoint Template

```markdown
# Session Checkpoint (Turn [X])

## 1. Completed Phases
- [x] Phase 1: Mapped codebase symbol dependencies (4,889 nodes / 21,849 edges).
- [x] Phase 2: Refactored top heavy skills to L1/L2/L3 progressive disclosure.

## 2. In-Progress / Next Action
- [ ] Phase 3: Run final verification gates and update walkthrough report.

## 3. Key Decisions & Rationale
1. Strict L2 Line Limit: Kept all L2 SKILL.md files under 100 lines.
2. Zero Secrets Rule: Never store keys/tokens in memory topic files.
3. Path Portability: Replaced hardcoded paths with relative markdown links.

## 4. Modified Files
- `.agent/skills/rust-pro/SKILL.md`
- `.agent/skills/game-development/SKILL.md`
```

---

## 2. Multi-Session Compaction Handoff

Save structured compaction files to `.tmp/handoffs/session_checkpoint.md` for seamless recovery across new agent sessions.