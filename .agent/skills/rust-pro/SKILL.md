---
name: rust-pro
description: "Master Rust 1.75+ with modern async patterns, advanced type system features, and production-ready systems programming. Expert in Tokio, Axum, and zero-cost abstractions."
when_to_use: "When writing Rust code, working with .rs files, Cargo.toml, Tokio async tasks, axum routing, Tauri Rust IPC backends, or WebAssembly codecs."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.3.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / rust-pro
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Blocking calls in Tokio tasks, raw `.unwrap()` panics, UTF-8 char boundary slicing crashes, closed tracing span clones, or cancellation leaks.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Rust Systems Engineering & Async Architecture

> **Philosophy**: Zero-cost abstractions, fearless concurrency, strict type-level safety.  
> **Ecosystem Standard**: Rust 1.75+ (editions 2021/2024), Tokio 1.54+, Axum 0.8+, `thiserror`, `serde`, `tracing`, `wasm-bindgen`.  
> **Skill Synergy & Boundaries**:
> - **Desktop IPC & Window Lifecycle**: Delegates to [`tauri-pro`](../tauri-pro/SKILL.md).
> - **Database Migrations & Relational Schemas**: Delegates to [`database-design`](../database-design/SKILL.md).
> - **Workflow Bindings**: Integrated into [`/create`](../../workflows/create.md), [`/refactor`](../../workflows/refactor.md), and [`/audit`](../../workflows/audit.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/async_axum_and_tokio_patterns.md`](./references/async_axum_and_tokio_patterns.md) | Axum StateHub decomposition, Tokio channel selection, cancellation safety, tracing span lifecycles | Writing backend routes & Tokio actors |
| [`references/wasm_and_binary_codecs.md`](./references/wasm_and_binary_codecs.md) | `wasm-codec` compilation, `wasm-bindgen` memory buffers, `postcard` binary serialization | Interfacing WebAssembly codecs with JavaScript |
| [`scripts/rust_auditor.py`](./scripts/rust_auditor.py) | Automated static auditor checking for blocking sync calls and production `unwrap()` panics | Auditing Rust source code before compilation |

---

## ⚡ 1. Modern Rust Core Principles

1. **Decomposed State Hubs**: In `server-rs`, inject state through `AppState` containing 5 decoupled hubs (`comm`, `gov`, `reg`, `res`, `sec`).
2. **Non-Blocking Runtime**: Never call `std::fs`, `std::thread::sleep`, or blocking mutexes in async functions. Use `tokio::task::spawn_blocking` for CPU-heavy tasks and `tokio::time::sleep` for delays.
3. **Structured Errors**: Use `thiserror` for internal domain errors and return explicit `Result<T, AppError>` bubbles with `?`.
4. **Mutex Discipline**: Never hold a `std::sync::MutexGuard` across an `.await` point. Use `tokio::sync::Mutex` or restructure to release locks prior to `.await`.
5. **Tokio Cancellation Safety**: Any `.await` in a `tokio::select!` branch or Axum request handler can be cancelled if the client disconnects or times out. Multi-step state transitions must be atomic or protected by RAII rollback guards to avoid partial writes.
6. **UTF-8 Char Boundary Safety**: Never slice strings via raw byte indices (`&s[..len]`). Always check `s.is_char_boundary(len)` or use `s.floor_char_boundary(len)` to prevent panic crashes on multi-byte characters.
7. **Safe Collection Indexing**: Never use unchecked raw index `vec[0]`. Always use `.first()`, `.last()`, or `.get(index)` with explicit `Option` handling to prevent index out-of-bounds panics.
8. **Tracing Span Lifecycle Safety**: Never clone or re-enter closed OpenTelemetry spans across asynchronous yields; use detached spans or `.in_scope()` blocks.
9. **Borrow Checker Discipline**: Avoid knee-jerk `.clone()`. Split large structs into independent domain pieces to allow concurrent mutable borrows of distinct fields.

---

## 🧠 2. Socratic Gate for Rust Systems

Before modifying Rust backend routes or engines, verify:
- **Gate 1 (Zero-Panics)**: Are all `.unwrap()` and `.expect()` calls eliminated from non-test production execution paths?
- **Gate 2 (Tokio Thread Safety)**: Are all async operations using Tokio-native primitives rather than blocking `std` I/O?
- **Gate 3 (Cancellation Safety)**: If this handler is aborted midway through `.await`, does it leave state hubs or database transactions consistent?
- **Gate 4 (UTF-8 Slicing Guard)**: Are all string truncations guarded against mid-character byte splits using `.floor_char_boundary()`?
- **Gate 5 (Safe Indexing Guard)**: Are all collection lookups using `.first()` or `.get()` rather than raw `[0]`?
- **Gate 6 (Clippy Gate)**: Does `cargo clippy --manifest-path server-rs/Cargo.toml -- -D warnings` exit cleanly with 0 warnings?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Raw String Byte Slicing** | `&s[..n]` splits multi-byte UTF-8 char and panics. | Use `s.floor_char_boundary(n)` or `.char_indices()`. |
| **Unchecked Vector Indexing** | `vec[0]` on empty list panics with index out of bounds. | Use `vec.first()` or `vec.get(0)` returning `Option`. |
| **Cloning Closed Tracing Spans** | Cloned closed span panics in `tracing-subscriber`. | Use `.in_scope()` or enter active child spans. |
| **Blocking in Tokio Task** | `std::thread::sleep` starves Tokio worker threads. | Use `tokio::time::sleep(...).await`. |
| **Sync Mutex Across `.await`** | Causes `!Send` compile errors or runtime deadlocks. | Use `tokio::sync::Mutex` or drop guard before `.await`. |
| **Production `unwrap()`** | Malformed input causes panic and drops network connections. | Return `Result<T, E>` and bubble with `?`. |
| **Cancellation Inconsistency** | Dropping an async task leaves partially mutated state. | Guard multi-step writes with atomic commits or `tokio::spawn`. |

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