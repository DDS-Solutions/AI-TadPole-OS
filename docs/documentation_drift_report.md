> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / documentation_drift_report
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# Documentation Drift Report

## Summary
**Audited**: application source, Rust routes/error metadata, governance scripts, and the complete `docs/` publication set
**Date**: 2026-08-16 (addendum 2026-10-01)
**Baseline**: `IDENTITY.md v1.2.1`  
**Tool**: symbol graph blast guards + `parity_guard.py` + `verify_ai_context.py` + observability sync + VitePress build

---

## Gaps Closed (2026-08-16)

- Reconciled `ERROR_REGISTRY.json` with the Rust RFC 9457 metadata engine, including dynamic code patterns and a registry-loading regression test.
- Regenerated `TELEMETRY_MAP.json` from the current source tree and restored AI context headers across all 983 scanned code files.
- Corrected outward gateway documentation: the limiter is fixed-window, administrative routes require bearer authentication, imports validate before locking, and profile saves use the shared authenticated client.
- Removed the false PDF-import success path. PDF ingestion is now explicitly unavailable rather than fabricating a catalog item.
- Promoted documentation drift, observability synchronization, AI context alignment, VitePress build, and dependency audit checks to failing CI gates.
- Regenerated `API_REFERENCE.md` and documented the four outward A2A endpoints in the API contract and OpenAPI description set.

---

## Gaps Closed (2026-07-13)

### directives/ — 15 Gaps Closed
- **P0**: `TestWorkflow.md` stub replaced with full 5-phase SOP; `neural_handoff.md` upgraded to 5-field Neural Lineage Schema; `incident_response.md` Phase 0 AI-indexable check added; `emergency_shutdown.md` STASIS vs SIGKILL decision tree added.
- **P1**: `orchestrate.md` Logic-Blocker circuit breaker added; `compliance_check.md` Compliance Heartbeat cross-ref added; `security_audit.md` STASIS behavior test added; `LONG_TERM_MEMORY.md` 5 irrelevant weather API entries pruned + IDENTITY.md Sync tag added; `deploy_to_prod.md` STASIS pre-deploy gate added; `documentation_policy.md` Section 6 (Directive Version Sync Policy) added.
- **P2**: 3 duplicate PascalCase stub files deleted (`Deep Analysis.md`, `Ops Review.md`, `User Feedback Analysis.md`); `FAULT_REGISTRY.md` version header added; `sme_discovery.md` Socratic Gate link added.

### docs/ — 14 Gaps Closed
- **P0**: `ERROR_REGISTRY.json` 4 Sovereign error codes added (`BUDGET_BREACH`, `STASIS_ACTIVE`, `LOGIC_BLOCKER`, `COMPLIANCE_DRIFT`); `TROUBLESHOOTING.md` version updated to 1.2.1 + Phase 0 banner added; `SWARM_ORCHESTRATION.md` Budget Gate updated to STASIS model + Neural Lineage Schema ref added in §2; `GOVERNANCE_ROLE_GUIDE.md` 5-field handoff schema table added to CEO role section.
- **P1**: `Security_Model.md` version updated to 1.2.1 + STASIS mode added to Financial Governance section; `ARCHITECTURE.md` Rule #6 (IDENTITY.md governance + ERROR_REGISTRY first-step) added to AI context section; `RELEASE_PROCESS.md` STASIS pre-release gate + date updated; `DEPLOYMENT_GUIDE.md` STASIS pre-deploy gate + date updated; `Agent_Runner_Workflow.md` implementation map version corrected + IDENTITY.md primary authority ref added.
- **P2**: `SWARM_ORCHESTRATION.md` `(NEW)` label removed from §6; `GOVERNANCE_ROLE_GUIDE.md` duplicate version blocks consolidated.



---

## Audit Addendum — 2026-10-01 (v1.1.463 / tip `bbcf0d4`)

