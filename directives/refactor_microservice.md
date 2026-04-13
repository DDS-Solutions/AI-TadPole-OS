# ⚒️ Directive: Refactor Microservice (SOP-DEV-01)

## 🎯 Primary Objective
Modernize and decouple engine sub-modules to improve maintainability, reduce "Prop-Drilling" fatigue, and enhance testability. Refactors must prioritize **Thread-Safety** and **Memory-Efficiency**.

---

## 🏗️ Refactor Patterns

### 1. Hub Decoupling
- **Goal**: Transition logic from monolithic files into specialized hubs.
- **Action**: Identify "Leaky Abstractions" where one module (e.g., `registry.rs`) has too much knowledge of another (e.g., `metering.rs`).
- **Standard**: Use `Arc<Mutex<T>>` or `tokio::sync::broadcast` for cross-hub state communication.

### 2. Interface Consistency
- **Goal**: Ensure 100% trait alignment.
- **Action**: Every service sub-module must implement a `Service` trait with `init()` and `shutdown()` methods for lifecycle uniformity.

### 3. Async Optimization
- **Goal**: Zero blocking of the Tokio executor.
- **Action**: Offload blocking I/O (e.g., legacy JSON writes) to `spawn_blocking` or migrate to `tokio::fs`.

---

## 🛠️ Verification Suite
1. **Lint**: `cargo clippy` and `npm run lint`.
2. **Tests**: `cargo test` on the specific module and `npm run test` for frontend parity.
3. **Benchmarks**: Run `benchmarks.rs` to ensure no regression in latency or throughput.

## 📝 Change Manifest
Every refactor must include an `ANALYSIS.md` doc explaining the **WHY** behind the structural changes and mapping the "Before vs After" dependency graph.