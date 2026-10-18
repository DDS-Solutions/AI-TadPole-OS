---
name: rust-pro
description: "Master Rust 1.75+ with modern async patterns, advanced type system features, and production-ready systems programming. Expert in Tokio, Axum, and zero-cost abstractions."
when_to_use: "When writing Rust code, working with .rs files, Cargo.toml, Tokio async tasks, axum routing, or Tauri Rust IPC backends."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / rust-pro
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Blocking calls in async Tokio tasks, unhandled panics via raw `unwrap()`, or holding sync Mutexes across `.await`.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Rust Systems Engineering & Async Architecture

> **Philosophy**: Zero-cost abstractions, fearless concurrency, strict type-level safety.  
> **Ecosystem Standard**: Rust 1.75+, Tokio runtime, Axum 0.7+, `thiserror`, `serde`.  
> **Workflow Bindings**: Integrated into [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md), [`/refactor`](file:///D:/TadpoleOS-Dev/.agent/workflows/refactor.md), and [`/audit`](file:///D:/TadpoleOS-Dev/.agent/workflows/audit.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/async_axum_and_tokio_patterns.md`](file:///D:/TadpoleOS-Dev/.agent/skills/rust-pro/references/async_axum_and_tokio_patterns.md) | Axum StateHub decomposition, Tokio channel selection, async task spawning | Writing backend routes & Tokio actors |
| [`scripts/rust_auditor.py`](file:///D:/TadpoleOS-Dev/.agent/skills/rust-pro/scripts/rust_auditor.py) | Automated static auditor checking for blocking sync calls and production `unwrap()` panics | Auditing Rust source code before compilation |

---

## ⚡ 1. Modern Rust Core Principles

1. **Decomposed State Hubs**: In `server-rs`, inject state through `AppState` containing 5 decoupled hubs (`comm`, `gov`, `reg`, `res`, `sec`).
2. **Non-Blocking Runtime**: Never call `std::fs`, `std::thread::sleep`, or blocking mutexes in async functions. Use `tokio::task::spawn_blocking` for CPU-heavy tasks and `tokio::time::sleep` for delays.
3. **Structured Errors**: Use `thiserror` for internal domain errors and return explicit `Result<T, AppError>` bubbles with `?`.
4. **Mutex Discipline**: Never hold a `std::sync::MutexGuard` across an `.await` point. Use `tokio::sync::Mutex` or restructure to release locks prior to `.await`.

---

## 🧠 2. Socratic Gate for Rust Systems

Before modifying Rust backend routes or engines, verify:
- **Gate 1 (Zero-Panics)**: Are all `.unwrap()` and `.expect()` calls eliminated from non-test production execution paths?
- **Gate 2 (Tokio Thread Safety)**: Are all async operations using Tokio-native primitives rather than blocking `std` I/O?
- **Gate 3 (Clippy Gate)**: Does `cargo clippy --manifest-path server-rs/Cargo.toml -- -D warnings` exit cleanly with 0 warnings?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Blocking in Tokio Task** | `std::thread::sleep` starves Tokio worker threads. | Use `tokio::time::sleep(...).await`. |
| **Sync Mutex Across `.await`** | Causes `!Send` compile errors or runtime deadlocks. | Use `tokio::sync::Mutex` or drop guard before `.await`. |
| **Production `unwrap()`** | Malformed input causes panic and drops network connections. | Return `Result<T, E>` and bubble with `?`. |
| **Excessive Cloning** | Cloning large vectors or strings on every request. | Pass references `&[T]`, `&str`, or use `Arc<T>` / `Cow<'a, str>`. |

---

## 🛠️ 4. Deterministic Verification Gate

Audit and verify Rust code:

```powershell
# 1. Run built-in self-tests
python .agent/skills/rust-pro/scripts/rust_auditor.py --test

# 2. Check Rust code with static auditor
python .agent/skills/rust-pro/scripts/rust_auditor.py --audit-dir server-rs/src

# 3. Compile and check borrow checker
cargo check --manifest-path server-rs/Cargo.toml

# 4. Enforce strict Clippy warnings
cargo clippy --manifest-path server-rs/Cargo.toml -- -D warnings
```