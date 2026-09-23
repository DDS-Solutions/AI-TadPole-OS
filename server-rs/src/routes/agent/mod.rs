//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / agent
//!
//! ### AI Assist Note
//! - Task request claims are atomic; duplicate deliveries must not start additional runners.
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod chat;
pub mod crud;
pub mod missions;
pub mod models;
pub mod recovery;
pub mod tasks;

pub use chat::*;
pub use crud::*;
pub use missions::*;
pub use models::*;
pub use recovery::*;
pub use tasks::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agent::types::{AgentIdentity, EngineAgent},
        error::AppError,
        state::AppState,
    };
    use axum::extract::{Path, Query, State};
    use std::sync::Arc;

    #[test]
    fn test_task_request_claim_is_atomic() {
        let requests = dashmap::DashMap::new();
        let barrier = std::sync::Barrier::new(16);
        let now = std::time::Instant::now();
        let claims = std::thread::scope(|scope| {
            let workers: Vec<_> = (0..16)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        claim_task_request(&requests, "agent:request:payload", now)
                    })
                })
                .collect();
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .filter(|claimed| *claimed)
                .count()
        });
        assert_eq!(claims, 1);
        assert_eq!(requests.len(), 1);
    }

    #[test]
    fn test_task_request_claim_expires_without_duplicate_extension() {
        let requests = dashmap::DashMap::new();
        let now = std::time::Instant::now();
        assert!(claim_task_request(&requests, "first", now));
        assert!(!claim_task_request(
            &requests,
            "first",
            now + std::time::Duration::from_secs(14)
        ));
        assert_eq!(*requests.get("first").unwrap(), now);
        assert!(claim_task_request(&requests, "second", now));
        assert!(claim_task_request(
            &requests,
            "first",
            now + std::time::Duration::from_secs(DEDUP_WINDOW_SECS)
        ));
    }

    #[tokio::test]
    async fn test_get_agent_not_found() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let result = get_agent(Path("non-existent-agent-id".to_string()), State(state)).await;
        assert!(result.is_err());
        if let Err(AppError::NotFound(msg)) = result {
            assert!(msg.contains("non-existent-agent-id"));
        } else {
            panic!("Expected AppError::NotFound");
        }
    }

    #[tokio::test]
    async fn test_list_agents_empty() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let result = get_agents(
            State(state),
            Query(crate::routes::pagination::PaginationParams::default()),
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_agent_route() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let mut agent = EngineAgent {
            identity: AgentIdentity {
                id: "test-delete-agent-route".to_string(),
                name: "Delete Me".to_string(),
                role: "Tester".to_string(),
                ..Default::default()
            },
            version: 1,
            ..Default::default()
        };
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .expect("Failed to save test agent");
        state
            .registry
            .agents
            .insert("test-delete-agent-route".to_string(), agent);

        let res = delete_agent(
            State(state.clone()),
            Path("test-delete-agent-route".to_string()),
        )
        .await;
        assert!(res.is_ok());
        assert!(!state
            .registry
            .agents
            .contains_key("test-delete-agent-route"));
    }
}
