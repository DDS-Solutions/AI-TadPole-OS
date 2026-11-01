//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - **Subsystem**: Agent context / configuration and policy resolution
//! - Preserve ordered preparation, workspace/privacy scope and shared usage accounting.
//! - Persistent quotas and payload limits may tighten a mission cap, never widen it implicitly.
//!
//! ### 🔍 Debugging & Observability
//! - Failures retain their existing AppError or nonfatal tracing behavior.

use super::super::{AgentIdentity, AgentRunner, Environment, MissionState, RunContext};
use super::{validate_and_resolve_workspace_dir, MICRO_DOLLARS_PER_USD};
use crate::agent::types::{EngineAgent, ModelConfig, ModelEntry, TaskPayload};
use crate::error::AppError;
use uuid::Uuid;

impl AgentRunner {
    /// Resolves the full agent context from registries, applying payload overrides.
    pub(crate) async fn resolve_agent_context(
        &self,
        agent_id: &str,
        payload: &TaskPayload,
        mission_id: &str,
        depth: u32,
        lineage: &[String],
    ) -> Result<RunContext, AppError> {
        let a = self
            .state
            .registry
            .agents
            .get(agent_id)
            .map(|e| e.value().clone())
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;

        let active_slot = payload
            .active_model_slot
            .as_deref()
            .or(a.models.active_model_slot.as_deref());

        // 1. Resolve Target Model ID
        let is_swarm_child = depth > 0 || payload.swarm_depth.unwrap_or(0) > 0;
        let target_model_id = if is_swarm_child {
            // Swarm subagents preserve their own configured model slot
            match active_slot {
                Some("planning") => a
                    .models
                    .planning_slot
                    .as_ref()
                    .map(|s| &s.model_id)
                    .filter(|s| !s.trim().is_empty()),
                Some("execution") => a
                    .models
                    .execution_slot
                    .as_ref()
                    .map(|s| &s.model_id)
                    .filter(|s| !s.trim().is_empty()),
                _ => a.models.model_id.as_ref().filter(|s| !s.trim().is_empty()),
            }
            .or_else(|| payload.model_id.as_ref().filter(|s| !s.trim().is_empty()))
            .unwrap_or(&a.models.model.model_id)
            .clone()
        } else {
            payload
                .model_id
                .as_ref()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| match active_slot {
                    Some("planning") => a
                        .models
                        .planning_slot
                        .as_ref()
                        .map(|s| &s.model_id)
                        .filter(|s| !s.trim().is_empty()),
                    Some("execution") => a
                        .models
                        .execution_slot
                        .as_ref()
                        .map(|s| &s.model_id)
                        .filter(|s| !s.trim().is_empty()),
                    _ => a.models.model_id.as_ref().filter(|s| !s.trim().is_empty()),
                })
                .unwrap_or(&a.models.model.model_id)
                .clone()
        };

        // 2. Resolve Base Configuration
        let slot_cfg = match active_slot {
            Some("planning")
                if a.models
                    .planning_slot
                    .as_ref()
                    .is_some_and(|s| !s.model_id.trim().is_empty()) =>
            {
                a.models.planning_slot.as_ref().unwrap()
            }
            Some("execution")
                if a.models
                    .execution_slot
                    .as_ref()
                    .is_some_and(|s| !s.model_id.trim().is_empty()) =>
            {
                a.models.execution_slot.as_ref().unwrap()
            }
            _ => &a.models.model,
        };
        let mut resolved_config = self.resolve_base_config(&a, &target_model_id, slot_cfg)?;

        // 3. Apply Payload Overrides
        self.merge_payload_overrides(&mut resolved_config, payload, is_swarm_child);

        // Model-Provider Alignment
        if resolved_config.provider == crate::agent::types::ModelProvider::Openai {
            if let Some(detected) =
                crate::agent::types::ModelProvider::from_model_id(&resolved_config.model_id)
            {
                resolved_config.provider = detected;
            }
        }

        // 4. Resolve Workspace Root
        let workspace_root = self.resolve_workspace_paths(payload.cluster_id.as_deref())?;
        let fs_adapter = crate::adapter::filesystem::FilesystemAdapter::new(workspace_root.clone());

