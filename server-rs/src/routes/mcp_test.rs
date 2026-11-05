//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / mcp_test
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `mcp_test::tests::*`

#[cfg(test)]
mod tests {
    use crate::routes::mcp::{execute_mcp_tool, list_mcp_tools};
    use crate::state::AppState;
    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_list_mcp_tools_endpoint() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let response = list_mcp_tools(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_execute_system_tool_api_returns_accepted() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let args = json!({"agent_id": "test", "task_description": "test task"});

        // Whitelist the tool for the test
        sqlx::query("INSERT INTO permission_policies (tool_name, mode) VALUES (?, ?)")
            .bind("recruit_specialist")
            .bind("allow")
            .execute(&state.resources.pool)
            .await
            .expect("Failed to whitelist tool");
        state
            .security
            .permission_policy
            .refresh_cache()
            .await
            .expect("Failed to refresh permission cache");

        let response = execute_mcp_tool(
            Path("recruit_specialist".to_string()),
            State(state),
            axum::Json(args),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_execute_tool_denied_by_policy() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let args = json!({});

        sqlx::query("INSERT INTO permission_policies (tool_name, mode) VALUES (?, ?)")
            .bind("dangerous_tool")
            .bind("deny")
            .execute(&state.resources.pool)
            .await
            .expect("insert deny policy");
        state
            .security
            .permission_policy
            .refresh_cache()
            .await
            .expect("refresh cache");

        let response = execute_mcp_tool(
            Path("dangerous_tool".to_string()),
            State(state),
            axum::Json(args),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_execute_tool_privacy_mode_block() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let args = json!({});
        let response = execute_mcp_tool(
            Path("openai_whisper".to_string()),
            State(state),
            axum::Json(args),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