**Scope**: Docs-vs-code gap review against main tip `bbcf0d4b001e6e2d0a9ffeebc71fb49f219cd61c`.  
**PR intent**: Close onboarding/ops documentation drift (and minimal scrape/Dockerfile alignment). **Not** a claim that all gaps are closed.

### Addressed in the companion docs PR (this change set)
- **H1** — Document global ~60s TimeoutLayer → HTTP 408; WS for long runs (GETTING_STARTED + TROUBLESHOOTING); fix misleading router comment (agent HTTP tasks are **not** on extended timeout).
- **H2** — Document `CAPABILITY_KEY_CURR` 64-hex / ephemeral / panic.
- **H3** — Document `ADMIN_TOKEN` / `NEURAL_ADMIN_TOKEN` production requirement + must differ from `NEURAL_TOKEN`.
- **H4** — Prometheus scrape path → `/v1/engine/metrics` + bearer note; OpenAPI/API_CONTRACT/OPERATIONS metrics auth corrected; phantom `/metrics` removed from OpenAPI.
- **H5** — Dockerfile `frontend-builder` bumped `node:20-slim` → `node:22-slim` to match `package.json` engines.
- **H6** — GETTING_STARTED Vite **8** (not 6).
- **H7** — Document chat/completions last-user-only + default slot; point Neural Pivot at UI / tasks API.
- **H8** — Compose `0.0.0.0:8000` + Grafana default `admin` callout.
- **M1** — `AUTO_APPROVE_SAFE_SKILLS` default documented as **false**.
- **M2** — `VITE_NEURAL_TOKEN` local-only / do not bake into production web builds.
- **M4** — Agent card localhost `url` rewrite note for Pages.
- **M5/M6** — Agent PUT curls camelCase; `modelConfig2`/`modelConfig3` vs `planningSlot` note.
- **M7** — SECURITY.md verified-against bump to **1.1.463**.
- **M9** — Stuck-agent → `POST /v1/engine/kill` in GETTING_STARTED + TROUBLESHOOTING.
- **M10** — OpenAPI metrics security aligned.

### Remaining / deferred after Round-1 PR body (see Round-2 addendum below for closures)
- **Code follow-ups (explicitly out of Round-1/2 docs PRs)**: attach extended timeout to agent task/completions HTTP; chat.rs multi-turn/slot behavior; MCP `env_clear` hardening; Live Voice Settings-token wiring; optional tracked agents seed.
- Round-1 **M3/M11–M14/L*** deferred items are addressed as **R2-*** in the Round-2 addendum when this stacked PR lands.

### Prior "Gaps Found — Current State" (2026-08-16)
The empty/all-closed statements below this addendum reflected the 2026-08-16 audit only. Treat the 2026-10-01 list above as the current remaining-work view.

## Gaps Found — Current State (2026-08-16 snapshot; see addendum above)

### Missing Skill Documentation
- No missing skills found.

### Missing Architectural Sections
- All core sections present. ADG-04/05 sprint changes reflected in ARCHITECTURE.md v1.2.5.
- IDENTITY.md v1.2.1 governance layer now referenced in all critical entry-point docs.

### Stale Documentation
- All files in `directives/` and `docs/` updated to IDENTITY.md v1.2.1 compliance.
- `documentation_policy.md` Section 6 now enforces version string tracking for future `IDENTITY.md` version bumps.

---

## Round-2 Addendum — 2026-10-01 (stacked on PR #261)

