> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[TODO]` in audit logs.
>
> ### AI Assist Note
> Tadpole OS Technical Debt & Backlog
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Tadpole OS Technical Debt & Backlog

## Maintenance & Hygiene [COMPLETE]
- [x] Prune unused imports across the `server-rs` project.
- [x] Modularize `AppState` hubs into `src/state/hubs/`.
- [x] Simplify `router.rs` with extracted middleware and sub-routers.
- [x] Standardize `execution/` scripts with `[OK]` / `[FAIL]` reporting.
- [x] Clean root directory (moved `.log`, `.txt`, `.py`, `.ps1` to appropriate subdirs).

## Upcoming Features
- [ ] Implement advanced RAG weighting for mission clusters.
- [ ] Add support for local ONNX embeddings in the engine.

## Nexus Architectural Refactoring [COMPLETE]
- [x] **Command Processor Refactor** (`src/logic/command_processor.ts`): Transition from monolithic dispatcher to Strategy Pattern / Plug-in Architecture.
- [x] **State Management Hardening** (`src/stores/workspace_store.ts` & `provider_store.ts`): Extract side-effects and API logic into dedicated Service Layers.
- [x] **Backend Orchestration Modularization** (`server-rs/src/agent/runner/synthesis.rs`): Decompose the synthesis cycle into discrete, trait-based modules.
- [x] **Layout Logic Extraction** (`src/layouts/Dashboard_Layout.tsx`): Move global state glue and event listeners into HOCs or specialized hooks.
- [x] **Documentation Engine Migration** (`src/pages/Docs.tsx`): Transition to a Markdown-driven content management system.

[//]: # (Metadata: [TODO])
