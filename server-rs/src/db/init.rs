//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database & Migrations / init
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Database]`, `[Recovery]`

use anyhow::{Context, Result};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

use crate::db::migrations::run_migrations;
use crate::db::seed::seed_default_data;

/// Policy dictating whether the database should seed baseline data upon startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedPolicy {
    Seed,
    SkipSeed,
}

/// Resolves the seed policy from the `SKIP_DB_SEED` environment variable.
///
/// Handles `true`, `1`, `yes`, `on` case-insensitively with trimming.
pub fn seed_policy_from_env() -> SeedPolicy {
    match std::env::var("SKIP_DB_SEED") {
        Ok(v) => match v.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => SeedPolicy::SkipSeed,
            _ => SeedPolicy::Seed,
        },
        Err(_) => SeedPolicy::Seed,
    }
}

/// Parameters owned by the Tadpole OS persistence framework and stripped before SQLite processing.
const OWNED_PARAMS: &[&str] = &["skip_seed"];

/// Parses and cleans a SQLite database URL, resolving the effective `SeedPolicy` and stripping internal params.
///
/// Rejects hazardous driver query parameters (like `immutable=1`, `mode=ro`, `_fk=0`) that silently alter
/// transactional durability or integrity guarantees.
pub fn parse_database_url(raw: &str) -> Result<(String, SeedPolicy)> {
    let (base, query) = raw.split_once('?').map_or((raw, ""), |(b, q)| (b, q));
    let mut kept: Vec<&str> = Vec::new();
    let mut policy = seed_policy_from_env();

    for param in query.split('&').filter(|p| !p.is_empty()) {
        let (k, v) = param.split_once('=').unwrap_or((param, ""));
        let k_lower = k.trim().to_ascii_lowercase();

        if OWNED_PARAMS.iter().any(|o| k_lower == *o) {
            policy = match v.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "" | "yes" | "on" => SeedPolicy::SkipSeed,
                "false" | "0" | "no" | "off" => SeedPolicy::Seed,
                other => {
                    anyhow::bail!("Invalid value for parameter '{}': '{}'", k, other);
                }
            };
            continue;
        }

        // Defend against reserved parameters that silently break SQLite write capabilities or FK safety
        if matches!(k_lower.as_str(), "_fk" | "immutable" | "vfs") {
            anyhow::bail!(
                "Reserved or hazardous connection parameter '{}' is disallowed in DATABASE_URL",
                k
            );
        }

        kept.push(param);
    }

    let clean = if kept.is_empty() {
        base.to_string()
    } else {
        format!("{}?{}", base, kept.join("&"))
    };

    Ok((clean, policy))
}

/// Helper to extract `skip_seed` parameter from a SQLite database URL and return `(clean_url, skip_seed)`.
///
/// Maintained for backward compatibility. Delegated to `parse_database_url`.
pub fn strip_skip_seed_param(database_url: &str) -> (String, bool) {
    match parse_database_url(database_url) {
        Ok((clean, policy)) => (clean, policy == SeedPolicy::SkipSeed),
        Err(_) => {
            // Fallback for resilient parsing on malformed inputs
            (database_url.to_string(), false)
        }
    }
}

/// Single Source of Truth for production SQLite connection options and performance PRAGMAs.
pub fn production_options(clean_url: &str) -> Result<SqliteConnectOptions, sqlx::Error> {
    SqliteConnectOptions::from_str(clean_url).map(|opts| {
        opts.create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .pragma("synchronous", "NORMAL") // Relax strict fsync for WAL speed
            .pragma("cache_size", "-64000") // Use 64MB of memory for the page cache
            .pragma("temp_store", "memory") // Keep temp tables in RAM
            .pragma("mmap_size", "268435456") // Memory-map 256MB for ultra-fast reads
            .pragma("busy_timeout", "10000") // Wait up to 10s if DB is locked
            .pragma("foreign_keys", "ON")
    })
}

/// Initializes the SQLite database pool and executes pending migrations.
///
/// Sets high-performance defaults (WAL mode, busy timeout) and ensures that the
/// backend schema is in sync with the `migrations/` directory.
pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    let (clean_url, seed_policy) = parse_database_url(database_url)
        .with_context(|| format!("parsing DATABASE_URL '{}'", database_url))?;

    let options = production_options(&clean_url)
        .with_context(|| format!("configuring SQLite options for URL '{}'", clean_url))?;

    let pool = SqlitePool::connect_with(options)
        .await
        .with_context(|| format!("connecting to database at '{}'", clean_url))?;

    tracing::info!(
        "🔌 [Database] Connected to database pool (clean_url: {})",
        clean_url
    );

    // Run schema migrations
    run_migrations(&pool)
        .await
        .with_context(|| "executing database schema migrations")?;

    // Sweep orphaned active missions from previous crash or shutdown
    if let Err(e) = crate::agent::mission::sweep_interrupted_missions(&pool).await {
        tracing::warn!("⚠️ [Recovery] Failed to sweep interrupted missions: {}", e);
    }

    // Sweep orphaned pending oversight records from previous crash or shutdown
    let sweep_result = sqlx::query(
        "UPDATE oversight_log SET status = 'rejected', decision = 'interrupted', decided_at = CURRENT_TIMESTAMP, decided_by = 'system_recovery' WHERE status = 'pending'",
    )
    .execute(&pool)
    .await;

    match sweep_result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                tracing::info!(
                    "🧹 [Recovery] Swept {} pending oversight records to rejected/interrupted",
                    res.rows_affected()
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                "⚠️ [Recovery] Failed to sweep orphaned pending oversight entries: {}",
                e
            );
        }
    }

    // Seed default data unless explicitly skipped
    if seed_policy == SeedPolicy::Seed {
        seed_default_data(&pool)
            .await
            .with_context(|| "seeding default baseline data")?;
    } else {
        tracing::info!(
            "ℹ [Database] Skipping default data seeding (policy: {:?})",
            seed_policy
        );
    }

    // Hydrate paired companion devices from SQLite (RED-06)
    if let Err(e) = crate::routes::remote::load_paired_devices(&pool).await {
        tracing::warn!(
            "⚠️ [Database] Failed to hydrate paired remote devices: {}",
            e
        );
    }

    // Hydrate active remote request nonces from SQLite (anti-replay defense across restarts)
    crate::security::remote_protocol::init_nonce_persistence(&pool)
        .await
        .with_context(|| "initializing anti-replay remote request nonce store")?;

    tracing::info!("✅ [Database] Connection pool initialized & migrations verified.");
    Ok(pool)
}

/// Executes a passive SQLite WAL checkpoint to keep log file size small during high write throughput.
#[allow(dead_code)]
pub async fn checkpoint_wal(pool: &SqlitePool) -> Result<()> {
    sqlx::query("PRAGMA wal_checkpoint(PASSIVE);")
        .execute(pool)
        .await
        .with_context(|| "executing PRAGMA wal_checkpoint(PASSIVE)")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_database_url_skip_seed_variations() {
        for truthy in ["true", "TRUE", "1", "yes", "on", "YES", "ON"] {
            let url = format!("sqlite:test.db?skip_seed={}", truthy);
            let (clean, policy) = parse_database_url(&url).unwrap();
            assert_eq!(clean, "sqlite:test.db", "expected param stripped");
            assert_eq!(policy, SeedPolicy::SkipSeed);
        }

        for falsy in ["false", "FALSE", "0", "no", "off", "NO", "OFF"] {
            let url = format!("sqlite:test.db?skip_seed={}", falsy);
            let (clean, policy) = parse_database_url(&url).unwrap();
            assert_eq!(
                clean, "sqlite:test.db",
                "expected param stripped even when false"
            );
            assert_eq!(policy, SeedPolicy::Seed);
        }
    }

    #[test]
    fn test_parse_database_url_reserved_params_rejected() {
        assert!(parse_database_url("sqlite:test.db?_fk=0").is_err());
        assert!(parse_database_url("sqlite:test.db?immutable=1").is_err());
        assert!(parse_database_url("sqlite:test.db?vfs=memdb").is_err());
    }

    #[test]
    fn test_strip_skip_seed_backward_compatibility() {
        let (clean, skip) = strip_skip_seed_param("sqlite:test.db?skip_seed=true&cache=shared");
        assert_eq!(clean, "sqlite:test.db?cache=shared");
        assert!(skip);

        let (clean, skip) = strip_skip_seed_param("sqlite:test.db?cache=shared&skip_seed=1");
        assert_eq!(clean, "sqlite:test.db?cache=shared");
        assert!(skip);

        let (clean, skip) = strip_skip_seed_param("sqlite:test.db?SKIP_SEED=TRUE");
        assert_eq!(clean, "sqlite:test.db");
        assert!(skip);

        let (clean, skip) = strip_skip_seed_param("sqlite:test.db");
        assert_eq!(clean, "sqlite:test.db");
        assert!(!skip);
    }

    #[test]
    fn test_production_options_builder() {
        let opts = production_options("sqlite::memory:").expect("valid options");
        drop(opts);
    }
}
