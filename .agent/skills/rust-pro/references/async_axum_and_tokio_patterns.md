> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / rust-pro
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Async runtime blocking or unhandled Tokio task panics.
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

## 3. Strict Non-Blocking Rule

- **Never call blocking std I/O or sleep inside async tasks**: Use `tokio::fs`, `tokio::time::sleep`, or `tokio::task::spawn_blocking`.