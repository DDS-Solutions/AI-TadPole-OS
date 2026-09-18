//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / audio
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InternalServerError`, `AppError::NotImplemented`
//! - **Telemetry Targets**: `[audio]`
//! - **Witness Tests**: `audio::tests::test_speak_missing_text`, `audio::tests::test_speak_browser_fallback`, `audio::tests::test_transcribe_missing_audio`

use crate::error::AppError;
use crate::state::AppState;

use axum::{
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
#[cfg(feature = "neural-audio")]
use tracing::{error, info};

pub const MAX_TTS_TEXT_CHARS: usize = 4096;

/// Parameters for the `text_to_speech` endpoint.
#[derive(Deserialize)]
pub struct SpeechRequest {
    /// The raw text to convert to audio.
    pub text: String,
    /// Targeted voice model (e.g., "alloy", "echo").
    pub voice: Option<String>,
    /// The engine to use: "browser", "piper" (local), or "high-tier" (OpenAI).
    pub engine: Option<String>,
}

/// Generates audio from text using the requested engine.
///
/// Handles browser fallback, zero-latency Piper streaming,
/// and high-fidelity OpenAI synthesis.
pub async fn text_to_speech(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SpeechRequest>,
) -> Result<impl IntoResponse, AppError> {
    let trimmed = payload.text.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("Text is required".to_string()));
    }

    if payload.text.len() > MAX_TTS_TEXT_CHARS {
        return Err(AppError::BadRequest(format!(
            "Text exceeds maximum allowed length of {} characters",
            MAX_TTS_TEXT_CHARS
        )));
    }

    // Default to browser-side TTS if no engine is specified or if configured as such
    let engine = payload.engine.as_deref().unwrap_or("browser");

    if engine == "browser" {
        // Return a signal that browser should handle it locally
        return Ok((
            StatusCode::OK,
            Json(json!({
                "status": "browser_fallback",
                "message": "Use Web Speech API for this request"
            })),
        )
            .into_response());
    }

    if engine == "piper" {
        #[cfg(feature = "neural-audio")]
        {
            // Check Bunker Cache for zero-latency hit
            if let Ok(Some(cached_audio)) = state.resources.audio_cache.get(&payload.text).await {
                info!("[BunkerCache] Zero-latency hit for: {}", payload.text);
                let tx = state.comms.audio_stream_tx.clone();
                if let Err(e) = tx.send(cached_audio) {
                    tracing::debug!("[audio] Audio stream receiver closed: {:?}", e);
                }
                return Ok((
                    StatusCode::OK,
                    Json(json!({
                        "status": "cached",
                        "message": "Serving from zero-latency Bunker Cache"
                    })),
                )
                    .into_response());
            }

            let tx = state.comms.audio_stream_tx.clone();
            let engine = state.resources.get_audio_engine().await;
            let cache = state.resources.audio_cache.clone();
            let text = payload.text.clone();

            tokio::spawn(async move {
                if let Err(e) = engine.speak_stream(&text, tx, cache).await {
                    error!("Piper stream error: {}", e);
                }
            });

            return Ok((
                StatusCode::OK,
                Json(json!({
                    "status": "streaming",
                    "message": "Audio chunks are being broadcast over WebSocket"
                })),
            )
                .into_response());
        }

        #[cfg(not(feature = "neural-audio"))]
        {
            return Err(AppError::NotImplemented(
                "Local Piper TTS is disabled in this 'Legacy' build. Use 'browser' or 'high-tier' (OpenAI) instead.".to_string()
            ));
        }
    }

    // High-fidelity logic (OpenAI)
    let api_key = if let Some(openai_provider) = state.registry.providers.get("openai") {
        openai_provider
            .api_key
            .clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
    } else {
        std::env::var("OPENAI_API_KEY").ok()
    };

    let api_key = match api_key {
        Some(k) if !k.trim().is_empty() => k,
        _ => {
            return Err(AppError::NotImplemented(
                "High-tier TTS requires OpenAI API key configuration in provider settings or OPENAI_API_KEY environment variable.".to_string(),
            ));
        }
    };

    let voice = payload.voice.as_deref().unwrap_or("alloy");
    let model = "tts-1"; // Baseline top-tier model

    let client = &state.resources.http_client;
    let res = client
        .post("https://api.openai.com/v1/audio/speech")
        .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
        .json(&json!({
            "model": model,
            "input": payload.text,
            "voice": voice,
        }))
        .send()
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Network Error: {}",
                state.security.secret_redactor.redact(&e.to_string())
            ))
        })?;

    if res.status().is_success() {
        let bytes = res.bytes().await.map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to read TTS audio stream: {}",
                state.security.secret_redactor.redact(&e.to_string())
            ))
        })?;
        Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "audio/mpeg")],
            bytes,
        )
            .into_response())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        let safe_err = state.security.secret_redactor.redact(&err_text);
        Err(AppError::InternalServerError(format!(
            "TTS Provider Error: {}",
            safe_err
        )))
    }
}

/// Standardized audio transcription endpoint.
///
/// Accepts a multipart form with an audio file and dispatches it
/// to the Groq/Whisper engine for high-speed text extraction.
pub async fn transcribe_audio(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut audio_data = Vec::new();
    let mut filename = "speech.wav".to_string();

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                let name = field.name().unwrap_or_default().to_string();
                if name == "file" {
                    filename = field.file_name().unwrap_or("speech.wav").to_string();
                    let mut bytes_acc = Vec::new();
                    let mut field = field;
                    while let Some(chunk) = field
                        .chunk()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("Failed to read chunk: {}", e)))?
                    {
                        if bytes_acc.len() + chunk.len() > 25 * 1024 * 1024 {
                            return Err(AppError::BadRequest(
                                "Audio file size exceeds limit of 25MB".to_string(),
                            ));
                        }
                        bytes_acc.extend_from_slice(&chunk);
                    }
                    audio_data = bytes_acc;
                }
            }
            Ok(None) => break,
            Err(e) => {
                return Err(AppError::BadRequest(format!(
                    "Malformed multipart payload: {}",
                    e
                )));
            }
        }
    }

    if audio_data.is_empty() {
        return Err(AppError::BadRequest("No audio file provided".to_string()));
    }

    let engine = "groq"; // Default for now

    if engine == "local" {
        #[cfg(feature = "neural-audio")]
        {
            match state
                .resources
                .get_audio_engine()
                .await
                .listen(audio_data)
                .await
            {
                Ok(text) => {
                    return Ok(Json(json!({
                        "status": "success",
                        "text": text
                    })))
                }
                Err(e) => {
                    return Err(AppError::InternalServerError(format!(
                        "Local Whisper Error: {}",
                        e
                    )))
                }
            }
        }

        #[cfg(not(feature = "neural-audio"))]
        {
            return Err(AppError::NotImplemented(
                "Local Whisper transcription is disabled in this 'Legacy' build. Using cloud-based Groq/Whisper fallback instead.".to_string()
            ));
        }
    }

    // Initialize Groq Provider
    let (api_key, model_id) = if let Some(groq_provider) = state.registry.providers.get("groq") {
        let key = groq_provider
            .api_key
            .clone()
            .or_else(|| std::env::var("GROQ_API_KEY").ok())
            .ok_or_else(|| AppError::InternalServerError("Missing GROQ_API_KEY".to_string()))?;

        let model = groq_provider
            .audio_model
            .clone()
            .unwrap_or_else(|| "whisper-large-v3".to_string());

        (key, model)
    } else {
        let key = std::env::var("GROQ_API_KEY")
            .map_err(|_| AppError::InternalServerError("Missing GROQ_API_KEY".to_string()))?;
        (key, "whisper-large-v3".to_string())
    };

    let config = crate::agent::types::ModelConfig {
        provider: crate::agent::types::ModelProvider::Groq,
        model_id,
        api_key: Some(api_key.clone()),
        base_url: None,
        system_prompt: None,
        temperature: None,
        max_tokens: None,
        external_id: None,
        rpm: None,
        rpd: None,
        tpm: None,
        tpd: None,
        skills: None,
        workflows: None,
        mcp_tools: None,
        connector_configs: None,
        extra_parameters: None,
        steering_vectors: None,
        reasoning_depth: None,
        act_threshold: None,
        max_turns: None,
    };

    // Use the shared HTTP client from AppState (PERF-01 fix)
    let client = (*state.resources.http_client).clone();
    let provider = crate::agent::groq::GroqProvider::new(client, api_key, config);

    let text = provider
        .transcribe(audio_data, &filename)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "text": text
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::post;
    use axum::Router;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_speak_missing_text() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app: Router = Router::new()
            .route("/", post(text_to_speech))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({ "text": "" })).unwrap(),
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_speak_browser_fallback() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app: Router = Router::new()
            .route("/", post(text_to_speech))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&json!({
                    "text": "Hello",
                    "engine": "browser"
                }))
                .unwrap(),
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["status"], "browser_fallback");
    }

    #[tokio::test]
    async fn test_transcribe_missing_audio() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app: Router = Router::new()
            .route("/", post(transcribe_audio))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("Content-Type", "multipart/form-data; boundary=X")
            .body(Body::from("--X--"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
