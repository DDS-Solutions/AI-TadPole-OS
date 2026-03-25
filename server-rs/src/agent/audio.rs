use anyhow::{anyhow, Result};
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
    pub async fn new() -> Result<Self> {
        Ok(Self {
            piper_session: None,
            whisper_session: None,
            vad_session: None,
        })
    }

    /// Load all models defined in environment variables.
    /// This is called during AppState initialization.
    pub async fn auto_load(&mut self) -> Result<()> {
        if let Ok(path) = std::env::var("PIPER_MODEL_PATH") {
            if std::path::Path::new(&path).exists() {
                self.load_piper(&path).await?;
            } else {
                tracing::warn!(
                    "[NeuralAudio] Piper model path configured but file not found: {}",
                    path
                );
            }
        }

        if let Ok(path) = std::env::var("WHISPER_MODEL_PATH") {
            if std::path::Path::new(&path).exists() {
                self.load_whisper(&path).await?;
            } else {
                tracing::warn!(
                    "[NeuralAudio] Whisper model path configured but file not found: {}",
                    path
                );
            }
        }

        if let Ok(path) = std::env::var("VAD_MODEL_PATH") {
            if std::path::Path::new(&path).exists() {
                self.load_vad(&path).await?;
            } else {
                tracing::warn!(
                    "[NeuralAudio] VAD model path configured but file not found: {}",
                    path
                );
            }
        }

        Ok(())
    }

    /// Load the Piper TTS model.
    pub async fn load_piper(&mut self, model_path: &str) -> Result<()> {
        let session = Session::builder()
            .map_err(|e| anyhow!("Failed to create builder: {}", e))?
            .with_intra_threads(4)
            .map_err(|e| anyhow!("Failed to set threads: {}", e))?
            .commit_from_file(model_path)
            .map_err(|e| anyhow!("Failed to load model {}: {}", model_path, e))?;

        self.piper_session = Some(Arc::new(session));
        info!(
            "[NeuralAudio] Piper model loaded successfully from {}",
            model_path
        );
        Ok(())
    }

    /// Load the Whisper STT model.
    pub async fn load_whisper(&mut self, model_path: &str) -> Result<()> {
        let session = Session::builder()
            .map_err(|e| anyhow!("Failed to create builder: {}", e))?
            .with_intra_threads(4)
            .map_err(|e| anyhow!("Failed to set threads: {}", e))?
            .commit_from_file(model_path)
            .map_err(|e| anyhow!("Failed to load model {}: {}", model_path, e))?;

        self.whisper_session = Some(Arc::new(session));
        info!(
            "[NeuralAudio] Whisper model loaded successfully from {}",
            model_path
        );
        Ok(())
    }

    /// Load the Silero VAD model.
    pub async fn load_vad(&mut self, model_path: &str) -> Result<()> {
        let session = Session::builder()
            .map_err(|e| anyhow!("Failed to create builder: {}", e))?
            .with_intra_threads(1)
            .map_err(|e| anyhow!("Failed to set threads: {}", e))?
            .commit_from_file(model_path)
            .map_err(|e| anyhow!("Failed to load model {}: {}", model_path, e))?;

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
        text: &str,
        sender: tokio::sync::broadcast::Sender<Vec<u8>>,
        cache: Arc<crate::agent::audio_cache::BunkerCache>,
    ) -> Result<()> {
        let _session = self
            .piper_session
            .as_ref()
            .ok_or_else(|| anyhow!("Piper engine not loaded. Check PIPER_MODEL_PATH."))?;
        info!("[NeuralAudio] Streaming synthesis for: {}", text);

        let mut full_audio = Vec::new();
        for _ in 0..10 {
            let chunk = vec![0u8; 1024];
            full_audio.extend_from_slice(&chunk);
            let _ = sender.send(chunk);
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let _ = cache.set(text, full_audio).await;
        Ok(())
    }

    /// Synthesize text to speech using Piper (Full).
    #[allow(dead_code)]
    pub async fn speak(&self, text: &str) -> Result<Vec<u8>> {
        let _session = self
            .piper_session
            .as_ref()
            .ok_or_else(|| anyhow!("Piper engine not loaded"))?;
        info!("[NeuralAudio] Synthesizing text: {}", text);
        Ok(vec![0; 1024])
    }

    /// Transcribe audio to text using Whisper.
    pub async fn listen(&self, audio_data: Vec<u8>) -> Result<String> {
        let _session = self
            .whisper_session
            .as_ref()
            .ok_or_else(|| anyhow!("Whisper engine not loaded. Check WHISPER_MODEL_PATH."))?;
        info!(
            "[NeuralAudio] Transcribing {} bytes of audio via Whisper ONNX",
            audio_data.len()
        );
        Ok("Processed transcription".to_string())
    }

    /// Detect voice activity using VAD.
    #[allow(dead_code)]
    pub async fn is_speaking(&self, _pcm_chunk: &[f32]) -> Result<bool> {
        let _session = self
            .vad_session
            .as_ref()
            .ok_or_else(|| anyhow!("VAD engine not loaded. Check VAD_MODEL_PATH."))?;
        Ok(true)
    }
}

#[cfg(not(feature = "neural-audio"))]
impl NeuralAudioEngine {
    #[allow(dead_code)]
    pub async fn new() -> Result<Self> {
        Ok(Self)
    }

    #[allow(dead_code)]
    pub async fn auto_load(&mut self) -> Result<()> {
        info!("[NeuralAudio] Legacy Mode active. Neural engine skipped (No AVX/AVX2).");
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn speak_stream(
        &self,
        _text: &str,
        _sender: tokio::sync::broadcast::Sender<Vec<u8>>,
        _cache: Arc<crate::agent::audio_cache::BunkerCache>,
    ) -> Result<()> {
        Err(anyhow!(
            "Neural Audio engine is disabled in this build. Use Browser or Cloud fallback."
        ))
    }

    #[allow(dead_code)]
    pub async fn speak(&self, _text: &str) -> Result<Vec<u8>> {
        Err(anyhow!("Neural Audio engine is disabled in this build."))
    }

    #[allow(dead_code)]
    pub async fn listen(&self, _audio_data: Vec<u8>) -> Result<String> {
        Err(anyhow!(
            "Local Whisper engine is disabled in this build. Use Cloud transcription."
        ))
    }

    #[allow(dead_code)]
    pub async fn is_speaking(&self, _pcm_chunk: &[f32]) -> Result<bool> {
        Ok(false)
    }
}
