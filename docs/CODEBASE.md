# 📂 Tadpole OS Codebase Guide

## Project Structure

This project is a high-performance **Tadpole OS** runtime with a **Rust-based Backend** and a **React/Vite Frontend**.

### Top-Level Directories

- **`server-rs/`**: The core **Tadpole OS** engine.
    - **`migrations/`**: Database schema evolution scripts (P3 remediation).
    - **`src/main.rs`**: Clean, modular Axum entry point.
    - **`src/startup.rs`**: Engine entry point and background task management (including debounced persistence flush loop).
    - **`src/router.rs`**: Decoupled routing and middleware configuration.
    - **`src/agent/`**: The Intelligence Layer.
        - `runner/`: Refactored core execution engine (Modular & Parallel).
        - `mcp.rs`: Unified MCP Tool Host & Discovery logic.
        - `benchmarks.rs`: Performance tracking and regression analytics.
        - `hooks.rs`: Lifecycle execution governance (`pre-tool`/`post-tool`).
        - `persistence.rs`: SQLite-backed agent registry. Supports `working_memory` and debounced usage flushing.
        - `skills.rs`: Dynamic file-system registry for Skills & Workflows.
        - `gemini.rs`: Google provider with concurrent tool call support.
        - `groq.rs`: Groq provider with self-healing Llama retry.
    - **`src/security/`**: The Core Defense Layer.
        - `audit.rs`: ED25519 Merkle Audit Trail for untamperable logging.
        - `scanning.rs`: Proactive script safety and secret detection.
        - `metering.rs`: Core Budget Guard logic. Implements debounced persistence with `DashMap` buffers.
    - **`src/routes/`**: API handlers.
        - `error.rs`: RFC 9457 ProblemDetails implementation.
        - `benchmarks.rs`: Performance analytics endpoints.
        - `ws.rs` : Unified high-concurrency WebSocket hub.
    - **`src/utils/`**: Common helpers.
        - `serialization.rs`: Implements `SanitizedSanitizedJson` for string pruning (2KB limit).
        - `mod.rs`: Utility exports.
- **`src/`**: The React Dashboard (Frontend).
    - **`pages/`**: Views: `OpsDashboard`, `Missions`, `Hierarchy`, `Standups`.
    - **`components/`**: UI Elements: `AgentCard`, `CommandPalette`, `NeuralPulse`.
    - **`hooks/`**: Custom logic: `useDashboardData`.
    - **`services/`**: Decoupled API architecture:
        - `AgentApiService.ts`, `MissionApiService.ts`, `SystemApiService.ts`.
        - `BaseApiService.ts` (Core logic), `TadpoleOSService.ts` (Proxy).
- **`workspaces/`**: Physical sandboxes isolated per cluster.

## Key Files & Entry Points

- **Backend Start**: `npm run engine` (port 8000)
- **Frontend Start**: `npm run dev` (port 5173)
- **Primary Logic**: `server-rs/src/agent/runner.rs`
- **Identity & Life**: `server-rs/data/context/IDENTITY.md`

## Coding Conventions

### Language
- **Rust**: Functional-style async concurrency using `FuturesUnordered` for parallel tasking.
- **TypeScript**: Typed state management via **Zustand**.

### State Management
- **Backend (Rust)**: High-concurrency `DashMap` for agent and mission registries.
- **Frontend (TS)**: Reactive stores for agents, roles, settings, tabs (`tabStore`), and headers (`headerStore`).

### Design System: "Neural Glass"
- **Aesthetic**: Cinematic dark mode, glassmorphism, pulsating status rings.

## Build & Run

1. `npm run engine`
2. `npm run dev`

## Troubleshooting

- **"OFFLINE" Dashboard**: Check if `NEURAL_TOKEN` is set in `.env` and engine is at :8000.
- **Parallel Failures**: Verify RPM/TPM limits in the Model Registry.
