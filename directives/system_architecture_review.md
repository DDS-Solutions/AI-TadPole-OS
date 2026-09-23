> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / system_architecture_review
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[system_architecture_review]`)

# 🗺️ Directive: System Architecture Review (SOP-ARC-01)

## 🎯 Primary Objective
Verify that the Tadpole OS codebase adheres to the **Hybrid Sovereign** architecture defined in `ARCHITECTURE.md`. This review ensures structural modularity and prevents the accumulation of technical debt in core hubs.

> 💡 **Bound Skills**:
> - Use `@[skills/improve-codebase-architecture]` to scan git hot spots and generate interactive visual HTML refactoring proposals (`.tmp/reports/architecture-review-<timestamp>.html`).
> - Use `@[skills/code-review-graph]` (`npm run graph:blast`) to measure symbol call graphs and blast radius before modifying Rust hubs or React stores.

---

## 🏗️ Review Checklist

### 1. Hub Modularity (`server-rs/src/state/mod.rs`)
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

## 🤖 Fused Audit Role: The Nexus Engineer
When executing this SOP, the orchestrator/agent MUST assume the **Nexus Engineer (v3.0)** role. Review findings using the four-pillar framework:
1. **Architectural Integrity**: Map components, identify God objects (>500 LOC), Zustand store atomic selector compliance, and Tauri IPC interfaces.
2. **Reliability & Robustness**: Concurrency safety, resource leaks (SQLite pools, WebSockets, LanceDB vector memory sweeps), and OpenTelemetry semantic alignment.
3. **Security Posture**: Neural key exposure, Tauri command arguments validation, and CORS origin checking (`is_allowed_origin`).
4. **Testing Rigor**: Complex unit identification, full coverage (happy-path, failure-path, and edge-case test planning), and mutation testing score target.

---

## 🛠️ Tools for Verification
- **Code Graph**: Review `docs/CODEBASE_MAP.md` for logic accuracy.
- **Sovereign Symbol Graph**: Run symbol graph commands (`npm run graph:blast`, `npm run graph:lookup`, `npm run graph:file`) to verify callers, callees, and compute blast radius before editing.
- **Parity Guard**: Run `python execution/parity_guard.py .` to ensure documentation matches implementation reality.
- **AI Context Alignment**: Run `python execution/verify_ai_context.py .` to verify context notes and metadata blocks.

## 📝 Findings Protocol
Categorize findings into `Refactor Recommendations` and `Architectural Alerts`. Update `long_term_memory.md` with any fundamental shifts decided during the review.