//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Environment Config (Schema Validator)**: Orchestrates the safe
//! discovery and validation of environment variables for the
//! Tadpole OS engine. Features **Safe Metadata Disclosure**:
//! returns variable names, descriptions, and "isSet" booleans
//! without exposing sensitive secret values. Implements **Dynamic
//! `.env.schema` Loading**: ensures that the engine's operational
//! requirements are synchronized with the available configuration
//! at runtime. AI agents should use this endpoint to identify
//! missing prerequisite keys (e.g., `OPENAI_API_KEY`) before
//! attempting high-tier operations (ENV-02).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 404 Not Found for `.env.schema` file,
//!   malformed schema syntax causing parse errors, or incorrect
//!   "isSet" reporting due to cache staleness in the `EnvSchema`
//!   instance.
//! - **Telemetry Link**: Search for `Failed to load schema` in
//!   `tracing` logs for configuration audit events.
//! - **Trace Scope**: `server-rs::routes::env_schema`

use crate::error::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

/// Returns safe metadata about all known environment variables.
/// Sensitive values are NEVER included — only whether they are set.
#[tracing::instrument(skip(state), name = "system::get_env_schema")]
pub async fn get_env_schema(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let schema_path = std::path::Path::new(".env.schema");

    match crate::env_schema::EnvSchema::load(schema_path) {
        Ok(schema) => {
            let metadata = schema.to_safe_metadata();
            Ok(Json(serde_json::json!({
                "status": "ok",
                "count": metadata.len(),
                "variables": metadata
            })))
        }
        Err(e) => {
            let safe_err = state.security.secret_redactor.redact(&format!("{}", e));
            Ok(Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to load schema: {}", safe_err),
                "variables": []
            })))
        }
    }
}

// Metadata: [env_schema]

// Metadata: [env_schema]
