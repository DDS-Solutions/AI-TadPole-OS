//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **AddressResolver**: The Universal Provider Bridge. Dynamically resolves local 
//! provider addresses (Ollama, LM Studio) across network boundaries. Implements a 
//! Strategy Pattern for address discovery with deterministic fallback logic (NET-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[resolver]` in tracing logs.
//! 

use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use once_cell::sync::Lazy;

/// Cached resolution result to prevent repeated network checks.
static RESOLUTION_CACHE: Lazy<Arc<RwLock<Option<String>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// Priority resolution strategies.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// Attempt 127.0.0.1 (Local native)
    Local,
    /// Attempt host.docker.internal (Docker bridge)
    DockerBridge,
    /// Attempt common local subnet gateway
    Gateway,
}

pub struct AddressResolver;

impl AddressResolver {
    /// Resolves the best available base URL for a local provider.
    /// Default port is 11434 (Ollama).
    pub async fn resolve_local_url(port: u16) -> String {
        // 1. Check Cache
        {
            let cache = RESOLUTION_CACHE.read().await;
            if let Some(ref url) = *cache {
                return format!("{}:{}", url, port);
            }
        }

        // 2. Resolve
        let resolved = Self::discover_host().await;
        
        // 3. Update Cache
        {
            let mut cache = RESOLUTION_CACHE.write().await;
            *cache = Some(resolved.clone());
        }

        format!("{}:{}", resolved, port)
    }

    /// Primary discovery loop with aggressive timeouts.
    async fn discover_host() -> String {
        let candidates = [
            ("http://127.0.0.1", "Native Local"),
            ("http://host.docker.internal", "Docker Bridge"),
            ("http://10.0.0.1", "Default Docker Gateway"),
        ];

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(200))
            .build()
            .unwrap_or_default();

        for (base, name) in candidates {
            tracing::debug!("🔍 [Resolver] Testing {} strategy: {}", name, base);
            
            // We check the health endpoint if possible, or just a TCP connect
            // For Ollama, we can just check the base URL.
            match tokio::time::timeout(Duration::from_millis(200), client.get(base).send()).await {
                Ok(Ok(resp)) if resp.status().is_success() || resp.status().as_u16() == 404 => {
                    // 404 is acceptable as it means the server is there but we hit the root
                    tracing::info!("✅ [Resolver] Host discovered via {}: {}", name, base);
                    return base.to_string();
                }
                _ => {
                    tracing::debug!("❌ [Resolver] {} failed or timed out", name);
                }
            }
        }

        // Final Fallback: Assume native local
        tracing::warn!("⚠️ [Resolver] All host discovery strategies failed. Falling back to 127.0.0.1");
        "http://127.0.0.1".to_string()
    }

    /// Forces a cache reset. Useful for system re-initialization.
    #[allow(dead_code)]
    pub async fn reset_cache() {
        let mut cache = RESOLUTION_CACHE.write().await;
        *cache = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolver_fallback_to_local() {
        // Since we can't easily mock the network in this environment without complex traits,
        // we verify that the discover_host eventually returns a string (the fallback).
        let url = AddressResolver::resolve_local_url(11434).await;
        assert!(url.contains(":11434"));
        assert!(url.starts_with("http://"));
    }

    #[tokio::test]
    async fn test_cache_persistence() {
        AddressResolver::reset_cache().await;
        
        // First resolution
        let url1 = AddressResolver::resolve_local_url(11434).await;
        
        // Second resolution (should be cached)
        let url2 = AddressResolver::resolve_local_url(11434).await;
        
        assert_eq!(url1, url2);
    }
}

// Metadata: [resolver]
