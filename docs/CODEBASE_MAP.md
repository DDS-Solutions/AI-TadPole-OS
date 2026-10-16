> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / CODEBASE_MAP
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🗺️ Codebase Map

> **Status**: Stable  
> **Version**: 1.2.5  
> **Last Updated**: 2026-07-11 (Symbol Graph & Doc Guard Hardening + graph_query Modularization)  
> **Classification**: Sovereign  

---

## Overview

This map is the lightweight navigation layer for the current Tadpole OS repository. It points to the live code roots and the highest-signal entry points used by the engine, dashboard, and desktop shell.

## Top-Level Map

| Directory | Role | Notes |
| :--- | :--- | :--- |
| `server-rs` | Rust engine | Axum routes, AppState hubs, telemetry bridge, agent runtime, security, persistence. |
| `src` | React dashboard | Pages, layouts, stores, services, visualizers, detached portal UI. |
| `src-tauri` | Desktop wrapper | Tauri packaging and native bundle targets for desktop distribution. |
| `wasm-codec` | Shared codec module | WASM-friendly serialization helpers used by the repo. |
| `execution` | Execution tools | Deterministic Python scripts, IPC bridge client library (`execution/lib`). |
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
- `server-rs/src/db/contract_tests.rs` implements behavioral contract tests for DashMap and SQLite state stores.
- `server-rs/src/telemetry/mod.rs` owns the tracing-to-frontend telemetry bridge.
- `server-rs/src/routes/templates.rs` handles starter-kit and template installation.
- `server-rs/src/routes/knowledge.rs` handles the IKS and OKF metadata API endpoints.
- `server-rs/src/bin/graph_query/` is the modular intelligence CLI (ADG-05): `main.rs`, `path_utils.rs`, `visualizer.rs`, `query_manager.rs`, `doc_guard.rs`.
- `execution/lib/mcp_client.py` zero-dependency Python client for the IPC bridge.
- `src/layouts/Dashboard_Layout.tsx` wires the main application shell and lazily loaded side panels.
- `src/services/tadpoleos_service.ts` is the frontend service facade for engine interactions.
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
- Template installs currently stage agents under `data/swarm_config/`, workflows into `directives/`, and specialized skills/scripts into `execution/`.
- The documentation parity checks rely on this file for path validation, so entries here should stay coarse, current, and rooted in paths that exist on disk.