//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Init Channels
//! - **Primary Entrypoints**: `BroadcastChannels`, `init_channels`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use std::sync::Arc;
use tokio::sync::broadcast;

pub struct BroadcastChannels {
    pub tx: broadcast::Sender<crate::types::LogEntry>,
    pub event_tx: broadcast::Sender<serde_json::Value>,
    pub audio_stream_tx: broadcast::Sender<Vec<u8>>,
    pub pulse_tx: broadcast::Sender<Arc<crate::telemetry::pulse_types::SwarmPulse>>,
}

pub fn init_channels() -> BroadcastChannels {
    let (tx, _) = broadcast::channel(1000);
    let (event_tx, _) = broadcast::channel(1000);
    let (audio_stream_tx, _) = broadcast::channel(5000);
    let (pulse_tx, _) = broadcast::channel(1000);

    BroadcastChannels {
        tx,
        event_tx,
        audio_stream_tx,
        pulse_tx,
    }
}
