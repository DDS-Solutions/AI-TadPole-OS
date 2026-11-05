//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - Implements lightweight 3-shot micro-probing for local models to detect delimiter drift and reasoning loops.
//! - Probes dispatch real `/api/generate` calls to the local Ollama runtime; scoring is pure and unit-tested.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Model Manager / Probe
//! - **Primary Entrypoints**: `probe_model_handler`, `evaluate_probe_response`, `resolve_local_ollama_base`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Micro-probes are strictly restricted to local/private Ollama endpoints (fail-closed 400 otherwise).
//! - `[Structural]` Each probe turn is bounded by a 10s strict timeout and a 160-token generation cap.
//! - `[Structural]` If calibration fails, recommended sampling parameters are clamped (repeat_penalty >= 1.15).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::NotFound, AppError::InternalServerError
//! - **Telemetry Targets**: `[MODEL_PROBE]`
//! - **Witness Tests**: none declared

use crate::{error::AppError, state::AppState};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

/// Per-turn wall-clock bound for a single probe generation.
pub const PROBE_TIMEOUT_SECS: u64 = 10;
/// Generation cap per probe turn (keeps probes cheap on small local models).
const PROBE_MAX_TOKENS: u32 = 160;
/// Minimum repeat penalty recommended when a model fails calibration.
pub const CLAMPED_REPEAT_PENALTY: f32 = 1.20;

const PROBE_SCHEMA: &str = "Return ONLY a JSON object with keys \"status\" (string) and \"count\" (integer). No prose, no code fences.";
const PROBE_DELIMITER: &str = "Reply with exactly one line in this form and nothing else: Action: read_file(path=\"main.rs\")";
const PROBE_LOOP: &str = "In about 80 words, list distinct strategies for debugging a failing unit test. Never repeat a sentence.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedOptions {
    pub repeat_penalty: f32,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProbeResult {
    pub model_id: String,
    pub calibrated: bool,
    pub score: f32,
    pub recommended_options: RecommendedOptions,
    pub failure_mode: Option<String>,
}

/// Computes token repetition ratio across consecutive 3-grams in a text string.
pub fn calculate_repetition_ratio(text: &str) -> f32 {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 4 {
        return 0.0;
    }
    let total_ngrams = words.len() - 2;
    let unique_ngrams: std::collections::HashSet<(&str, &str, &str)> =
        words.windows(3).map(|w| (w[0], w[1], w[2])).collect();
    1.0 - (unique_ngrams.len() as f32 / total_ngrams as f32)
}

/// Evaluates a local model's three probe responses (schema, delimiter, loop resistance).
pub fn evaluate_probe_response(
    schema_output: &str,
    delimiter_output: &str,
    loop_output: &str,
) -> (bool, f32, Option<String>) {
    let mut score: f32 = 1.0;
    let mut failures = Vec::new();

    // 1. Schema: output must contain a parseable JSON object.
    if !contains_json_object(schema_output) {
        score -= 0.35;
        failures.push("schema_not_json".to_string());
    }

    // 2. Delimiter containment: raw chat-template control tokens must never leak.
    const CONTROL_TOKENS: [&str; 4] = [
        "<|im_start|>",
        "<|im_end|>",
        "<|eot_id|>",
        "<|start_header_id|>",
    ];
    if CONTROL_TOKENS.iter().any(|t| delimiter_output.contains(t)) {
        score -= 0.30;
        failures.push("raw_control_token_leakage".to_string());
    }

    // 3. Loop propensity: measure 3-gram repetition.
    let rep_ratio = calculate_repetition_ratio(loop_output);
    if rep_ratio > 0.40 {
        score -= 0.40;
        failures.push(format!("high_loop_propensity(ratio={:.2})", rep_ratio));
    }

    let score = score.clamp(0.0, 1.0);
    let failure_mode = (!failures.is_empty()).then(|| failures.join("; "));
    (score >= 0.70, score, failure_mode)
}

/// Returns sampling recommendations; failing models get clamped, conservative parameters.
pub fn recommended_options(calibrated: bool) -> RecommendedOptions {
    if calibrated {
        RecommendedOptions {
            repeat_penalty: 1.05,
            temperature: 0.7,
            top_p: 0.95,
        }
    } else {
        RecommendedOptions {
            repeat_penalty: CLAMPED_REPEAT_PENALTY,
            temperature: 0.2,
            top_p: 0.85,
        }
    }
}

/// True if `text` contains a `{...}` span that parses as a JSON object.
fn contains_json_object(text: &str) -> bool {
    match (text.find('{'), text.rfind('}')) {
        (Some(start), Some(end)) if end > start => {
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&text[start..=end])
                .is_ok()
        }
        _ => false,
    }
}

/// Normalizes an Ollama host into a bare base URL and enforces the local-only invariant.
pub fn resolve_local_ollama_base(raw_host: &str) -> Result<String, AppError> {
    let clean = raw_host.trim().trim_end_matches('/');
    let base = clean
        .strip_suffix("/v1")
        .unwrap_or(clean)
        .trim_end_matches('/');
    let parsed = url::Url::parse(base)
        .map_err(|e| AppError::BadRequest(format!("[MODEL_PROBE] Invalid Ollama host: {}", e)))?;
    let host = parsed.host_str().unwrap_or_default();
    if !super::is_host_private_or_loopback(host) {
        return Err(AppError::BadRequest(
            "[MODEL_PROBE] Probes are restricted to local/private inference endpoints.".to_string(),
        ));
    }
    Ok(base.to_string())
}

