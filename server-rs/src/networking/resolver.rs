//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / resolver
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Resolver]`

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// TTL for fallback (failed discovery) cache entries.
/// Short enough to recover when a service starts after the resolver,
/// long enough to avoid probe storms during sustained outages.
pub const FALLBACK_TTL: Duration = Duration::from_secs(30);

/// TTL for successful discoveries. Allows eventual recovery if container or service IPs change.
pub const SUCCESS_TTL: Duration = Duration::from_secs(600);

/// Request timeout for individual host probe requests.
pub const PROBE_TIMEOUT: Duration = Duration::from_millis(200);

/// TCP connect timeout for host probe requests.
pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(150);

/// Allowlist of common local inference/AI ports permitted for active outbound discovery probing.
/// Prevents using loopback resolution as an internal network / port scanner (SSRF mitigation).
pub const ALLOWED_PROBE_PORTS: &[u16] = &[
    11434, // Ollama default
    11435, // Ollama secondary
    1234,  // LM Studio
    8000,  // vLLM / LocalAI default
    8080,  // Generic local inference HTTP
    8081,  // SGLang / Alternative local inference
    5000,  // text-generation-webui
    5001,  // KoboldCPP
];

/// Checks if a port is permitted for active loopback probing.
pub fn is_allowed_probe_port(port: u16) -> bool {
    ALLOWED_PROBE_PORTS.contains(&port)
}

/// A cache entry distinguishing discovered hosts from fallback defaults.
#[derive(Clone, Debug)]
struct CacheEntry {
    /// Bare host name or IP address (e.g., "127.0.0.1", "host.docker.internal").
    host: String,
    resolved_at: Instant,
    is_fallback: bool,
}

impl CacheEntry {
    /// Returns `true` if this entry has expired and should be re-probed.
    fn is_expired(&self) -> bool {
        if self.is_fallback {
            self.resolved_at.elapsed() > FALLBACK_TTL
        } else {
            self.resolved_at.elapsed() > SUCCESS_TTL
        }
    }
}

/// Cached resolution results by port to prevent repeated network checks.
static RESOLUTION_CACHE: LazyLock<RwLock<HashMap<u16, CacheEntry>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Process-wide single-flight gate ensuring only one discovery probe runs at a time per port.
static DISCOVERY_GATE: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Reusable hardened HTTP client for host probing.
/// Strict timeouts, no redirects, no proxy, bounded pooling.
static PROBE_CLIENT: LazyLock<Option<reqwest::Client>> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .pool_max_idle_per_host(2)
        .build()
        .inspect_err(|e| {
            tracing::error!(
                "⚠️ [Resolver] Failed to build hardened probe client: {}. Active discovery disabled.",
                e
            );
        })
        .ok()
});

/// Candidate host endpoint descriptor for sequential discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateEndpoint {
    pub host: &'static str,
    pub name: &'static str,
}

pub const CANDIDATE_ENDPOINTS: &[CandidateEndpoint] = &[
    CandidateEndpoint {
        host: "127.0.0.1",
        name: "Native Local",
    },
    CandidateEndpoint {
        host: "host.docker.internal",
        name: "Docker Bridge",
    },
    CandidateEndpoint {
        host: "10.0.0.1",
        name: "Default Docker Gateway",
    },
];

pub struct AddressResolver;

impl AddressResolver {
    /// Resolves the best available base URL for a local provider.
    /// Default port is 11434 (Ollama).
    pub async fn resolve_local_url(port: u16) -> String {
        let host = Self::resolve_local_host("http", port).await;
        format!("http://{}:{}", host, port)
    }

    /// Resolves the bare host for a given scheme and port with single-flight and caching.
    pub async fn resolve_local_host(scheme: &str, port: u16) -> String {
        // Fast path: Check read cache
        {
            let cache = RESOLUTION_CACHE.read();
            if let Some(entry) = cache.get(&port) {
                if !entry.is_expired() {
                    return entry.host.clone();
                }
            }
        }

        // Single-flight lock: coalesce concurrent discovery attempts
        let _guard = DISCOVERY_GATE.lock().await;

        // Re-check cache under single-flight in case a concurrent task already resolved it
        {
            let cache = RESOLUTION_CACHE.read();
            if let Some(entry) = cache.get(&port) {
                if !entry.is_expired() {
                    return entry.host.clone();
                }
            }
        }

        // Perform sequential host discovery
        let (host, is_fallback) = Self::discover_host(scheme, port).await;

        // Write cache with no-downgrade rule
        {
            let mut cache = RESOLUTION_CACHE.write();
            if let Some(existing) = cache.get(&port) {
                // Never overwrite a confirmed successful discovery with a fallback
                if !existing.is_expired() && !existing.is_fallback && is_fallback {
                    return existing.host.clone();
                }
            }
            cache.insert(
                port,
                CacheEntry {
                    host: host.clone(),
                    resolved_at: Instant::now(),
                    is_fallback,
                },
            );
        }

        host
    }

    /// If the URL is a local loopback address, resolves it to the correct reactive host.
    /// Preserves user credentials, paths, query parameters, and fragments.
    pub async fn resolve_url_if_local(url: &str) -> String {
        // Fast path: pre-filter check
        if !Self::looks_local(url) {
            return url.to_string();
        }

        let url_to_parse = if !url.contains("://") {
            format!("http://{}", url)
        } else {
            url.to_string()
        };

        let mut parsed = match url::Url::parse(&url_to_parse) {
            Ok(p) => p,
            Err(_) => {
                tracing::debug!(%url, "⚠️ [Resolver] Could not parse URL, returning as-is");
                return url.to_string();
            }
        };

        let host = match parsed.host_str() {
            Some(h) => h,
            None => return url.to_string(),
        };

        let is_loopback = matches!(
            host.to_ascii_lowercase().as_str(),
            "localhost" | "127.0.0.1" | "::1" | "[::1]" | "0.0.0.0"
        );

        if !is_loopback {
            return url.to_string();
        }

        let scheme = parsed.scheme().to_string();

        if let Some(port) = parsed.port() {
            if !is_allowed_probe_port(port) {
                // Safety: Disallowed port probing -> resolve statically according to container environment
                let default_host = if crate::utils::is_docker() {
                    "host.docker.internal"
                } else {
                    "127.0.0.1"
                };
                let _ = parsed.set_host(Some(default_host));
                return parsed.to_string();
            }

            let resolved_host = Self::resolve_local_host(&scheme, port).await;
            let _ = parsed.set_host(Some(&resolved_host));
            parsed.to_string()
        } else {
            // No port: environment-based substitution
            let new_host = if crate::utils::is_docker() {
                "host.docker.internal"
            } else {
                "127.0.0.1"
            };
            let _ = parsed.set_host(Some(new_host));
            parsed.to_string()
        }
    }

    /// Sequential priority discovery with bounded timeouts.
    /// Returns `(host, is_fallback)` where `host` is the bare host name or IP address.
    async fn discover_host(scheme: &str, port: u16) -> (String, bool) {
        let Some(client) = PROBE_CLIENT.as_ref() else {
            tracing::error!("⚠️ [Resolver] Probe client unavailable. Falling back to 127.0.0.1");
            return ("127.0.0.1".to_string(), true);
        };

        for endpoint in CANDIDATE_ENDPOINTS {
            tracing::debug!(
                "🔍 [Resolver] Testing endpoint: {}://{}:{}",
                scheme,
                endpoint.host,
                port
            );

            if Self::probe_host(client, scheme, endpoint.host, port).await {
                tracing::info!(
                    "✅ [Resolver] Host discovered via {}: {}://{}:{}",
                    endpoint.name,
                    scheme,
                    endpoint.host,
                    port
                );
                return (endpoint.host.to_string(), false);
            }

            tracing::debug!(
                "❌ [Resolver] Endpoint {} ({}) failed or timed out for port {}",
                endpoint.name,
                endpoint.host,
                port
            );
        }

        // Final fallback: assume native local 127.0.0.1
        tracing::warn!(
            "⚠️ [Resolver] All host discovery strategies failed for port {}. \
             Falling back to 127.0.0.1 (TTL: {}s)",
            port,
            FALLBACK_TTL.as_secs()
        );
        ("127.0.0.1".to_string(), true)
    }

    /// Probes a single candidate host by attempting an HTTP GET.
    /// Returns `true` if any valid HTTP response is received (2xx, 3xx, 4xx, 5xx).
    async fn probe_host(client: &reqwest::Client, scheme: &str, host: &str, port: u16) -> bool {
        let url = format!("{}://{}:{}", scheme, host, port);
        match client.get(&url).send().await {
            Ok(resp) => {
                // Drain response body to cleanly return socket to pool
                let _ = resp.bytes().await;
                true
            }
            Err(_) => false,
        }
    }

    /// Returns `true` if the URL string appears to reference a local/loopback address.
    /// Case-insensitive fast pre-filter.
    fn looks_local(url: &str) -> bool {
        let lower = url.to_ascii_lowercase();
        lower.contains("localhost")
            || lower.contains("127.0.0.1")
            || lower.contains("[::1]")
            || lower.contains("0.0.0.0")
            || lower.contains("::1")
    }

    /// Forces a cache reset.
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
            host: "127.0.0.1".into(),
            resolved_at: Instant::now(),
            is_fallback: false,
        };
        assert!(
            !fresh_success.is_expired(),
            "fresh success should not be expired"
        );

        let fresh_fallback = CacheEntry {
            host: "127.0.0.1".into(),
            resolved_at: Instant::now(),
            is_fallback: true,
        };
        assert!(
            !fresh_fallback.is_expired(),
            "fresh fallback should not be expired yet"
        );

        let old_fallback_time = Instant::now()
            .checked_sub(FALLBACK_TTL + Duration::from_secs(1))
            .unwrap_or_else(Instant::now);
        let old_fallback = CacheEntry {
            host: "127.0.0.1".into(),
            resolved_at: old_fallback_time,
            is_fallback: true,
        };
        assert!(
            old_fallback.is_expired(),
            "expired fallback should report expired"
        );

        let old_success_time = Instant::now()
            .checked_sub(SUCCESS_TTL + Duration::from_secs(1))
            .unwrap_or_else(Instant::now);
        let old_success = CacheEntry {
            host: "127.0.0.1".into(),
            resolved_at: old_success_time,
            is_fallback: false,
        };
        assert!(
            old_success.is_expired(),
            "success beyond SUCCESS_TTL should expire to allow network updates"
        );
    }

    #[test]
    fn test_looks_local_case_insensitive() {
        assert!(AddressResolver::looks_local("http://localhost:11434/v1"));
        assert!(AddressResolver::looks_local("http://LOCALHOST:11434/v1"));
        assert!(AddressResolver::looks_local("http://LocalHost:11434/v1"));
        assert!(AddressResolver::looks_local("http://127.0.0.1:8080"));
        assert!(AddressResolver::looks_local("http://[::1]:3000/api"));
        assert!(AddressResolver::looks_local("http://0.0.0.0:11434"));
        assert!(AddressResolver::looks_local("0.0.0.0:11434"));
        assert!(!AddressResolver::looks_local("https://api.openai.com/v1"));
        assert!(!AddressResolver::looks_local(
            "http://host.docker.internal:11434"
        ));
    }

    #[test]
    fn test_candidate_endpoint_order() {
        assert_eq!(CANDIDATE_ENDPOINTS.len(), 3);
        assert_eq!(CANDIDATE_ENDPOINTS[0].host, "127.0.0.1");
        assert_eq!(CANDIDATE_ENDPOINTS[1].host, "host.docker.internal");
        assert_eq!(CANDIDATE_ENDPOINTS[2].host, "10.0.0.1");
    }

    #[tokio::test]
    async fn test_url_credentials_and_query_preservation() {
        let input = "http://user:secret123@localhost:11434/v1/chat?model=llama3&temp=0.7#section-1";
        let resolved = AddressResolver::resolve_url_if_local(input).await;

        assert!(
            resolved.contains("user:secret123@"),
            "User credentials must be preserved: {}",
            resolved
        );
        assert!(
            resolved.contains("/v1/chat"),
            "Path must be preserved: {}",
            resolved
        );
        assert!(
            resolved.contains("model=llama3&temp=0.7"),
            "Query params must be preserved: {}",
            resolved
        );
        assert!(
            resolved.ends_with("#section-1"),
            "Fragment must be preserved: {}",
            resolved
        );
    }

    #[tokio::test]
    async fn test_ssrf_disallowed_port_skips_probing() {
        AddressResolver::reset_cache();
        // Redis port 6379 is not in ALLOWED_PROBE_PORTS
        assert!(!is_allowed_probe_port(6379));

        let input = "http://localhost:6379/data";
        let resolved = AddressResolver::resolve_url_if_local(input).await;

        // Must be rewritten statically without active scanning or probe caching
        assert!(
            resolved.contains(":6379/data"),
            "Disallowed port resolved safely: {}",
            resolved
        );

        // RESOLUTION_CACHE must NOT have cached port 6379
        let cache = RESOLUTION_CACHE.read();
        assert!(
            !cache.contains_key(&6379),
            "Disallowed port must never be cached in discovery cache"
        );
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
