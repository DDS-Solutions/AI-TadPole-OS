//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / swarm_persistence
//!
//! ### AI Assist Note
//! - Keep directive persistence fields aligned with the receiving agent context.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::agent::runner::RunContext;
use crate::error::AppError;
use sqlx::SqlitePool;
use uuid::Uuid;

/// An explicit ID survives paraphrasing; otherwise identical arguments deduplicate.
pub(crate) fn assignment_key(
    ctx: &RunContext,
    fc: &crate::agent::types::ToolCall,
) -> Result<String, crate::agent::runner::tools::error::ToolExecutionError> {
    use crate::agent::runner::tools::error::ToolExecutionError;
    use sha2::{Digest, Sha256};
    let mut args = fc.args.clone();
    if let Some(id) = fc.args.get("assignment_id") {
        let id = id
            .as_str()
            .filter(|id| {
                !id.trim().is_empty() && id.len() <= 128 && !id.chars().any(char::is_control)
            })
            .ok_or_else(|| {
                ToolExecutionError::Validation(
                    "assignment_id must be a non-empty string of at most 128 bytes".into(),
                )
            })?;
        let object = args.as_object_mut().ok_or_else(|| {
            ToolExecutionError::Validation("Delegation arguments must be an object".into())
        })?;
        object.remove("message");
        object.remove("directive");
        object.insert("assignment_id".into(), serde_json::json!(id.trim()));
    }
    let scope = serde_json::json!({
        "mission": ctx.mission_state.mission_id, "parent": ctx.identity.agent_id,
        "tool": fc.name, "args": args, "allowed_files": ctx.allowed_files,
        "safe_mode": ctx.safe_mode, "workspace": ctx.env.workspace_root,
        "model": ctx.model_config.model_id, "plan": ctx.conductor_plan,
        "role": ctx.identity.role, "authority": ctx.identity.authority_level,
        "skills": ctx.skills, "mcp_tools": ctx.mcp_tools,
    });
    Ok(hex::encode(Sha256::digest(scope.to_string().as_bytes())))
}

/// Claim before dispatch; interrupted work requires review instead of automatic replay.
pub(crate) async fn execute_assignment_once<F, Fut>(
    pool: &SqlitePool,
    ctx: &RunContext,
    fc: &crate::agent::types::ToolCall,
    work: F,
) -> Result<String, crate::agent::runner::tools::error::ToolExecutionError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<
        Output = Result<String, crate::agent::runner::tools::error::ToolExecutionError>,
    >,
{
    use crate::agent::runner::tools::error::ToolExecutionError;
    let key = assignment_key(ctx, fc)?;
    let claimed = sqlx::query("INSERT INTO delegation_assignments (assignment_key, mission_id, source_agent_id, status) VALUES (?, ?, ?, 'running') ON CONFLICT(assignment_key) DO NOTHING")
        .bind(&key).bind(&ctx.mission_state.mission_id).bind(&ctx.identity.agent_id)
        .execute(pool).await.map_err(AppError::Sqlx)?.rows_affected() == 1;
    if !claimed {
        let (status, result): (String, Option<String>) = sqlx::query_as(
            "SELECT status, result FROM delegation_assignments WHERE assignment_key = ?",
        )
        .bind(&key)
        .fetch_one(pool)
        .await
        .map_err(AppError::Sqlx)?;
        return if status == "completed" {
            Ok(result.unwrap_or_default())
        } else {
            Err(ToolExecutionError::Validation(format!("ASSIGNMENT_REVIEW_REQUIRED: This assignment is {status}. Inspect the existing execution before deliberately issuing new work with a different assignment_id.")))
        };
    }
    let result = work().await;
    let status = if result.is_ok() {
        "completed"
    } else {
        "failed"
    };
    sqlx::query("UPDATE delegation_assignments SET status = ?, result = ?, updated_at = CURRENT_TIMESTAMP WHERE assignment_key = ?")
        .bind(status).bind(result.as_ref().ok()).bind(&key).execute(pool).await.map_err(AppError::Sqlx)?;
    result
}

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
    pub reasoning_trace: Option<String>,
    pub artifacts: Option<String>,
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
    .bind(&ctx.mission_state.mission_id)
    .bind(&ctx.identity.agent_id)
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
        "SELECT id, mission_id, source_agent_id, target_agent_id, instruction, status, result, reasoning_trace, artifacts
         FROM agent_directives 
         WHERE target_agent_id = ? AND status IN ('pending', 'acknowledged')
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

