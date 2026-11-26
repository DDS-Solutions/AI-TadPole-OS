//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / rate_limiter
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[RateLimiter]`
//! - **Witness Tests**: none declared

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
/// Rate limiter for LLM provider API calls.
///
/// Enforces four independent limits from ModelEntry/ModelConfig:
///   - RPM (requests per minute): a rolling window via a `Semaphore` with timed release.
///   - TPM (tokens per minute): an atomic counter reset every 60 seconds.
///   - RPD (requests per day): a 24-hour quota window tracking daily request counts.
///   - TPD (tokens per day): a 24-hour quota window tracking daily token consumption.
///
/// Limits are opt-in — if limits are `None` in the model config, no throttling occurs for that metric.
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

    /// Daily request count in current 24h window.
    pub(crate) requests_today: Arc<AtomicU32>,
    rpd_limit: Option<u32>,

    /// Daily tokens consumed in current 24h window.
    pub(crate) tokens_used_today: Arc<AtomicU64>,
    tpd_limit: Option<u32>,

    /// Timestamp of the start of the current 24h window.
    daily_window_start: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(rpm: Option<u32>, tpm: Option<u32>) -> Self {
        Self::with_daily_limits(rpm, tpm, None, None)
    }

    pub fn with_daily_limits(
        rpm: Option<u32>,
        tpm: Option<u32>,
        rpd: Option<u32>,
        tpd: Option<u32>,
    ) -> Self {
        let rpm_semaphore = rpm.map(|r| Arc::new(Semaphore::new(r as usize)));
        Self {
            rpm_semaphore,
            rpm_limit: rpm,
            tokens_used: Arc::new(AtomicU32::new(0)),
            tpm_limit: tpm,
            window_start: Arc::new(Mutex::new(Instant::now())),
            requests_today: Arc::new(AtomicU32::new(0)),
            rpd_limit: rpd,
            tokens_used_today: Arc::new(AtomicU64::new(0)),
            tpd_limit: tpd,
            daily_window_start: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Acquires a request slot, blocking if RPM, TPM, RPD, or TPD limits would be exceeded.
    /// `estimated_tokens`: an estimate of the tokens this request will consume.
    pub async fn acquire(&self, estimated_tokens: u32) {
        // ── Daily Window Maintenance & Enforcement (TPD & RPD) ───────────────
        if self.tpd_limit.is_some() || self.rpd_limit.is_some() {
            loop {
                let mut day_start = self.daily_window_start.lock().await;
                let elapsed = day_start.elapsed();

                if elapsed >= Duration::from_secs(86_400) {
                    // New daily window: reset counters and timestamp
                    self.requests_today.store(0, Ordering::SeqCst);
                    self.tokens_used_today.store(0, Ordering::SeqCst);
                    *day_start = Instant::now();
                }

                let tpd_ok = match self.tpd_limit {
                    Some(tpd) => {
                        let current = self.tokens_used_today.load(Ordering::SeqCst);
                        current + (estimated_tokens as u64) <= (tpd as u64)
                    }
                    None => true,
                };

                let rpd_ok = match self.rpd_limit {
                    Some(rpd) => {
                        let current = self.requests_today.load(Ordering::SeqCst);
                        current < rpd
                    }
                    None => true,
                };

                if tpd_ok && rpd_ok {
                    drop(day_start);
                    break;
                }

                let wait = Duration::from_secs(86_400).saturating_sub(elapsed);
                drop(day_start);

                if !tpd_ok {
                    if let Some(tpd) = self.tpd_limit {
                        tracing::warn!(
                            "⏳ [RateLimiter] TPD limit ({} tokens/day) reached. Waiting {}s for daily reset.",
                            tpd,
                            wait.as_secs()
                        );
                    }
                } else if !rpd_ok {
                    if let Some(rpd) = self.rpd_limit {
                        tracing::warn!(
                            "⏳ [RateLimiter] RPD limit ({} requests/day) reached. Waiting {}s for daily reset.",
                            rpd,
                            wait.as_secs()
                        );
                    }
                }

                tokio::time::sleep(wait).await;
            }
        }

        // Increment daily request count
        self.requests_today.fetch_add(1, Ordering::SeqCst);

        // ── TPM enforcement ──────────────────────────────────────────────────
        if let Some(tpm) = self.tpm_limit {
            loop {
                let mut start = self.window_start.lock().await;
                let elapsed = start.elapsed();

                if elapsed >= Duration::from_secs(60) {
                    // New minute window: reset counter and timestamp
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
            match sem.clone().acquire_owned().await {
                Ok(permit) => {
                    // Schedule permit return after 60s (sliding window)
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        drop(permit);
                    });
                }
                Err(_) => {
                    tracing::error!("🚨 [RateLimiter] RPM semaphore closed unexpectedly. Proceeding without rate limit.");
                }
            }
        }
    }

    /// Records the actual tokens consumed after a successful API call.
    pub fn record_usage(&self, actual_tokens: u64) {
        self.tokens_used
            .fetch_add(actual_tokens as u32, Ordering::SeqCst);
        self.tokens_used_today
            .fetch_add(actual_tokens, Ordering::SeqCst);
    }

    /// Convenience: returns true if this limiter has any active constraints.
    pub fn is_active(&self) -> bool {
        self.rpm_limit.is_some()
            || self.tpm_limit.is_some()
            || self.rpd_limit.is_some()
            || self.tpd_limit.is_some()
    }

    /// Returns the current number of requests handled in the current 24h window.
    pub fn requests_today(&self) -> u32 {
        self.requests_today.load(Ordering::SeqCst)
    }

    /// Returns the total tokens consumed in the current 24h window.
    pub fn tokens_used_today(&self) -> u64 {
        self.tokens_used_today.load(Ordering::SeqCst)
    }
}