**Branch**: `docs/round2-deploy-ws-seed-privacy-2026-10`  
**Base**: `docs/getting-started-ops-drift-2026-10` (PR #261 @ `1aa2335`). After #261 merges, retarget this PR base to `main` (or rebase).

### Closed in Round 2 (docs + tiny metadata)

| ID | Fix |
| :--- | :--- |
| **R2-H1** | GETTING_STARTED seed truth — gitignore / Alpha-only fallback; create agents 2/3 |
| **R2-H2** | DEPLOYMENT_GUIDE refresh — GHCR compose, ADMIN/CAPABILITY/PRIVACY/bind; remove dead `publish-public.ps1` |
| **R2-H3** | WEBSOCKET_EVENTS match `ws.rs`; document `/v1/engine/live-voice`; fix oversight wire shape; mark `agent:send` / `agent:user_answer` REST-only |
| **R2-H4** | Live Voice `tadpole_token` orphan caveat (GETTING_STARTED + WEBSOCKET + API_REFERENCE) |
| **R2-H5** | Fast-path / SpecReview / Conductor operator boxes (GETTING_STARTED, TEST_MISSIONS, Agent_Runner) |
| **R2-H6** | OPERATIONS verified banner → 1.1.463 / `bbcf0d4`; footer stamps |
| **R2-M1** | PRIVACY_MODE ≤15B + NullProvider / `is_degraded` |
| **R2-M2** | ALLOWED_ORIGINS full `cors.rs` defaults |
| **R2-M3** | Compose `${ADMIN_TOKEN:-}` / CAPABILITY / PRIVACY inject (empty defaults; demos still boot) |
| **R2-M4** | `version.json` `version_updated_at` → 2026-10-01 |
| **R2-M5** | DEVELOPMENT clone/Tokio/GHCR; CONTRIBUTING drop `deploy.ps1` + version.json; README badges |
| **R2-M6** | CLI_TOOLS pointer to `scripts/doctor.mjs`; dead publish script removed from DEPLOYMENT |
| **R2-M9** | Knowledge_Memory GHCR/`vector-memory` callout; SWARM `spawn_subagent` vs `recruit_specialist` |
| **R2-M8** | API_REFERENCE live-voice + timeout/metrics/vector-memory caveats |

### Still deferred / product follow-ups

- Code: Live Voice should read Settings bearer instead of `tadpole_token`; optional tracked seed JSON; agent-card Pages URL placeholder.
- #261 Round-1 items remain on the stack until that PR merges.
- Do **not** treat chat.rs timeout/MCP `env_clear` as Round-2 doc work.


## Round-3 Addendum — 2026-10-01 (stacked on Round-2 / #262)

**Branch**: `docs/round3-wiki-security-stores-starter-tauri-2026-10`  
**Base**: `docs/round2-deploy-ws-seed-privacy-2026-10` (PR #262). Tip audited: `bbcf0d4` / v1.1.463.  
**Does not redo** Round-1/2 GETTING_STARTED/ops/OpenAPI/WS/seed/privacy content.

### Closed in Round 3 (docs + starter kit skill remaps)

| ID | Fix |
| :--- | :--- |
| **R3-H1** | Wiki mass-rewrite `Tadpole-OS` → `AI-TadPole-OS` GitHub links; refresh last-verified/`bbcf0d4` where frontmatter present |
| **R3-H2** | Wiki Configuration env: `AUTO_APPROVE` default **false**, ADMIN tokens, CAPABILITY 64-hex/panic, ALLOWED_ORIGINS built-ins, PRIVACY ≤15B/NullProvider |
| **R3-H3** | `Security_Model.md` → product **1.1.463**; STASIS/`BUDGET_BREACH` marked aspirational; vault persistence aligned with SEC-02 |
| **R3-H4** | Starter-kit phantom skills remapped to registry-real tools + `STARTER_KITS.md` caveat |
| **R3-H5** | `Frontend_State_Management.md` full store list; WS **`/v1/engine/ws`**; Settings/Tauri token seeding |
| **R3-H6** | Desktop `.neural_token` + `get_neural_token` IPC in GETTING_STARTED, DEPLOYMENT, wiki Installation |
| **R3-M1** | Wiki Continuity stub + sidebar; points at `/v1/continuity/*` and `continuity/workflow/` |
| **R3-M3** | Mythos `intelligence/` path + dual `act_threshold` (0.9 vs 0.95); GEV repo URL; ADG `file://` → relative |
| **R3-M4** | ERROR_REGISTRY: `BUDGET_BREACH`/`STASIS_ACTIVE` `emitted:false`; add `WORKFLOW_STEP_FAILED`; TELEMETRY_MAP FE-only note in AGENTS/CLAUDE + generator; SECURITY_REGISTRY date bump |
| **R3-M5** | `POLLYWOG-DEBT.md` stub; CLAUDE↔AGENTS `graph:blast:guard` alignment; ROADMAP Piper/template-count tone-down |
| **R3-M6** | README pointer to `apps/mobile-android` |

### Still deferred / out of Round-3 docs PR

- Code: STASIS runtime, Live Voice Settings-token wiring, chat.rs/MCP hardening (explicitly excluded).
- Full wiki line-anchor audit; TELEMETRY_MAP Rust scanner extension; ADG manifest `verified_at` regen; full Continuity operator manual; Benchmark_Spec / token_telemetry_mission deep refresh (optional follow-ups).
  - *(Kill-Switches anchors + Benchmark_Spec header + Criterion honesty addressed in Round-4.)*


## Round-4 Addendum — 2026-10-01 (stacked on Round-3 / #263)

**Branch**: `docs/round4-arch-glossary-benchmark-2026-10`  
**Base**: `docs/round3-wiki-security-stores-starter-tauri-2026-10` (PR #263). Tip audited: `bbcf0d4` / v1.1.463.  
**Does not redo** Round-1–3 GETTING_STARTED/ops/WS/seed/privacy/wiki-repo-rename/Security_Model primary STASIS narrative (except Kill-Switches anchors + SEC-08 residual).

### Closed in Round 4 (docs)

| ID | Fix |
| :--- | :--- |
| **R4-H1** | `ARCHITECTURE.md` / `Architecture_Overview.md` → product **1.1.463**; tone down 100%/sub-ms/zero-stall / Verified Production-Ready |
| **R4-H2** | `CODEBASE_MAP` + ARCH directory tree: `templates/`, `/v1/engine/ws`, add missing `adapter`/`middleware`/`networking`/`services`/`system`/`types`; drop Merkle-folder fiction |
| **R4-H3** | Wiki Kill-Switches line anchors → L61/L127/L223/L239/L243; budget row no longer invents Kill-Switch auto-halt via `BudgetExhausted`/STASIS |
| **R4-H4** | BLOG / org_singularity / DESIGN_SYNERGY / design.md — marketing disclaimers, version honesty, corrupt leading fence fixed |
| **R4-M1** | `Benchmark_Spec.md` header → 1.1.463; document `/v1/benchmarks` CRUD; keep Criterion benches absent honesty |
| **R4-M2** | `GLOSSARY` Micro-Dollar Merkle invention removed; `/v1/benchmarks` path; verified-against stamp |
| **R4-M3** | `GOVERNANCE_ROLE_GUIDE` SoT → `agent/types/oversight.rs`; version **1.1.463** |
| **R4-M4** | `QWEN_LOCAL_INTEGRATION` Linux/macOS path + Secure Credentials Vault rename; version bump |
| **R4-M5** | `RELEASE_PROCESS` STASIS aspirational (not release gate); drop dead `publish-public.ps1`; WS `/v1/engine/ws` |
| **R4-M6** | `agent-contract-spec` `active_model_slot`; `persistence/` module |
| **R4-M7** | `SECURITY_REGISTRY` → 1.0.2; SEC-08 metering/`check_budget` honesty (no STASIS kill-switch) |
| **R4-M8** | `SUPPORT.md` placeholder removed; `CHANGELOG_RECONSTRUCTION` public-unverifiable SHA note |

### Still deferred / out of Round-4 docs PR

- Code: STASIS runtime, Live Voice Settings-token wiring, chat.rs/MCP hardening (explicitly excluded).
- Full wiki mass line-anchor audit beyond Kill-Switches; TELEMETRY_MAP Rust scanner; ADG `verified_at` regen.
- Deep Criterion bench implementation (docs correctly leave checklists unchecked).

