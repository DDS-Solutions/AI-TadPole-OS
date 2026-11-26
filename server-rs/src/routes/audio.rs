//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / audio
//! - **Primary Entrypoints**: `audio_capabilities`, `text_to_speech`, `transcribe_audio`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Structural]` Cloud audio synthesis and transcription respect Privacy Shield (`privacy_mode`).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::InternalServerError`, `AppError::NotImplemented`
//! - **Telemetry Targets**: `[audio]`

use crate::error::AppError;
use crate::state::AppState;

use axum::{
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
#[cfg(feature = "neural-audio")]
use tracing::{error, info};

pub const MAX_TTS_TEXT_CHARS: usize = 4096;

#[derive(Debug, Serialize)]
pub struct AudioEngineCapability {
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AudioCapabilitiesResponse {
    pub privacy_mode: bool,
    pub tts: BTreeMap<String, AudioEngineCapability>,
    pub stt: BTreeMap<String, AudioEngineCapability>,
}

fn resolve_provider_key(state: &AppState, provider_ids: &[&str], env_var: &str) -> Option<String> {
    for id in provider_ids {
        if let Some(provider) = state.registry.providers.get(*id) {
            if let Some(ref key) = provider.api_key {
                if !key.trim().is_empty() {
                    return Some(key.clone());
                }
            }
        }
    }
    std::env::var(env_var).ok().filter(|k| !k.trim().is_empty())
}

fn is_privacy_mode_active(state: &AppState) -> bool {
    state
        .governance
        .privacy_mode
        .load(std::sync::atomic::Ordering::Acquire)
}

/// Validates and normalizes a local loopback audio server URL (SSRF-safe).
/// Only `http`/`https` targeting strict loopback hosts (`localhost`, `127.0.0.1`, `::1`) are allowed.
pub(crate) fn validate_loopback_audio_url(raw: &str, default_path: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Local audio URL is empty.".to_string());
    }

    let mut url = reqwest::Url::parse(trimmed)
        .map_err(|e| format!("Invalid local audio URL '{}': {}", trimmed, e))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(format!(
            "Local audio URL must use http or https scheme (got '{}').",
            url.scheme()
        ));
    }

    let is_loopback = match url.host() {
        Some(url::Host::Domain(d)) => d.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    };

    if !is_loopback {
        return Err(
            "Local audio URL must target a loopback address (localhost, 127.0.0.1, or ::1)."
                .to_string(),
        );
    }

    if url.path().is_empty() || url.path() == "/" {
        url.set_path(default_path);
    }

    Ok(url.to_string())
}

fn resolve_local_audio_endpoint(
    env_var: &str,
    default_path: &str,
) -> Option<Result<String, String>> {
    let raw = std::env::var(env_var).ok()?;
    if raw.trim().is_empty() {
        return None;
    }
    Some(validate_loopback_audio_url(&raw, default_path))
}

async fn synthesize_via_local_http(
    client: &reqwest::Client,
    endpoint: &str,
    model: &str,
    voice: &str,
    text: &str,
    redactor: &crate::secret_redactor::SecretRedactor,
) -> Result<(String, axum::body::Bytes), AppError> {
    let res = client
        .post(endpoint)
        .json(&json!({
            "model": model,
            "input": text,
            "voice": voice,
        }))
        .send()
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Local TTS server unreachable at {}: {}",
                endpoint,
                redactor.redact(&e.to_string())
            ))
        })?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(AppError::InternalServerError(format!(
            "Local TTS server error: {}",
            redactor.redact(&err_text)
        )));
    }

    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "audio/mpeg".to_string());

    let bytes = res.bytes().await.map_err(|e| {
        AppError::InternalServerError(format!(
            "Failed to read local TTS audio response: {}",
            redactor.redact(&e.to_string())
        ))
    })?;

    Ok((content_type, bytes))
}

