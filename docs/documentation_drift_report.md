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

### Remaining / deferred (do **not** treat as closed)
- **Code follow-ups (explicitly out of this PR)**: attach extended timeout to agent task/completions HTTP; chat.rs multi-turn/slot behavior; MCP `env_clear` hardening.
- **M3** — Full `ALLOWED_ORIGINS` default set still understated vs `cors.rs` (partial).
- **M11** — `DEPLOYMENT_GUIDE.md` Last Verified / GHCR-compose refresh still stale.
- **M12** — Fast-path vs Conductor / SpecReview operator narrative still thin in GETTING_STARTED.
- **M13** — `version.json` `version_updated_at` metadata still dated 2026-09-14.
- **M14** — `docker-compose.yml` still does not inject `ADMIN_TOKEN` / `CAPABILITY_KEY_*` (docs-only callouts for now).
- **L*** — Rust pin badge nuance, seed roster fragility, PRIVACY_MODE nuance, thin DEVELOPMENT.md — deferred.

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