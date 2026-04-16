//! Rate Limiting — Token buckets and traffic shaping for LLM providers
//!
//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Traffic Shaper**: Enforces multi-dimensional rate limits for LLM
//! providers. Tracks **RPM (Requests Per Minute)** via a sliding-window
//! semaphore and **TPM (Tokens Per Minute)** via atomic counters.
//! Opt-in limits are derived from `ModelConfig`. Features **Self-Healing
//! Window Resets** to recover from long-tail API stalls.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Resource exhaustion due to overlapping 60s windows,
//!   atomic counter overflow, or semaphore deadlocks in high-concurrency
//!   spawn contexts.
//! - **Trace Scope**: `server-rs::agent::rate_limiter`

use std::sync::atomic::{AtomicU32, Ordering};
/// Rate limiter for LLM provider API calls.
///
/// Enforces two independent limits from ModelEntry/ModelConfig:
///   - RPM (requests per minute): a rolling window via a `Semaphore` with timed release.
///   - TPM (tokens per minute): an atomic counter reset every 60 seconds.
///
/// Both limits are opt-in — if rpm/tpm are `None` in the model config, no throttling occurs.
///
/// # Usage
/// ```
/// let limiter = RateLimiter::new(60, 100_000); // 60 RPM, 100k TPM
/// limiter.acquire(512).await; // "I'm about to use ~512 tokens"
/// // make your API call
/// limiter.record_usage(420); // "I actually used 420 tokens"
/// ```
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use tokio::time::{Duration, Instant};

pub struct RateLimiter {
    /// Semaphore permits == max concurrent requests in the current window.
    rpm_semaphore: Option<Arc<Semaphore>>,
    rpm_limit: Option<u32>,

    /// Tokens used in current minute window.
    pub(crate) tokens_used: Arc<AtomicU32>,
    tpm_limit: Option<u32>,

    /// Timestamp of the start of the current 60s window.
    window_start: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(rpm: Option<u32>, tpm: Option<u32>) -> Self {
        let rpm_semaphore = rpm.map(|r| Arc::new(Semaphore::new(r as usize)));
        Self {
            rpm_semaphore,
            rpm_limit: rpm,
            tokens_used: Arc::new(AtomicU32::new(0)),
            tpm_limit: tpm,
            window_start: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Acquires a request slot, blocking if RPM or TPM limits would be exceeded.
    /// `estimated_tokens`: an estimate of the tokens this request will consume.
    pub async fn acquire(&self, estimated_tokens: u32) {
        // ── TPM enforcement ──────────────────────────────────────────────────
        if let Some(tpm) = self.tpm_limit {
            loop {
                let mut start = self.window_start.lock().await;
                let elapsed = start.elapsed();

                if elapsed >= Duration::from_secs(60) {
                    // New window: reset counter and timestamp
                    self.tokens_used.store(0, Ordering::SeqCst);
                    *start = Instant::now();
                }

                let current = self.tokens_used.load(Ordering::SeqCst);
                if current + estimated_tokens <= tpm {
                    break;
                }

                // Calculate how long until the window resets
                let wait = Duration::from_secs(60).saturating_sub(elapsed);
                drop(start); // release lock before sleeping

                tracing::warn!(
                    "⏳ [RateLimiter] TPM limit ({} tokens/min) reached. Waiting {}s for window reset.",
                    tpm,
                    wait.as_secs()
                );
                tokio::time::sleep(wait).await;
            }
        }

        // ── RPM enforcement ──────────────────────────────────────────────────
        if let Some(ref sem) = self.rpm_semaphore {
            let permit = sem.clone().acquire_owned().await.expect("Semaphore closed");

            // Schedule permit return after 60s (sliding window)
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(60)).await;
                drop(permit);
            });
        }
    }

    /// Records the actual tokens consumed after a successful API call.
    pub fn record_usage(&self, actual_tokens: u32) {
        self.tokens_used.fetch_add(actual_tokens, Ordering::SeqCst);
    }

    /// Convenience: returns true if this limiter has any active constraints.
    pub fn is_active(&self) -> bool {
        self.rpm_limit.is_some() || self.tpm_limit.is_some()
    }
}
