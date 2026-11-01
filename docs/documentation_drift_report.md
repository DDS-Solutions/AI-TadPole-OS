> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / documentation_drift_report
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# Documentation Drift Report

## Summary
**Audited**: application source, Rust routes/error metadata, governance scripts, and the complete `docs/` publication set
**Date**: 2026-10-29
**Baseline**: `IDENTITY.md v1.1.464` (Dual-Plane Sovereign Hierarchy)  
**Tool**: symbol graph blast guards + `parity_guard.py` + `verify_ai_context.py` + observability sync + VitePress build

**Latest audit**: 2026-10-30 — see `reports/DOC_AUDIT_20261030.md` for source-alignment findings, corrections, verification results, and the remaining comment-density finding.

---

## Documentation alignment (2026-10-29 — Dual-Plane Sovereign Hierarchy)

- **Dual-Plane Sovereign Hierarchy (`IDENTITY.md v1.1.464`)**:
  - Codified **Plane 1 (`Sovereign Codebase Lifecycle Engine` / `Engine-Maintenance`)** and **Plane 2 (`Company-Agnostic SMB AI Digital Twin` / `SMB-Digital-Twin`)** across `directives/IDENTITY.md`, `AGENTS.md`, `GEMINI.md`, `CLAUDE.md`, `README.md`, `docs/ARCHITECTURE.md`, `docs/Architecture_Overview.md`, `docs/Security_Model.md`, `docs/OPERATIONS_MANUAL.md`, `docs/GETTING_STARTED.md`, `docs/SWARM_ORCHESTRATION.md`, `docs/GOVERNANCE_ROLE_GUIDE.md`, `docs/GLOSSARY.md`, `docs/wiki/Glossary.md`, and `docs/wiki/Home.md`.
  - Declared the **`FR-11` Permission Firewall** across `.agent/skills/intelligent-routing/SKILL.md`, `route_dispatcher.py`, `.agent/skills/behavioral-modes/SKILL.md`, `execution/map_agent_skills.py`, `directives/neural_handoff.md`, and `directives/LONG_TERM_MEMORY.md`. These declarations do not enforce runtime permissions; `route_dispatcher.py` classifies and routes prompts only.

## Dual-plane implementation status correction (2026-10-30)

The earlier report overstated `FR-11` enforcement. The Rust runner now fails closed only when an agent's stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not activate the guard. It does not provide company-scoped business execution, and the SMB digital-twin specification still marks isolation and connector implementation as pending. Plane 2 skill metadata is restricted to read-only discovery tools, but that metadata is not a substitute for runtime authorization or workspace isolation. Do not report `FR-11` or `NFR-1` as fully implemented until the server-side company boundary is in place and verified.

## Runtime budget and governance status correction (2026-10-30)

The earlier notes that listed `BUDGET_BREACH`, `STASIS_ACTIVE`, `LOGIC_BLOCKER`, and `COMPLIANCE_DRIFT` as executable runtime errors overstated the implementation. The Rust source defines per-agent and per-mission quota reservations and the `BUDGET_EXHAUSTED` error; it contains no global STASIS state, budget-triggered swarm suspension, special budget snapshot or alert, three-cycle `LOGIC_BLOCKER`, or automated 10-operation compliance heartbeat. The four labels were removed from `docs/ERROR_REGISTRY.json`. Related security, swarm, shutdown, release, and troubleshooting guidance now marks those actions as manual policy or pending runtime work.

The current emergency stop is the persisted `dispatch_paused` control documented in `OPERATIONS_MANUAL.md`; it is independent of budget exhaustion. `execution/sovereign_audit.py`, previously referenced by several directives, is not present. Current repository checks are `execution/parity_guard.py`, `execution/verify_ai_context.py`, and `npm run adg:verify`.

## Documentation and security-boundary correction (2026-10-30)

