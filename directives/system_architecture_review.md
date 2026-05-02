> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[system_architecture_review]` in audit logs.
>
> ### AI Assist Note
> 🗺️ Directive: System Architecture Review (SOP-ARC-01)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🗺️ Directive: System Architecture Review (SOP-ARC-01)

## 🎯 Primary Objective
Verify that the Tadpole OS codebase adheres to the **Hybrid Sovereign** architecture defined in `ARCHITECTURE.md`. This review ensures structural modularity and prevents the accumulation of technical debt in core hubs.

---

## 🏗️ Review Checklist

### 1. Hub Modularity (`state.rs`)
- **Check**: Ensure `AppState` remains a thin orchestrator that delegates to specialized hubs (Registry, Governance, Security, Resources).
- **Rule**: Hubs must not have circular dependencies. Registry must not depend on Governance; if needed, use a shared trait or messaging.

### 2. Provider Decoupling
- **Check**: Audit `agent/runner/mod.rs` for provider-specific logic. 
- **Rule**: 100% of LLM interaction must flow through the `llm_provider` trait. No direct imports of `gemini.rs` or `openai.rs` in the runner lifecycle.

### 3. Frontend Store Isolation
- **Check**: Verify Zustand stores (`agent_store`, `settings_store`, etc.) are atomic.
- **Rule**: Avoid large cross-store `useEffect` chains. Use the `commandProcessor.ts` for complex multi-store orchestration.

### 4. Dependency Hygiene
- **Check**: Run `cargo tree` (backend) and `npm list` (frontend).
- **Rule**: Minimize external dependencies. Favor native Rust implementations or standardized libraries (Axum, Tokio, React).

---

## 🛠️ Tools for Verification
- **Code Graph**: Review `docs/CODEBASE_MAP.md` for logic accuracy.
- **Parity Guard**: `python execution/parity_guard.py .` to ensure documentation matches implementation reality.

## 📝 Findings Protocol
Categorize findings into `Refactor Recommendations` and `Architectural Alerts`. Update `long_term_memory.md` with any fundamental shifts decided during the review.
[//]: # (Metadata: [system_architecture_review])