/// Completes all pending directives targeting an agent upon mission/run finalization.
/// Returns the number of directives updated.
pub async fn complete_pending_directives_for_agent(
    pool: &SqlitePool,
    agent_id: &str,
    result_summary: Option<&str>,
) -> Result<u64, AppError> {
    let res = sqlx::query(
        "UPDATE agent_directives
         SET status = 'completed',
             result = CASE
                 WHEN result IS NOT NULL AND length(result) > 0 THEN result
                 ELSE ?
             END
         WHERE target_agent_id = ? AND status IN ('pending', 'acknowledged')",
    )
    .bind(result_summary)
    .bind(agent_id)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;

    Ok(res.rows_affected())
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
    .bind(&ctx.mission_state.mission_id)
    .bind(&ctx.identity.agent_id)
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

    #[tokio::test]
    async fn assignment_results_follow_mission_retention() {
        let state = crate::state::AppState::new_minimal_mock().await;
        let mut agent = crate::agent::types::EngineAgent::default();
        agent.identity.id = "assignment-retention".into();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();
        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            &agent.identity.id,
            "Retention",
            1.0,
        )
        .await
        .unwrap();
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = agent.identity.id;
        ctx.mission_state.mission_id = mission.id.clone();
        let fc = crate::agent::types::ToolCall {
            name: "spawn_subagent".into(),
            args: serde_json::json!({"message":"private task"}),
        };
        execute_assignment_once(&state.resources.pool, &ctx, &fc, || async {
            Ok("private result".into())
        })
        .await
        .unwrap();
        sqlx::query("DELETE FROM mission_history WHERE id = ?")
            .bind(&mission.id)
            .execute(&state.resources.pool)
            .await
            .unwrap();
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM delegation_assignments WHERE mission_id = ?")
                .bind(&mission.id)
                .fetch_one(&state.resources.pool)
                .await
                .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn assignment_replay_returns_original_result_and_new_work_remains_possible() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        let state = crate::state::AppState::new_minimal_mock().await;
        let ctx = RunContext::default();
        let mut fc = crate::agent::types::ToolCall {
            name: "spawn_subagent".into(),
            args: serde_json::json!({"agent_id":"Alpha", "assignment_id":"audit-1", "message":"Inspect execution"}),
        };
        let calls = Arc::new(AtomicUsize::new(0));
        for message in ["Inspect execution", "Please inspect the same execution"] {
            fc.args["message"] = serde_json::json!(message);
            let result = execute_assignment_once(&state.resources.pool, &ctx, &fc, || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok("Original attributed result".to_string())
            })
            .await
            .unwrap();
            assert_eq!(result, "Original attributed result");
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        fc.args["assignment_id"] = serde_json::json!("audit-2");
        execute_assignment_once(&state.resources.pool, &ctx, &fc, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok("Different assignment".into())
        })
        .await
        .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let mut narrower = ctx.clone();
        narrower.allowed_files = Some(vec!["public.rs".into()]);
        assert_ne!(
            assignment_key(&ctx, &fc).unwrap(),
            assignment_key(&narrower, &fc).unwrap()
        );
    }

    #[tokio::test]
    async fn concurrent_and_interrupted_assignments_are_not_dispatched_twice() {
        use std::sync::Arc;
        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        let ctx = RunContext::default();
        let fc = crate::agent::types::ToolCall {
            name: "spawn_subagent".into(),
            args: serde_json::json!({"agent_id":"Alpha", "message":"same work"}),
        };
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let cloned_state = state.clone();
        let cloned_ctx = ctx.clone();
        let cloned_fc = fc.clone();
        let first = tokio::spawn(async move {
            execute_assignment_once(
                &cloned_state.resources.pool,
                &cloned_ctx,
                &cloned_fc,
                || async {
                    let _ = started_tx.send(());
                    std::future::pending::<()>().await;
                    Ok("unused".into())
                },
            )
            .await
        });
        started_rx.await.unwrap();
        let duplicate = execute_assignment_once(&state.resources.pool, &ctx, &fc, || async {
            panic!("Duplicate work must not run")
        })
        .await;
        assert!(duplicate
            .unwrap_err()
            .to_string()
            .contains("ASSIGNMENT_REVIEW_REQUIRED"));
        first.abort();
        let _ = first.await;
        let after_interruption =
            execute_assignment_once(&state.resources.pool, &ctx, &fc, || async {
                panic!("Interrupted work must not replay automatically")
            })
            .await;
        assert!(after_interruption
            .unwrap_err()
            .to_string()
            .contains("running"));
    }

    #[tokio::test]
    async fn failed_assignments_require_deliberate_retry() {
        let state = crate::state::AppState::new_minimal_mock().await;
        let ctx = RunContext::default();
        let fc = crate::agent::types::ToolCall {
            name: "spawn_subagent".into(),
            args: serde_json::json!({"message":"failed work"}),
        };
        let _ = execute_assignment_once(&state.resources.pool, &ctx, &fc, || async {
            Err(
                crate::agent::runner::tools::error::ToolExecutionError::Validation(
                    "test failure".into(),
                ),
            )
        })
        .await;
        let duplicate = execute_assignment_once(&state.resources.pool, &ctx, &fc, || async {
            panic!("Failure must not silently replay")
        })
        .await;
        assert!(duplicate.unwrap_err().to_string().contains("failed"));
    }

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
                reasoning_trace TEXT,
                artifacts TEXT,
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

    #[tokio::test]
    async fn test_complete_pending_directives_for_agent() {
        let pool = setup_db().await;

        insert_directive(
            &pool,
            "id-a",
            "mission-003",
            "agent-src",
            "agent-tgt",
            "Directive A",
        )
        .await;
        insert_directive(
            &pool,
            "id-b",
            "mission-003",
            "agent-src",
            "agent-tgt",
            "Directive B",
        )
        .await;
        insert_directive(
            &pool,
            "id-c",
            "mission-003",
            "agent-src",
            "agent-other",
            "Directive C",
        )
        .await;

        let before = get_pending_directives(&pool, "agent-tgt").await.unwrap();
        assert_eq!(before.len(), 2);

        let count =
            complete_pending_directives_for_agent(&pool, "agent-tgt", Some("Mission executed"))
                .await
                .unwrap();

        assert_eq!(count, 2);

        let after_tgt = get_pending_directives(&pool, "agent-tgt").await.unwrap();
        assert!(after_tgt.is_empty());

        // Other agent's pending directive is untouched
        let other = get_pending_directives(&pool, "agent-other").await.unwrap();
        assert_eq!(other.len(), 1);
    }
}
