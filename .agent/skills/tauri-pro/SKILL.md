---
name: tauri-pro
description: "Master Tauri v1/v2 desktop development with Rust commands, IPC bridge, security capabilities, window state, and event bus."
when_to_use: "When writing or auditing Tauri Rust commands, handling frontend invoke() IPC calls, managing Tauri event listeners, or configuring desktop shell capabilities."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.2.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / tauri-pro
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Desktop crashes, unhandled Rust panics in IPC commands, permission capability denial, Base64 JSON memory bloat.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Tauri Desktop Architecture & IPC Engineering Protocol

> **Purpose**: Standardize the communication contract, security boundary, and state management between the Rust core engine and the React desktop frontend in Tauri v2 (`tadpole-os`).
> **Skill Synergy & Boundaries**:
> - **Core Rust Engine**: Delegates pure business logic, Axum routes, and Tokio actors to [`rust-pro`](../rust-pro/SKILL.md).
> - **Frontend UI State**: Delegates component state subscriptions and store mutations to [`zustand-patterns`](../zustand-patterns/SKILL.md).
> - **Workflow Bindings**: Integrated directly into [`/create`](../../workflows/create.md), [`/refactor`](../../workflows/refactor.md), and [`/audit`](../../workflows/audit.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/tauri_ipc_and_security.md`](./references/tauri_ipc_and_security.md) | Rust `#[tauri::command]` invariants, typed React `invoke()` wrappers, Tauri v2 capabilities schema, MessagePack binary streaming | Implementing desktop IPC routes, capabilities, or large data streaming |
| [`scripts/tauri_auditor.py`](./scripts/tauri_auditor.py) | Deterministic scanner for Rust command `Result<T, E>` types, unhandled panics, and capability sync | Auditing Tauri command boundaries or self-test verification |

---

## 🏛️ Core Architectural Invariants

1. **Panic-Free Command Contracts**: Every `#[tauri::command]` MUST return `Result<T, E>`. Never call `.unwrap()` or `.expect()` inside a command handler, as panics terminate the desktop process.
2. **Asynchronous Command Execution**: Long-running operations must be `async fn` and leverage Tokio tasks (`tokio::task::spawn_blocking` for CPU-bound tasks) to avoid locking the desktop main UI thread.
3. **Payload Sanitization & Serialization**: Always pass serializable structs with `serde::Serialize` and `serde::Deserialize`.
4. **Binary Channel Streaming for Large Payloads**: For payloads $> 64\text{ KB}$ (e.g. vector dumps, log batches, audio buffers), avoid Base64-in-JSON which causes 33% memory bloat. Use Tauri IPC channels (`tauri::ipc::Channel`) or `@msgpack/msgpack` binary envelopes.
5. **Clean Event Teardowns**: In React components, always unsubscribe from Tauri event listeners (`listen()`) inside `useEffect` cleanup callbacks.
6. **Tauri v2 Capability Discipline**: Commands and plugins must be explicitly permitted under `src-tauri/capabilities/*.json` with minimal least-privilege scopes.

---

## 🛡️ Socratic Gate: Desktop Shell Security & IPC Boundaries

Before adding or exposing a new Tauri command:
1. **Capability Scope**: Is this command explicitly permitted in `src-tauri/capabilities/default.json` for the target window?
2. **Filesystem Sandbox**: Does file I/O respect the allowed application directory without traversing parent folders (`..`)?
3. **Thread Safety**: Is shared state accessed via `tauri::State` wrapped in thread-safe primitives (`Arc<Mutex<T>>` or `Arc<RwLock<T>>`)?
4. **Binary Efficiency**: Does this command return large arrays that should be streamed via `tauri::ipc::Channel` rather than a monolithic JSON response?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Calling `.unwrap()` in command | Crashes the entire desktop application | Propagate error as `Result<T, String>` with `?` |
| Blocking the main thread with heavy compute | Freezes the desktop GUI and locks OS window events | Spawn background task with `tokio::task::spawn_blocking` |
| Monolithic Base64 JSON for large datasets | Spikes WebView memory and triggers garbage collection jank | Use `tauri::ipc::Channel` streaming or MessagePack buffers |
| Dangling event listeners in React | Memory leaks and duplicate callback triggers | Clean up listener promise in `useEffect` return function |
| Raw unvalidated shell command execution | Remote code execution vulnerability | Whitelist allowed binaries in capabilities and validate arguments strictly |

---

## 🛠️ Verification & Execution Commands

```powershell
# Audit Rust commands for panic-free Result returns
python .agent/skills/tauri-pro/scripts/tauri_auditor.py src-tauri/src

# Run self-test suite
python .agent/skills/tauri-pro/scripts/tauri_auditor.py --test
```
