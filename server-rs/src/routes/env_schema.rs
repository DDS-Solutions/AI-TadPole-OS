//! Environment Config — Schema discovery and validation API
//!
//! Returns schema metadata (variable names, types, descriptions, isSet booleans) 
//! without exposing any actual secret values.
//!
//! @docs ARCHITECTURE:Networking

use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

/// Returns safe metadata about all known environment variables.
/// Sensitive values are NEVER included — only whether they are set.
pub async fn get_env_schema(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let schema_path = std::path::Path::new(".env.schema");

    match crate::env_schema::EnvSchema::load(schema_path) {
        Ok(schema) => {
            let metadata = schema.to_safe_metadata();
            Json(serde_json::json!({
                "status": "ok",
                "count": metadata.len(),
                "variables": metadata
            }))
        }
        Err(e) => {
            let safe_err = state.security.secret_redactor.redact(&format!("{}", e));
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to load schema: {}", safe_err),
                "variables": []
            }))
        }
    }
}