async fn transcribe_via_local_http(
    client: &reqwest::Client,
    endpoint: &str,
    model: &str,
    audio_data: Vec<u8>,
    filename: &str,
    mime_type: &str,
    redactor: &crate::secret_redactor::SecretRedactor,
) -> Result<String, AppError> {
    let part = reqwest::multipart::Part::bytes(audio_data)
        .file_name(filename.to_string())
        .mime_str(mime_type)
        .map_err(|e| AppError::BadRequest(format!("Invalid audio MIME type: {}", e)))?;

    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("model", model.to_string());

    let res = client
        .post(endpoint)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Local Whisper server unreachable at {}: {}",
                endpoint,
                redactor.redact(&e.to_string())
            ))
        })?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(AppError::InternalServerError(format!(
            "Local Whisper server error: {}",
            redactor.redact(&err_text)
        )));
    }

    let body = res.text().await.map_err(|e| {
        AppError::InternalServerError(format!(
            "Failed to read local Whisper response: {}",
            redactor.redact(&e.to_string())
        ))
    })?;

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
        if let Some(text) = parsed.get("text").and_then(|v| v.as_str()) {
            return Ok(text.to_string());
        }
    }

    Ok(body.trim().to_string())
}

/// Normalizes uploaded audio filename and MIME type so cloud decoders receive
/// an accurate container format (WebM, OGG, MP4/M4A, MP3, or WAV).
pub(crate) fn normalize_audio_upload_meta(
    raw_filename: Option<&str>,
    raw_mime: Option<&str>,
) -> (String, String) {
    let base_mime = raw_mime
        .and_then(|m| m.split(';').next())
        .map(|m| m.trim().to_ascii_lowercase())
        .filter(|m| !m.is_empty());

    let inferred_from_mime = match base_mime.as_deref() {
        Some("audio/webm") | Some("video/webm") => Some(("speech.webm", "audio/webm")),
        Some("audio/ogg") | Some("application/ogg") => Some(("speech.ogg", "audio/ogg")),
        Some("audio/mp4") | Some("audio/x-m4a") | Some("audio/m4a") => {
            Some(("speech.m4a", "audio/mp4"))
        }
        Some("audio/mpeg") | Some("audio/mp3") => Some(("speech.mp3", "audio/mpeg")),
        Some("audio/wav") | Some("audio/x-wav") | Some("audio/wave") => {
            Some(("speech.wav", "audio/wav"))
        }
        Some("audio/flac") => Some(("speech.flac", "audio/flac")),
        _ => None,
    };

    if let Some((default_name, canonical_mime)) = inferred_from_mime {
        let ext = default_name.rsplit('.').next().unwrap_or("webm");
        let name = match raw_filename.map(str::trim).filter(|n| !n.is_empty()) {
            Some(existing) if existing.to_ascii_lowercase().ends_with(&format!(".{ext}")) => {
                existing.to_string()
            }
            _ => default_name.to_string(),
        };
        return (name, canonical_mime.to_string());
    }

    if let Some(name) = raw_filename.map(str::trim).filter(|n| !n.is_empty()) {
        let lower = name.to_ascii_lowercase();
        let mime = if lower.ends_with(".webm") {
            "audio/webm"
        } else if lower.ends_with(".ogg") || lower.ends_with(".opus") {
            "audio/ogg"
        } else if lower.ends_with(".m4a") || lower.ends_with(".mp4") {
            "audio/mp4"
        } else if lower.ends_with(".mp3") {
            "audio/mpeg"
        } else if lower.ends_with(".flac") {
            "audio/flac"
        } else {
            "audio/wav"
        };
        return (name.to_string(), mime.to_string());
    }

    ("speech.webm".to_string(), "audio/webm".to_string())
}

