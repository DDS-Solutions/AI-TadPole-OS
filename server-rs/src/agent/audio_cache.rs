use anyhow::Result;
use md5;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::PathBuf;
use tracing::info;

/// Service for caching synthesized audio chunks to avoid redundant neural inference.
pub struct BunkerCache {
    pool: SqlitePool,
}

impl BunkerCache {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;

        // Initialize table
        sqlx::query(
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

    /// Retrieve audio data for a given text prompt.
    pub async fn get(&self, text: &str) -> Result<Option<Vec<u8>>> {
        let hash = format!("{:x}", md5::compute(text));
        let row: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT audio_data FROM audio_cache WHERE hash = ?")
                .bind(hash)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.map(|r| r.0))
    }

    /// Store audio data for a given text prompt.
    pub async fn set(&self, text: &str, audio_data: Vec<u8>) -> Result<()> {
        let hash = format!("{:x}", md5::compute(text));
        sqlx::query("INSERT OR REPLACE INTO audio_cache (hash, audio_data) VALUES (?, ?)")
            .bind(hash)
            .bind(audio_data)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Create a mock in-memory instance for testing
    pub fn mock() -> Self {
        let pool =
            SqlitePool::connect_lazy("sqlite::memory:").expect("Failed to create mock memory pool");
        Self { pool }
    }
}
