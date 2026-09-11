//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / env_schema
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InternalServerError`
//! - **Telemetry Targets**: `[EnvSchema]`
//! - **Witness Tests**: `env_schema::tests::test_env_schema_redaction_invariant`

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
    let schema_path = if state.base_dir.join(".env.schema").exists() {
        state.base_dir.join(".env.schema")
    } else if std::path::Path::new(".env.schema").exists() {
        std::path::PathBuf::from(".env.schema")
    } else if let Some(p) = crate::routes::docs::find_doc_path(".env.schema") {
        p
    } else {
        std::path::PathBuf::from(".env.schema")
    };

    match crate::env_schema::EnvSchema::load(&schema_path) {
        Ok(schema) => {
            let metadata = schema.to_safe_metadata();
            Ok(Json(serde_json::json!({
                "status": "ok",
                "count": metadata.len(),
                "variables": metadata
            })))
        }
        Err(e) => {
            let safe_err = state.security.secret_redactor.redact(&e.to_string());
            tracing::error!("❌ [EnvSchema] Failed to load schema: {}", safe_err);
            Err(AppError::InternalServerError(format!(
                "Failed to load environment schema: {}",
                safe_err
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_env_schema_redaction_invariant() {
        let sample_schema_content = r#"
# @required @sensitive
# Secret OpenAI API Key
OPENAI_API_KEY=

# Server listening port
PORT=8080
"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let schema_file = temp_dir.path().join(".env.schema");
        std::fs::write(&schema_file, sample_schema_content).unwrap();

        std::env::set_var("OPENAI_API_KEY", "sk-secret-key-1234567890abcdef");

        let schema = crate::env_schema::EnvSchema::load(&schema_file).unwrap();
        let metadata = schema.to_safe_metadata();
        let serialized = serde_json::to_string(&metadata).unwrap();

        assert!(!serialized.contains("sk-secret-key"));
        assert!(serialized.contains("OPENAI_API_KEY"));
        assert!(serialized.contains("isSet"));
    }
}
