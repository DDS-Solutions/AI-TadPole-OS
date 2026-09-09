//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP HTTP Body Reader
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Bounded response body consumption strictly bounded by MAX_RESPONSE_BODY_BYTES.
//! - `[Structural]` Source error preservation in InfrastructureError detail.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InfrastructureError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::mcp::client::http::tests::*`

use super::limits::MAX_RESPONSE_BODY_BYTES;
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};

pub async fn read_bounded_body(
    mut response: reqwest::Response,
    server_name: &str,
) -> Result<Vec<u8>, AppError> {
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::NetworkError,
            detail: format!("Failed to read response body from '{}': {}", server_name, e),
            help_link: None,
        })?
    {
        if body.len().saturating_add(chunk.len()) > MAX_RESPONSE_BODY_BYTES {
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP server '{}' exceeded the response body limit",
                    server_name
                ),
                help_link: None,
            });
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}
