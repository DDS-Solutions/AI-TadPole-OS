# Swarm Subsystem Code Review Findings & Triage Plan

> **Date:** 2026-10-28  
> **Subsystem:** Sovereign Engine / Agent Runner Swarm Subsystem  
> **Files Audited:**
> - `server-rs/src/agent/runner/swarm/governance.rs`
> - `server-rs/src/agent/runner/swarm/dispatcher.rs`
> - `server-rs/src/agent/runner/swarm/mod.rs`
> - `server-rs/src/agent/runner/swarm/recruitment.rs`
> - `server-rs/src/agent/runner/conductor.rs`
> - `server-rs/src/agent/runner/mod.rs`

---

## 1. Executive Summary & Verdict

A thorough, line-by-line ground-truth investigation was conducted against the Sovereign Engine's Agent Runner Swarm Subsystem. 

**Verdict:** The swarm subsystem's foundational architecture (scored recruitment, tiered authority, topological wave execution, lineage cycles) is solid. However, **several critical governance and dispatch defects exist in active code paths**, most notably:
1. **Governance Fail-Open on Context Resolution Failure:** A misplaced `continue` skips both COO and Overlord approvals for a suspended agent.
2. **COO Substring Match:** Loose role matching (`role.contains("coo")`) allows "coordinator" agents to auto-approve Gate 1.
3. **UTF-8 Slicing Panic:** Byte slicing at offset 4000 can panic if a multibyte UTF-8 codepoint is split.
4. **Conductor Resolved ID Discard:** Conductor dispatches raw request tokens instead of resolved/staged agent IDs.
5. **Conductor Dispatch Policy Bypass:** Conductor plans bypass the executive CEO ("1") check and target count caps.
6. **Governance vs Recruitment Identity Mismatch:** Governance evaluates raw tokens, allowing skill-requested agents to bypass suspended status checks.
7. **Permanent Ambient Skill Pruning:** Temporary environment dependency failures permanently delete skills from SQLite.

Conversely, several reviewer findings were **overstated or false** (such as claims of duplicate step ID drops, unhandled oversight timeouts, unconstrained branch budgets, or corrupted capability registration).

---

## 2. Ground-Truth Triage Table

