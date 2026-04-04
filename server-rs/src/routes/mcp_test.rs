//! MCP Handler Tests — Neural bridge and protocol state verification
//!
//! @docs ARCHITECTURE:Agent

#[cfg(test)]
mod tests {
    use crate::state::AppState;
    use axum::{http::StatusCode, response::IntoResponse};
    use serde_json::json;
    use std::sync::Arc;
    // Since we want to test the HANDLERS specifically with a mock state:

    use crate::routes::mcp::{execute_mcp_tool, list_mcp_tools};

    #[tokio::test]
    async fn test_list_mcp_tools_endpoint() {
        let state = Arc::new(AppState::new().await.expect("Failed to initialize state for MCP tests"));

        // Manual call to handler
        let response = list_mcp_tools(axum::extract::State(state))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        // Verify body contains system tools
        // (Simplified check as it's a response object)
    }

    #[tokio::test]
    async fn test_execute_system_tool_api_returns_accepted() {
        let state = Arc::new(AppState::new().await.expect("Failed to initialize state for MCP tool execution tests"));
        let args = json!({"agent_id": "test", "task_description": "test task"});

        let response = execute_mcp_tool(
            axum::extract::Path("recruit_specialist".to_string()),
            axum::extract::State(state),
            axum::Json(args),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }
}
