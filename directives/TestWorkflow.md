> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[TestWorkflow]` in audit logs.
>
> ### AI Assist Note
> SOP-QA-01: Test & Verification Workflow. Operational gate for all Tadpole OS code changes.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py` and `execution/verify_all.py`.
>
> ## 🎯 Primary Objective
> Prove that code changes are correct by **running them**, not merely inspecting them. Every modification to the Tadpole OS codebase must pass this verification gate before being considered complete.
>
> > [!IMPORTANT]
> > This directive is the operational implementation of `GEMINI.md § Final Checklist`. Running this SOP satisfies the `IDENTITY.md Directive #8` Compliance Heartbeat requirement for the current session.
>
> ---
>
> ## 🏗️ Verification Phases
>
> ### Phase 1: Static Analysis
> - **Rust**: `cargo clippy --workspace --all-targets -- -D warnings`
> - **Frontend**: `npm run lint` (or `eslint src/`)
> - **Fix All**: No warnings or errors may be suppressed without a documented justification.
>
> ### Phase 2: Unit & Integration Tests
> - **Rust**: `cargo test --workspace`
> - **Frontend**: `npm run test`
> - **Requirement**: All tests must pass. No `#[ignore]` tags permitted without a linked issue.
>
> ### Phase 3: Sovereign Full-Suite Audit
> - **Execution**: `python execution/verify_all.py .`
> - **Order**: Security → Lint → Schema → Tests → UX → SEO → E2E
> - **Gate**: Do not proceed to Phase 4 until all Critical blockers are resolved.
>
> ### Phase 4: Parity Guard
> - **Execution**: `python execution/parity_guard.py .`
> - **Goal**: Confirm documentation matches implementation. Any `[MISMATCH]` output is a blocker.
>
> ### Phase 5: AI Context Alignment
> - **Execution**: `python execution/verify_ai_context.py .`
> - **Goal**: Confirm all AI Assist Note headers are present and version-pinned to `IDENTITY.md v1.2.1`.
>
> ---
>
> ## 🚦 Gate Conditions
> - **PASS**: All 5 phases complete with zero critical/high findings.
> - **FAIL**: Any critical blocker in Phases 1–4. Must be resolved before the task-slug is marked complete.
> - **Logic-Blocker**: If Phase 3 fails 3 consecutive times without convergence, escalate to Entity 0 per `IDENTITY.md Directive #3`.
>
> ## 📝 Reporting
> Log verification results to `{task-slug}.md` under a `## ✅ Verification` section before context compaction.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.
[//]: # (Metadata: [TestWorkflow])
