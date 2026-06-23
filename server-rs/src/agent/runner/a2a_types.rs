//! @docs ARCHITECTURE:Registry
//! @docs ARCHITECTURE:Runner
//! 
//! ### AI Assist Note
//! **A2A Payment Core Types**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[a2a_types]` in tracing logs.

use crate::error::AppError;

pub type Amount = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum Address {
    Local(String),
    Web3(String),
}

impl Address {
    pub fn to_string_repr(&self) -> String {
        match self {
            Address::Local(id) => format!("local:{}", id),
            Address::Web3(addr) => format!("web3:{}", addr),
        }
    }

    pub fn parse(s: &str) -> Result<Self, AppError> {
        if let Some(id) = s.strip_prefix("local:") {
            Ok(Address::Local(id.to_string()))
        } else if let Some(addr) = s.strip_prefix("web3:") {
            Ok(Address::Web3(addr.to_string()))
        } else {
            Ok(Address::Local(s.to_string()))
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum EconomicZone {
    Dev,
    Staging,
    Prod,
}

#[allow(dead_code)]
impl EconomicZone {
    pub fn to_string_repr(&self) -> String {
        match self {
            EconomicZone::Dev => "DEV".to_string(),
            EconomicZone::Staging => "STAGING".to_string(),
            EconomicZone::Prod => "PROD".to_string(),
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "STAGING" => EconomicZone::Staging,
            "PROD" => EconomicZone::Prod,
            _ => EconomicZone::Dev,
        }
    }
}

// Metadata: [a2a_types]
