//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / swarm_persistence
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_save_and_get_pending_directives`, `test_update_directive_status_completes_record`

use crate::agent::runner::RunContext;
use crate::error::AppError;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct AgentDirective {
    pub id: String,
    pub mission_id: String,
    pub source_agent_id: String,
    pub target_agent_id: String,
    pub instruction: String,
    pub status: String,
    pub result: Option<String>,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct PeerReviewRequest {
    pub id: String,
    pub mission_id: String,
    pub requester_id: String,
    pub reviewer_id: String,
    pub content_to_review: String,
    pub criteria: Option<String>,
    pub status: String,
}

/// Saves a new mission directive to the database.
pub async fn save_directive(
    pool: &SqlitePool,
    ctx: &RunContext,
    target_agent_id: &str,
    instruction: &str,
) -> Result<String, AppError> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO agent_directives (id, mission_id, source_agent_id, target_agent_id, instruction, status) 
         VALUES (?, ?, ?, ?, ?, 'pending')"
    )
    .bind(&id)
    .bind(&ctx.mission_id)
    .bind(&ctx.agent_id)
    .bind(target_agent_id)
    .bind(instruction)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;

    Ok(id)
}

/// Retrieves all pending directives for a specific agent.
pub async fn get_pending_directives(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<AgentDirective>, AppError> {
    let rows = sqlx::query_as::<_, AgentDirective>(
        "SELECT id, mission_id, source_agent_id, target_agent_id, instruction, status, result 
         FROM agent_directives 
         WHERE target_agent_id = ? AND status = 'pending'
         ORDER BY created_at ASC",
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Sqlx)?;

    Ok(rows)
}

/// Updates the status of a directive.
#[allow(dead_code)]
pub async fn update_directive_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    result: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE agent_directives SET status = ?, result = ? WHERE id = ?")
        .bind(status)
        .bind(result)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Sqlx)?;

    Ok(())
}

/// Submits a peer review result.
pub async fn submit_review(
    pool: &SqlitePool,
    id: &str,
    feedback: &str,
    status: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE peer_reviews SET status = ?, feedback = ? WHERE id = ?")
        .bind(status)
        .bind(feedback)
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Sqlx)?;

    Ok(())
}

/// Submits a peer review request.
pub async fn save_review_request(
    pool: &SqlitePool,
    ctx: &RunContext,
    reviewer_id: &str,
    content: &str,
    criteria: Option<&str>,
) -> Result<String, AppError> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO peer_reviews (id, mission_id, requester_id, reviewer_id, content_to_review, criteria, status) 
         VALUES (?, ?, ?, ?, ?, ?, 'requested')"
    )
    .bind(&id)
    .bind(&ctx.mission_id)
    .bind(&ctx.agent_id)
    .bind(reviewer_id)
    .bind(content)
    .bind(criteria)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;

    Ok(id)
}

/// Retrieves all review requests for a specific reviewer.
pub async fn get_pending_reviews(
    pool: &SqlitePool,
    reviewer_id: &str,
) -> Result<Vec<PeerReviewRequest>, AppError> {
    let rows = sqlx::query_as::<_, PeerReviewRequest>(
        "SELECT id, mission_id, requester_id, reviewer_id, content_to_review, criteria, status 
         FROM peer_reviews 
         WHERE reviewer_id = ? AND status = 'requested'
         ORDER BY created_at ASC",
    )
    .bind(reviewer_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Sqlx)?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_directives (
                id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                source_agent_id TEXT NOT NULL,
                target_agent_id TEXT NOT NULL,
                instruction TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                result TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    /// Helper that inserts a directive without requiring a RunContext
    async fn insert_directive(
        pool: &SqlitePool,
        id: &str,
        mission_id: &str,
        source: &str,
        target: &str,
        instruction: &str,
    ) {
        sqlx::query(
            "INSERT INTO agent_directives
             (id, mission_id, source_agent_id, target_agent_id, instruction, status)
             VALUES (?, ?, ?, ?, ?, 'pending')",
        )
        .bind(id)
        .bind(mission_id)
        .bind(source)
        .bind(target)
        .bind(instruction)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_and_get_pending_directives() {
        let pool = setup_db().await;

        insert_directive(
            &pool,
            "id-1",
            "mission-001",
            "agent-src",
            "agent-tgt",
            "First instruction",
        )
        .await;
        insert_directive(
            &pool,
            "id-2",
            "mission-001",
            "agent-src",
            "agent-tgt",
            "Second instruction",
        )
        .await;

        // Should only show pending directives for agent-tgt
        let pending = get_pending_directives(&pool, "agent-tgt").await.unwrap();
        assert_eq!(pending.len(), 2);
        assert!(pending.iter().any(|d| d.id == "id-1"));
        assert!(pending.iter().any(|d| d.id == "id-2"));

        // Another target has no directives
        let other = get_pending_directives(&pool, "agent-other").await.unwrap();
        assert!(other.is_empty());
    }

    #[tokio::test]
    async fn test_update_directive_status_completes_record() {
        let pool = setup_db().await;

        insert_directive(
            &pool,
            "id-x",
            "mission-002",
            "agent-src",
            "agent-tgt",
            "Do task X",
        )
        .await;

        // Verify it appears as pending
        let before = get_pending_directives(&pool, "agent-tgt").await.unwrap();
        assert_eq!(before.len(), 1);

        // Mark it completed
        update_directive_status(&pool, "id-x", "completed", Some("Task X done"))
            .await
            .unwrap();

        // Should no longer appear in pending list
        let after = get_pending_directives(&pool, "agent-tgt").await.unwrap();
        assert!(after.is_empty());
    }
}