- Reframed Privacy Shield documentation as runner model routing plus selected MCP name checks. It is not a system-wide egress firewall; the global monitor itself periodically sends HTTP `HEAD` reachability probes while global mode is active.
- Corrected credential-flow guidance: encrypted provider keys are stored in browser storage, then the selected decrypted key is included in the browser-to-engine task request and used for the provider call. The vault is not end-to-end encryption from the engine.
- Corrected embedding guidance: the documented BGE-Small ONNX default was not present in the current text-embedding code. The knowledge store calls Gemini `text-embedding-004`; connector ingestion resolves an embedding provider from agent configuration.
- Corrected storage-path guidance: the default SQLite file is `data/state/tadpole.db`; when it is absent, startup copies a legacy `data/tadpole.db` and its WAL/SHM sidecars without removing the old source. Runner cluster workspaces are created under `data/workspaces/<cluster-id>`. Memory listing and snapshots prefer `data/state/workspaces/` when present, otherwise use `data/workspaces/`, and do not merge both roots.
- Removed the claim that cluster workspaces are fully isolated. Runner file tools apply checks to their configured workspace root, but that path boundary is not company/tenant authorization and does not restrict every tool or network path.
- Rewrote remaining “100% local”, “zero data leaks”, and production-ready claims that implied guarantees the code does not provide. Historical evaluations and launch copy are marked as historical or draft material.
- Updated network, vault, cost, quota, kill-switch, compliance, onboarding, and starter-kit guidance. Budget exhaustion remains per-agent/per-mission call reservation enforcement; it does not globally suspend the swarm.
- Removed stale handler paths from `API_REFERENCE.md`; router/OpenAPI parity and documentation-link checks pass.

Verification for this audit is recorded in `reports/DOC_AUDIT_20261030.md`. Source-comment density remains below the directive's 15% target in `src/` (13.8%), `server-rs/` (9.2%), and `src-tauri/` (9.8%); this is tracked as a code-comment documentation-quality finding, not as factual documentation drift.

---

## Gaps Closed (2026-10-06 — Phase 1–4 Completion Failure Remediation)

- **Phase 1: Workspace Files & Task Dispatch Envelope Alignment**:
  - Reconciled `/v1/system/workspaces/files` response type contract (`{ files: string[], total: number, truncated: boolean }`) with `Terminal.tsx` autocomplete parser to gracefully handle both plain array and object envelopes without breaking file search.
  - Updated `POST /v1/agents/{id}/tasks` to synchronously return `task_id` in the `202 Accepted` response, and updated `dispatch_service.ts` to return `{ success, task_id }` with telemetry event emission.
- **Phase 2: Agent Pause/Resume Lifecycle & Workflow Execution Route**:
  - Replaced silent `let _ =` queries in `pause_agent` and `reset_agent` with explicit DB error propagation. Updated `resume_agent` response contract to return `{ status: "ok", agent_id, state: "idle", reopened_mission: false }` with actionable status messaging.
  - Added missing `POST /v1/continuity/workflows/{id}/run` route in `routes/continuity.rs` and `router.rs`, added `run_workflow` to `continuity_api.ts`, registered key in `system_api_service.ts` allowlist, and added full unit tests.
- **Phase 3: Auth Integrity & Model Infrastructure Truthfulness**:
  - Enhanced `decide_oversight` in `oversight_api.ts` to sign requests whenever valid keys exist (not strictly restricted to `key_source === 'operator'`), and throw an actionable error in production when operator key pair is missing.
  - Added local Ollama proxy fallback for `node_id == "local" || "localhost"` in `pull_model` (`model_manager.rs`), and added fallback "Local Engine (Ollama)" node in `Model_Store.tsx` when no remote swarm bunkers are discovered.
  - Returned `requires_restart: true` in `UpdateEnvironmentResponse` from `/v1/system/environment` (`system.rs` & `engine_api.ts`) to clarify that running runtime state requires engine restart for altered variables to take effect.
  - Added explicit handling in `CapabilityRegistryService` to wrap HTTP 403 Forbidden responses into clear administrative privilege errors.
- **Phase 4: Knowledge Base, CAS History & Search Transparency**:
  - Updated `/v1/docs/knowledge` in `docs.rs` to return `200 OK` with `[]` instead of `404 Not Found` when the knowledge documentation directory is unindexed, preventing frontend crashes.
  - Added `confirm_knowledge` client method in `intelligence_api_service.ts` matching `POST /v1/knowledge/{id}/confirm`.
  - Added `get_file_history` and `restore_file_version` in `workspace_api.ts` and `system_api_types.ts` for `/v1/cas/history` and `/v1/cas/restore`.
  - Annotated queried retrieval sources `["bm25", "trustgraph", "knowledge_meta"]` in `/v1/memory/search/hybrid` (`memory.rs`).
  - Added comprehensive verification steps and Mission 11 (Workflow Pipeline) to `docs/TEST_MISSIONS.md`.

