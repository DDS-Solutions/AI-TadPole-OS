//! @docs ARCHITECTURE:Agent
//! 
//! ### AI Assist Note
//! **MCP Route Verification (Handler Tests)**: Orchestrates the 
//! verification of the MCP API handlers and protocol state for the 
//! Tadpole OS engine. Features **Handler Unit Testing**: provides 
//! direct testing of the `list_mcp_tools` and `execute_mcp_tool` 
//! functions without requiring a full HTTP server stack. Implements 
//! **Protocol Response Validation**: ensures that system tools 
//! requiring dynamic context return the correct `202 Accepted` 
//! (Delegated) status when called via the API. AI agents should run 
//! these tests to verify that the neural bridge between the REST API 
//! and the `McpHost` remains functional after routing or state 
//! refactors (MCP-05).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 500 Internal Server Error for tool execution due 
//!   to missing workspace directories, incorrect path parameter 
//!   extraction, or state initialization failures in the test context.
//! - **Trace Scope**: `server-rs::routes::mcp_test`

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
