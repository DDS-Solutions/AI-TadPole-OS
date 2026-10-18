//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / Agent Chat Completions
//! - **Primary Entrypoints**: `create_chat_completion`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::models::{
    ChatCompletionChoice, ChatCompletionChoiceMessage, ChatCompletionRequest,
    ChatCompletionResponse, ChatCompletionUsage,
};
use super::tasks::validate_agent_preflight;
use crate::{
    agent::{runner::AgentRunner, types::TaskPayload},
    error::AppError,
    state::AppState,
};
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

/// POST /v1/chat/completions
///
/// OpenAI-compatible completion endpoint that routes the task to a specific Swarm agent (the "model")
/// and executes the task synchronously, returning the final report/result.
pub async fn create_chat_completion(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let agent_id = req.model.clone();

    let last_user_message = req
        .messages
        .iter()
        .rfind(|m| m.role == "user")
        .map(|m| m.content.clone())
        .ok_or_else(|| {
            AppError::BadRequest("No user message found in completion history".to_string())
        })?;

    let mut payload = TaskPayload {
        message: last_user_message,
        active_model_slot: Some("default".to_string()),
        ..Default::default()
    };

    // Preflight validation for chat completion (Existence, Status, Budget, and Traceparent)
    validate_agent_preflight(&state, &agent_id, &headers, &mut payload)?;

    let _permit = state.comms.runner_semaphore.acquire().await.map_err(|_| {
        AppError::InternalServerError("Runner execution throttle semaphore closed".to_string())
    })?;

    tracing::info!(
        "📡 [Completions API] Dispatching synchronous task to agent {}",
        agent_id
    );
    let runner = AgentRunner::new(state.clone());
    let run_res = match runner.run_with_output(agent_id.clone(), payload).await {
        Ok(res) => res,
        Err(err) => {
            let status = err.status_code();
            let (err_type, message) = match &err {
                AppError::NotFound(msg) => ("not_found_error", msg.clone()),
                AppError::Unauthorized(msg) => ("authentication_error", msg.clone()),
                AppError::RateLimit(msg) => ("rate_limit_error", msg.clone()),
                AppError::BadRequest(msg) => ("invalid_request_error", msg.clone()),
                _ => ("api_error", err.to_string()),
            };
            return Ok((
                status,
                Json(serde_json::json!({
                    "error": {
                        "message": message,
                        "type": err_type,
                        "param": null,
                        "code": status.as_u16()
                    }
                })),
            )
                .into_response());
        }
    };

    let (prompt_tokens, completion_tokens, total_tokens) = if let Some(ref usage) = run_res.usage {
        (usage.input_tokens, usage.output_tokens, usage.total_tokens)
    } else {
        (0, 0, 0)
    };

    let response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion",
        created: chrono::Utc::now().timestamp(),
        model: agent_id,
        choices: vec![ChatCompletionChoice {
            index: 0,
            message: ChatCompletionChoiceMessage {
                role: "assistant",
                content: run_res.text,
            },
            finish_reason: "stop",
        }],
        usage: ChatCompletionUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        },
    };

    Ok(Json(response).into_response())
}
