//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / oversight ledger
//!
//! ### AI Assist Note
//! - Keep each oversight boundary in its own focused route module.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::*;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct OversightSettingsPayload {
    #[serde(default)]
    pub dispatch_paused: Option<bool>,
    pub auto_approve_safe_skills: Option<bool>,
    pub privacy_mode: Option<bool>,
    pub cluster_privacy_policies: Option<std::collections::HashMap<String, bool>>,
    pub max_agents: Option<u32>,
    pub max_clusters: Option<u32>,
    pub max_swarm_depth: Option<u32>,
    pub max_task_length: Option<usize>,
    pub default_budget_usd: Option<f64>,
    pub default_model: Option<String>,
    pub default_provider: Option<String>,
}

/// GET /v1/oversight/settings
///
/// Retrieves global governance constraints, including swarm limits,
/// auto-approval policies, and per-cluster privacy shield policies.
///
/// @docs OPERATIONS_MANUAL:GovernanceSettings
#[tracing::instrument(skip(state), name = "governance::get_settings")]
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let cluster_policies: std::collections::HashMap<String, bool> = state
        .governance
        .cluster_privacy_policies
        .iter()
        .map(|kv| (kv.key().clone(), *kv.value()))
        .collect();

    let payload = OversightSettingsPayload {
        dispatch_paused: Some(
            state
                .comms
                .engine_stopped
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        auto_approve_safe_skills: Some(
            state
                .governance
                .auto_approve_safe_skills
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        privacy_mode: Some(
            state
                .governance
                .privacy_mode
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        cluster_privacy_policies: Some(cluster_policies),
        max_agents: Some(
            state
                .governance
                .max_agents
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        max_clusters: Some(
            state
                .governance
                .max_clusters
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        max_swarm_depth: Some(
            state
                .governance
                .max_swarm_depth
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        max_task_length: Some(
            state
                .governance
                .max_task_length
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        default_budget_usd: Some(*state.governance.default_budget_usd.read()),
        default_model: Some(state.governance.default_model.read().clone()),
        default_provider: Some(state.governance.default_provider.read().clone()),
    };

    Ok(Json(payload))
}

/// PUT /v1/oversight/settings
///
/// Updates global governance constraints, including swarm depth,
/// auto-approval policies, and budget limitations.
///
/// @docs OPERATIONS_MANUAL:GovernanceSettings
#[tracing::instrument(skip(state, payload), name = "governance::update_settings")]
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<OversightSettingsPayload>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validation First (Atomic Guard: never partially apply invalid configurations)
    if let Some(val) = payload.max_agents {
        if !(MIN_AGENTS_FLOOR..=MAX_AGENTS_CEILING).contains(&val) {
            return Err(AppError::BadRequest(format!(
                "max_agents must be between {} and {}",
                MIN_AGENTS_FLOOR, MAX_AGENTS_CEILING
            )));
        }
    }
    if let Some(val) = payload.max_clusters {
        if !(MIN_CLUSTERS_FLOOR..=MAX_CLUSTERS_CEILING).contains(&val) {
            return Err(AppError::BadRequest(format!(
                "max_clusters must be between {} and {}",
                MIN_CLUSTERS_FLOOR, MAX_CLUSTERS_CEILING
            )));
        }
    }
    if let Some(val) = payload.max_swarm_depth {
        if !(MIN_SWARM_DEPTH_FLOOR..=MAX_SWARM_DEPTH_CEILING).contains(&val) {
            return Err(AppError::BadRequest(format!(
                "max_swarm_depth must be between {} and {}",
                MIN_SWARM_DEPTH_FLOOR, MAX_SWARM_DEPTH_CEILING
            )));
        }
    }
    if let Some(val) = payload.max_task_length {
        if !(MIN_TASK_LENGTH_FLOOR..=MAX_TASK_LENGTH_CEILING).contains(&val) {
            return Err(AppError::BadRequest(format!(
                "max_task_length must be between {} and {}",
                MIN_TASK_LENGTH_FLOOR, MAX_TASK_LENGTH_CEILING
            )));
        }
    }
    if let Some(val) = payload.default_budget_usd {
        if !val.is_finite() || !(MIN_BUDGET_USD..=MAX_BUDGET_USD).contains(&val) {
            return Err(AppError::BadRequest(format!(
                "default_budget_usd must be a finite non-negative number <= {}",
                MAX_BUDGET_USD
            )));
        }
    }
    if let Some(ref val) = payload.default_model {
        let trimmed = val.trim();
        if trimmed.is_empty()
            || trimmed.len() > 128
            || !trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':'))
        {
            return Err(AppError::BadRequest(
                "default_model contains invalid characters or exceeds 128 bytes".to_string(),
            ));
        }
    }
    if let Some(ref val) = payload.default_provider {
        let trimmed = val.trim();
        if trimmed.is_empty()
            || trimmed.len() > 64
            || !trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            return Err(AppError::BadRequest(
                "default_provider contains invalid characters or exceeds 64 bytes".to_string(),
            ));
        }
    }

    // 2. Apply Validated Settings
    if let Some(val) = payload.auto_approve_safe_skills {
        state
            .governance
            .auto_approve_safe_skills
            .store(val, std::sync::atomic::Ordering::Release);
    }

    if let Some(val) = payload.privacy_mode {
        state
            .governance
            .privacy_mode
            .store(val, std::sync::atomic::Ordering::Release);

        if val {
            tracing::info!(
                "🛡️ Privacy Shield ENABLED: Hard-blocking external API requests (OpenAI/Anthropic/Google)."
            );
        }

        state.broadcast_sys(
            &format!(
                "🛡️ Privacy Shield: Mode set to {}. {}",
                if val {
                    "ON (Local-First)"
                } else {
                    "OFF (Hybrid)"
                },
                if val {
                    "External APIs are hard-blocked."
                } else {
                    ""
                }
            ),
            if val { "success" } else { "info" },
            None,
        );
    }

    if let Some(ref policies) = payload.cluster_privacy_policies {
        for (cluster_id, mode) in policies {
            state
                .governance
                .cluster_privacy_policies
                .insert(cluster_id.clone(), *mode);
            tracing::info!(
                "🛡️ Cluster Privacy Shield updated: cluster '{}' privacy_mode={}",
                cluster_id,
                mode
            );
        }
    }

    if let Some(val) = payload.max_agents {
        state
            .governance
            .max_agents
            .store(val, std::sync::atomic::Ordering::Release);
    }
    if let Some(val) = payload.max_clusters {
        state
            .governance
            .max_clusters
            .store(val, std::sync::atomic::Ordering::Release);
    }
    if let Some(val) = payload.max_swarm_depth {
        state
            .governance
            .max_swarm_depth
            .store(val, std::sync::atomic::Ordering::Release);
    }
    if let Some(val) = payload.max_task_length {
        state
            .governance
            .max_task_length
            .store(val, std::sync::atomic::Ordering::Release);
    }
    if let Some(val) = payload.default_budget_usd {
        *state.governance.default_budget_usd.write() = val;
    }
    let default_model_val = if let Some(ref val) = payload.default_model {
        *state.governance.default_model.write() = val.clone();
        val.clone()
    } else {
        state.governance.default_model.read().clone()
    };
    let default_provider_val = if let Some(ref val) = payload.default_provider {
        *state.governance.default_provider.write() = val.clone();
        val.clone()
    } else {
        state.governance.default_provider.read().clone()
    };

    tracing::info!(
        "🛡️ Governance updated: Auto-Approve={:?}, MaxAgents={:?}, MaxClusters={:?}, MaxDepth={:?}, DefaultModel={}, DefaultProvider={}",
        payload.auto_approve_safe_skills,
        payload.max_agents,
        payload.max_clusters,
        payload.max_swarm_depth,
        default_model_val,
        default_provider_val
    );

    let audit_params = serde_json::to_string(&payload).unwrap_or_default();
    if let Err(e) = state
        .security
        .audit_trail
        .record("admin", None, None, "[SETTINGS_UPDATE]", &audit_params)
        .await
    {
        tracing::error!(
            "🚨 Failed to record [SETTINGS_UPDATE] in audit trail: {:?}",
            e
        );
    }

    let state_clone = state.clone();
    tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            state_clone.save_governance_settings();
        }),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Saving settings timed out".to_string()))?
    .map_err(|e| AppError::InternalServerError(format!("Spawn blocking failed: {}", e)))?;

    let cluster_policies: std::collections::HashMap<String, bool> = state
        .governance
        .cluster_privacy_policies
        .iter()
        .map(|kv| (kv.key().clone(), *kv.value()))
        .collect();

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "auto_approve_safe_skills": state.governance.auto_approve_safe_skills.load(std::sync::atomic::Ordering::Acquire),
            "max_agents": state.governance.max_agents.load(std::sync::atomic::Ordering::Acquire),
            "max_clusters": state.governance.max_clusters.load(std::sync::atomic::Ordering::Acquire),
            "max_swarm_depth": state.governance.max_swarm_depth.load(std::sync::atomic::Ordering::Acquire),
            "max_task_length": state.governance.max_task_length.load(std::sync::atomic::Ordering::Acquire),
            "default_budget_usd": *state.governance.default_budget_usd.read(),
            "default_model": state.governance.default_model.read().clone(),
            "default_provider": state.governance.default_provider.read().clone(),
            "privacy_mode": state.governance.privacy_mode.load(std::sync::atomic::Ordering::Acquire),
            "cluster_privacy_policies": cluster_policies
        })),
    ))
}
