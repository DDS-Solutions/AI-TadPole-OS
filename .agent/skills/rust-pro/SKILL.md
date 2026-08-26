---
name: rust-pro
description: Master Rust 1.75+ with modern async patterns, advanced type system features, and production-ready systems programming. Expert in Tokio, Axum, and zero-cost abstractions.
when_to_use: "When writing Rust code, working with .rs files, Cargo.toml, Tokio, axum, or any Rust ecosystem tools."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / rust-pro
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Rust Systems Engineering & Async Architecture

> **Philosophy**: Zero-cost abstractions. Fearless concurrency. Strict type-level safety.
> **Ecosystem Standard**: Rust 1.75+, Tokio runtime, Axum 0.7+, `thiserror`, `serde`.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core Rust engineering rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/async_axum_and_tokio_patterns.md`](./references/async_axum_and_tokio_patterns.md) | Axum StateHub decomposition, Tokio channel selection, async task spawning | Writing backend routes & Tokio actors |

---

## ⚡ 1. Modern Rust Core Principles

1. **Decomposed State Hubs**: In `server-rs`, inject state through `AppState` containing 5 decoupled hubs (`comm`, `gov`, `reg`, `res`, `sec`).
2. **Non-Blocking Runtime**: Never call `std::fs`, `std::thread::sleep`, or blocking mutexes in async functions. Use `tokio::task::spawn_blocking` for CPU-heavy tasks.
3. **Structured Errors**: Use `thiserror` for domain errors and return explicit `Result<T, AppError>`.
4. **Zero-Copy Serialization**: Leverage `serde` with `&str` / `Cow<'a, str>` when deserializing high-throughput JSON.

---

## 🛠️ 2. Verification & Lint Gates

```powershell
# 1. Check syntax and borrow checker
cargo check --manifest-path server-rs/Cargo.toml

# 2. Enforce strict Clippy warnings
cargo clippy --manifest-path server-rs/Cargo.toml -- -D warnings

# 3. Execute unit and integration test suites
cargo test --manifest-path server-rs/Cargo.toml
```