//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / blueprints
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::RoleBlueprint;
use crate::error::AppError;
use sqlx::SqlitePool;

/// ### ⚖️ Governance: Blueprint Discovery
/// Loads all registered Role Blueprints from the database ordered deterministically by name.
pub async fn load_blueprints(pool: &SqlitePool) -> Result<Vec<RoleBlueprint>, AppError> {
    let rows = sqlx::query_as::<_, RoleBlueprint>(
        "SELECT id, name, department, description, skills, workflows, mcp_tools, requires_oversight, model_id, created_at \
         FROM role_blueprints ORDER BY name ASC"
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
/// Deletes a Role Blueprint from the system. Returns `AppError::NotFound` if the blueprint does not exist.
pub async fn delete_blueprint<'c, E>(executor: E, id: &str) -> Result<(), AppError>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    let res = sqlx::query("DELETE FROM role_blueprints WHERE id = ?")
        .bind(id)
        .execute(executor)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Role blueprint '{}' not found",
            id
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE role_blueprints (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                department TEXT NOT NULL,
                description TEXT NOT NULL,
                skills TEXT,
                workflows TEXT,
                mcp_tools TEXT,
                requires_oversight BOOLEAN DEFAULT 0,
                model_id TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_blueprints_crud_and_ordering() {
        let pool = setup_test_db().await;

        let b1 = RoleBlueprint {
            id: "analyst".to_string(),
            name: "Zebra Analyst".to_string(),
            department: "Data".to_string(),
            description: "Data analysis role".to_string(),
            skills: "[]".to_string(),
            workflows: "[]".to_string(),
            mcp_tools: "[]".to_string(),
            requires_oversight: false,
            model_id: None,
            created_at: None,
        };

        let b2 = RoleBlueprint {
            id: "architect".to_string(),
            name: "Alpha Architect".to_string(),
            department: "Engineering".to_string(),
            description: "System design role".to_string(),
            skills: "[]".to_string(),
            workflows: "[]".to_string(),
            mcp_tools: "[]".to_string(),
            requires_oversight: true,
            model_id: None,
            created_at: None,
        };

        save_blueprint(&pool, &b1).await.unwrap();
        save_blueprint(&pool, &b2).await.unwrap();

        let loaded = load_blueprints(&pool).await.unwrap();
        assert_eq!(loaded.len(), 2);
        // Ordered by name ASC ("Alpha Architect" before "Zebra Analyst")
        assert_eq!(loaded[0].id, "architect");
        assert_eq!(loaded[1].id, "analyst");

        // Delete existing
        delete_blueprint(&pool, "analyst").await.unwrap();
        let after_delete = load_blueprints(&pool).await.unwrap();
        assert_eq!(after_delete.len(), 1);

        // Delete non-existent returns NotFound
        let not_found_res = delete_blueprint(&pool, "non_existent").await;
        assert!(matches!(not_found_res, Err(AppError::NotFound(_))));
    }
}
