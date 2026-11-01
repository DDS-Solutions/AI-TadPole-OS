//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! - Session internals preserve authentication, bounded transport and permit ownership.
//!
//! ### 🔍 Debugging & Observability
//! - Session failures use the existing sanitized tracing targets.

mod live;
mod shared;
mod telemetry;

pub use live::live_voice_handler;
pub use telemetry::ws_handler;

pub const WS_HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
pub const WS_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);
pub const WS_AUTH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
pub const WS_LIVE_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
pub const LIVE_MAX_SESSION_BYTES: u64 = 100 * 1024 * 1024;
pub const LIVE_DEBOUNCE_BYTES: u64 = 256 * 1024;
pub const LIVE_ESTIMATED_COST_PER_SEC: f64 = 0.00005;

pub const WS_CLOSE_AUTH_INVALID: u16 = 4401;
pub const WS_CLOSE_QUOTA_EXHAUSTED: u16 = 4402;
pub const WS_CLOSE_AUTH_TIMEOUT: u16 = 4408;
pub const WS_CLOSE_IDLE_TIMEOUT: u16 = 4409;
pub const WS_CLOSE_SESSION_CAP_EXCEEDED: u16 = 4410;
