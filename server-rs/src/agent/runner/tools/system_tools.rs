//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / system_tools
//!
//! ### AI Assist Note
//! Manages persistent working memory and active model slot switches with OCC retry.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use crate::agent::runner::{AgentRunner, RunContext};
use crate::error::AppError;

/// Performs a null-aware shallow merge of `patch` into `target`.
/// Keys whose value is `null` in the patch are removed from the target (RFC 7396 JSON Merge Patch).
fn merge_working_memory(
    target: &mut serde_json::Map<String, serde_json::Value>,
    patch: &serde_json::Map<String, serde_json::Value>,
) {
    for (k, v) in patch {
        if v.is_null() {
            target.remove(k);
        } else {
            target.insert(k.clone(), v.clone());
        }
    }
}

fn apply_working_memory_update(
    working_memory: &mut serde_json::Value,
    new_memory: &serde_json::Value,
) {
    let (Some(old_obj), Some(new_obj)) = (working_memory.as_object_mut(), new_memory.as_object())
    else {
        *working_memory = new_memory.clone();
        return;
    };
    merge_working_memory(old_obj, new_obj);
}

fn validate_slot_name(slot: &str) -> bool {
    matches!(slot, "planning" | "execution" | "default")
}

fn resolve_model_slot_config(
    agent: &crate::agent::types::EngineAgent,
    slot: &str,
) -> Option<(String, String)> {
    let config = match slot {
        "planning" => agent.models.planning_slot.as_ref(),
        "execution" => agent.models.execution_slot.as_ref(),
        _ => Some(&agent.models.model),
    }?;
    Some((config.model_id.clone(), config.provider.to_string()))
}

impl AgentRunner {
    async fn sync_agent_in_registry(
        &self,
        agent_id: &str,
        current_version: u32,
        agent_clone: &crate::agent::types::EngineAgent,
    ) -> Result<(), AppError> {
        let pool = self.state.resources.pool.clone();
        let Some(mut entry) = self.state.registry.agents.get_mut(agent_id) else {
            return Ok(());
        };
        let agent = entry.value_mut();
        if agent.version == current_version {
            *agent = agent_clone.clone();
        } else if let Some(db_agent) =
            crate::agent::persistence::load_agent_by_id_db(&pool, agent_id).await?
        {
            *agent = db_agent;
        }

        self.state.emit_event(serde_json::json!({
            "type": "agent:update",
            "agent_id": agent_id,
            "data": agent.clone()
        }));
        Ok(())
    }