/// Reports live runtime audio capabilities (TTS & STT engines) for capability-driven UI.
pub async fn audio_capabilities(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let privacy_mode = is_privacy_mode_active(&state);
    let has_openai = resolve_provider_key(&state, &["openai"], "OPENAI_API_KEY").is_some();
    let has_groq = resolve_provider_key(&state, &["groq"], "GROQ_API_KEY").is_some();
    let has_google =
        resolve_provider_key(&state, &["google", "gemini"], "GOOGLE_API_KEY").is_some();

    let mut tts = BTreeMap::new();
    tts.insert(
        "browser".to_string(),
        AudioEngineCapability {
            available: true,
            reason: None,
        },
    );
    tts.insert(
        "openai".to_string(),
        AudioEngineCapability {
            available: !privacy_mode && has_openai,
            reason: if privacy_mode {
                Some("Disabled while Privacy Shield (Local-First) is active.".to_string())
            } else if !has_openai {
                Some(
                    "Configure an OpenAI API key in Provider Settings or OPENAI_API_KEY."
                        .to_string(),
                )
            } else {
                None
            },
        },
    );
    tts.insert(
        "gemini-live".to_string(),
        AudioEngineCapability {
            available: !privacy_mode && has_google,
            reason: if privacy_mode {
                Some("Disabled while Privacy Shield (Local-First) is active.".to_string())
            } else if !has_google {
                Some(
                    "Configure a Google/Gemini API key in Provider Settings or GOOGLE_API_KEY."
                        .to_string(),
                )
            } else {
                None
            },
        },
    );
    let local_tts_endpoint = resolve_local_audio_endpoint("LOCAL_TTS_URL", "/v1/audio/speech");
    let local_whisper_endpoint =
        resolve_local_audio_endpoint("LOCAL_WHISPER_URL", "/v1/audio/transcriptions");

    tts.insert(
        "piper".to_string(),
        match &local_tts_endpoint {
            Some(Ok(_)) => AudioEngineCapability {
                available: true,
                reason: None,
            },
            Some(Err(err)) => AudioEngineCapability {
                available: false,
                reason: Some(err.clone()),
            },
            None => AudioEngineCapability {
                available: false,
                reason: Some(if cfg!(feature = "neural-audio") {
                    "Set LOCAL_TTS_URL (e.g. http://127.0.0.1:8880) or configure local Piper ONNX weights.".to_string()
                } else {
                    "Set LOCAL_TTS_URL to a loopback TTS server (e.g. http://127.0.0.1:8880) or build with --features neural-audio.".to_string()
                }),
            },
        },
    );
    tts.insert(
        "groq".to_string(),
        AudioEngineCapability {
            available: false,
            reason: Some(
                "Groq TTS is not supported on this engine; use Browser, OpenAI, Local (Piper/Kokoro), or Gemini Live."
                    .to_string(),
            ),
        },
    );

    let mut stt = BTreeMap::new();
    stt.insert(
        "browser".to_string(),
        AudioEngineCapability {
            available: true,
            reason: None,
        },
    );
    stt.insert(
        "groq".to_string(),
        AudioEngineCapability {
            available: !privacy_mode && has_groq,
            reason: if privacy_mode {
                Some("Disabled while Privacy Shield (Local-First) is active.".to_string())
            } else if !has_groq {
                Some("Configure a Groq API key in Provider Settings or GROQ_API_KEY.".to_string())
            } else {
                None
            },
        },
    );
    stt.insert(
        "whisper".to_string(),
        match &local_whisper_endpoint {
            Some(Ok(_)) => AudioEngineCapability {
                available: true,
                reason: None,
            },
            Some(Err(err)) => AudioEngineCapability {
                available: false,
                reason: Some(err.clone()),
            },
            None => AudioEngineCapability {
                available: false,
                reason: Some(if cfg!(feature = "neural-audio") {
                    "Set LOCAL_WHISPER_URL (e.g. http://127.0.0.1:8001) or configure local Whisper ONNX weights.".to_string()
                } else {
                    "Set LOCAL_WHISPER_URL to a loopback Whisper server (e.g. http://127.0.0.1:8001) or build with --features neural-audio.".to_string()
                }),
            },
        },
    );

    Ok(Json(AudioCapabilitiesResponse {
        privacy_mode,
        tts,
        stt,
    }))
}

/// Parameters for the `text_to_speech` endpoint.
#[derive(Deserialize)]
pub struct SpeechRequest {
    /// The raw text to convert to audio.
    pub text: String,
    /// Targeted voice model (e.g., "alloy", "echo").
    pub voice: Option<String>,
    /// The engine to use: "browser", "piper" (local), or "openai"/"high-tier".
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
    let engine = payload.engine.as_deref().unwrap_or("browser").trim();
    let voice = payload.voice.as_deref().unwrap_or("alloy");

    if engine == "browser" {
        return Ok((
            StatusCode::OK,
            Json(json!({
                "status": "browser_fallback",
                "message": "Use Web Speech API for this request"
            })),
        )
            .into_response());
    }

