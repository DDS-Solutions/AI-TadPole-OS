//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Test Support
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Single-connection SQLite in-memory test database avoiding multi-connection table drops.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none

#[cfg(test)]
pub async fn create_test_pool() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS permission_policies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tool_name TEXT NOT NULL UNIQUE,
            mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS agent_permission_policies (
            agent_id TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (agent_id, tool_name)
        );
        CREATE TABLE IF NOT EXISTS role_permission_policies (
            role TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (role, tool_name)
        );
        CREATE TABLE IF NOT EXISTS capability_policies (
            capability_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            mode TEXT NOT NULL,
            PRIMARY KEY (capability_class, resource_pattern)
        );
        CREATE TABLE IF NOT EXISTS agent_capability_policies (
            agent_id TEXT NOT NULL,
            capability_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            mode TEXT NOT NULL,
            PRIMARY KEY (agent_id, capability_class, resource_pattern)
        );
        CREATE TABLE IF NOT EXISTS role_capability_policies (
            role TEXT NOT NULL,
            capability_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            mode TEXT NOT NULL,
            PRIMARY KEY (role, capability_class, resource_pattern)
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}