        // 5. Capability Merging & Security Gates
        let mut skills = resolved_config
            .skills
            .clone()
            .unwrap_or_else(|| a.capabilities.skills.clone());
        let mut workflows = resolved_config
            .workflows
            .clone()
            .unwrap_or_else(|| a.capabilities.workflows.clone());
        let mut mcp_tools = resolved_config
            .mcp_tools
            .clone()
            .unwrap_or_else(|| a.capabilities.mcp_tools.clone());

        let safe_mode = payload.safe_mode.unwrap_or(false);
        self.apply_security_gates(safe_mode, &mut skills, &mut workflows, &mut mcp_tools);

        // 6. Mission Financials
        let (mut budget_usd, current_cost_usd) = if mission_id != "system-internal"
            && !mission_id.is_empty()
        {
            match crate::agent::mission::get_mission_by_id(&self.state.resources.pool, mission_id)
                .await
            {
                Ok(Some(m)) => (m.budget_usd, m.cost_usd),
                _ => (payload.budget_usd.unwrap_or(0.0), 0.0),
            }
        } else {
            (0.0, 0.0)
        };
        if let Some(requested) = payload.budget_usd.filter(|&budget| budget > 0.0) {
            budget_usd = if budget_usd > 0.0 {
                budget_usd.min(requested)
            } else {
                requested
            };
        }

        // SEC-02: Synchronize with persistent budget_guard (mission_quotas)
        if mission_id != "system-internal" && !mission_id.is_empty() {
            let quota_target = payload.cluster_id.as_deref().unwrap_or(mission_id);
            match sqlx::query_scalar::<_, i64>(
                "SELECT budget_usd FROM mission_quotas WHERE cluster_id = ?1 OR cluster_id = ?2 LIMIT 1",
            )
            .bind(mission_id)
            .bind(quota_target)
            .fetch_optional(&self.state.resources.pool)
            .await
            {
                Ok(Some(quota_budget)) => {
                    // Convert micro-dollars to float (unified to micro-dollars in database)
                    let normalized_budget = (quota_budget as f64) / MICRO_DOLLARS_PER_USD;
                    if normalized_budget > 0.0 {
                        // Quotas constrain the existing mission cap even when a
                        // child payload does not repeat the root's budget.
                        if budget_usd > 0.0 {
                            budget_usd = budget_usd.min(normalized_budget);
                        } else {
                            budget_usd = normalized_budget;
                        }
                    } else if normalized_budget == 0.0 {
                        budget_usd = 0.0;
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [BudgetGuard] Failed to fetch mission quota for '{}': {}",
                        mission_id,
                        e
                    );
                }
            }
        }

        let authority_level = crate::agent::types::RoleAuthorityLevel::from_role(&a.identity.role);
        let primary_goal = payload
            .primary_goal
            .clone()
            .or_else(|| Some(payload.message.clone()));

        let identity = AgentIdentity {
            agent_id: agent_id.to_string(),
            name: a.identity.name.clone(),
            role: a.identity.role.clone(),
            department: a.identity.department.clone(),
            description: a.identity.description.clone(),
            authority_level,
        };

        let mission_state = MissionState {
            mission_id: mission_id.to_string(),
            cluster_id: payload.cluster_id.clone(),
            user_id: payload.user_id.clone(),
            depth,
            lineage: lineage.to_vec(),
            primary_goal,
            budget_usd,
            current_cost_usd,
            sub_budget_usd: payload.sub_budget_usd,
        };

        let env = Environment {
            workspace_root,
            fs_adapter,
            base_dir: self.state.base_dir.clone(),
        };

        let mut ctx = RunContext::new_base(
            identity,
            mission_state,
            env,
            resolved_config,
            a.models.clone(),
            skills,
            workflows,
            mcp_tools,
            a.state.working_memory.clone(),
        );
        ctx.safe_mode = safe_mode;
        ctx.analysis = payload.analysis.unwrap_or(false);
        ctx.traceparent = payload.traceparent.clone();
        ctx.allowed_files = payload.allowed_files.clone();
        ctx.recent_findings = payload.recent_findings.clone();
        ctx.visible_transcript = Some(std::sync::Arc::new(parking_lot::Mutex::new(
            payload.visible_transcript.clone().unwrap_or_default(),
        )));

