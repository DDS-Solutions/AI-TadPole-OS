//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / resolver
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Advisory: UNVERIFIED]` Correctness: fallback (failed) resolutions use a short TTL (30s) to allow recovery
//!   when services start after the resolver. Successful discoveries are cached permanently.
//! - `[Advisory: UNVERIFIED]` Priority: discovery probes run in deterministic order: Local → DockerBridge → Gateway.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Resolver]`
//! - **Witness Tests**: `test_resolve_url_if_local`

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

/// TTL for fallback (failed discovery) cache entries.
/// Short enough to recover when a service starts after the resolver,
/// long enough to avoid probe storms during sustained outages.
const FALLBACK_TTL: Duration = Duration::from_secs(30);

/// Timeout for individual host probe requests.
const PROBE_TIMEOUT: Duration = Duration::from_millis(200);

/// A cache entry distinguishing discovered hosts from fallback defaults.
#[derive(Clone, Debug)]
struct CacheEntry {
    host: String,
    resolved_at: Instant,
    is_fallback: bool,
}

impl CacheEntry {
    /// Returns `true` if this entry has expired and should be re-probed.
    /// Successful discoveries never expire; fallback entries expire after `FALLBACK_TTL`.
    fn is_expired(&self) -> bool {
        self.is_fallback && self.resolved_at.elapsed() > FALLBACK_TTL
    }
}

/// Cached resolution results by port to prevent repeated network checks.
/// Fallback entries expire after [`FALLBACK_TTL`]; successful entries are permanent.
static RESOLUTION_CACHE: LazyLock<RwLock<HashMap<u16, CacheEntry>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Reusable HTTP client for host probing. Built once, shared across all discovery calls.
/// Uses [`PROBE_TIMEOUT`] as the request-level timeout.
static PROBE_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()
        .map_err(|e| {
            tracing::error!(
                "⚠️ [Resolver] Failed to build probe client: {}. Using default (no timeout).",
                e
            );
            e
        })
        .unwrap_or_default()
});

/// Deterministic priority order for host discovery.
/// Probed sequentially: first successful responder wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// Attempt 127.0.0.1 (Local native) — highest priority
    Local,
    /// Attempt host.docker.internal (Docker bridge)
    DockerBridge,
    /// Attempt common Docker default gateway (10.0.0.1)
    ///
    /// NOTE: Only valid for Docker's *default* Linux bridge network.
    /// Custom/compose networks use different subnets (172.18.x, 192.168.x).
    /// Ideally this would be derived from the container's routing table.
    Gateway,
}

impl ResolutionStrategy {
    /// Returns all strategies in priority order.
    fn all() -> &'static [(ResolutionStrategy, &'static str, &'static str)] {
        &[
            (
                ResolutionStrategy::Local,
                "http://127.0.0.1",
                "Native Local",
            ),
            (
                ResolutionStrategy::DockerBridge,
                "http://host.docker.internal",
                "Docker Bridge",
            ),
            (
                ResolutionStrategy::Gateway,
                "http://10.0.0.1",
                "Default Docker Gateway",
            ),
        ]
    }
}

pub struct AddressResolver;

impl AddressResolver {
    /// Resolves the best available base URL for a local provider.
    /// Default port is 11434 (Ollama).
    ///
    /// Results are cached: successful discoveries permanently, fallback results
    /// for [`FALLBACK_TTL`] (allowing recovery when services start late).
    pub async fn resolve_local_url(port: u16) -> String {
        // 1. Check cache (valid, non-expired entries only)
        {
            let cache = RESOLUTION_CACHE.read();
            if let Some(entry) = cache.get(&port) {
                if !entry.is_expired() {
                    return format!("{}:{}", entry.host, port);
                }
                tracing::debug!(
                    "🔄 [Resolver] Cached fallback for port {} expired after {}s, re-probing",
                    port,
                    entry.resolved_at.elapsed().as_secs()
                );
            }
        }

        // 2. Discover (sequential priority order)
        let (host, is_fallback) = Self::discover_host(port).await;

        // 3. Update cache
        {
            let mut cache = RESOLUTION_CACHE.write();
            cache.insert(
                port,
                CacheEntry {
                    host: host.clone(),
                    resolved_at: Instant::now(),
                    is_fallback,
                },
            );
        }

        format!("{}:{}", host, port)
    }

