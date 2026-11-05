> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / rust-pro
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Async runtime blocking, unhandled Tokio task panics, or cancellation state leaks.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[RUST_PRO]`)

# Rust Systems Architecture & Tokio Patterns Reference (L3)

---

## 1. Decomposed Axum State Pattern (`server-rs`)

```rust
use axum::{extract::State, Json, response::IntoResponse};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub comm: Arc<CommHub>,
    pub gov: Arc<GovHub>,
    pub reg: Arc<RegHub>,
    pub res: Arc<ResHub>,
    pub sec: Arc<SecHub>,
}

#[tracing::instrument(skip(state), fields(subsystem = "gov"))]
pub async fn get_system_health(
    State(state): State<AppState>
) -> impl IntoResponse {
    let health = state.gov.check_health().await;
    Json(health)
}
```

---

## 2. Tokio Concurrency & Channel Primitives

| Channel Type | Capacity | Pattern |
|---|---|---|
| **`tokio::sync::mpsc`** | Bounded buffer | Producer-consumer worker tasks |
| **`tokio::sync::broadcast`** | Multi-producer multi-consumer | WebSocket broadcast event streaming |
| **`tokio::sync::watch`** | Single-value retain | Configuration / Kill-switch state changes |
| **`tokio::sync::oneshot`** | Single message | Response handshake across tasks |

---

## 3. Tokio Cancellation Safety

In Tokio, when a `tokio::select!` branch completes, other branches are dropped immediately. If a task is awaiting I/O inside an Axum handler and the HTTP connection is closed, the handler future is dropped.

### Cancellation-Unsafe (Anti-Pattern):
```rust
// ❌ If dropped after deduct_credits, mission is never launched!
account.deduct_credits(10).await?;
mission_engine.launch_mission().await?;
```

### Cancellation-Safe Remedies:
1. **Background Task Decoupling**: Offload critical side effects into a separate detached task:
   ```rust
   // ✅ Spawn decoupled background job so client abort doesn't cancel it
   tokio::spawn(async move {
       account.deduct_credits(10).await?;
       mission_engine.launch_mission().await?;
   });
   ```
2. **Atomic Rollback Guards**: Use RAII drop guards that roll back pending transactions unless explicitly marked committed.

---

## 4. Borrow-Checker Bisection Patterns

When fighting complex lifetime or borrow-checker errors across mutable references:
1. **Split Structs**: Group mutable fields into sub-structs so methods borrow only the needed subset (`&mut self.config` vs `&mut self`).
2. **Two-Phase Scoping**: Extract necessary read data into local immutable variables before entering a mutable borrow scope.
3. **Reborrowing in Loops**: Ensure loops do not move resources; reborrow explicitly via `&mut *ref`.

---

## 5. UTF-8 Character Boundary Slicing

Never slice strings with raw byte offsets `&s[..len]`, which panics with `self.is_char_boundary(new_len)` if `len` falls in the middle of a multi-byte character (emojis, unicode symbols).

```rust
// ❌ PANIC TRAP on multi-byte characters:
// let truncated = &text[..max_bytes];

// ✅ SAFE TRUNCATION (Rust 1.75+):
let safe_len = text.floor_char_boundary(max_bytes);
let truncated = &text[..safe_len];

// ✅ ALTERNATIVE via Character Iteration:
let safe_string: String = text.chars().take(max_chars).collect();
```

---

## 6. OpenTelemetry & Tracing Span Lifecycles

In `tracing-subscriber`, cloning or re-entering a closed span panics with `assertion left != right failed: tried to clone a span that already closed`.

```rust
// ❌ PANIC TRAP: Re-entering or cloning a span across async yields
let span = tracing::info_span!("subsystem_worker");
let _guard = span.enter(); // Guard dropped when entering await
tokio::time::sleep(duration).await; // Thread context switched!

// ✅ SAFE ASYNC INSTRUMENTATION:
// Use .instrument() future wrapper rather than holding Span guards across await
async fn perform_work() { /* ... */ }

perform_work()
    .instrument(tracing::info_span!("subsystem_worker"))
    .await;
```

---

## 7. Safe Collection Indexing

Never use raw indexing `vec[0]` or `slice[i]`, which causes unhandled index out-of-bounds panics if collections are empty.

```rust
// ❌ PANIC TRAP on empty collections:
// let first_node = &graph.nodes[0];

// ✅ SAFE LOOKUPS:
let first_node = graph.nodes.first(); // Returns Option<&Node>
let nth_node = graph.nodes.get(index); // Returns Option<&Node>
```