> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.

# 🗺️ Codebase Map

> **Status**: Stable  
> **Version**: 1.1.13  
> **Last Updated**: 2026-04-17 (Alignment Patch)  
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

## Key Entry Points

- `server-rs/src/main.rs` boots the Rust engine.
- `server-rs/src/router.rs` defines the HTTP and WebSocket surface, including `/v1/*` routes and `/engine/ws`.
- `server-rs/src/state/mod.rs` is the current AppState module and global state root.
- `server-rs/src/telemetry/mod.rs` owns the tracing-to-frontend telemetry bridge.
- `server-rs/src/routes/templates.rs` handles starter-kit and template installation.
- `src/layouts/Dashboard_Layout.tsx` wires the main application shell and lazily loaded side panels.
- `src/services/tadpoleos_service.ts` is the frontend service facade for engine interactions.
- `src/components/ui/Portal_Window.tsx` powers detached windows while preserving shared state.
- `starter_kits/` contains the built-in SMB starter swarms shipped with the repo.
- `execution/` contains parity, audit, and operator verification tooling.

## Notes For Operators

- Persistent runtime data is rooted from `AppState.base_dir`, which defaults to the repo root and stores data under `data/`.
- Template installs currently stage agents under `data/swarm_config/`, workflows into `directives/`, and specialized skills/scripts into `execution/`.
- The documentation parity checks rely on this file for path validation, so entries here should stay coarse, current, and rooted in paths that exist on disk.

[//]: # (Metadata: [CODEBASE_MAP])

[//]: # (Metadata: [CODEBASE_MAP])
