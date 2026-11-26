//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - SEC-09 runtime surface: reports the content digest the local Ollama runtime holds for a model and
//!   flags drift against the first digest observed by this engine process.
//! - Byte-level weight hashing and tensor fingerprints remain in L3 `execution/weight_space/attest.py`.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Model Manager / Attestation
//! - **Primary Entrypoints**: `get_model_attestation`, `record_baseline`, `find_tag_entry`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Digests are read only from local/private Ollama endpoints (shared guard with probe).
//! - `[Structural]` Runtime-registry digests are Grade B (measured locally); never reported as Grade A.
//! - `[Structural]` A digest change for a known model emits an `engine:model_attestation_drift` warning.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::NotFound, AppError::InternalServerError
//! - **Telemetry Targets**: `[WEIGHTS]`
//! - **Witness Tests**: none declared

use crate::{error::AppError, state::AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::Duration,
};

/// Process-lifetime baseline digests (first observation wins). Intentionally in-memory:
/// persistent baselines are a follow-up and must not add schema drift.
static BASELINES: OnceLock<parking_lot::Mutex<HashMap<String, String>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ModelAttestation {
    pub model_id: String,
    pub weight_digest: String,
    pub size_bytes: u64,
    pub family: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
    pub grade: &'static str,
    pub baseline_digest: String,
    pub drift: bool,
}

/// Records `digest` as the baseline for `model_id` if none exists; returns (baseline, drift).
pub fn record_baseline(
    baselines: &mut HashMap<String, String>,
    model_id: &str,
    digest: &str,
) -> (String, bool) {
    let baseline = baselines
        .entry(model_id.to_string())
        .or_insert_with(|| digest.to_string())
        .clone();
    let drift = baseline != digest;
    (baseline, drift)
}

/// Finds a model in an Ollama `/api/tags` payload by `name` or `model`.
pub fn find_tag_entry<'a>(
    tags: &'a serde_json::Value,
    model_id: &str,
) -> Option<&'a serde_json::Value> {
    tags.get("models")?.as_array()?.iter().find(|m| {
        m.get("name").and_then(|v| v.as_str()) == Some(model_id)
            || m.get("model").and_then(|v| v.as_str()) == Some(model_id)
    })
}

fn detail(entry: &serde_json::Value, key: &str) -> Option<String> {
    entry
        .get("details")
        .and_then(|d| d.get(key))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// GET /v1/infra/models/{id}/attestation
///
/// Returns the SEC-09 provenance record for a locally served model.
#[tracing::instrument(skip(state), fields(model_id = %model_id), name = "infra_models::attestation")]
pub async fn get_model_attestation(
    State(state): State<Arc<AppState>>,
    Path(model_id): Path<String>,
) -> Result<Json<ModelAttestation>, AppError> {
    super::probe::validate_model_id(&model_id)?;
    let base = super::probe::resolve_local_ollama_base(&super::get_ollama_host().await)?;
    let url = format!("{}/api/tags", base);

    let send = state.resources.http_client.get(&url).send();
    let resp = tokio::time::timeout(Duration::from_secs(super::probe::PROBE_TIMEOUT_SECS), send)
        .await
        .map_err(|_| {
            AppError::InternalServerError("[WEIGHTS] Ollama tag listing timed out".into())
        })?
        .map_err(|e| {
            let safe = state.security.secret_redactor.redact(&e.to_string());
            AppError::InternalServerError(format!("[WEIGHTS] Ollama unreachable: {}", safe))
        })?;
    if !resp.status().is_success() {
        return Err(AppError::InternalServerError(format!(
            "[WEIGHTS] Ollama returned HTTP {}",
            resp.status()
        )));
    }
    let tags: serde_json::Value = resp.json().await.map_err(|e| {
        AppError::InternalServerError(format!("[WEIGHTS] Invalid Ollama tag payload: {}", e))
    })?;

    let entry = find_tag_entry(&tags, &model_id).ok_or_else(|| {
        AppError::NotFound(format!(
            "[WEIGHTS] Model '{}' is not installed on the local runtime",
            model_id
        ))
    })?;
    let raw_digest = entry
        .get("digest")
        .and_then(|v| v.as_str())
        .filter(|d| !d.is_empty())
        .ok_or_else(|| {
            AppError::InternalServerError("[WEIGHTS] Runtime did not report a digest".into())
        })?;
    let weight_digest = if raw_digest.starts_with("sha256:") {
        raw_digest.to_string()
    } else {
        format!("sha256:{}", raw_digest)
    };

    let (baseline_digest, drift) = {
        let mut guard = BASELINES
            .get_or_init(|| parking_lot::Mutex::new(HashMap::new()))
            .lock();
        record_baseline(&mut guard, &model_id, &weight_digest)
    };

    if drift {
        tracing::warn!(
            "⚠️ [WEIGHTS] engine:model_attestation_drift model={} baseline={} current={}",
            model_id,
            baseline_digest,
            weight_digest
        );
        state.broadcast_sys(
            &format!(
                "engine:model_attestation_drift — model '{}' digest changed since first attestation (SEC-09)",
                model_id
            ),
            "warning",
            None,
        );
    }

    Ok(Json(ModelAttestation {
        size_bytes: entry.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
        family: detail(entry, "family"),
        parameter_size: detail(entry, "parameter_size"),
        quantization_level: detail(entry, "quantization_level"),
        model_id,
        weight_digest,
        grade: "B",
        baseline_digest,
        drift,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_baseline_detects_drift() {
        let mut map = HashMap::new();
        assert_eq!(
            record_baseline(&mut map, "m", "sha256:aa"),
            ("sha256:aa".to_string(), false)
        );
        assert_eq!(
            record_baseline(&mut map, "m", "sha256:aa"),
            ("sha256:aa".to_string(), false)
        );
        // A changed digest is drift, and the original baseline is retained.
        assert_eq!(
            record_baseline(&mut map, "m", "sha256:bb"),
            ("sha256:aa".to_string(), true)
        );
    }

    #[test]
    fn test_find_tag_entry_matches_name_or_model() {
        let tags = serde_json::json!({
            "models": [
                { "name": "llama3:latest", "model": "llama3:latest", "digest": "abc" },
                { "name": "alias", "model": "phi3:mini", "digest": "def" }
            ]
        });
        assert_eq!(
            find_tag_entry(&tags, "llama3:latest").unwrap()["digest"],
            "abc"
        );
        assert_eq!(find_tag_entry(&tags, "phi3:mini").unwrap()["digest"], "def");
        assert!(find_tag_entry(&tags, "missing").is_none());
        assert!(find_tag_entry(&serde_json::json!({}), "x").is_none());
    }
}
