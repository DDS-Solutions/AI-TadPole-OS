> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / CODEBASE_MAP
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🗺️ Codebase Map

> **Status**: Stable  
> **Version**: 1.3.0<br>
> **Last Updated**: 2026-10-22 (refactor ownership map; macOS Universal Build Support retained)<br>
> **Classification**: Sovereign  

---

## Overview

This map is the lightweight navigation layer for the current Tadpole OS repository. It points to the live code roots and the highest-signal entry points used by the engine, dashboard, and desktop shell.

## Top-Level Map

| Directory | Role | Notes |
| :--- | :--- | :--- |
| `server-rs` | Rust engine | Axum routes, AppState hubs, telemetry bridge, agent runtime, security, persistence. |
| `src` | React dashboard | Pages, layouts, stores, services, visualizers, detached portal UI. |
| `src-tauri` | Desktop wrapper | Tauri packaging for Windows (NSIS), Linux (AppImage/.deb), and macOS Universal (fat `.dmg`). |
| `wasm-codec` | Shared codec module | WASM-friendly serialization helpers used by the repo. |
| `execution` | Execution tools | Deterministic Python scripts, IPC bridge client library (`execution/lib`). |
| `scripts` | Build & deploy helpers | `build-linux-light.ps1`, `build-macos-universal.sh`, `verify-macos-bundle.sh`, `deploy-linuxlite.ps1`, `publish-public.ps1`. |
| `tests/mission_bench` | Evaluation rig | Scenario-driven blind-judge mission benchmarks (`run.py`, `judge.py`). |

## Key Entry Points

- `server-rs/src/main.rs` boots the Rust engine.
- `server-rs/src/router.rs` defines the HTTP and WebSocket surface, including `/v1/*` routes and `/engine/ws`.
- `server-rs/src/state/mod.rs` is the current AppState module and global state root.
- `server-rs/src/agent/knowledge_store/mod.rs` governs SQLite-backed IKS and OKF metadata storage.
- `server-rs/src/agent/context_manager.rs` houses the $O(N)$ linear dialogue compactor.
- `server-rs/src/agent/mcp/ipc_bridge.rs` powers the local zero-dependency IPC bridge (Named Pipe/UDS) for Code Mode.
- `server-rs/src/agent/runner/execution_metrics.rs` computes per-mission execution metrics and telemetry rollups.
- `server-rs/src/agent/runner/tools/` houses the Zero-Trust tool pipeline, CBS, and trait-based registry.
- `server-rs/src/agent/runner/tools/resolved_policy.rs` resolves shared policy metadata for runner and MCP boundaries.
- `server-rs/src/agent/runner/context/` separates resolution, ordered preparation, and memory augmentation behind `context.rs`.
- `server-rs/src/agent/runner/service_traits/mission_state/outcome.rs` owns durable completion and conflict recovery.
- `server-rs/src/agent/script_skills/` separates artifact codecs, sources, validation, and filesystem persistence.
- `server-rs/src/security/permissions/` separates decision types, policy evaluation, SQL lookup, and approval lifecycle.
- `server-rs/src/routes/ws/` separates telemetry, live voice, and shared admission helpers behind `ws.rs`.
- `server-rs/src/db/contract_tests.rs` implements behavioral contract tests for DashMap and SQLite state stores.
- `server-rs/src/telemetry/mod.rs` owns the tracing-to-frontend telemetry bridge.
- `server-rs/src/routes/templates.rs` handles starter-kit and template installation.
- `server-rs/src/routes/knowledge.rs` handles the IKS and OKF metadata API endpoints.
- `server-rs/src/routes/missions/` houses the Mission Flight Recorder composite endpoints (`/v1/missions`, `/v1/missions/:id/dossier`, `/v1/missions/:id/pin`) with Merkle chain validation and CAS diffing.
- `server-rs/src/bin/graph_query/` is the modular intelligence CLI (ADG-05): `main.rs`, `path_utils.rs`, `visualizer.rs`, `query_manager.rs`, `doc_guard.rs`.
- `execution/lib/mcp_client.py` zero-dependency Python client for the IPC bridge.
- `src/layouts/Dashboard_Layout.tsx` wires the main application shell and lazily loaded side panels.
- `src/services/tadpoleos_service.ts` is the frontend service facade for engine interactions.
- `src/services/mission_history_service.ts` provides typed client access to the flight recorder endpoints.
- `src/services/provider_configuration.ts` and `src/services/model_commands.ts` own validated configuration writes and selective recovery.
- `src/domain/agents/state_reducer.ts` merges API snapshots and socket patches.
- `src/domain/chat/` owns transcript transitions; `src/services/chat/` owns decisions, replication, and diagnostic projection.
- `src/components/missions/history/` contains the Mission Flight Recorder & Run History master-detail dossier UI and audit viewers.
- `src/components/chat/OpenUI_Renderer.tsx` provides in-chat generative UI rendering for typed OpenUI DSL.
- `src/components/chat/Question_Choice_Pill.tsx` interactive choice pills for structured user question prompts.
- `src/components/chat/Mission_Metrics_Badge.tsx` HUD badge for real-time mission execution metrics.
- `src/components/ui/Portal_Window.tsx` powers detached windows while preserving shared state.
- `src/components/intelligence/KnowledgeGraph.tsx` renders the primary Neural Map visualization supporting Symbols and OKF force-graphs.
- `starter_kits/` contains the built-in SMB starter swarms shipped with the repo.
- `execution/` contains parity, audit, and operator verification tooling.
- `tests/mission_bench/` contains the scenario-driven blind-judge benchmarking harness (`run.py`, `judge.py`).

## Notes For Operators

- Persistent runtime data is rooted from `AppState.base_dir`, which defaults to the repo root and stores data under `data/`.
- **macOS**: `src-tauri/src/data_dir.rs`, called by `resolve_data_dir()` in the desktop entrypoint, prepares `~/Library/Application Support/com.tadpole.os/` for the database, neural token, logs and PID locks. An absolute `TADPOLE_DATA_DIR` overrides it; invalid or unavailable directories stop startup before sidecar launch.
- Template installs currently stage agents under `data/swarm_config/`, workflows into `directives/`, and specialized skills/scripts into `execution/`.
- The documentation parity checks rely on this file for path validation, so entries here should stay coarse, current, and rooted in paths that exist on disk.
- Universal macOS CI is configured as the `macos-universal` matrix entry on `macos-14` in `.github/workflows/build.yml`. It uses the shared build helper, stages both target-specific sidecars plus the universal engine, and verifies bundle slices/signatures/artifacts. See `docs/BLUEPRINT-universal-macos-build.md` for native verification status.
