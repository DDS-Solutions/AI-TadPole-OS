//! @docs ARCHITECTURE:Registry:Mcp
//! ### AI Assist Note
//! Operation ID caching, idempotent replay prevention, and bounded LRU eviction.
//! ### 🔍 Debugging & Observability
//! Enforces UUID invariants, argument binding immutability, and MAX_OPERATION_BINDINGS limits.

use super::super::headers::hash_operation_binding;
use super::super::limits::MAX_OPERATION_BINDINGS;
use crate::error::AppError;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct OperationState<'a> {
    pub bindings: &'a mut HashMap<String, String>,
    pub retryable: &'a mut HashSet<String>,
    pub order: &'a mut VecDeque<String>,
}

fn evict_oldest_operation(state: &mut OperationState<'_>) {
    let non_retryable_idx = state
        .order
        .iter()
        .position(|id| !state.retryable.contains(id));
    let evicted_id = if let Some(idx) = non_retryable_idx {
        state.order.remove(idx)
    } else {
        state.order.pop_front()
    };

    if let Some(ref id) = evicted_id {
        state.bindings.remove(id);
        state.retryable.remove(id);
    }
}

pub fn remember_operation(
    state: &mut OperationState<'_>,
    operation_id: &str,
    name: &str,
    arguments: &Value,
) -> Result<(), AppError> {
    uuid::Uuid::parse_str(operation_id)
        .map_err(|_| AppError::BadRequest("MCP operation_id must be a UUID".to_string()))?;

    let binding = hash_operation_binding(name, arguments);

    if let Some(existing) = state.bindings.get(operation_id) {
        if existing != &binding {
            return Err(AppError::Conflict(
                "MCP operation_id cannot be reused with different tool arguments".to_string(),
            ));
        }
        if !state.retryable.remove(operation_id) {
            return Err(AppError::Conflict(
                "MCP operation_id may be retried only after an explicit retryable outcome"
                    .to_string(),
            ));
        }
        return Ok(());
    }

    if state.bindings.len() >= MAX_OPERATION_BINDINGS {
        evict_oldest_operation(state);
    }

    state.bindings.insert(operation_id.to_string(), binding);
    state.order.push_back(operation_id.to_string());
    Ok(())
}