    if engine == "piper" || engine == "local" {
        if let Some(endpoint_res) =
            resolve_local_audio_endpoint("LOCAL_TTS_URL", "/v1/audio/speech")
        {
            let endpoint = endpoint_res.map_err(AppError::Forbidden)?;
            let local_model = std::env::var("LOCAL_TTS_MODEL")
                .ok()
                .filter(|m| !m.trim().is_empty())
                .unwrap_or_else(|| "kokoro".to_string());
            let (content_type, bytes) = synthesize_via_local_http(
                &state.resources.http_client,
                &endpoint,
                &local_model,
                voice,
                &payload.text,
                &state.security.secret_redactor,
            )
            .await?;
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                bytes,
            )
                .into_response());
        }

        if engine == "local" {
            return Err(AppError::NotImplemented(
                "Local TTS requires LOCAL_TTS_URL (e.g. http://127.0.0.1:8880) pointing to a loopback TTS server."
                    .to_string(),
            ));
        }

        #[cfg(feature = "neural-audio")]
        {
            let cache_key =
                crate::agent::audio_cache::BunkerCache::scoped_key("piper", voice, &payload.text);
            if let Ok(Some(cached_audio)) = state.resources.audio_cache.get(&cache_key).await {
                info!("[BunkerCache] Zero-latency hit for scoped voice key");
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
            let audio_engine = state.resources.get_audio_engine().await?;
            let cache = state.resources.audio_cache.clone();
            let text = payload.text.clone();

            tokio::spawn(async move {
                if let Err(e) = audio_engine.speak_stream(&text, tx, cache).await {
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
                "Local Piper TTS is not configured. Set LOCAL_TTS_URL (e.g. http://127.0.0.1:8880), build with --features neural-audio, or use 'browser'/'openai'."
                    .to_string(),
            ));
        }
    }

    if engine != "openai" && engine != "high-tier" {
        return Err(AppError::NotImplemented(format!(
            "TTS engine '{}' is not supported by /v1/engine/speak. Use 'browser', 'openai', or Live Voice WebSocket for 'gemini-live'.",
            engine
        )));
    }

    if is_privacy_mode_active(&state) {
        return Err(AppError::Forbidden(
            "Cloud TTS is disabled while Privacy Shield (Local-First) is active.".to_string(),
        ));
    }

    // High-fidelity logic (OpenAI)
    let api_key = match resolve_provider_key(&state, &["openai"], "OPENAI_API_KEY") {
        Some(k) => k,
        None => {
            return Err(AppError::NotImplemented(
                "High-tier TTS requires OpenAI API key configuration in provider settings or OPENAI_API_KEY environment variable.".to_string(),
            ));
        }
    };

    let model = "tts-1";
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
/// Accepts a multipart form with an audio `file` and optional `engine` (`groq` | `whisper`),
/// forwarding accurate container MIME/filename metadata to the configured STT engine.
pub async fn transcribe_audio(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut audio_data = Vec::new();
    let mut raw_filename: Option<String> = None;
    let mut raw_mime: Option<String> = None;
    let mut requested_engine: Option<String> = None;

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                let name = field.name().unwrap_or_default().to_string();
                if name == "file" {
                    raw_filename = field.file_name().map(str::to_string);
                    raw_mime = field.content_type().map(str::to_string);
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
                } else if name == "engine" {
                    if let Ok(val) = field.text().await {
                        let trimmed = val.trim().to_ascii_lowercase();
                        if !trimmed.is_empty() {
                            requested_engine = Some(trimmed);
                        }
                    }
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

    let local_whisper_endpoint =
        resolve_local_audio_endpoint("LOCAL_WHISPER_URL", "/v1/audio/transcriptions");

    let engine = match requested_engine.as_deref() {
        Some(explicit) => explicit,
        None if is_privacy_mode_active(&state) && matches!(local_whisper_endpoint, Some(Ok(_))) => {
            "whisper"
        }
        None => "groq",
    };

    if engine == "local" || engine == "whisper" {
        if let Some(endpoint_res) = local_whisper_endpoint {
            let endpoint = endpoint_res.map_err(AppError::Forbidden)?;
            let local_model = std::env::var("LOCAL_WHISPER_MODEL")
                .ok()
                .filter(|m| !m.trim().is_empty())
                .unwrap_or_else(|| "whisper-1".to_string());
            let (filename, mime_type) =
                normalize_audio_upload_meta(raw_filename.as_deref(), raw_mime.as_deref());
            let text = transcribe_via_local_http(
                &state.resources.http_client,
                &endpoint,
                &local_model,
                audio_data,
                &filename,
                &mime_type,
                &state.security.secret_redactor,
            )
            .await?;
            return Ok(Json(json!({
                "status": "success",
                "text": text
            })));
        }

        #[cfg(feature = "neural-audio")]
        {
            let audio_engine = state.resources.get_audio_engine().await?;
            match audio_engine.listen(audio_data).await {
                Ok(text) => {
                    return Ok(Json(json!({
                        "status": "success",
                        "text": text
                    })))
                }
                Err(e) => {
                    return Err(AppError::NotImplemented(format!(
                        "Local Whisper unavailable: {}",
                        e
                    )))
                }
            }
        }

        #[cfg(not(feature = "neural-audio"))]
        {
            return Err(AppError::NotImplemented(
                "Local Whisper transcription is not configured. Set LOCAL_WHISPER_URL (e.g. http://127.0.0.1:8001), select Groq STT, or build with --features neural-audio.".to_string()
            ));
        }
    }

    if engine != "groq" {
        return Err(AppError::BadRequest(format!(
            "Unsupported STT engine '{}'. Supported values: 'groq', 'whisper'.",
            engine
        )));
    }

    if is_privacy_mode_active(&state) {
        return Err(AppError::Forbidden(
            "Cloud STT transcription is disabled while Privacy Shield (Local-First) is active."
                .to_string(),
        ));
    }

    // Initialize Groq Provider
    let api_key = resolve_provider_key(&state, &["groq"], "GROQ_API_KEY").ok_or_else(|| {
        AppError::NotImplemented(
            "Groq STT requires a Groq API key in Provider Settings or GROQ_API_KEY environment variable."
                .to_string(),
        )
    })?;

    let model_id = state
        .registry
        .providers
        .get("groq")
        .and_then(|p| p.audio_model.clone())
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| "whisper-large-v3".to_string());

    let (filename, mime_type) =
        normalize_audio_upload_meta(raw_filename.as_deref(), raw_mime.as_deref());

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
        custom_headers: None,
        context_window: None,
        input_cost_per_m: None,
        output_cost_per_m: None,
    };

    let client = (*state.resources.http_client).clone();
    let provider = crate::agent::groq::GroqProvider::new(client, api_key, config);

    let text = provider
        .transcribe(audio_data, &filename, &mime_type)
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
    use axum::routing::{get, post};
    use axum::Router;
    use tower::util::ServiceExt;

    #[test]
    fn test_normalize_audio_upload_meta_preserves_webm_and_strips_codec_params() {
        let (name, mime) =
            normalize_audio_upload_meta(Some("speech.wav"), Some("audio/webm;codecs=opus"));
        assert_eq!(name, "speech.webm");
        assert_eq!(mime, "audio/webm");

        let (ogg_name, ogg_mime) =
            normalize_audio_upload_meta(Some("clip.ogg"), Some("audio/ogg; codecs=opus"));
        assert_eq!(ogg_name, "clip.ogg");
        assert_eq!(ogg_mime, "audio/ogg");

        let (m4a_name, m4a_mime) = normalize_audio_upload_meta(None, Some("audio/mp4"));
        assert_eq!(m4a_name, "speech.m4a");
        assert_eq!(m4a_mime, "audio/mp4");
    }

    #[tokio::test]
    async fn test_audio_capabilities_reports_privacy_and_browser_availability() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Release);
        let app: Router = Router::new()
            .route("/", get(audio_capabilities))
            .with_state(state);

        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(response.into_body(), 8192)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["privacy_mode"], true);
        assert_eq!(body["tts"]["browser"]["available"], true);
        assert_eq!(body["tts"]["openai"]["available"], false);
        assert_eq!(body["tts"]["groq"]["available"], false);
        assert_eq!(body["stt"]["groq"]["available"], false);
    }

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
    async fn test_speak_rejects_unsupported_groq_tts_engine() {
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
                    "engine": "groq"
                }))
                .unwrap(),
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
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

    #[tokio::test]
    async fn test_transcribe_respects_whisper_engine_selection() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app: Router = Router::new()
            .route("/", post(transcribe_audio))
            .with_state(state);

        let multipart_body = concat!(
            "--X\r\n",
            "Content-Disposition: form-data; name=\"engine\"\r\n\r\n",
            "whisper\r\n",
            "--X\r\n",
            "Content-Disposition: form-data; name=\"file\"; filename=\"speech.webm\"\r\n",
            "Content-Type: audio/webm\r\n\r\n",
            "fakeaudio\r\n",
            "--X--\r\n"
        );

        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("Content-Type", "multipart/form-data; boundary=X")
            .body(Body::from(multipart_body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[test]
    fn test_validate_loopback_audio_url_enforces_strict_loopback_only() {
        assert_eq!(
            validate_loopback_audio_url("http://127.0.0.1:8001", "/v1/audio/transcriptions")
                .unwrap(),
            "http://127.0.0.1:8001/v1/audio/transcriptions"
        );
        assert_eq!(
            validate_loopback_audio_url(
                "http://localhost:8880/v1/audio/speech",
                "/v1/audio/speech"
            )
            .unwrap(),
            "http://localhost:8880/v1/audio/speech"
        );
        assert!(
            validate_loopback_audio_url("http://[::1]:8001", "/v1/audio/transcriptions").is_ok()
        );

        // Reject cloud, LAN, metadata, and non-HTTP schemes
        assert!(validate_loopback_audio_url(
            "https://api.openai.com/v1/audio/speech",
            "/v1/audio/speech"
        )
        .is_err());
        assert!(validate_loopback_audio_url(
            "http://10.0.0.1:8001",
            "/v1/audio/transcriptions"
        )
        .is_err());
        assert!(
            validate_loopback_audio_url("http://10.0.0.1/latest", "/v1/audio/speech")
                .is_err()
        );
        assert!(validate_loopback_audio_url("file:///etc/passwd", "/v1/audio/speech").is_err());
    }

    #[tokio::test]
    async fn test_local_http_tts_and_whisper_roundtrip_over_loopback() {
        async fn mock_tts(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
            let input = body["input"].as_str().unwrap_or_default().to_string();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "audio/wav")],
                format!("WAV:{}", input),
            )
        }

        async fn mock_stt(mut multipart: Multipart) -> impl IntoResponse {
            let mut got_filename = String::new();
            let mut got_model = String::new();
            while let Ok(Some(field)) = multipart.next_field().await {
                match field.name() {
                    Some("file") => {
                        got_filename = field.file_name().unwrap_or_default().to_string();
                        let _ = field.bytes().await;
                    }
                    Some("model") => {
                        got_model = field.text().await.unwrap_or_default();
                    }
                    _ => {}
                }
            }
            Json(json!({ "text": format!("{}:{}", got_model, got_filename) }))
        }

        let mock_app = Router::new()
            .route("/v1/audio/speech", post(mock_tts))
            .route("/v1/audio/transcriptions", post(mock_stt));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, mock_app).await.unwrap();
        });

        let client = reqwest::Client::new();
        let redactor = crate::secret_redactor::SecretRedactor::noop();

        let tts_url = validate_loopback_audio_url(
            &format!("http://127.0.0.1:{}", addr.port()),
            "/v1/audio/speech",
        )
        .unwrap();
        let (ct, audio_bytes) = synthesize_via_local_http(
            &client,
            &tts_url,
            "kokoro",
            "af_heart",
            "hello local",
            &redactor,
        )
        .await
        .unwrap();
        assert_eq!(ct, "audio/wav");
        assert_eq!(audio_bytes.as_ref(), b"WAV:hello local");

        let stt_url = validate_loopback_audio_url(
            &format!("http://127.0.0.1:{}", addr.port()),
            "/v1/audio/transcriptions",
        )
        .unwrap();
        let transcript = transcribe_via_local_http(
            &client,
            &stt_url,
            "whisper-1",
            b"fake-webm-bytes".to_vec(),
            "speech.webm",
            "audio/webm",
            &redactor,
        )
        .await
        .unwrap();
        assert_eq!(transcript, "whisper-1:speech.webm");

        server_handle.abort();
    }
}