        Ok(ctx)
    }

    /// Resolves the base model configuration from registries or agent defaults.
    pub(super) fn resolve_base_config(
        &self,
        a: &EngineAgent,
        target_model_id: &str,
        slot_cfg: &ModelConfig,
    ) -> Result<ModelConfig, AppError> {
        if let Some(model_entry) = self.state.registry.models.get(target_model_id) {
            self.construct_registry_config(a, &model_entry, slot_cfg)
        } else if let Some(found_entry) = self
            .state
            .registry
            .models
            .iter()
            .find(|kv| kv.value().name.to_lowercase() == target_model_id.to_lowercase())
        {
            self.construct_registry_config(a, found_entry.value(), slot_cfg)
        } else {
            // FALLBACK: Use agent's internal model config for the active slot
            let mut cfg = slot_cfg.clone();
            cfg.model_id = target_model_id.to_string();
            if cfg.system_prompt.as_deref().unwrap_or("").trim().is_empty() {
                if let Some(prompt) = a
                    .models
                    .planning_slot
                    .as_ref()
                    .and_then(|s| s.system_prompt.as_ref())
                    .filter(|s| !s.trim().is_empty())
                    .or_else(|| {
                        a.models
                            .model
                            .system_prompt
                            .as_ref()
                            .filter(|s| !s.trim().is_empty())
                    })
                {
                    cfg.system_prompt = Some(prompt.clone());
                }
            }
            cfg.skills = Some(a.capabilities.skills.clone());
            cfg.workflows = Some(a.capabilities.workflows.clone());
            cfg.mcp_tools = Some(a.capabilities.mcp_tools.clone());
            Ok(cfg)
        }
    }

    /// Helper to build a ModelConfig from registry entries.
    pub(super) fn construct_registry_config(
        &self,
        a: &EngineAgent,
        m: &ModelEntry,
        slot_cfg: &ModelConfig,
    ) -> Result<ModelConfig, AppError> {
        let provider_config = self
            .state
            .registry
            .providers
            .get(&m.provider_id)
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Provider {} not found for model {}",
                    m.provider_id, m.name
                ))
            })?;

        let model_id = if let Some(pm_id) = &m.provider_model_id {
            pm_id.clone()
        } else if Uuid::parse_str(&m.id).is_ok() {
            tracing::debug!(
                "Model {} has UUID primary key; using model name '{}' as model_id",
                m.id,
                m.name
            );
            m.name.clone()
        } else {
            m.id.clone()
        };

        let mut config = ModelConfig {
            provider: provider_config.protocol,
            model_id,
            api_key: provider_config.api_key.clone(),
            base_url: provider_config.base_url.clone(),
            system_prompt: if provider_config.supports_steering_vectors {
                Some("".to_string())
            } else {
                let candidate = slot_cfg
                    .system_prompt
                    .as_ref()
                    .filter(|s| !s.trim().is_empty());
                candidate
                    .or_else(|| {
                        a.models
                            .planning_slot
                            .as_ref()
                            .and_then(|s| s.system_prompt.as_ref())
                            .filter(|s| !s.trim().is_empty())
                    })
                    .or_else(|| {
                        a.models
                            .model
                            .system_prompt
                            .as_ref()
                            .filter(|s| !s.trim().is_empty())
                    })
                    .cloned()
            },
            temperature: slot_cfg.temperature,
            max_tokens: slot_cfg.max_tokens,
            external_id: provider_config.external_id.clone(),
            rpm: m.rpm,
            rpd: m.rpd,
            tpm: m.tpm,
            tpd: m.tpd,
            skills: Some(a.capabilities.skills.clone()),
            workflows: Some(a.capabilities.workflows.clone()),
            mcp_tools: Some(a.capabilities.mcp_tools.clone()),
            steering_vectors: if provider_config.supports_steering_vectors {
                Some(vec![format!("persona:{}", a.identity.role)])
            } else {
                None
            },
            reasoning_depth: slot_cfg.reasoning_depth,
            act_threshold: slot_cfg.act_threshold,
            max_turns: slot_cfg.max_turns,
            connector_configs: None,
            extra_parameters: None,
            custom_headers: provider_config.custom_headers.clone(),
            context_window: m.context_window,
            input_cost_per_m: m.input_cost_per_m,
            output_cost_per_m: m.output_cost_per_m,
        };

        if let Some(default) = &provider_config.default_config {
            config = default.merge(&config);
        }

        Ok(config)
    }

    /// Merges task payload overrides into the resolved configuration.
    pub(super) fn merge_payload_overrides(
        &self,
        config: &mut ModelConfig,
        payload: &TaskPayload,
        is_swarm_child: bool,
    ) {
        if !is_swarm_child {
            if let Some(p) = payload.provider {
                // SEC-02: Resolve Provider ID to protocol if it exists in registry
                if let Some(provider_config) = self.state.registry.providers.get(&p.to_string()) {
                    config.provider = provider_config.protocol;
                    if payload.api_key.is_none() {
                        config.api_key = provider_config.api_key.clone();
                    }
                    if payload.base_url.is_none() {
                        config.base_url = provider_config.base_url.clone();
                    }
                    if payload.external_id.is_none() {
                        config.external_id = provider_config.external_id.clone();
                    }
                    if config.custom_headers.is_none() {
                        config.custom_headers = provider_config.custom_headers.clone();
                    }
                } else {
                    config.provider = p;
                }
            } else {
                // Fallback: Ensure secrets are pulled from backend registry if protocol matches
                let current_p = config.provider.to_string();
                if let Some(provider_config) = self.state.registry.providers.get(&current_p) {
                    if config.api_key.is_none() {
                        config.api_key = provider_config.api_key.clone();
                    }
                    if config.base_url.is_none() {
                        config.base_url = provider_config.base_url.clone();
                    }
                    if config.external_id.is_none() {
                        config.external_id = provider_config.external_id.clone();
                    }
                    if config.custom_headers.is_none() {
                        config.custom_headers = provider_config.custom_headers.clone();
                    }
                }
            }
        } else {
            // Swarm child: preserve child's configured provider, hydrate missing secrets/base_url from registry
            let current_p = config.provider.to_string();
            if let Some(provider_config) = self.state.registry.providers.get(&current_p) {
                if config.api_key.is_none() {
                    config.api_key = provider_config.api_key.clone();
                }
                if config.base_url.is_none() {
                    config.base_url = provider_config.base_url.clone();
                }
                if config.external_id.is_none() {
                    config.external_id = provider_config.external_id.clone();
                }
                if config.custom_headers.is_none() {
                    config.custom_headers = provider_config.custom_headers.clone();
                }
            }
        }

        // Provider hijacking defense: subagent task payloads cannot override credentials, base_url, or external_id
        if is_swarm_child {
            if payload.api_key.is_some()
                || payload.base_url.is_some()
                || payload.external_id.is_some()
            {
                tracing::warn!("⚠️ [Security] Discarding api_key/base_url/external_id override in subagent task payload to prevent provider hijacking.");
            }
        } else {
            if let Some(key) = &payload.api_key {
                config.api_key = Some(key.clone());
            }
            if let Some(url) = &payload.base_url {
                config.base_url = Some(url.clone());
            }
            if let Some(eid) = &payload.external_id {
                config.external_id = Some(eid.clone());
            }
            if let Some(m) = &payload.model_id {
                config.model_id = m.clone();
            }
        }

        if let Some(ref mut url) = config.base_url {
            if url.contains("api.ollama.com") {
                *url = url.replace("api.ollama.com", "ollama.com");
            }
        }
    }

    /// Resolves and validates the workspace root path.
    pub(super) fn resolve_workspace_paths(
        &self,
        cluster_id: Option<&str>,
    ) -> Result<std::path::PathBuf, AppError> {
        validate_and_resolve_workspace_dir(&self.state.base_dir, cluster_id, "executive-core")
    }

    /// Applies security restrictions to the tool list if safe_mode is active.
    pub(super) fn apply_security_gates(
        &self,
        safe_mode: bool,
        skills: &mut Vec<String>,
        workflows: &mut Vec<String>,
        mcp_tools: &mut Vec<String>,
    ) {
        if safe_mode {
            skills.retain(|s| {
                if let Some(tool) = self.state.registry.tool_registry.get(s) {
                    !tool.is_dangerous()
                } else if let Some(skill_manifest) = self.state.registry.skills.skills.get(s) {
                    !skill_manifest.oversight_required
                } else {
                    // Block unknown tools in safe mode by default
                    false
                }
            });
            workflows.clear();
            mcp_tools.clear();
        }
    }
}
