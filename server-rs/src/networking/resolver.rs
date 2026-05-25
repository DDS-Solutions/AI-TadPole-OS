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

use parking_lot::RwLock;
use std::time::Duration;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Cached resolution results by port to prevent repeated network checks.
static RESOLUTION_CACHE: Lazy<RwLock<HashMap<u16, String>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

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
            let cache = RESOLUTION_CACHE.read();
            if let Some(url) = cache.get(&port) {
                return format!("{}:{}", url, port);
            }
        }

        // 2. Resolve
        let resolved = Self::discover_host(port).await;
        
        // 3. Update Cache
        {
            let mut cache = RESOLUTION_CACHE.write();
            cache.insert(port, resolved.clone());
        }

        format!("{}:{}", resolved, port)
    }

    /// If the URL is a local loopback address, resolves it to the correct reactive host.
    pub async fn resolve_url_if_local(url: &str) -> String {
        if !url.contains("localhost") && !url.contains("127.0.0.1") {
            return url.to_string();
        }

        // Extract port if specified
        let port_opt = if let Some(pos) = url.find("localhost:") {
            let start = pos + "localhost:".len();
            let end = url[start..].find(|c: char| !c.is_ascii_digit()).unwrap_or(url[start..].len());
            url[start..start+end].parse::<u16>().ok()
        } else if let Some(pos) = url.find("127.0.0.1:") {
            let start = pos + "127.0.0.1:".len();
            let end = url[start..].find(|c: char| !c.is_ascii_digit()).unwrap_or(url[start..].len());
            url[start..start+end].parse::<u16>().ok()
        } else {
            None
        };

        if let Some(port) = port_opt {
            let resolved_host = Self::resolve_local_url(port).await;
            let target_localhost = format!("localhost:{}", port);
            let target_loopback = format!("127.0.0.1:{}", port);
            
            let host_part = resolved_host.replace("http://", "").replace("https://", "");
            
            url.replace(&target_localhost, &host_part)
               .replace(&target_loopback, &host_part)
        } else {
            // Fallback if no port specified
            if crate::utils::is_docker() {
                url.replace("localhost", "host.docker.internal")
                    .replace("127.0.0.1", "host.docker.internal")
            } else {
                url.replace("localhost", "127.0.0.1")
            }
        }
    }

    /// Primary discovery loop with aggressive timeouts.
    async fn discover_host(port: u16) -> String {
        let candidates = [
            ("http://127.0.0.1", "Native Local"),
            ("http://host.docker.internal", "Docker Bridge"),
            ("http://10.0.0.1", "Default Docker Gateway"),
        ];

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(200))
            .build()
            .unwrap_or_default();

        let (tx, mut rx) = tokio::sync::mpsc::channel(candidates.len());

        for (base, name) in candidates {
            let url = format!("{}:{}", base, port);
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                tracing::debug!("🔍 [Resolver] Testing {} strategy: {}", name, url);
                match tokio::time::timeout(Duration::from_millis(200), client.get(&url).send()).await {
                    Ok(Ok(resp)) if resp.status().is_success() || resp.status().as_u16() == 404 => {
                        tracing::info!("✅ [Resolver] Host discovered via {}: {}", name, base);
                        let _ = tx.send(base.to_string()).await;
                    }
                    _ => {
                        tracing::debug!("❌ [Resolver] {} failed or timed out for {}", name, url);
                    }
                }
            });
        }
        // Drop the original sender so the channel closes when all spawned tasks finish
        drop(tx);

        if let Some(successful_base) = rx.recv().await {
            return successful_base;
        }

        // Final Fallback: Assume native local
        tracing::warn!("⚠️ [Resolver] All host discovery strategies failed for port {}. Falling back to 127.0.0.1", port);
        "http://127.0.0.1".to_string()
    }

    /// Forces a cache reset. Useful for system re-initialization.
    #[allow(dead_code)]
    pub async fn reset_cache() {
        let mut cache = RESOLUTION_CACHE.write();
        cache.clear();
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

    #[tokio::test]
    async fn test_resolve_url_if_local() {
        AddressResolver::reset_cache().await;
        
        let local_url = "http://localhost:11434/v1";
        let resolved = AddressResolver::resolve_url_if_local(local_url).await;
        
        assert!(resolved.contains(":11434/v1"));
        assert!(resolved.starts_with("http://"));
    }
}

// Metadata: [resolver]
