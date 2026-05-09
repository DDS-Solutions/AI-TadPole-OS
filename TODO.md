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

[//]: # (Metadata: [TODO])
