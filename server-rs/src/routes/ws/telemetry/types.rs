//! @docs ARCHITECTURE:Networking
//! ### AI Assist Note
//! Signaling types for internal WebSocket stream coordination.
//! ### 🔍 Debugging & Observability
//! Signals represent state transitions between receiver and sender tasks.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvSignal {
    Pong,
    AuthOk,
    IdleClose,
}
