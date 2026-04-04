//! Rate Limiter Verification — Throughput and windowing tests
//!
//! @docs ARCHITECTURE:Agent

#[cfg(test)]
mod tests {
    use crate::agent::rate_limiter::RateLimiter;

    #[tokio::test]
    async fn rate_limiter_inactive_without_limits() {
        let limiter = RateLimiter::new(None, None);
        assert!(!limiter.is_active());
    }

    #[tokio::test]
    async fn rate_limiter_active_with_rpm() {
        let limiter = RateLimiter::new(Some(10), None);
        assert!(limiter.is_active());
    }

    #[tokio::test]
    async fn rate_limiter_active_with_tpm() {
        let limiter = RateLimiter::new(None, Some(100_000));
        assert!(limiter.is_active());
    }

    #[tokio::test]
    async fn rate_limiter_active_with_both() {
        let limiter = RateLimiter::new(Some(60), Some(100_000));
        assert!(limiter.is_active());
    }

    #[tokio::test]
    async fn rate_limiter_acquires_under_tpm_limit() {
        let limiter = RateLimiter::new(None, Some(10_000));
        // Should not block: 512 < 10_000
        limiter.acquire(512).await;
        // No panic = pass
    }

    #[tokio::test]
    async fn rate_limiter_records_usage() {
        let limiter = RateLimiter::new(None, Some(10_000));
        limiter.record_usage(500);
        limiter.record_usage(300);
        let total = limiter
            .tokens_used
            .load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(total, 800);
    }

    #[tokio::test]
    async fn rate_limiter_acquires_under_rpm_limit() {
        let limiter = RateLimiter::new(Some(100), None);
        // Should not block: first request of 100 available permits
        limiter.acquire(0).await;
        // No panic = pass
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn rate_limiter_tpm_blocks_when_exceeded() {
        let limiter = RateLimiter::new(None, Some(100));

        // Use up all available token budget
        limiter.record_usage(100);

        // This should block because we've exhausted our TPM budget.
        // We use a timeout to prevent the test from hanging forever.
        let result =
            tokio::time::timeout(std::time::Duration::from_millis(200), limiter.acquire(50)).await;

        // Should timeout because it's blocked waiting for window reset
        assert!(
            result.is_err(),
            "Expected timeout because TPM limit was exceeded"
        );
    }
}
