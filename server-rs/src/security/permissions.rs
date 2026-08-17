//! Sovereign Permission System - Granular Tool Authorization
//!
//! Provides a strict authorization layer ensuring the user retains final
//! control over destructive, sensitive, or high-cost tool executions.
//!
//! @docs ARCHITECTURE:SecurityModel
//!
//! ### AI Assist Note
//! **Sovereign Permission System**: Orchestrates the granular
//! **Tool Authorization** layer for the Tadpole OS engine. Enforces
//! the **Sovereign Safety** principle: any tool not explicitly
//! **Whitelisted** (`Allow`) or **Guardrailed** (`Prompt`) defaults
//! to a manual user approval cycle. AI agents must check
//! `PermissionMode` before attempting high-risk or high-cost
//! executions (filesystem writes, external network access) to
//! ensure the user retains final state control (PERM-01).
//!
//! ### Formalized Authorization Precedence Hierarchy
//!
//! | Priority Tier | Policy Source | Evaluation Order | Can Weaken? | Can Strengthen? |
//! |---|---|---|---|---|
//! | **1. Security Floor** | Mandatory Floor (`SEC-06`) | Applied to final decision | No (Floor clamps `Allow` -> `Prompt`/`Deny`) | Yes |
//! | **2. Signed Manifest** | `SEC-05` Active Manifest | Overrides Floor if active & verified | Yes (with cryptographically verified key) | Yes |
//! | **3. Agent Override** | `agent_capability_policies` | Evaluated 1st in candidate search | Yes | Yes |
//! | **4. Role Override** | `role_capability_policies` | Evaluated 2nd in candidate search | Yes | Yes |
//! | **5. Global Capability** | `capability_policies` | Evaluated 3rd in candidate search | Yes | Yes |
//! | **6. Domain Policy** | Inferred Namespace (`infer_domain`) | Evaluated 4th in candidate search | Yes | Yes |
//! | **7. Legacy Tool Policy**| `permission_policies` | Fallback for `Execute` capability | Yes | Yes |
//! | **8. Default Fallback** | Sovereign Default Safety | Evaluated last | No | No (Defaults to `Prompt`) |
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Permission denial for unrecognized tools,
//!   UI-blocking during waiting-for-approval states, or
//!   misconfiguration of the internal tool whitelist.
//! - **Trace Scope**: `server-rs::security::permissions`

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum PermissionMode {
    /// Tool is always allowed (e.g., read-only safe commands).
    Allow,
    /// Tool is always denied (e.g., restricted system access).
    Deny,
    /// Tool execution is paused until the user provides explicit approval.
    Prompt,
}

impl PermissionMode {
    #[allow(dead_code)]
    pub fn as_canonical_str(&self) -> &'static str {
        match self {
            PermissionMode::Allow => "allow",
            PermissionMode::Deny => "deny",
            PermissionMode::Prompt => "prompt",
        }
    }

    /// Clamps permission mode against a mandatory security floor (SEC-06).
    /// Controls can be tightened (e.g. Prompt -> Deny) but never weakened below the floor.
    #[allow(dead_code)]
    pub fn clamp_to_floor(self, floor: PermissionMode) -> PermissionMode {
        match (self, floor) {
            (PermissionMode::Deny, _) => PermissionMode::Deny,
            (PermissionMode::Allow, PermissionMode::Prompt) => PermissionMode::Prompt,
            (PermissionMode::Allow, PermissionMode::Deny) => PermissionMode::Deny,
            (PermissionMode::Prompt, PermissionMode::Deny) => PermissionMode::Deny,
            (mode, _) => mode,
        }
    }
}

impl std::str::FromStr for PermissionMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "allow" => Ok(PermissionMode::Allow),
            "deny" => Ok(PermissionMode::Deny),
            "prompt" => Ok(PermissionMode::Prompt),
            _ => Err(anyhow::anyhow!("Invalid permission mode: {}", s)),
        }
    }
}

impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionMode::Allow => write!(f, "Allow"),
            PermissionMode::Deny => write!(f, "Deny"),
            PermissionMode::Prompt => write!(f, "Prompt"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum CapabilityClass {
    Execute,
    Install,
    Modify,
    Delete,
    Approve,
}

impl CapabilityClass {
    #[allow(dead_code)]
    pub fn as_canonical_str(&self) -> &'static str {
        match self {
            CapabilityClass::Execute => "execute",
            CapabilityClass::Install => "install",
            CapabilityClass::Modify => "modify",
            CapabilityClass::Delete => "delete",
            CapabilityClass::Approve => "approve",
        }
    }

    /// Returns the non-overridable security floor (SEC-06).
    #[allow(dead_code)]
    pub fn mandatory_floor(&self) -> PermissionMode {
        match self {
            CapabilityClass::Execute => PermissionMode::Prompt,
            CapabilityClass::Install => PermissionMode::Prompt,
            CapabilityClass::Modify => PermissionMode::Prompt,
            CapabilityClass::Delete => PermissionMode::Prompt,
            CapabilityClass::Approve => PermissionMode::Deny,
        }
    }
}

impl std::str::FromStr for CapabilityClass {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "execute" => Ok(CapabilityClass::Execute),
            "install" => Ok(CapabilityClass::Install),
            "modify" => Ok(CapabilityClass::Modify),
            "delete" => Ok(CapabilityClass::Delete),
            "approve" => Ok(CapabilityClass::Approve),
            _ => Err(anyhow::anyhow!("Invalid capability class: {}", s)),
        }
    }
}

impl std::fmt::Display for CapabilityClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityClass::Execute => write!(f, "Execute"),
            CapabilityClass::Install => write!(f, "Install"),
            CapabilityClass::Modify => write!(f, "Modify"),
            CapabilityClass::Delete => write!(f, "Delete"),
            CapabilityClass::Approve => write!(f, "Approve"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionDecisionSource {
    AgentPolicy,
    RolePolicy,
    GlobalCapabilityPolicy,
    GlobalLegacyPolicy,
    DomainPolicy,
    DefaultPrompt,
    DatabaseFailure,
    SecurityFloor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionDecision {
    pub mode: PermissionMode,
    pub source: PermissionDecisionSource,
    pub reason: Option<String>,
}

impl PermissionDecision {
    pub fn new(
        mode: PermissionMode,
        source: PermissionDecisionSource,
        reason: Option<String>,
    ) -> Self {
        Self {
            mode,
            source,
            reason,
        }
    }

    pub fn with_reason(
        mode: PermissionMode,
        source: PermissionDecisionSource,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            mode,
            source,
            reason: Some(reason.into()),
        }
    }
}

/// Strongly typed cache key for zero-allocation structural matching & invalidation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PolicyCacheKey {
    Legacy {
        agent_id: Option<String>,
        role: Option<String>,
        tool_name: String,
    },
    Capability {
        agent_id: Option<String>,
        role: Option<String>,
        capability: CapabilityClass,
        resource: String,
    },
}

pub trait PermissionPrompter: Send + Sync {
    /// Prompts the user for a decision on a pending tool execution.
    /// This may be implemented via a Tauri modal or a CLI prompt.
    fn prompt_user(&self, tool_name: &str, arguments: &str) -> anyhow::Result<PermissionMode>;
}

pub struct PermissionPolicy {
    pool: sqlx::SqlitePool,
    #[allow(dead_code)]
    prompter: Option<std::sync::Arc<dyn PermissionPrompter>>,
    decision_cache: dashmap::DashMap<PolicyCacheKey, PermissionDecision>,
}

