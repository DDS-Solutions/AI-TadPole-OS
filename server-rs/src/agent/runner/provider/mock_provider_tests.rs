//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mock_provider_tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

#[cfg(test)]
mod tests {
    use super::super::ProviderVariant;
    use crate::agent::null_provider::NullProvider;
    use crate::error::{AppError, InfrastructureErrorKind, ProviderId};

    #[test]
    fn test_error_normalization_mapping() {
        let null_provider = NullProvider::new(
            "test-agent",
            crate::agent::null_provider::NullReason::TestMode,
        );
        let variant = ProviderVariant::Null(null_provider);

        // Map a raw BadRequest error
        let raw_err = AppError::BadRequest("API key revoked".to_string());
        let normalized = variant.normalize_error(raw_err);

        match normalized {
            AppError::InfrastructureError {
                provider_id,
                kind,
                detail,
                ..
            } => {
                assert_eq!(provider_id, ProviderId::Runner);
                assert_eq!(kind, InfrastructureErrorKind::ApiError);
                assert!(detail.contains("API key revoked"));
            }
            other => panic!("Expected InfrastructureError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_null_provider_returns_degraded_msg() {
        let null_provider = NullProvider::new(
            "test-agent",
            crate::agent::null_provider::NullReason::TestMode,
        );
        let variant = ProviderVariant::Null(null_provider);

        let result = variant.generate("system prompt", "user prompt", None).await;
        assert!(result.is_ok());
        let (degraded_msg, tools, usage) = result.unwrap();
        assert!(degraded_msg.contains("DEGRADED"));
        assert!(tools.is_empty());
        assert!(usage.is_none());
    }
}
