//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Provider Mock Tests**: Unit tests verifying provider mock responses and error normalizer logic.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Mismatch in mapping raw error categories to normalized ones.

#[cfg(test)]
mod tests {
    use super::super::ProviderVariant;
    use crate::agent::null_provider::NullProvider;
    use crate::error::{AppError, ProviderId, InfrastructureErrorKind};

    #[test]
    fn test_error_normalization_mapping() {
        let null_provider = NullProvider::new("test-agent", crate::agent::null_provider::NullReason::TestMode);
        let variant = ProviderVariant::Null(null_provider);

        // Map a raw BadRequest error
        let raw_err = AppError::BadRequest("API key revoked".to_string());
        let normalized = variant.normalize_error(raw_err);

        match normalized {
            AppError::InfrastructureError { provider_id, kind, detail, .. } => {
                assert_eq!(provider_id, ProviderId::Runner);
                assert_eq!(kind, InfrastructureErrorKind::ApiError);
                assert!(detail.contains("API key revoked"));
            }
            other => panic!("Expected InfrastructureError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_null_provider_returns_degraded_msg() {
        let null_provider = NullProvider::new("test-agent", crate::agent::null_provider::NullReason::TestMode);
        let variant = ProviderVariant::Null(null_provider);

        let result = variant.generate("system prompt", "user prompt", None).await;
        assert!(result.is_ok());
        let (degraded_msg, tools, usage) = result.unwrap();
        assert!(degraded_msg.contains("DEGRADED"));
        assert!(tools.is_empty());
        assert!(usage.is_none());
    }
}
