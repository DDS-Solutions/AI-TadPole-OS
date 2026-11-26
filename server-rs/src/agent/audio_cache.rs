//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / audio_cache
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::PathBuf;
use tracing::info;

fn hash_cache_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Service for caching synthesized audio chunks to avoid redundant neural inference.
pub struct BunkerCache {
    pool: SqlitePool,
}

impl BunkerCache {
    pub async fn new(db_path: PathBuf) -> Result<Self, AppError> {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;

        // Initialize table
        sqlx::query::<sqlx::Sqlite>(
            "CREATE TABLE IF NOT EXISTS audio_cache (
                hash TEXT PRIMARY KEY,
                audio_data BLOB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await?;

        info!("[BunkerCache] Audio cache initialized.");
        Ok(Self { pool })
    }

    /// Build a collision-resistant cache key scoped by engine, voice, and text.
    #[allow(dead_code)]
    pub fn scoped_key(engine: &str, voice: &str, text: &str) -> String {
        format!("{engine}:{voice}:{text}")
    }

    /// Retrieve audio data for a given text prompt or scoped cache key.
    #[allow(dead_code)]
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, AppError> {
        let hash = hash_cache_key(key);
        let row: Option<(Vec<u8>,)> = sqlx::query_as::<sqlx::Sqlite, (Vec<u8>,)>(
            "SELECT audio_data FROM audio_cache WHERE hash = ?",
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0))
    }

    /// Store audio data for a given text prompt or scoped cache key.
    #[allow(dead_code)]
    pub async fn set(&self, key: &str, audio_data: Vec<u8>) -> Result<(), AppError> {
        let hash = hash_cache_key(key);
        sqlx::query::<sqlx::Sqlite>(
            "INSERT OR REPLACE INTO audio_cache (hash, audio_data) VALUES (?, ?)",
        )
        .bind(hash)
        .bind(audio_data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a mock in-memory instance for testing
    #[allow(dead_code)]
    pub fn mock() -> Self {
        let pool =
            SqlitePool::connect_lazy("sqlite::memory:").expect("Failed to create mock memory pool");
        Self { pool }
    }

    /// Create an uninitialized no-op instance for emergency fallback
    pub async fn new_noop() -> Self {
        let pool = SqlitePool::connect_lazy("sqlite::memory:")
            .expect("Failed to initialize memory fallback pool");
        Self { pool }
    }
}