## Gaps Closed (2026-10-06 — Mission, Continuity, Benchmark & Graph Remediation)

- **Mission Sync vs. Execution & Clone Lifecycle**: Clarified `sync_mission` response contract (`status: "synchronized"`, `executed: false`, explicit message explaining dispatch via `/tasks` is required for execution); documented `clone_mission` creating a clean `pending` draft record with zeroed financial cost without copying prior findings or auto-starting.
- **Scheduled Job Run Correlation**: In `run_job_now_handler`, synchronously generate and return `run_id` in the `202 Accepted` response while emitting the `continuity:job_triggered` telemetry event with `run_id`, allowing immediate UI tracking and linking dispatched runs into `execute_job_with_run`.
- **Benchmark Suite Truthfulness & Failure Persistence**: Persisted `status: "FAIL"` results with error details into the SQLite benchmark ledger when background suites encounter execution errors; reclassified `BM-RUN-01` from "Agent Runner Baseline" to "Agent Registry Resolution Latency" (`category: "Registry"`) with accurate descriptions.
- **Symbol Graph Degradation & Memory Source Truthfulness**: Gracefully degraded `/v1/intelligence/graph` to `200 OK` with `status: "unindexed"` and empty node/edge sets when symbol graph indexing is unavailable, preventing HTTP 500 error cascades in the Neural Map UI; annotated `/v1/memory/graph` with `"source": "markdown_concept_notes"` and node/edge count metadata.

## Gaps Closed (2026-10-05 — Security Review Remediation)

- **Centralized Subprocess Isolation (`create_isolated_command`)**: Consolidated process execution across all 8 call sites (`system_tools.rs`, `skill.rs`, `hooks.rs`, `plugin.rs`, `native.rs`, `skillspector.rs`, `deploy.rs`, `templates/source.rs`) onto a single spawn helper that enforces `cmd.env_clear()`, safe host environment variable allowlisting, and process group cleanup (`kill_on_drop`).
- **Plugin Runner & Integrity Sandboxing**: Enforced `interpreters_trusted()` (`TADPOLE_TRUST_INTERPRETERS`) on plugin interpreters and `run_integrity_check`, bounded plugin child execution with a 60-second timeout, and eliminated ambient parent credential inheritance.
- **Oversight Public Key Pinning**: Refused signed oversight approvals in `verify_oversight_signature_canonical` when `OVERSIGHT_PUBLIC_KEY` is not pinned, preventing self-approved decisions. Added fail-fast startup validation requiring a valid 32-byte hex `OVERSIGHT_PUBLIC_KEY` in production.
- **Template Clone Target URL Binding**: Pinned `git clone` to the pre-validated `validated_target.url` rather than caller-supplied raw strings, and isolated git subprocess environments.
- **Fail-Closed NullProvider**: Replaced silent degraded completions and zeroed embeddings with `AppError::ServiceUnavailable` outside explicit test harnesses, preventing corrupted mission rankings.
- **Node Discovery Contract Alignment**: Replaced stubbed discovery scans that returned false positive success with explicit `AppError::NotImplemented` responses.
- **Audio Synthesis Contract Unification**: Aligned `/v1/engine/speak` responses to consistently stream binary audio bytes, rejecting browser fallback or unsupported configurations with appropriate HTTP errors.

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

## Gaps Found — Current State

### Missing Skill Documentation
- No missing skills found.

### Missing Architectural Sections
- All core sections present. ADG-04/05 sprint changes reflected in ARCHITECTURE.md v1.2.5.
- `IDENTITY.md v1.1.464` Dual-Plane Sovereign Hierarchy governance layer now referenced in all critical entry-point docs.

### Stale Documentation
- The 2026-10-29 sweep updated the files listed above. The 2026-10-30 review corrected its Plane 2 status claims; this historical statement is not evidence of current complete conformance.
- `documentation_policy.md` Section 6 now enforces version string tracking for future `IDENTITY.md` version bumps.
