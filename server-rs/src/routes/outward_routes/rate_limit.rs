//! @docs ARCHITECTURE:OutwardGateway
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / outward_routes::rate_limit
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Maximum distinct IPs tracked concurrently in rate limiter memory.
pub(super) const MAX_TRACKED_IPS: usize = 10_000;

/// In-memory IP Fixed-Window Counter Rate Limiter (Default: 60 requests per 60-second window per IP).
#[derive(Debug, Clone)]
pub struct IpRateLimiter {
    max_requests: u32,
    window_duration: Duration,
    pub(super) records: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

impl IpRateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_duration: Duration::from_secs(window_secs),
            records: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if IP address is allowed to proceed. Returns true if request is permitted.
    pub fn check_and_record(&self, ip: &str) -> bool {
        if self.max_requests == 0 {
            return false;
        }

        let Ok(mut guard) = self.records.lock() else {
            return false;
        };
        let now = Instant::now();

        if guard.len() >= MAX_TRACKED_IPS {
            evict_overflow(&mut guard, now, self.window_duration);
        }

        let Some((count, window_start)) = guard.get_mut(ip) else {
            guard.insert(ip.to_string(), (1, now));
            return true;
        };

        if now.duration_since(*window_start) >= self.window_duration {
            *count = 1;
            *window_start = now;
            return true;
        }

        if *count < self.max_requests {
            *count += 1;
            return true;
        }

        false
    }
}

/// Memory DoS Protection: drop expired entries first, then evict the oldest 20% if still at capacity.
fn evict_overflow(guard: &mut HashMap<String, (u32, Instant)>, now: Instant, window: Duration) {
    guard.retain(|_, (_, start)| now.duration_since(*start) < window);
    if guard.len() < MAX_TRACKED_IPS {
        return;
    }

    let mut by_age: Vec<(String, Instant)> =
        guard.iter().map(|(k, (_, s))| (k.clone(), *s)).collect();
    by_age.sort_by_key(|(_, s)| *s);
    let evict_count = (guard.len() / 5).max(1);
    for (key, _) in by_age.into_iter().take(evict_count) {
        guard.remove(&key);
    }
}
