//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / audio
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[NeuralAudio]`
//! - **Witness Tests**: none declared

use crate::error::AppError;
#[cfg(feature = "neural-audio")]
use crate::error::{InfrastructureErrorKind, ProviderId};
use std::sync::Arc;
use tracing::info;

#[cfg(feature = "neural-audio")]
use ort::session::Session;

/// Engine responsible for high-performance, local-first audio processing.
#[cfg(feature = "neural-audio")]
#[derive(Clone, Default)]
pub struct NeuralAudioEngine {
    piper_session: Option<Arc<Session>>,
    whisper_session: Option<Arc<Session>>,
    vad_session: Option<Arc<Session>>,
}

#[cfg(not(feature = "neural-audio"))]
#[derive(Clone, Default)]
pub struct NeuralAudioEngine;

#[cfg(feature = "neural-audio")]
#[allow(dead_code)] // Public API for runtime model loading, wired during startup
impl NeuralAudioEngine {
    pub async fn new() -> Result<Self, AppError> {
        Ok(Self {
            piper_session: None,
            whisper_session: None,
            vad_session: None,
        })
    }

    /// Load all models defined in environment variables.
    /// This is called during AppState initialization.
    pub async fn auto_load(&mut self) -> Result<(), AppError> {
        let piper_path = std::env::var("PIPER_MODEL_PATH")
            .unwrap_or_else(|_| "data/models/piper.onnx".to_string());
        let whisper_path = std::env::var("WHISPER_MODEL_PATH")
            .unwrap_or_else(|_| "data/models/whisper.onnx".to_string());
        let vad_path = std::env::var("VAD_MODEL_PATH")
            .unwrap_or_else(|_| "data/models/silero_vad.onnx".to_string());

        if std::path::Path::new(&piper_path).exists() {
            let _ = self.load_piper(&piper_path).await;
        } else {
            tracing::warn!(
                "[NeuralAudio] Piper model not found at {}. TTS will be disabled.",
                piper_path
            );
        }

        if std::path::Path::new(&whisper_path).exists() {
            let _ = self.load_whisper(&whisper_path).await;
        } else {
            tracing::warn!(
                "[NeuralAudio] Whisper model not found at {}. STT will be disabled.",
                whisper_path
            );
        }

        if std::path::Path::new(&vad_path).exists() {
            let _ = self.load_vad(&vad_path).await;
        } else {
            tracing::warn!(
                "[NeuralAudio] VAD model not found at {}. Voice detection will be disabled.",
                vad_path
            );
        }

        Ok(())
    }

    /// Load the Piper TTS model.
    pub async fn load_piper(&mut self, model_path: &str) -> Result<(), AppError> {
        let session = match Session::builder() {
            Ok(builder) => builder
                .with_intra_threads(4)
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to set threads: {}", e))
                })?
                .commit_from_file(model_path)
                .map_err(|e| AppError::InfrastructureError {
                    provider_id: ProviderId::Audio,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!("Failed to load model {}: {}", model_path, e),
                    help_link: None,
                })?,
            Err(e) => {
                tracing::error!("[NeuralAudio] Critical error initializing ORT SessionBuilder: {}. This usually means native libraries are missing or incompatible.", e);
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Audio,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!("ORT init failure: {}", e),
                    help_link: None,
                });
            }
        };

        self.piper_session = Some(Arc::new(session));
        info!(
            "[NeuralAudio] Piper model loaded successfully from {}",
            model_path
        );
        Ok(())
    }

    /// Load the Whisper STT model.
    pub async fn load_whisper(&mut self, model_path: &str) -> Result<(), AppError> {
        let session = match Session::builder() {
            Ok(builder) => builder
                .with_intra_threads(4)
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to set threads: {}", e))
                })?
                .commit_from_file(model_path)
                .map_err(|e| AppError::InfrastructureError {
                    provider_id: ProviderId::Audio,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!("Failed to load model {}: {}", model_path, e),
                    help_link: None,
                })?,
            Err(e) => {
                tracing::error!("[NeuralAudio] Critical error initializing ORT SessionBuilder: {}. This usually means native libraries are missing or incompatible.", e);
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Audio,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!("ORT init failure: {}", e),
                    help_link: None,
                });
            }
        };

        self.whisper_session = Some(Arc::new(session));
        info!(
            "[NeuralAudio] Whisper model loaded successfully from {}",
            model_path
        );
        Ok(())
    }

    /// Load the Silero VAD model.
    pub async fn load_vad(&mut self, model_path: &str) -> Result<(), AppError> {
        let session = match Session::builder() {
            Ok(builder) => builder
                .with_intra_threads(1)
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to set threads: {}", e))
                })?
                .commit_from_file(model_path)
                .map_err(|e| AppError::InfrastructureError {
                    provider_id: ProviderId::Audio,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!("Failed to load model {}: {}", model_path, e),
                    help_link: None,
                })?,
            Err(e) => {
                tracing::error!("[NeuralAudio] Critical error initializing ORT SessionBuilder: {}. This usually means native libraries are missing or incompatible.", e);
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Audio,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!("ORT init failure: {}", e),
                    help_link: None,
                });
            }
        };

        self.vad_session = Some(Arc::new(session));
        info!(
            "[NeuralAudio] Silero VAD model loaded successfully from {}",
            model_path
        );
        Ok(())
    }

    /// Synthesize text to speech using Piper (Streaming).
    pub async fn speak_stream(
        &self,
        _text: &str,
        _sender: tokio::sync::broadcast::Sender<Vec<u8>>,
        _cache: Arc<crate::agent::audio_cache::BunkerCache>,
    ) -> Result<(), AppError> {
        let _session =
            self.piper_session
                .as_ref()
                .ok_or_else(|| AppError::InfrastructureError {
                    provider_id: ProviderId::Audio,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: "Piper engine not loaded. Check PIPER_MODEL_PATH.".to_string(),
                    help_link: None,
                })?;

        // F25: Return explicit unsupported outcome rather than fabricating dummy zero-filled audio
        Err(AppError::NotImplemented(
            "Local Piper streaming inference runtime is not configured. Use browser or cloud TTS instead.".to_string(),
        ))
    }

    /// Synthesize text to speech using Piper (Full).
    #[allow(dead_code)]
    pub async fn speak(&self, _text: &str) -> Result<Vec<u8>, AppError> {
        let _session =
            self.piper_session
                .as_ref()
                .ok_or_else(|| AppError::InfrastructureError {
                    provider_id: ProviderId::Audio,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: "Piper engine not loaded".to_string(),
                    help_link: None,
                })?;

        Err(AppError::NotImplemented(
            "Local Piper batch inference runtime is not configured. Use browser or cloud TTS instead.".to_string(),
        ))
    }

    /// Transcribe audio to text using Whisper.
    pub async fn listen(&self, _audio_data: Vec<u8>) -> Result<String, AppError> {
        let _session =
            self.whisper_session
                .as_ref()
                .ok_or_else(|| AppError::InfrastructureError {
                    provider_id: ProviderId::Audio,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: "Whisper engine not loaded. Check WHISPER_MODEL_PATH.".to_string(),
                    help_link: None,
                })?;

        Err(AppError::NotImplemented(
            "Local Whisper ONNX inference runtime is not configured. Use cloud transcription (Groq) instead.".to_string(),
        ))
    }

    /// Detect voice activity using VAD.
    #[allow(dead_code)]
    pub async fn is_speaking(&self, _pcm_chunk: &[f32]) -> Result<bool, AppError> {
        let _session = self
            .vad_session
            .as_ref()
            .ok_or_else(|| AppError::InfrastructureError {
                provider_id: ProviderId::Audio,
                kind: InfrastructureErrorKind::ApiError,
                detail: "VAD engine not loaded. Check VAD_MODEL_PATH.".to_string(),
                help_link: None,
            })?;

        Err(AppError::NotImplemented(
            "Local VAD ONNX inference runtime is not configured.".to_string(),
        ))
    }
}

#[cfg(not(feature = "neural-audio"))]
impl NeuralAudioEngine {
    #[allow(dead_code)]
    pub async fn new() -> Result<Self, AppError> {
        Ok(Self)
    }

    #[allow(dead_code)]
    pub async fn auto_load(&mut self) -> Result<(), AppError> {
        info!("[NeuralAudio] Legacy Mode active. Neural engine skipped (No AVX/AVX2).");
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn speak_stream(
        &self,
        _text: &str,
        _sender: tokio::sync::broadcast::Sender<Vec<u8>>,
        _cache: Arc<crate::agent::audio_cache::BunkerCache>,
    ) -> Result<(), AppError> {
        Err(AppError::BadRequest(
            "Neural Audio engine is disabled in this build. Use Browser or Cloud fallback."
                .to_string(),
        ))
    }

    #[allow(dead_code)]
    pub async fn speak(&self, _text: &str) -> Result<Vec<u8>, AppError> {
        Err(AppError::BadRequest(
            "Neural Audio engine is disabled in this build.".to_string(),
        ))
    }

    #[allow(dead_code)]
    pub async fn listen(&self, _audio_data: Vec<u8>) -> Result<String, AppError> {
        Err(AppError::BadRequest(
            "Local Whisper engine is disabled in this build. Use Cloud transcription.".to_string(),
        ))
    }

    #[allow(dead_code)]
    pub async fn is_speaking(&self, _pcm_chunk: &[f32]) -> Result<bool, AppError> {
        Ok(false)
    }
}
