//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Client Pool
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Single-flight client acquisition and generation-checked eviction preventing stale removal.
//! - `[Structural]` Orphan client teardown with shutdown() awaiting on redundancy or eviction.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InfrastructureError`
//! - **Telemetry Targets**: none declared

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use super::client::McpClient;
use crate::error::AppError;

/// A handle to an active MCP client tagged with a monotonic generation ID.
#[derive(Clone)]
pub struct ClientHandle {
    pub client: Arc<Mutex<McpClient>>,
    pub generation: u64,
}

/// Thread-safe client connection pool providing generation-safe eviction and single-flight spawns.
#[derive(Clone, Default)]
pub struct ClientPool {
    clients: Arc<Mutex<HashMap<String, ClientHandle>>>,
    generation_seq: Arc<AtomicU64>,
}

impl ClientPool {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            generation_seq: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Retrieves an active client handle if present in the pool.
    pub async fn get(&self, server_name: &str) -> Option<ClientHandle> {
        let clients = self.clients.lock().await;
        clients.get(server_name).cloned()
    }

    /// Inserts a newly spawned client and returns a generation-tagged handle.
    pub async fn insert(&self, server_name: &str, client: McpClient) -> ClientHandle {
        let gen = self.generation_seq.fetch_add(1, Ordering::SeqCst);
        let handle = ClientHandle {
            client: Arc::new(Mutex::new(client)),
            generation: gen,
        };
        let mut clients = self.clients.lock().await;
        clients.insert(server_name.to_string(), handle.clone());
        handle
    }

    /// Evicts a client from the pool only if its cached generation matches `expected_generation`.
    /// Returns true if the client was evicted, or false if eviction was skipped due to a generation mismatch.
    pub async fn evict(&self, server_name: &str, expected_generation: Option<u64>) -> bool {
        let mut clients = self.clients.lock().await;
        if let Some(existing) = clients.get(server_name) {
            if let Some(expected_gen) = expected_generation {
                if existing.generation != expected_gen {
                    warn!(
                        "⚠️ [MCP-Pool] Skipped stale eviction for '{}' (cached gen {}, expected gen {})",
                        server_name, existing.generation, expected_gen
                    );
                    return false;
                }
            }
            if let Some(removed) = clients.remove(server_name) {
                info!(
                    "[MCP-Pool] Evicted MCP client for '{}' (gen {})",
                    server_name, removed.generation
                );
                // Graceful async shutdown in background
                tokio::spawn(async move {
                    let mut lock = removed.client.lock().await;
                    let _ = lock.shutdown().await;
                });
                return true;
            }
        }
        false
    }

    /// Atomically gets an existing client or executes `spawn_fn` to create one,
    /// ensuring that concurrent stampedes do not leak orphan processes.
    pub async fn get_or_spawn<F, Fut>(
        &self,
        server_name: &str,
        spawn_fn: F,
    ) -> Result<ClientHandle, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<McpClient, AppError>>,
    {
        // 1. Fast path: check if client already exists
        if let Some(handle) = self.get(server_name).await {
            return Ok(handle);
        }

        // 2. Slow path: spawn new client
        let new_client = spawn_fn().await?;

        // 3. Re-acquire lock to insert or reuse concurrently spawned client
        let mut clients = self.clients.lock().await;
        if let Some(existing) = clients.get(server_name) {
            // Another task spawned first: shut down our duplicate client cleanly
            tokio::spawn(async move {
                let mut c = new_client;
                let _ = c.shutdown().await;
            });
            return Ok(existing.clone());
        }

        let gen = self.generation_seq.fetch_add(1, Ordering::SeqCst);
        let handle = ClientHandle {
            client: Arc::new(Mutex::new(new_client)),
            generation: gen,
        };
        clients.insert(server_name.to_string(), handle.clone());
        Ok(handle)
    }

    /// Returns the number of currently active client connections in the pool.
    pub async fn len(&self) -> usize {
        self.clients.lock().await.len()
    }

    /// Checks if the pool is empty.
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_pool_stale_eviction_is_rejected() {
        let pool = ClientPool::new();

        // Simulate client handles with different generation counts
        let gen1 = pool.generation_seq.fetch_add(1, Ordering::SeqCst);
        let gen2 = pool.generation_seq.fetch_add(1, Ordering::SeqCst);

        // Put a fake client at generation 2
        {
            let mut clients = pool.clients.lock().await;
            // Note: McpClient cannot be easily constructed without connection, but we can verify eviction generation logic
            // via insert and evict with simulated generations
            clients.insert(
                "test-server".to_string(),
                ClientHandle {
                    client: Arc::new(Mutex::new(
                        crate::agent::mcp::McpClient::connect_http(
                            "test-server",
                            "http://127.0.0.1:9",
                            None,
                            None,
                        )
                        .unwrap(),
                    )),
                    generation: gen2,
                },
            );
        }

        // Try to evict with stale generation 1 -> must fail and preserve client
        let evicted_stale = pool.evict("test-server", Some(gen1)).await;
        assert!(!evicted_stale);
        assert!(pool.get("test-server").await.is_some());

        // Evict with matching generation 2 -> must succeed
        let evicted_fresh = pool.evict("test-server", Some(gen2)).await;
        assert!(evicted_fresh);
        assert!(pool.get("test-server").await.is_none());
    }
}
