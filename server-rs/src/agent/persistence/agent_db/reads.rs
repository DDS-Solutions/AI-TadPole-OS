//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! - Keep agent read queries and row decoding on the same selected column contract.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Persistence / agent reads
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::Sqlx`
//! - **Telemetry Targets**: none declared

use super::{
    row::{SELECT_AGENTS_QUERY, SELECT_AGENT_BY_ID_QUERY},
    FlatAgentRow,
};
use crate::agent::types::EngineAgent;
use crate::error::AppError;
use sqlx::SqlitePool;

/// Loads all agents from the SQLite database.
pub async fn load_agents_db(pool: &SqlitePool) -> Result<Vec<EngineAgent>, AppError> {
    let rows = sqlx::query_as::<_, FlatAgentRow>(SELECT_AGENTS_QUERY)
        .fetch_all(pool)
        .await
        .map_err(AppError::Sqlx)?;

    Ok(rows.into_iter().map(|r| r.to_engine_agent()).collect())
}

/// Loads a single agent by ID from the SQLite database.
pub async fn load_agent_by_id(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<Option<EngineAgent>, AppError> {
    let row = sqlx::query_as::<_, FlatAgentRow>(SELECT_AGENT_BY_ID_QUERY)
        .bind(agent_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Sqlx)?;

    Ok(row.map(|r| r.to_engine_agent()))
}

/// Alias for [`load_agent_by_id`].
#[inline]
pub async fn load_agent_by_id_db(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<Option<EngineAgent>, AppError> {
    load_agent_by_id(pool, agent_id).await
}