    /// If the URL is a local loopback address, resolves it to the correct reactive host.
    /// Supports `localhost`, `127.0.0.1`, and `[::1]` detection.
    /// Uses proper URL parsing to avoid rewriting host strings in paths or query parameters.
    pub async fn resolve_url_if_local(url: &str) -> String {
        // Fast path: if it doesn't look local at all, return as-is
        if !Self::looks_local(url) {
            return url.to_string();
        }

        // Parse with the `url` crate for safe host-only rewriting
        let parsed = match url::Url::parse(url) {
            Ok(p) => p,
            Err(_) => {
                tracing::debug!(
                    "⚠️ [Resolver] Could not parse URL '{}', returning as-is",
                    url
                );
                return url.to_string();
            }
        };

        let host = match parsed.host_str() {
            Some(h) => h,
            None => return url.to_string(),
        };

        // Only rewrite if the host component is actually a loopback address
        if !matches!(host, "localhost" | "127.0.0.1" | "::1") {
            return url.to_string();
        }

        if let Some(port) = parsed.port() {
            // Port specified: resolve via discovery and rewrite host+port
            let resolved_full = Self::resolve_local_url(port).await;
            // resolved_full is "http://host:port" — extract just "host:port"
            let host_port = resolved_full
                .strip_prefix("http://")
                .or_else(|| resolved_full.strip_prefix("https://"))
                .unwrap_or(&resolved_full);

            // Reconstruct: scheme + resolved host:port + path + query + fragment
            let mut result = format!("{}://{}", parsed.scheme(), host_port);
            result.push_str(parsed.path());
            if let Some(query) = parsed.query() {
                result.push('?');
                result.push_str(query);
            }
            if let Some(fragment) = parsed.fragment() {
                result.push('#');
                result.push_str(fragment);
            }
            result
        } else {
            // No port: simple host substitution based on environment
            let mut rewritten = parsed.clone();
            let new_host = if crate::utils::is_docker() {
                "host.docker.internal"
            } else {
                "127.0.0.1"
            };
            let _ = rewritten.set_host(Some(new_host));
            rewritten.to_string()
        }
    }

    /// Sequential priority discovery with bounded timeouts.
    /// Probes candidates in [`ResolutionStrategy`] priority order (Local → DockerBridge → Gateway).
    /// Returns `(host_base, is_fallback)` — `is_fallback` is `true` when all probes failed.
    ///
    /// Worst-case latency: `N_candidates × PROBE_TIMEOUT` (~600ms for 3 candidates).
    async fn discover_host(port: u16) -> (String, bool) {
        let client = &*PROBE_CLIENT;

        for (strategy, base, name) in ResolutionStrategy::all() {
            tracing::debug!(
                "🔍 [Resolver] Testing {:?} strategy: {}:{}",
                strategy,
                base,
                port
            );
            if Self::probe_host(client, base, port).await {
                tracing::info!("✅ [Resolver] Host discovered via {}: {}", name, base);
                return (base.to_string(), false);
            }
            tracing::debug!(
                "❌ [Resolver] {:?} ({}) failed or timed out for port {}",
                strategy,
                name,
                port
            );
        }

        // Final fallback: assume native local. Cached with short TTL so we re-probe soon.
        tracing::warn!(
            "⚠️ [Resolver] All host discovery strategies failed for port {}. \
             Falling back to 127.0.0.1 (TTL: {}s)",
            port,
            FALLBACK_TTL.as_secs()
        );
        ("http://127.0.0.1".to_string(), true)
    }