| # | Severity | Review Claim | Codebase Ground Truth | Status |
|---|---|---|---|---|
| **#1** | 🔴 Critical | `continue` in `governance.rs` fails open (skips both gates) | **CONFIRMED REAL**: Line 93 `continue;` in `governance.rs` continues the `for sub_agent_id in target_ids` loop, skipping both COO and Overlord approval for that suspended agent. | **Valid (P0)** |
| **#2** | 🔴 Critical | `"coordinator".contains("coo")` grants COO privilege | **CONFIRMED REAL**: Line 56 checks `a.identity.role.to_lowercase().contains("coo")`. Any role like "coordinator" auto-approves Gate 1. | **Valid (P0)** |
| **#3** | 🔴 Critical | `&output[..4000]` panics on UTF-8 boundary | **CONFIRMED REAL**: Lines 236–240 in `dispatcher.rs` slice directly on byte 4000. Any multibyte UTF-8 character crossing byte 4000 causes an uncatchable thread panic. | **Valid (P0)** |
| **#4** | 🔴 Critical | Conductor path discards resolved agent IDs | **CONFIRMED REAL**: `swarm/mod.rs` resolves IDs, but `execute_swarm_dispatch` ignores `target_ids` when `conductor_plan.is_some()`. `execute_conductor_dispatch` dispatches `step.target_agent` (raw token) directly. | **Valid (P0)** |
| **#5** | 🟡 Low | `PROTOCOL_VIOLATION` output is classified as Completed | **OVERSTATED**: Circular recruitment and max-depth guards in `execute_swarm_branch` explicitly return `SwarmBranchStatus::Blocked`. Only downstream outputs matching this prefix are unhandled. | **Partially Valid (Minor)** |
| **#6** | 🟠 High | Conductor path bypasses CEO & target count caps | **CONFIRMED REAL**: Lines 287–309 in `swarm/mod.rs` do not check `TADPOLE_CEO_ID` or cap targets to `MAX_SWARM_TARGETS_PER_CALL` (allows up to 16 steps). | **Valid (P1)** |
| **#7** | ⚪ False | Duplicate `step_id` silently drops steps | **FALSE**: `conductor.rs` lines 300–307 and 353–360 explicitly validate and reject duplicate `step_id`s and missing dependencies prior to execution. | **Debunked** |
| **#8** | ⚪ False | `step_id.unwrap_or(0)` key collision | **DEFENSIVE ONLY**: In the conductor path, `step_id` is always `Some(step.step_id)`. | **Low / Defensive** |
| **#9** | ⚪ False | `branch_count` should be wave size, not plan size | **COUNTER-PRODUCTIVE**: Child sub-budgets are derived by dividing remaining budget by `branch_count`. Using wave size would over-allocate across waves. | **Debunked** |
| **#10** | ⚪ False | `register_capability` receives whole JSON object | **FALSE**: `register_capability` in `script_skills.rs` line 234 explicitly takes `serde_json::Value` and deserializes `SkillDefinition` from it. | **Debunked** |
| **#11** | 🟡 Low | Fabricated/fallback agents omit inline API key | **CONFIRMED REAL**: Provider fallback copies provider, model_id, base_url, but not in-memory `api_key`. Needed for ephemeral non-env credentials. | **Valid (P1)** |
| **#12 & #15** | 🟠 High | Governance identity ≠ recruitment identity (TOCTOU & non-determinism) | **CONFIRMED REAL**: `governance.rs` checks raw tokens against ID/name/role before `recruitment.rs` resolves via skills/workflows. Skill tokens bypass suspended checks. | **Valid (P1)** |
| **#13** | 🟡 Low | Staged agent published with last-write-wins | **CONFIRMED REAL**: Matched existing agents are cloned and re-inserted in full, which could overwrite concurrent task/working memory changes. | **Valid (P1)** |
| **#14** | 🟠 High | Ambient unfulfilled skills are permanently pruned from stored agent | **CONFIRMED REAL**: Lines 329 & 384 overwrite `target_agent.capabilities.skills = valid_skills;` and persist to SQLite, permanently stripping skills when env vars are missing. | **Valid (P1)** |
| **#16** | ⚪ False | DB-only suspended agents invisible to governance | **CONDITIONAL**: Registry is hydrated from SQLite at boot (`state/mod.rs:112`). Only unhydrated or drifted state would be missed. | **Low Priority** |
| **#17** | ⚪ False | No timeouts on approval path (unbounded) | **FALSE**: Oversight has an explicit 300-second timeout (`oversight.rs:156`), and provider calls have network client timeouts. | **Debunked** |
| **#18** | 🟡 Low | Branch token usage not charged to parent | **PARTIALLY VALID**: Child token counts are not returned into parent `usage` struct, though monetary costs are persisted to the shared mission. | **Valid (P2)** |
| **#19** | ⚪ False | Branch budgets not drawn from mission | **FALSE**: `derive_fanout_subtask_payload` in `runner/mod.rs:281` derives bounded sub-budgets from parent remaining budget. | **Debunked** |
| **#21** | 🟡 Low | Alpha directive error taxonomy maps execution failures to HTTP 400 | **CONFIRMED REAL**: Line 463 in `swarm/mod.rs` maps all non-AppErrors to `AppError::BadRequest`. | **Valid (P2)** |
| **#22** | 🟠 High | Prompt-injection & markdown delimiter breakout on untrusted branch output | **CONFIRMED REAL**: `render_markdown` uses unescaped ```` ```text ```` fences, and dependency context uses unescaped `=== END UNTRUSTED DATA ===` markers. | **Valid (P1)** |
| **#23** | ⚪ False | Hung branch stalls wave indefinitely | **OVERSTATED**: Underlying LLM provider calls have timeouts. Adding an explicit branch-level deadline is defensive. | **Low Priority** |

---

## 3. High-Priority Shortlist (Items for Immediate Remediation)

1. **#1 (P0)**: Fix fail-open `continue` in `governance.rs` (lines 86–95) — Fail closed or fall through to Overlord Gate 2 with `coo_available = false`.
2. **#2 (P0)**: Fix loose `"coo"` role matching in `governance.rs` (line 56) — Use exact token / word-boundary checking against `"coo"` or `TADPOLE_COO_ID`.
3. **#3 (P0)**: Fix UTF-8 slicing panic in `dispatcher.rs` (line 236) — Implement `truncate_output` respecting UTF-8 char boundaries.
4. **#4 (P0)**: Thread resolved agent IDs through to Conductor dispatcher — Pass `resolved_by_target: HashMap<String, String>` into `execute_conductor_dispatch` and dispatch the resolved ID.
5. **#6 (P1)**: Enforce dispatch policy (`TADPOLE_CEO_ID` rejection + target limit) uniformly in Conductor path.
6. **#12 & #15 (P1)**: Unify resolution between Governance and Recruitment so skill-targeted agents cannot bypass suspended agent gates.
7. **#14 (P1)**: Decouple runtime capability filtering from database persistence in `recruitment.rs` so temporary missing env vars do not prune SQLite records.
8. **#22 (P1)**: Escape delimiters in `dispatcher.rs` dependency context and markdown fences in `render_markdown`.