impl PermissionPolicy {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self {
            pool,
            prompter: None,
            decision_cache: dashmap::DashMap::new(),
        }
    }

    pub fn pool(&self) -> &sqlx::SqlitePool {
        &self.pool
    }

    /// Construct typed cache key for legacy tool lookups.
    pub fn legacy_cache_key(
        agent_id: Option<&str>,
        role: Option<&str>,
        tool_name: &str,
    ) -> PolicyCacheKey {
        PolicyCacheKey::Legacy {
            agent_id: agent_id.map(|s| s.to_string()),
            role: role.map(|s| s.to_string()),
            tool_name: tool_name.to_string(),
        }
    }

    /// Construct typed cache key for capability lookups.
    pub fn capability_cache_key(
        agent_id: Option<&str>,
        role: Option<&str>,
        capability: CapabilityClass,
        resource: &str,
    ) -> PolicyCacheKey {
        PolicyCacheKey::Capability {
            agent_id: agent_id.map(|s| s.to_string()),
            role: role.map(|s| s.to_string()),
            capability,
            resource: resource.to_string(),
        }
    }

    /// Normalize resource path separators and infer domain namespace using strict path-anchors.
    pub fn infer_domain(resource: &str) -> Option<String> {
        let stripped = resource
            .strip_prefix("path:")
            .unwrap_or(resource)
            .replace('\\', "/");

        if stripped.starts_with("domain:") {
            return Some(stripped);
        }

        // Collapse duplicate slashes and path components
        let parts: Vec<&str> = stripped
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .collect();

        if parts.is_empty() {
            return None;
        }

        for (i, part) in parts.iter().enumerate() {
            if (*part == ".agent" && parts.get(i + 1) == Some(&"skills")) || *part == "skills" {
                return Some("domain:skills".to_string());
            }
            if *part == "directives" {
                return Some("domain:directives".to_string());
            }
            if *part == "execution" || *part == "scripts" {
                return Some("domain:execution".to_string());
            }
            if *part == "server-rs" {
                return Some("domain:system".to_string());
            }
        }
        None
    }

    /// Reloads the policy cache from database tables using typed keys.
    pub async fn refresh_cache(&self) -> anyhow::Result<()> {
        self.decision_cache.clear();

        // 1. Load legacy global policies
        let legacy_rows: Vec<(String, String)> =
            sqlx::query_as("SELECT tool_name, mode FROM permission_policies")
                .fetch_all(&self.pool)
                .await?;

        for (name, mode_str) in legacy_rows {
            if let Ok(mode) = mode_str.parse::<PermissionMode>() {
                let key = Self::legacy_cache_key(None, None, &name);
                self.decision_cache.insert(
                    key,
                    PermissionDecision::new(
                        mode,
                        PermissionDecisionSource::GlobalLegacyPolicy,
                        None,
                    ),
                );
            }
        }

        // 2. Load global capability policies
        let cap_rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT capability_class, resource_pattern, mode FROM capability_policies",
        )
        .fetch_all(&self.pool)
        .await?;

        for (cap_str, pattern, mode_str) in cap_rows {
            if let (Ok(cap), Ok(mode)) = (
                cap_str.parse::<CapabilityClass>(),
                mode_str.parse::<PermissionMode>(),
            ) {
                let key = Self::capability_cache_key(None, None, cap, &pattern);
                self.decision_cache.insert(
                    key,
                    PermissionDecision::new(
                        mode,
                        PermissionDecisionSource::GlobalCapabilityPolicy,
                        None,
                    ),
                );
            }
        }

        tracing::info!(
            "✅ [Security] Permission cache refreshed ({} entries).",
            self.decision_cache.len()
        );
        Ok(())
    }

    /// Helper for DB query execution distinguishing between RowNotFound vs infrastructure error.
    async fn fetch_policy_mode(
        &self,
        query: &'static str,
        binds: &[&str],
    ) -> Result<Option<PermissionMode>, anyhow::Error> {
        let mut q = sqlx::query_as::<_, (String,)>(query);
        for b in binds {
            q = q.bind(b);
        }

        match q.fetch_one(&self.pool).await {
            Ok((mode_str,)) => match mode_str.parse::<PermissionMode>() {
                Ok(mode) => Ok(Some(mode)),
                Err(e) => {
                    tracing::error!("⚠️ [Security] Malformed permission mode in DB: {}", e);
                    Err(anyhow::anyhow!("Malformed permission mode: {}", e))
                }
            },
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => {
                tracing::error!(
                    "🚨 [Security] Database error during permission evaluation: {}",
                    e
                );
                Err(anyhow::Error::new(e))
            }
        }
    }

    #[allow(dead_code)]
    async fn evaluate_raw_capability_decision(
        &self,
        agent_id: Option<&str>,
        role: Option<&str>,
        capability: CapabilityClass,
        resource: &str,
    ) -> PermissionDecision {
        let cap_str = capability.to_string();
        let domain_opt = Self::infer_domain(resource);
        let domain_different = domain_opt.as_deref().map_or(false, |d| d != resource);

        let candidates: [(&str, bool); 2] = [
            (resource, false),
            (domain_opt.as_deref().unwrap_or(""), true),
        ];
        let count = if domain_different { 2 } else { 1 };

        for &(target, is_domain) in &candidates[..count] {
            // A. Check Agent-Specific Policy
            if let Some(aid) = agent_id {
                match self
                    .fetch_policy_mode(
                        "SELECT mode FROM agent_capability_policies WHERE agent_id = ? AND capability_class = ? AND resource_pattern = ?",
                        &[aid, &cap_str, target],
                    )
                    .await
                {
                    Ok(Some(mode)) => {
                        let src = if is_domain { PermissionDecisionSource::DomainPolicy } else { PermissionDecisionSource::AgentPolicy };
                        return PermissionDecision::with_reason(mode, src, format!("Agent policy override for {}", target));
                    }
                    Err(_) => {
                        return PermissionDecision::with_reason(PermissionMode::Deny, PermissionDecisionSource::DatabaseFailure, "Infrastructure DB failure");
                    }
                    Ok(None) => {}
                }
            }

            // B. Check Role-Based Policy
            if let Some(r) = role {
                match self
                    .fetch_policy_mode(
                        "SELECT mode FROM role_capability_policies WHERE role = ? AND capability_class = ? AND resource_pattern = ?",
                        &[r, &cap_str, target],
                    )
                    .await
                {
                    Ok(Some(mode)) => {
                        let src = if is_domain { PermissionDecisionSource::DomainPolicy } else { PermissionDecisionSource::RolePolicy };
                        return PermissionDecision::with_reason(mode, src, format!("Role policy override for {}", target));
                    }
                    Err(_) => return PermissionDecision::with_reason(PermissionMode::Deny, PermissionDecisionSource::DatabaseFailure, "Infrastructure DB failure"),
                    Ok(None) => {}
                }
            }

            // C. Check Global Capability Policy
            match self
                .fetch_policy_mode(
                    "SELECT mode FROM capability_policies WHERE capability_class = ? AND resource_pattern = ?",
                    &[&cap_str, target],
                )
                .await
            {
                Ok(Some(mode)) => {
                    let src = if is_domain { PermissionDecisionSource::DomainPolicy } else { PermissionDecisionSource::GlobalCapabilityPolicy };
                    return PermissionDecision::with_reason(mode, src, format!("Global capability policy for {}", target));
                }
                Err(_) => return PermissionDecision::with_reason(PermissionMode::Deny, PermissionDecisionSource::DatabaseFailure, "Infrastructure DB failure"),
                Ok(None) => {}
            }
        }

        // D. Legacy Fallback for Execute capability matching existing tool policies
        if capability == CapabilityClass::Execute {
            let legacy_decision = self.get_mode_decision(agent_id, role, resource).await;
            if legacy_decision.source != PermissionDecisionSource::DefaultPrompt {
                return legacy_decision;
            }
        }

        // Default to Prompt for unmapped capabilities (Sovereign Safety First)
        PermissionDecision::with_reason(
            PermissionMode::Prompt,
            PermissionDecisionSource::DefaultPrompt,
            "Unmapped capability defaults to prompt",
        )
    }

    /// Checks if a cryptographically verified signed capability manifest (SEC-05) is currently active for resource.
    #[allow(dead_code)]
    pub async fn is_signed_capability_active(
        &self,
        capability: CapabilityClass,
        resource: &str,
    ) -> bool {
        let cap_str = capability.to_string();
        // Escape SQL LIKE wildcards in resource parameter to prevent wildcard expansion
        let escaped_resource = resource.replace('%', "\\%").replace('_', "\\_");
        let res: Result<Option<(i32,)>, _> = sqlx::query_as(
            "SELECT 1 FROM signed_capability_manifests WHERE risk_class = ? AND (resource_pattern = ? OR ? LIKE resource_pattern ESCAPE '\\') AND status = 'active' AND (expiration IS NULL OR expiration > CURRENT_TIMESTAMP) LIMIT 1"
        )
        .bind(cap_str)
        .bind(resource)
        .bind(escaped_resource)
        .fetch_optional(&self.pool)
        .await;

        matches!(res, Ok(Some(_)))
    }

    /// Evaluates permission under granular CapabilityClass returning full decision metadata with SEC-06 clamping.
    #[allow(dead_code)]
    #[tracing::instrument(skip(self), fields(agent = ?agent_id, role = ?role, capability = %capability, resource = resource), name = "security::check_capability_decision")]
    pub async fn check_capability_decision(
        &self,
        agent_id: Option<&str>,
        role: Option<&str>,
        capability: CapabilityClass,
        resource: &str,
    ) -> PermissionDecision {
        let cache_key = Self::capability_cache_key(agent_id, role, capability, resource);

        // Memory Bounding: Prune cache if it exceeds 10,000 entries
        if self.decision_cache.len() > 10_000 {
            self.decision_cache.clear();
        }

        // 1. Try Decision Cache
        if let Some(decision) = self.decision_cache.get(&cache_key) {
            return decision.clone();
        }

        let raw_decision = self
            .evaluate_raw_capability_decision(agent_id, role, capability, resource)
            .await;
        let floor = capability.mandatory_floor();
        let effective_mode = raw_decision.mode.clamp_to_floor(floor);

        let (final_decision, cacheable) = if effective_mode != raw_decision.mode {
            if self.is_signed_capability_active(capability, resource).await {
                // Do NOT cache signed-manifest floor overrides long-term without expiration checking
                (raw_decision, false)
            } else {
                (
                    PermissionDecision::with_reason(
                        effective_mode,
                        PermissionDecisionSource::SecurityFloor,
                        format!(
                            "Clamped by SEC-06 mandatory security floor (evaluated {:?} clamped to {:?} for capability {})",
                            raw_decision.mode, effective_mode, capability
                        ),
                    ),
                    true,
                )
            }
        } else {
            (raw_decision, true)
        };

        if cacheable {
            self.decision_cache
                .insert(cache_key, final_decision.clone());
        }
        final_decision
    }

    /// Evaluates permission under granular CapabilityClass returning mode only.
    #[allow(dead_code)]
    pub async fn check_capability(
        &self,
        agent_id: Option<&str>,
        role: Option<&str>,
        capability: CapabilityClass,
        resource: &str,
    ) -> PermissionMode {
        self.check_capability_decision(agent_id, role, capability, resource)
            .await
            .mode
    }

    /// Determines the permission decision for a tool under agent and role isolation.
    #[tracing::instrument(skip(self), fields(agent = ?agent_id, role = ?role, tool = tool_name), name = "security::get_mode_decision")]
    pub async fn get_mode_decision(
        &self,
        agent_id: Option<&str>,
        role: Option<&str>,
        tool_name: &str,
    ) -> PermissionDecision {
        let cache_key = Self::legacy_cache_key(agent_id, role, tool_name);

        // 1. Try Decision Cache
        if let Some(decision) = self.decision_cache.get(&cache_key) {
            return decision.clone();
        }

        // 2. Try Agent-Specific Policy
        if let Some(aid) = agent_id {
            match self.fetch_policy_mode(
                "SELECT mode FROM agent_permission_policies WHERE agent_id = ? AND tool_name = ?",
                &[aid, tool_name],
            ).await {
                Ok(Some(mode)) => {
                    let decision = PermissionDecision::with_reason(mode, PermissionDecisionSource::AgentPolicy, format!("Agent legacy override for {}", tool_name));
                    self.decision_cache.insert(cache_key, decision.clone());
                    return decision;
                }
                Err(_) => return PermissionDecision::with_reason(PermissionMode::Deny, PermissionDecisionSource::DatabaseFailure, "Infrastructure DB failure"),
                Ok(None) => {}
            }
        }

        // 3. Try Role-Based Policy
        if let Some(r) = role {
            match self
                .fetch_policy_mode(
                    "SELECT mode FROM role_permission_policies WHERE role = ? AND tool_name = ?",
                    &[r, tool_name],
                )
                .await
            {
                Ok(Some(mode)) => {
                    let decision = PermissionDecision::with_reason(
                        mode,
                        PermissionDecisionSource::RolePolicy,
                        format!("Role legacy override for {}", tool_name),
                    );
                    self.decision_cache.insert(cache_key, decision.clone());
                    return decision;
                }
                Err(_) => {
                    return PermissionDecision::with_reason(
                        PermissionMode::Deny,
                        PermissionDecisionSource::DatabaseFailure,
                        "Infrastructure DB failure",
                    )
                }
                Ok(None) => {}
            }
        }

        // 4. Try Global Policy
        match self
            .fetch_policy_mode(
                "SELECT mode FROM permission_policies WHERE tool_name = ?",
                &[tool_name],
            )
            .await
        {
            Ok(Some(mode)) => {
                let decision = PermissionDecision::with_reason(
                    mode,
                    PermissionDecisionSource::GlobalLegacyPolicy,
                    format!("Global legacy policy for {}", tool_name),
                );
                self.decision_cache.insert(cache_key, decision.clone());
                return decision;
            }
            Err(_) => {
                return PermissionDecision::with_reason(
                    PermissionMode::Deny,
                    PermissionDecisionSource::DatabaseFailure,
                    "Infrastructure DB failure",
                )
            }
            Ok(None) => {}
        }

        // Default to prompt for unknown tools (Sovereign Safety First)
        let default_decision = PermissionDecision::with_reason(
            PermissionMode::Prompt,
            PermissionDecisionSource::DefaultPrompt,
            "Unknown tool defaults to prompt",
        );
        self.decision_cache
            .insert(cache_key, default_decision.clone());
        default_decision
    }

    /// Determines the permission mode for a tool under agent and role isolation.
    pub async fn get_mode(
        &self,
        agent_id: Option<&str>,
        role: Option<&str>,
        tool_name: &str,
    ) -> PermissionMode {
        self.get_mode_decision(agent_id, role, tool_name).await.mode
    }

    /// Manually sets the permission mode for a tool (used for tests and admin updates).
    /// Evicts both legacy tool cache AND capability Execute cache entries depending on legacy fallback.
    #[allow(dead_code)]
    pub async fn set_mode(&self, tool_name: &str, mode: PermissionMode) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO permission_policies (tool_name, mode) VALUES (?, ?) ON CONFLICT(tool_name) DO UPDATE SET mode = excluded.mode")
            .bind(tool_name)
            .bind(mode.to_string().to_lowercase())
            .execute(&self.pool)
            .await?;

        // Evict matching legacy tool keys AND capability Execute keys targeting tool_name
        self.decision_cache.retain(|key, _| match key {
            PolicyCacheKey::Legacy { tool_name: t, .. } => t != tool_name,
            PolicyCacheKey::Capability {
                capability,
                resource,
                ..
            } => !(*capability == CapabilityClass::Execute && resource == tool_name),
        });
        Ok(())
    }

    /// Manually sets capability mode for testing & administration with SEC-06 mandatory floor check.
    /// Evicts capability cache entries using exact structural match.
    #[allow(dead_code)]
    pub async fn set_capability_mode(
        &self,
        capability: CapabilityClass,
        resource_pattern: &str,
        mode: PermissionMode,
    ) -> anyhow::Result<()> {
        let floor = capability.mandatory_floor();
        if mode.clamp_to_floor(floor) != mode {
            return Err(anyhow::anyhow!(
                "Cannot set permission mode '{:?}' for capability '{}' - violates SEC-06 mandatory security floor '{:?}'",
                mode, capability, floor
            ));
        }

        self.set_capability_mode_signed(capability, resource_pattern, mode)
            .await
    }

    /// Sets an agent-specific capability policy with SEC-06 mandatory floor validation.
    #[allow(dead_code)]
    pub async fn set_agent_capability_mode(
        &self,
        agent_id: &str,
        capability: CapabilityClass,
        resource_pattern: &str,
        mode: PermissionMode,
    ) -> anyhow::Result<()> {
        let floor = capability.mandatory_floor();
        if mode.clamp_to_floor(floor) != mode {
            return Err(anyhow::anyhow!(
                "Cannot set agent permission mode '{:?}' for capability '{}' - violates SEC-06 mandatory security floor '{:?}'",
                mode, capability, floor
            ));
        }

        sqlx::query("INSERT INTO agent_capability_policies (agent_id, capability_class, resource_pattern, mode) VALUES (?, ?, ?, ?) ON CONFLICT(agent_id, capability_class, resource_pattern) DO UPDATE SET mode = excluded.mode")
            .bind(agent_id)
            .bind(capability.to_string())
            .bind(resource_pattern)
            .bind(mode.to_string().to_lowercase())
            .execute(&self.pool)
            .await?;

        self.decision_cache.retain(|key, _| match key {
            PolicyCacheKey::Capability { agent_id: a, .. } => a.as_deref() != Some(agent_id),
            PolicyCacheKey::Legacy { agent_id: a, .. } => a.as_deref() != Some(agent_id),
        });
        Ok(())
    }

    /// Sets a role-specific capability policy with SEC-06 mandatory floor validation.
    #[allow(dead_code)]
    pub async fn set_role_capability_mode(
        &self,
        role: &str,
        capability: CapabilityClass,
        resource_pattern: &str,
        mode: PermissionMode,
    ) -> anyhow::Result<()> {
        let floor = capability.mandatory_floor();
        if mode.clamp_to_floor(floor) != mode {
            return Err(anyhow::anyhow!(
                "Cannot set role permission mode '{:?}' for capability '{}' - violates SEC-06 mandatory security floor '{:?}'",
                mode, capability, floor
            ));
        }

        sqlx::query("INSERT INTO role_capability_policies (role, capability_class, resource_pattern, mode) VALUES (?, ?, ?, ?) ON CONFLICT(role, capability_class, resource_pattern) DO UPDATE SET mode = excluded.mode")
            .bind(role)
            .bind(capability.to_string())
            .bind(resource_pattern)
            .bind(mode.to_string().to_lowercase())
            .execute(&self.pool)
            .await?;

        self.decision_cache.retain(|key, _| match key {
            PolicyCacheKey::Capability { role: r, .. } => r.as_deref() != Some(role),
            PolicyCacheKey::Legacy { role: r, .. } => r.as_deref() != Some(role),
        });
        Ok(())
    }

    /// Bypasses SEC-06 floor check ONLY for cryptographically verified signed capability manifests.
    pub async fn set_capability_mode_signed(
        &self,
        capability: CapabilityClass,
        resource_pattern: &str,
        mode: PermissionMode,
    ) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO capability_policies (capability_class, resource_pattern, mode) VALUES (?, ?, ?) ON CONFLICT(capability_class, resource_pattern) DO UPDATE SET mode = excluded.mode")
            .bind(capability.to_string())
            .bind(resource_pattern)
            .bind(mode.to_string().to_lowercase())
            .execute(&self.pool)
            .await?;

        self.decision_cache.retain(|key, _| match key {
            PolicyCacheKey::Capability {
                capability: c,
                resource: r,
                ..
            } => {
                if *c != capability {
                    return true;
                }
                if r == resource_pattern {
                    return false;
                }
                if let Some(dom) = Self::infer_domain(r) {
                    if dom == resource_pattern {
                        return false;
                    }
                }
                true
            }
            _ => true,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_mode_parsing() {
        assert_eq!(
            "allow".parse::<PermissionMode>().unwrap(),
            PermissionMode::Allow
        );
        assert_eq!(
            "deny".parse::<PermissionMode>().unwrap(),
            PermissionMode::Deny
        );
        assert_eq!(
            "prompt".parse::<PermissionMode>().unwrap(),
            PermissionMode::Prompt
        );
        assert!("invalid".parse::<PermissionMode>().is_err());
    }

    #[test]
    fn test_as_canonical_str() {
        assert_eq!(PermissionMode::Allow.as_canonical_str(), "allow");
        assert_eq!(PermissionMode::Deny.as_canonical_str(), "deny");
        assert_eq!(PermissionMode::Prompt.as_canonical_str(), "prompt");

        assert_eq!(CapabilityClass::Execute.as_canonical_str(), "execute");
        assert_eq!(CapabilityClass::Install.as_canonical_str(), "install");
        assert_eq!(CapabilityClass::Modify.as_canonical_str(), "modify");
        assert_eq!(CapabilityClass::Delete.as_canonical_str(), "delete");
        assert_eq!(CapabilityClass::Approve.as_canonical_str(), "approve");
    }

    #[tokio::test]
    async fn test_permission_policy_default_prompt() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE permission_policies (tool_name TEXT PRIMARY KEY, mode TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("CREATE TABLE agent_permission_policies (agent_id TEXT, tool_name TEXT, mode TEXT, PRIMARY KEY(agent_id, tool_name))").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE role_permission_policies (role TEXT, tool_name TEXT, mode TEXT, PRIMARY KEY(role, tool_name))").execute(&pool).await.unwrap();

        let policy = PermissionPolicy::new(pool);
        // Unknown tool defaults to Prompt
        let mode = policy
            .get_mode(Some("agent-1"), Some("role-1"), "unknown_tool")
            .await;
        assert_eq!(mode, PermissionMode::Prompt);
    }
}

// Metadata: [permissions]