    fn init_agent_mutation<F>(
        &self,
        agent_id: &str,
        mut mutator: F,
    ) -> Result<crate::agent::types::EngineAgent, AppError>
    where
        F: FnMut(&mut crate::agent::types::EngineAgent),
    {
        let mut entry = self
            .state
            .registry
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| {
                AppError::NotFound(format!("Agent '{}' not found in registry", agent_id))
            })?;
        let agent = entry.value_mut();
        mutator(agent);
        Ok(agent.clone())
    }

    async fn retry_save_agent<F>(
        pool: &sqlx::SqlitePool,
        agent_id: &str,
        mut agent_clone: crate::agent::types::EngineAgent,
        mut mutator: F,
    ) -> Result<crate::agent::types::EngineAgent, AppError>
    where
        F: FnMut(&mut crate::agent::types::EngineAgent),
    {
        for retries in 0..=5 {
            let Err(e) = crate::agent::persistence::save_agent_db(pool, &mut agent_clone).await
            else {
                return Ok(agent_clone);
            };
            if !matches!(e, AppError::Conflict(_)) || retries == 5 {
                return Err(e);
            }
            let Some(mut db_agent) =
                crate::agent::persistence::load_agent_by_id_db(pool, agent_id).await?
            else {
                continue;
            };
            mutator(&mut db_agent);
            agent_clone = db_agent;
            tokio::time::sleep(std::time::Duration::from_millis(50 * (retries + 1))).await;
        }
        Ok(agent_clone)
    }

    /// Helper method for optimistic concurrency control (OCC) agent mutations with automatic DB conflict retry.
    async fn update_agent_with_retry<F>(
        &self,
        agent_id: &str,
        mut mutator: F,
    ) -> Result<crate::agent::types::EngineAgent, AppError>
    where
        F: FnMut(&mut crate::agent::types::EngineAgent),
    {
        let agent_clone = self.init_agent_mutation(agent_id, &mut mutator)?;
        let pool = self.state.resources.pool.clone();
        let saved = Self::retry_save_agent(&pool, agent_id, agent_clone, mutator).await?;
        self.sync_agent_in_registry(agent_id, saved.version, &saved)
            .await?;
        Ok(saved)
    }

    /// Updates the agent's persistent working memory (scratchpad).
    pub(crate) async fn handle_update_working_memory(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
    ) -> Result<(), AppError> {
        let new_memory = fc
            .args
            .get("memory")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let res = self
            .update_agent_with_retry(&ctx.identity.agent_id, |agent| {
                apply_working_memory_update(&mut agent.state.working_memory, &new_memory);
            })
            .await;

        match res {
            Ok(_) => *output_text = "(WORKING MEMORY UPDATED SUCCESSFULLY)".to_string(),
            Err(e) => *output_text = format!("(ERROR: Failed to update working memory: {})", e),
        }

        Ok(())
    }

    fn get_agent_slot_config(
        &self,
        agent_id: &str,
        slot: &str,
    ) -> Result<(String, String), String> {
        let entry = self.state.registry.agents.get(agent_id).ok_or_else(|| {
            "(ERROR: Agent not found in registry during model switch)".to_string()
        })?;
        resolve_model_slot_config(entry.value(), slot).ok_or_else(|| {
            format!(
                "(ERROR: Requested model slot '{}' has no configuration defined for this agent)",
                slot
            )
        })
    }

    async fn request_model_switch_oversight(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        requested_slot: &str,
        model_id: &str,
        provider: &str,
    ) -> Result<Option<String>, AppError> {
        self.broadcast_agent(
            ctx,
            &format!(
                "💎 Oversight: wants to switch active model slot to '{}' (model: {}, provider: {}). CRITICAL REVIEW REQUIRED.",
                requested_slot, model_id, provider
            ),
            "error",
        );

        let audit = crate::agent::types::ToolCallAudit {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: ctx.identity.agent_id.clone(),
            mission_id: Some(ctx.mission_state.mission_id.clone()),
            skill: "request_model_switch".to_string(),
            params: fc.args.clone(),
            department: ctx.identity.department.clone(),
            description: format!(
                "Switch active model slot to '{}' (model: {}, provider: {})",
                requested_slot, model_id, provider
            ),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let res = self
            .submit_oversight_resolution(audit, Some(ctx.mission_state.mission_id.clone()))
            .await?;

        if !res.approved {
            return Ok(None);
        }

        let target = res
            .override_slot
            .unwrap_or_else(|| requested_slot.to_string());
        Ok(Some(target))
    }

    fn sync_disk_agents(&self) {
        let base_dir = self.state.base_dir.clone();
        let agents: Vec<_> = self
            .state
            .registry
            .agents
            .iter()
            .map(|kv| kv.value().clone())
            .collect();

        tokio::spawn(async move {
            if let Err(err) = crate::agent::persistence::save_agents_json(&base_dir, agents).await {
                tracing::error!("Failed to sync agents.json during model switch: {}", err);
            }
        });
    }

    async fn apply_model_slot_switch(
        &self,
        ctx: &RunContext,
        target_slot: &str,
    ) -> Result<String, AppError> {
        let slot = target_slot.to_string();
        self.update_agent_with_retry(&ctx.identity.agent_id, move |agent| {
            agent.models.active_model_slot = Some(slot.clone());
        })
        .await?;
        self.sync_disk_agents();
        self.broadcast_agent(
            ctx,
            &format!(
                "🔄 System: Model slot successfully switched to '{}'.",
                target_slot
            ),
            "info",
        );
        Ok(format!(
            "(MODEL SLOT SUCCESSFULLY SWITCHED TO '{}')",
            target_slot
        ))
    }

    pub(crate) async fn handle_request_model_switch(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
    ) -> Result<(), AppError> {
        let requested_slot = fc
            .args
            .get("slot")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if !validate_slot_name(requested_slot) {
            *output_text = format!(
                "(ERROR: Invalid model slot '{}'. Valid slots are: 'planning', 'execution', 'default')",
                requested_slot
            );
            return Ok(());
        }

        let (model_id, provider) =
            match self.get_agent_slot_config(&ctx.identity.agent_id, requested_slot) {
                Ok(cfg) => cfg,
                Err(msg) => {
                    *output_text = msg;
                    return Ok(());
                }
            };

        let target_slot = match self
            .request_model_switch_oversight(ctx, fc, requested_slot, &model_id, &provider)
            .await?
        {
            Some(slot) => slot,
            None => {
                *output_text = "(Model slot switch REJECTED by Oversight)".to_string();
                return Ok(());
            }
        };

        if !validate_slot_name(&target_slot) {
            *output_text = format!("(ERROR: Invalid overridden model slot '{}')", target_slot);
            return Ok(());
        }

        if let Err(msg) = self.get_agent_slot_config(&ctx.identity.agent_id, &target_slot) {
            *output_text = msg;
            return Ok(());
        }

        match self.apply_model_slot_switch(ctx, &target_slot).await {
            Ok(msg) => *output_text = msg,
            Err(e) => *output_text = format!("(ERROR: Failed to update agent model slot: {})", e),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_working_memory_null_removals() {
        let mut target = serde_json::Map::new();
        target.insert("keep".to_string(), json!("stays"));
        target.insert("remove_me".to_string(), json!("to be deleted"));

        let mut patch = serde_json::Map::new();
        patch.insert("remove_me".to_string(), serde_json::Value::Null);

        merge_working_memory(&mut target, &patch);

        assert_eq!(target.get("keep"), Some(&json!("stays")));
        assert_eq!(target.get("remove_me"), None);
    }

    #[test]
    fn test_merge_working_memory_updates() {
        let mut target = serde_json::Map::new();
        target.insert("key1".to_string(), json!("old_val"));

        let mut patch = serde_json::Map::new();
        patch.insert("key1".to_string(), json!("new_val"));
        patch.insert("key2".to_string(), json!(42));

        merge_working_memory(&mut target, &patch);

        assert_eq!(target.get("key1"), Some(&json!("new_val")));
        assert_eq!(target.get("key2"), Some(&json!(42)));
    }
}