/// Rejects empty, oversized, or control-character model identifiers before they reach Ollama.
pub fn validate_model_id(model_id: &str) -> Result<(), AppError> {
    if model_id.is_empty()
        || model_id.len() > 256
        || model_id
            .chars()
            .any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(AppError::BadRequest(
            "[MODEL_PROBE] Model id must be 1-256 non-whitespace characters.".to_string(),
        ));
    }
    Ok(())
}

async fn run_probe_turn(
    state: &AppState,
    base: &str,
    model_id: &str,
    prompt: &str,
) -> Result<String, AppError> {
    let url = format!("{}/api/generate", base);
    let body = serde_json::json!({
        "model": model_id,
        "prompt": prompt,
        "stream": false,
        "options": { "temperature": 0.2, "num_predict": PROBE_MAX_TOKENS },
    });
    let send = state.resources.http_client.post(&url).json(&body).send();
    let resp = tokio::time::timeout(Duration::from_secs(PROBE_TIMEOUT_SECS), send)
        .await
        .map_err(|_| {
            AppError::InternalServerError(format!(
                "[MODEL_PROBE] Probe turn timed out after {}s",
                PROBE_TIMEOUT_SECS
            ))
        })?
        .map_err(|e| {
            let safe = state.security.secret_redactor.redact(&e.to_string());
            AppError::InternalServerError(format!("[MODEL_PROBE] Ollama unreachable: {}", safe))
        })?;

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::NotFound(format!(
            "[MODEL_PROBE] Model '{}' is not installed on the local runtime",
            model_id
        )));
    }
    if !status.is_success() {
        return Err(AppError::InternalServerError(format!(
            "[MODEL_PROBE] Ollama returned HTTP {}",
            status
        )));
    }
    let value: serde_json::Value = resp.json().await.map_err(|e| {
        AppError::InternalServerError(format!("[MODEL_PROBE] Invalid Ollama response: {}", e))
    })?;
    Ok(value
        .get("response")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string())
}

/// POST /v1/infra/models/{id}/probe
///
/// Runs the 3-shot micro-probe (schema, delimiter, loop resistance) against a local Ollama model.
#[tracing::instrument(skip(_admin, state), fields(model_id = %model_id), name = "infra_models::probe")]
pub async fn probe_model_handler(
    _admin: crate::middleware::auth::RequireAdmin,
    State(state): State<Arc<AppState>>,
    Path(model_id): Path<String>,
) -> Result<Json<ModelProbeResult>, AppError> {
    validate_model_id(&model_id)?;
    let base = resolve_local_ollama_base(&super::get_ollama_host().await)?;

    // Sequential on purpose: small local models serialize generation anyway.
    let schema_out = run_probe_turn(&state, &base, &model_id, PROBE_SCHEMA).await?;
    let delimiter_out = run_probe_turn(&state, &base, &model_id, PROBE_DELIMITER).await?;
    let loop_out = run_probe_turn(&state, &base, &model_id, PROBE_LOOP).await?;

    let (calibrated, score, failure_mode) =
        evaluate_probe_response(&schema_out, &delimiter_out, &loop_out);

    tracing::info!(
        "🧪 [MODEL_PROBE] model={} calibrated={} score={:.2} failures={:?}",
        model_id,
        calibrated,
        score,
        failure_mode
    );

    Ok(Json(ModelProbeResult {
        model_id,
        calibrated,
        score,
        recommended_options: recommended_options(calibrated),
        failure_mode,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_repetition_ratio() {
        let clean = "This is a clean and non-repeating sentence with diverse vocabulary words.";
        assert!(calculate_repetition_ratio(clean) < 0.1);

        let repeating =
            "I must recruit Alpha I must recruit Alpha I must recruit Alpha I must recruit Alpha";
        assert!(calculate_repetition_ratio(repeating) > 0.5);
    }

    #[test]
    fn test_evaluate_probe_response_healthy() {
        let (calibrated, score, failure) = evaluate_probe_response(
            "Sure: {\"status\": \"healthy\", \"count\": 3}",
            "Action: read_file(path=\"main.rs\")",
            "This is a thoughtful answer exploring alternate hypotheses.",
        );
        assert!(calibrated);
        assert_eq!(score, 1.0);
        assert!(failure.is_none());
    }

    #[test]
    fn test_evaluate_probe_response_looping() {
        let (calibrated, score, failure) = evaluate_probe_response(
            "invalid plain text",
            "<|im_start|> leaked token",
            "loop loop loop loop loop loop loop loop loop loop",
        );
        assert!(!calibrated);
        assert!(score < 0.5);
        assert!(failure.is_some());
        assert!(recommended_options(calibrated).repeat_penalty >= 1.15);
    }

    #[test]
    fn test_schema_probe_rejects_brace_lookalikes() {
        // Braces alone are not enough; the span must parse as a JSON object.
        assert!(!contains_json_object("use { and } in rust"));
        assert!(contains_json_object("```json\n{\"a\": 1}\n```"));
    }

    #[test]
    fn test_resolve_local_ollama_base_enforces_local_only() {
        assert_eq!(
            resolve_local_ollama_base("http://127.0.0.1:11434/v1/").unwrap(),
            "http://127.0.0.1:11434"
        );
        assert!(resolve_local_ollama_base("http://localhost:11434").is_ok());
        assert!(resolve_local_ollama_base("https://api.example.com").is_err());
        assert!(resolve_local_ollama_base("not a url").is_err());
    }

    #[test]
    fn test_validate_model_id() {
        assert!(validate_model_id("llama3.2:3b").is_ok());
        assert!(validate_model_id("").is_err());
        assert!(validate_model_id("bad id").is_err());
        assert!(validate_model_id(&"x".repeat(300)).is_err());
    }
}