---

## 4. Implementation Plan & Execution Phases

### Phase 1: Governance & Crash Protection (P0 Blockers)
- **Target Files:**
  - `server-rs/src/agent/runner/swarm/governance.rs`
  - `server-rs/src/agent/runner/swarm/dispatcher.rs`
- **Actions:**
  1. In `governance.rs`, replace `continue;` with `coo_available = false; coo_rationale = format!("COO context resolution failed: {}. Falling back to Overlord.", e);` so Gate 2 (Overlord) is never skipped.
  2. In `governance.rs`, replace `.contains("coo")` with exact word token comparison (`role.split(|c: char| !c.is_alphanumeric()).any(|w| w == "coo")`) and canonical ID matching.
  3. In `dispatcher.rs`, introduce `truncate_output(s: &str, max_bytes: usize) -> String` with `s.is_char_boundary(end)` backward stepping.

### Phase 2: Conductor Identity Resolution & Dispatch Policy (P0 & P1)
- **Target Files:**
  - `server-rs/src/agent/runner/swarm/mod.rs`
  - `server-rs/src/agent/runner/swarm/dispatcher.rs`
  - `server-rs/src/agent/runner/swarm/recruitment.rs`
- **Actions:**
  1. In `swarm/mod.rs`, construct `resolved_by_target: HashMap<String, String>` mapping original request tokens to resolved/staged agent IDs.
  2. Pass `resolved_by_target` into `execute_swarm_dispatch` and `execute_conductor_dispatch`.
  3. In `execute_conductor_dispatch`, look up `step.target_agent` in `resolved_by_target` before calling `execute_swarm_branch`.
  4. In `swarm/mod.rs`, apply `TADPOLE_CEO_ID` rejection and `MAX_SWARM_TARGETS_PER_CALL` to `ctx.conductor_plan` targets.
  5. In `recruitment.rs`, prevent exact-ID targeting of `TADPOLE_CEO_ID` or executive root agents without directive authority.

### Phase 3: Identity Alignment & Capability Persistence (P1)
- **Target Files:**
  - `server-rs/src/agent/runner/swarm/governance.rs`
  - `server-rs/src/agent/runner/swarm/recruitment.rs`
- **Actions:**
  1. In `recruitment.rs`, ensure `target_agent.capabilities.skills` in database persistence preserves declared skills; only runtime capabilities are filtered for execution.
  2. In `governance.rs`, align candidate resolution with `recruitment.rs` scoring so skill-requested suspended agents trigger pre-flight approval.

### Phase 4: Security Delimiters & Error Taxonomy (P1 / P2)
- **Target Files:**
  - `server-rs/src/agent/runner/swarm/dispatcher.rs`
  - `server-rs/src/agent/runner/swarm/mod.rs`
- **Actions:**
  1. In `dispatcher.rs`, escape `=== END UNTRUSTED DATA ===` markers in dependency output.
  2. In `swarm/mod.rs`, escape triple-backtick markdown fences in `render_markdown`.
  3. In `swarm/mod.rs`, map `handle_alpha_directive` execution errors to `AppError::Internal` instead of `AppError::BadRequest`.
  4. In `dispatcher.rs`, add `|| trimmed.starts_with("PROTOCOL_VIOLATION")` to `branch_output_status`.

### Phase 5: Verification & Safety Gates
- **Actions:**
  1. Run `cargo check --manifest-path server-rs/Cargo.toml` to verify Rust compilation.
  2. Add unit tests in `governance.rs`, `dispatcher.rs`, and `recruitment.rs` testing:
     - COO context resolution failure delegates to Gate 2 (does not fail open).
     - Role `"Coordinator"` does not bypass COO approval.
     - Multibyte UTF-8 dependency strings at boundary 4000 truncate without panic.
     - Conductor plan dispatches resolved agent ID.
     - Ambient skill dependency failure does not erase persisted skills in SQLite.
  3. Run `npm run adg:verify` and `npm run test:witness`.
  4. Run invariant test suite (`npx vitest run tests/invariants/`).
