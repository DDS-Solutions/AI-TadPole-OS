//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **Governance & Role Blueprints Persistence**: Handles loading, saving, and deletion of `RoleBlueprint` entities in SQLite.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Database query execution failure or schema serialization mismatch.
//! - **Telemetry Link**: Search for `[Blueprints]` in server logs.

use crate::agent::types::RoleBlueprint;
use crate::error::AppError;
use sqlx::SqlitePool;

/// ### ⚖️ Governance: Blueprint Discovery
/// Loads all registered Role Blueprints from the database.
pub async fn load_blueprints(pool: &SqlitePool) -> Result<Vec<RoleBlueprint>, AppError> {
    let rows = sqlx::query_as::<_, RoleBlueprint>(
        "SELECT id, name, department, description, skills, workflows, mcp_tools, requires_oversight, model_id, created_at FROM role_blueprints"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// ### ⚖️ Governance: Promote to Role
/// Persists a Role Blueprint to the database.
pub async fn save_blueprint(pool: &SqlitePool, blueprint: &RoleBlueprint) -> Result<(), AppError> {
    execute_save_blueprint(pool, blueprint).await
}

/// Transaction-compatible variant of `save_blueprint`.
pub async fn execute_save_blueprint<'c, E>(
    executor: E,
    blueprint: &RoleBlueprint,
) -> Result<(), AppError>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    sqlx::query(
        "INSERT INTO role_blueprints (id, name, department, description, skills, workflows, mcp_tools, requires_oversight, model_id, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            department = excluded.department,
            description = excluded.description,
            skills = excluded.skills,
            workflows = excluded.workflows,
            mcp_tools = excluded.mcp_tools,
            requires_oversight = excluded.requires_oversight,
            model_id = excluded.model_id"
    )
    .bind(&blueprint.id)
    .bind(&blueprint.name)
    .bind(&blueprint.department)
    .bind(&blueprint.description)
    .bind(&blueprint.skills)
    .bind(&blueprint.workflows)
    .bind(&blueprint.mcp_tools)
    .bind(blueprint.requires_oversight)
    .bind(&blueprint.model_id)
    .bind(blueprint.created_at.unwrap_or_else(chrono::Utc::now))
    .execute(executor)
    .await?;
    Ok(())
}

/// ### ⚖️ Governance: Role Retirement
/// Deletes a Role Blueprint from the system.
pub async fn delete_blueprint<'c, E>(executor: E, id: &str) -> Result<(), AppError>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    sqlx::query("DELETE FROM role_blueprints WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await?;
    Ok(())
}