    /// Probes a single candidate host by attempting an HTTP GET.
    /// Returns `true` if any HTTP response is received (success or 404).
    ///
    /// NOTE: A 404 counts as "host found" — we're detecting that *some* HTTP server
    /// is listening on that port, not that a specific service is running. This means
    /// an unrelated service on the same port will satisfy the probe. This is an
    /// acceptable tradeoff for a local service discovery mechanism.
    async fn probe_host(client: &reqwest::Client, base: &str, port: u16) -> bool {
        let url = format!("{}:{}", base, port);
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 404 => true,
            _ => false,
        }
    }

    /// Returns `true` if the URL string appears to reference a local/loopback address.
    /// Used as a fast pre-filter before full URL parsing.
    fn looks_local(url: &str) -> bool {
        url.contains("localhost") || url.contains("127.0.0.1") || url.contains("[::1]")
    }

    /// Forces a cache reset. Useful for system re-initialization.
    #[allow(dead_code)]
    pub fn reset_cache() {
        let mut cache = RESOLUTION_CACHE.write();
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Pure unit tests (no network) ---

    #[test]
    fn test_cache_entry_expiry() {
        let fresh_success = CacheEntry {
            host: "http://127.0.0.1".into(),
            resolved_at: Instant::now(),
            is_fallback: false,
        };
        assert!(
            !fresh_success.is_expired(),
            "successful entries should never expire"
        );

        let fresh_fallback = CacheEntry {
            host: "http://127.0.0.1".into(),
            resolved_at: Instant::now(),
            is_fallback: true,
        };
        assert!(
            !fresh_fallback.is_expired(),
            "fresh fallback should not be expired yet"
        );

        let old_fallback = CacheEntry {
            host: "http://127.0.0.1".into(),
            resolved_at: Instant::now() - FALLBACK_TTL - Duration::from_secs(1),
            is_fallback: true,
        };
        assert!(
            old_fallback.is_expired(),
            "expired fallback should report expired"
        );

        let old_success = CacheEntry {
            host: "http://127.0.0.1".into(),
            resolved_at: Instant::now() - Duration::from_secs(3600),
            is_fallback: false,
        };
        assert!(
            !old_success.is_expired(),
            "old successful entries should never expire"
        );
    }

    #[test]
    fn test_looks_local() {
        assert!(AddressResolver::looks_local("http://localhost:11434/v1"));
        assert!(AddressResolver::looks_local("http://127.0.0.1:8080"));
        assert!(AddressResolver::looks_local("http://[::1]:3000/api"));
        assert!(!AddressResolver::looks_local("https://api.openai.com/v1"));
        assert!(!AddressResolver::looks_local(
            "http://host.docker.internal:11434"
        ));
    }

    #[test]
    fn test_resolution_strategy_ordering() {
        let strategies = ResolutionStrategy::all();
        assert_eq!(strategies.len(), 3);
        assert_eq!(strategies[0].0, ResolutionStrategy::Local);
        assert_eq!(strategies[1].0, ResolutionStrategy::DockerBridge);
        assert_eq!(strategies[2].0, ResolutionStrategy::Gateway);
    }

    // --- Integration tests (hit network, use shared state) ---

    #[tokio::test]
    async fn test_resolver_fallback_to_local() {
        // Verifies that discover_host returns a well-formed URL even when
        // no service is listening (falls back to 127.0.0.1).
        // NOTE: This test hits the real network and shares global RESOLUTION_CACHE.
        AddressResolver::reset_cache();
        let url = AddressResolver::resolve_local_url(11434).await;
        assert!(url.contains(":11434"), "URL should contain port: {}", url);
        assert!(
            url.starts_with("http://"),
            "URL should start with http://: {}",
            url
        );
    }

    #[tokio::test]
    async fn test_cache_persistence() {
        AddressResolver::reset_cache();

        // First resolution
        let url1 = AddressResolver::resolve_local_url(11434).await;

        // Second resolution (should be cached — either as success or unexpired fallback)
        let url2 = AddressResolver::resolve_local_url(11434).await;

        assert_eq!(
            url1, url2,
            "Cached result should be stable within TTL window"
        );
    }

    #[tokio::test]
    async fn test_resolve_url_if_local() {
        AddressResolver::reset_cache();

        let test_cases = vec![
            // (input, should_contain, description)
            (
                "http://localhost:11434/v1",
                ":11434/v1",
                "localhost with port and path",
            ),
            (
                "http://127.0.0.1:8080/api",
                ":8080/api",
                "loopback with port and path",
            ),
            (
                "https://api.openai.com/v1",
                "api.openai.com",
                "non-local URL unchanged",
            ),
        ];

        for (input, expected_substr, desc) in test_cases {
            let resolved = AddressResolver::resolve_url_if_local(input).await;
            assert!(
                resolved.contains(expected_substr),
                "{}: expected '{}' to contain '{}', got '{}'",
                desc,
                input,
                expected_substr,
                resolved
            );
        }
    }

    #[tokio::test]
    async fn test_resolve_url_preserves_non_local() {
        let non_local = "https://api.openai.com/v1/chat/completions?key=abc#section";
        let result = AddressResolver::resolve_url_if_local(non_local).await;
        assert_eq!(
            result, non_local,
            "Non-local URLs must pass through unchanged"
        );
    }
}
