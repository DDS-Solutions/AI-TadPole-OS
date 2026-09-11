//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / a2a_types
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use std::fmt;

pub type Amount = u64;

/// Maximum safe amount for SQLite 64-bit signed integer storage.
pub const MAX_SAFE_AMOUNT: u64 = i64::MAX as u64;

/// Validates that an amount is non-zero and within SQLite safe bounds.
pub fn validate_amount(amount: Amount) -> Result<(), AppError> {
    if amount == 0 {
        return Err(AppError::BadRequest(
            "Transaction amount must be strictly greater than zero".to_string(),
        ));
    }
    if amount > MAX_SAFE_AMOUNT {
        return Err(AppError::BadRequest(format!(
            "Transaction amount {} exceeds maximum safe integer bound ({})",
            amount, MAX_SAFE_AMOUNT
        )));
    }
    Ok(())
}

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
        let trimmed = s.trim();
        if let Some(id) = trimmed.strip_prefix("local:") {
            if id.is_empty() {
                return Err(AppError::BadRequest(
                    "Local address cannot be empty".to_string(),
                ));
            }
            Ok(Address::Local(id.to_string()))
        } else if let Some(addr) = trimmed.strip_prefix("web3:") {
            if addr.is_empty() {
                return Err(AppError::BadRequest(
                    "Web3 address cannot be empty".to_string(),
                ));
            }
            Ok(Address::Web3(addr.to_string()))
        } else if trimmed.starts_with("0x") {
            // Sniff EVM/Web3 hex addresses (e.g. 0x1234... or full 42-char wallet)
            if trimmed.len() >= 4 && trimmed[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                Ok(Address::Web3(trimmed.to_string()))
            } else {
                Err(AppError::BadRequest(format!(
                    "Invalid 0x Web3 address format: '{}'",
                    trimmed
                )))
            }
        } else if !trimmed.is_empty()
            && trimmed.len() <= 64
            && trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            Ok(Address::Local(trimmed.to_string()))
        } else {
            Err(AppError::BadRequest(format!(
                "Invalid A2A address format: '{}'. Must be 'local:<id>', 'web3:<addr>', or valid agent identifier.",
                trimmed
            )))
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_repr())
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum EconomicZone {
    Dev,
    Staging,
    Prod,
}

impl EconomicZone {
    pub fn to_string_repr(&self) -> String {
        match self {
            EconomicZone::Dev => "DEV".to_string(),
            EconomicZone::Staging => "STAGING".to_string(),
            EconomicZone::Prod => "PROD".to_string(),
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "STAGING" => EconomicZone::Staging,
            "PROD" | "PRODUCTION" => EconomicZone::Prod,
            "DEV" | "DEVELOPMENT" => EconomicZone::Dev,
            other => {
                tracing::warn!(
                    "⚠️ [a2a_types] Unrecognized economic zone '{}', falling back to DEV",
                    other
                );
                EconomicZone::Dev
            }
        }
    }

    pub fn parse_strict(s: &str) -> Result<Self, AppError> {
        match s.trim().to_uppercase().as_str() {
            "DEV" | "DEVELOPMENT" => Ok(EconomicZone::Dev),
            "STAGING" => Ok(EconomicZone::Staging),
            "PROD" | "PRODUCTION" => Ok(EconomicZone::Prod),
            other => Err(AppError::BadRequest(format!(
                "Invalid economic zone: '{}'. Must be DEV, STAGING, or PROD.",
                other
            ))),
        }
    }
}

impl fmt::Display for EconomicZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_repr())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_validation() {
        assert!(validate_amount(0).is_err());
        assert!(validate_amount(1).is_ok());
        assert!(validate_amount(1_000_000).is_ok());
        assert!(validate_amount(MAX_SAFE_AMOUNT).is_ok());
        assert!(validate_amount(MAX_SAFE_AMOUNT + 1).is_err());
    }

    #[test]
    fn test_address_parsing() {
        assert_eq!(
            Address::parse("local:agent_1").unwrap(),
            Address::Local("agent_1".to_string())
        );
        assert_eq!(
            Address::parse("web3:0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            Address::Web3("0x1234567890abcdef1234567890abcdef12345678".to_string())
        );
        assert_eq!(
            Address::parse("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            Address::Web3("0x1234567890abcdef1234567890abcdef12345678".to_string())
        );
        assert_eq!(
            Address::parse("agent_1").unwrap(),
            Address::Local("agent_1".to_string())
        );

        // Failures
        assert!(Address::parse("").is_err());
        assert!(Address::parse("local:").is_err());
        assert!(Address::parse("web3:").is_err());
        assert!(Address::parse("0xZZZZ").is_err());
        assert!(Address::parse("invalid name with spaces!").is_err());
    }

    #[test]
    fn test_economic_zone_parsing() {
        assert_eq!(EconomicZone::parse("DEV"), EconomicZone::Dev);
        assert_eq!(EconomicZone::parse("staging"), EconomicZone::Staging);
        assert_eq!(EconomicZone::parse("PROD"), EconomicZone::Prod);
        assert_eq!(EconomicZone::parse("production"), EconomicZone::Prod);
        assert_eq!(EconomicZone::parse("unknown_zone"), EconomicZone::Dev);

        assert!(EconomicZone::parse_strict("DEV").is_ok());
        assert!(EconomicZone::parse_strict("STAGING").is_ok());
        assert!(EconomicZone::parse_strict("PROD").is_ok());
        assert!(EconomicZone::parse_strict("invalid").is_err());
    }
}
