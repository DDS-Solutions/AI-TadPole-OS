//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[lib.rs]` in tracing logs.

//! @docs ARCHITECTURE:Performance
//!
//! ### AI Assist Note
//! **Binary Codec**: Implements Postcard-based binary serialization for sub-millisecond
//! telemetry. Bridges the Rust backend with the WASM-powered frontend pulse decoder.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PulseNode {
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub status: u8, // 0: idle, 1: busy, 2: error, 3: degraded
    pub battery: u8,
    pub signal: u8,
    pub progress: f32,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PulseConnection {
    #[wasm_bindgen(getter_with_clone)]
    pub source: String,
    #[wasm_bindgen(getter_with_clone)]
    pub target: String,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwarmPulse {
    pub timestamp: u64,
    #[wasm_bindgen(getter_with_clone)]
    pub nodes: Vec<PulseNode>,
    #[wasm_bindgen(getter_with_clone)]
    pub edges: Vec<PulseConnection>,
}

#[wasm_bindgen]
impl SwarmPulse {
    #[wasm_bindgen(constructor)]
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, JsValue> {
        postcard::to_allocvec(self)
            .map_err(|e| JsValue::from_str(&format!("Encoding error: {}", e)))
    }

    pub fn decode(bytes: &[u8]) -> Result<SwarmPulse, JsValue> {
        postcard::from_bytes(bytes)
            .map_err(|e| JsValue::from_str(&format!("Decoding error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulse_codec_parity() {
        let mut pulse = SwarmPulse::new(123456789);
        pulse.nodes.push(PulseNode {
            id: "agent-1".to_string(),
            x: 10.0,
            y: 20.0,
            status: 1,
            battery: 85,
            signal: 90,
            progress: 0.5,
        });

        let encoded = pulse.encode().expect("Failed to encode");
        let decoded = SwarmPulse::decode(&encoded).expect("Failed to decode");

        assert_eq!(decoded.timestamp, pulse.timestamp);
        assert_eq!(decoded.nodes.len(), 1);
        assert_eq!(decoded.nodes[0].id, "agent-1");
        assert_eq!(decoded.nodes[0].status, 1);
    }
}
