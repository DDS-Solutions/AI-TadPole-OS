---
name: tauri-pro
description: "Master Tauri v1/v2 desktop development with Rust commands, IPC bridge, security capabilities, window state, and event bus."
when_to_use: "When writing or auditing Tauri Rust commands, handling frontend invoke() IPC calls, managing Tauri event listeners, or configuring desktop shell capabilities."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / tauri-pro
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Desktop crashes, unhandled Rust panics in IPC commands, permission capability denial.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Tauri Desktop Architecture & IPC Engineering Protocol

> **Purpose**: Standardize the communication contract, security boundary, and state management between the Rust core engine and the React desktop frontend.
> **Workflow Binding**: Used during [`/create`](../../workflows/create.md), [`/refactor`](../../workflows/refactor.md), and [`/audit`](../../workflows/audit.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/tauri_ipc_and_security.md`](./references/tauri_ipc_and_security.md) | Rust `#[tauri::command]` invariants, typed React `invoke()` wrappers, capabilities schema | Implementing desktop IPC routes or security scopes |
| [`scripts/tauri_auditor.py`](./scripts/tauri_auditor.py) | Deterministic scanner for Rust command `Result<T, E>` types and unhandled panics | Auditing Tauri command boundaries or self-test verification |

---

## 🏛️ Core Architectural Invariants

1. **Panic-Free Command Contracts**: Every `#[tauri::command]` MUST return `Result<T, E>`. Never call `.unwrap()` or `.expect()` inside a command handler, as panics terminate the desktop process.
2. **Asynchronous Command Execution**: Long-running operations must be `async fn` and leverage Tokio tasks to avoid locking the desktop main UI thread.
3. **Payload Sanitization & Envelopes**: Always pass serializable JSON structs with `serde::Serialize` and `serde::Deserialize`.
4. **Clean Event Teardowns**: In React components, always unsubscribe from Tauri event listeners (`listen()`) inside `useEffect` cleanup callbacks.

---

## 🛡️ Socratic Gate: Desktop Shell Security & IPC Boundaries

Before adding or exposing a new Tauri command:
1. **Capability Scope**: Is this command permitted in `src-tauri/capabilities/default.json`?
2. **Filesystem Sandbox**: Does file I/O respect the allowed application directory without traversing parent folders (`..`)?
3. **Thread Safety**: Is shared state accessed via `tauri::State` wrapped in thread-safe primitives (`Arc<Mutex<T>>` or `Arc<RwLock<T>>`)?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Calling `.unwrap()` in command | Crashes the entire desktop application | Propagate error as `Result<T, String>` |
| Blocking the main thread with heavy compute | Freezes the desktop GUI | Spawn background task with `tokio::task::spawn_blocking` |
| Dangling event listeners in React | Memory leaks and duplicate callback triggers | Clean up listener promise in `useEffect` return function |
| Raw unvalidated shell command execution | Remote code execution vulnerability | Whitelist allowed binaries and validate arguments strictly |

---

## 🛠️ Verification & Execution Commands

```powershell
# Audit Rust commands for panic-free Result returns
python .agent/skills/tauri-pro/scripts/tauri_auditor.py server-rs/src

# Run self-test suite
python .agent/skills/tauri-pro/scripts/tauri_auditor.py --test
```
