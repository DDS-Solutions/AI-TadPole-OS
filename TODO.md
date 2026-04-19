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
