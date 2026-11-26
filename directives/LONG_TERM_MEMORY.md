> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / LONG_TERM_MEMORY
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[LONG_TERM_MEMORY]`)

# 🧠 Tadpole Engine: Persistent Ledger (Long-Term Memory)
**Intelligence Level**: High (ECC Optimized)
**Source of Truth**: `server-rs/src/memory.rs`, `directives/LONG_TERM_MEMORY.md`
**Last Hardened**: 2026-10-29
**IDENTITY.md Sync**: v1.1.464
**Standard Compliance**: ECC-MEM (Enhanced Contextual Clarity - Memory Standards)


---

## 🧠 Memory Lifecycle & Retrieval

```mermaid
graph LR
    Event["New Insight (Event)"]
    Embed["Vectorize (Embedding)"]
    Store["LanceDB (Storage)"]
    Search["Cosine Search (Retrieval)"]
    Rerank["Heuristic Rerank (Context)"]

    Event --> Embed
    Embed --> Store
    Store -- "Query" --> Search
    Search --> Rerank
```

---

# Long-Term Memory (Persistent Ledger)

Last Updated: 2026.10.29

## 🏛️ System Policies & SOP Specifications (Operational Rules)
> [!NOTE]
> Items in this section define operational rules and SOP formats for *future* executions. They do NOT indicate that a task or report has already been executed. Agents MUST verify file existence on disk before asserting completion.

- **Dual-Plane Memory & Data Isolation [SOP Rule]**: `directives/LONG_TERM_MEMORY.md` stores strictly **Plane 1 (`Engine-Maintenance`)** architectural wisdom, crash reconciliations, and SOP rules. **Plane 2 (`SMB-Digital-Twin`)** customer records, invoices, and founder persona calibrations must remain isolated in company-scoped stores when that runtime exists; `FR-11` / `NFR-1` are requirements, and the store implementation remains pending. Until then, do not persist company data or persona artifacts through the generic runner; use synthetic inputs and stop at preview. The runner rejects only an agent whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not trigger that guard.
- **Glossary Compliance Alignment [SOP Rule]**: When a glossary compliance audit is executed, any discrepancies between `docs/wiki/Glossary.md` (SSOT) and codebase structures must be documented in a structured `Glossary_Drift_Report.md` and routed to the `Alpha` node (Department Lead). *(Do not assume this report exists until an audit is actively performed and written to disk).*
- **Doc Path Resolution**: Documentation files like `docs/wiki/Glossary.md` must be referenced using workspace-relative paths (e.g. `docs/wiki/Glossary.md`) rather than absolute OS paths to avoid Tauri sandbox security violations.
- **API Key Fallback Protocol**: When external API key errors (`invalid_api_key`) prevent subagent swarming, agents must pivot to centralized manual synthesis using local architecture models to maintain progress.
- **Bunker Deployment**: Filesystem access can support offline deployments when models, dependencies, and integrations are local; enforce a true disconnected boundary with host/network controls.
- **Clean Code Ratchet & Zero-Unwrap SOP [SOP Rule]**: All modifications across Rust, TypeScript, and Python must satisfy clean code constraints ($\le 500$ LOC/file, $\le 50$ LOC/function, $\le 4$ block nesting depth, $< 6$ parameters per function via parameter object bundling). In Rust engine production code (`server-rs/src/`), raw `.unwrap()` calls are strictly forbidden and must be replaced with fail-safe error propagation (`?`, `ok_or_else`, or default matchers). Pre-commit changes must pass `npm run lint:clean:diff`.
- **Agent Handoffs**: CEO-to-Alpha handoffs require explicit reasoning protocol injection and `[Execution-Plane]` classification.
- **Hook Registry**: Pre-tool hooks are mandatory for security auditing.

## 🧠 Verified Historical Engine Resolutions (Past Hardening)
- **Sub-Agent Model Override Isolation**: Clearing parent model details in `TaskPayload` via `derive_subtask_payload` ensures spawned sub-agents resolve their own model configurations instead of inheriting parent settings. (Verification: Correct model resolution in recruitment routing tests).
- **Specialist Recruitment Dispatching**: Correctly resolving and propagating the `target_id` inside `ensure_sub_agent_exists` prevents dispatcher failures and guarantees that swarms execute specialist roles accurately. (Verification: Full cargo test suite verification passes).
- **Safe Multi-byte String Pruning**: Emojis and multi-byte UTF-8 sequences cause Rust's `String::truncate` to panic if sliced at arbitrary byte indices. Always align the index using `floor_char_boundary(limit)` before truncating. (Verification: Resolves crash in `server-rs::utils::serialization`).
- **Engine Crash Reconciliation**: Startup crash reconciliation logs panic details to SQLite and marks crashed missions as failed, ensuring post-mortem analysis availability. (Verification: Resolves stuck active missions).
- **Workspace Verification Tools**: Repository checks available in the current tree include `execution/parity_guard.py`, `execution/verify_ai_context.py`, and `npm run adg:verify`. Tool access is limited by the active workspace permissions; `awaken.py` and `sovereign_audit.py` are not present in the repository.
- **High-Fidelity Context Retention**: Through the AI Awakening protocol, all core files now provide explicit architectural metadata to maintain consistent agent reasoning over long-term missions.
- **Codebase-Wide Clean Code & Zero-Unwrap Remediation**: Replaced 53 runtime `.unwrap()` instances in `server-rs/src/` with safe error propagation (`?`, `ok_or_else`, default matchers); decomposed monolithic modules across Rust (`customer_catalog/`, `bin/graph_query/doc_guard/`, `agent/business/service/`, `runner/intelligence/`), TypeScript (`vault_store.ts`, `settings_store.ts`, `useStandups.ts`, `agent_service.ts`, `system_api_service.ts`), and Python (`parity_guard.py` and `verify_ai_context.py` via `execution/lib/`); integrated CI ratchets `npm run lint:clean` and `npm run lint:complexity`. (Verification: 100% green across fast-gates, invariants, and cargo checks).
- **Integrity Sources**: Use the documented parity, AI-context, and ADG checks for repository validation. `version.json` is the product release authority; no `sovereign_audit.py` script is the repository integrity source.

## Session Markers
- Initializing high-security agentOS enhancements.
