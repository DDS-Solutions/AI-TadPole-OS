//! @docs ARCHITECTURE:SecurityModel
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / permissions
//!
//! ### AI Assist Note
//! - One facade owns the policy cache; typed repository facts and pure evaluation live in internals.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Security]`

mod evaluation;
mod prompt;
mod repository;
mod types;

pub use prompt::{AsyncOversightPrompter, PermissionPrompter};
pub use types::{
    CapabilityClass, PermissionDecision, PermissionDecisionSource, PermissionMode, PolicyCacheKey,
};

use anyhow::{anyhow, Result};
use dashmap::DashMap;
use sqlx::SqlitePool;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Maximum lifetime of cached permission decisions before re-querying the database (5 minutes).
pub const CACHE_TTL: Duration = Duration::from_secs(300);

/// Maximum cached decisions to prevent memory unbounded growth under dynamic input.
pub const MAX_CACHE_ENTRIES: usize = 5000;

#[derive(Debug, Clone)]
struct CachedDecision {
    decision: PermissionDecision,
    cached_at: Instant,
}

pub struct PermissionPolicy {
    pool: SqlitePool,
    #[allow(dead_code)]
    prompter: Option<Arc<dyn PermissionPrompter>>,
    decision_cache: DashMap<PolicyCacheKey, CachedDecision>,
}

impl PermissionPolicy {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            prompter: None,
            decision_cache: DashMap::new(),
        }
    }

    pub fn pool(&self) -> &SqlitePool {
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

    fn get_cached_decision(&self, key: &PolicyCacheKey) -> Option<PermissionDecision> {
        if let Some(entry) = self.decision_cache.get(key) {
            if entry.cached_at.elapsed() <= CACHE_TTL {
                return Some(entry.decision.clone());
            }
        }
        None
    }

    fn insert_cached_decision(&self, key: PolicyCacheKey, decision: PermissionDecision) {
        if self.decision_cache.len() >= MAX_CACHE_ENTRIES {
            self.decision_cache
                .retain(|_, v| v.cached_at.elapsed() <= CACHE_TTL);
            if self.decision_cache.len() >= MAX_CACHE_ENTRIES {
                let mut entries: Vec<(PolicyCacheKey, Instant)> = self
                    .decision_cache
                    .iter()
                    .map(|e| (e.key().clone(), e.value().cached_at))
                    .collect();
                entries.sort_by_key(|(_, t)| *t);
                for (k, _) in entries.into_iter().take(MAX_CACHE_ENTRIES / 4) {
                    self.decision_cache.remove(&k);
                }
            }
        }
        self.decision_cache.insert(
            key,
            CachedDecision {
                decision,
                cached_at: Instant::now(),
            },
        );
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

        let parts: Vec<&str> = stripped
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .collect();

        if parts.is_empty() {
            return None;
        }

        if parts.first() == Some(&".agent") && parts.get(1) == Some(&"skills") {
            return Some("domain:skills".to_string());
        }
        if parts.first() == Some(&"skills") {
            return Some("domain:skills".to_string());
        }
        if parts.first() == Some(&"directives") {
            return Some("domain:directives".to_string());
        }
        if parts.first() == Some(&"execution") || parts.first() == Some(&"scripts") {
            return Some("domain:execution".to_string());
        }
        if parts.first() == Some(&"server-rs") {
            return Some("domain:system".to_string());
        }

        None
    }

    /// Reloads the policy cache from database tables using typed keys.
    pub async fn refresh_cache(&self) -> Result<()> {
        self.decision_cache.clear();

        // 1. Load legacy global policies
        let legacy_rows: Vec<(String, String)> =
            sqlx::query_as("SELECT tool_name, mode FROM permission_policies")
                .fetch_all(&self.pool)
                .await?;

        for (name, mode_str) in legacy_rows {
            if let Ok(mode) = mode_str.parse::<PermissionMode>() {
                let key = Self::legacy_cache_key(None, None, &name);
                self.insert_cached_decision(
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
                // A raw mode below the floor needs evaluation against a live
                // signed exception. Such decisions must never be prewarmed.
                if mode.clamp_to_floor(cap.mandatory_floor()) != mode {
                    continue;
                }
                let key = Self::capability_cache_key(None, None, cap, &pattern);
                self.insert_cached_decision(
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

    async fn evaluate_raw_capability_decision(
        &self,
        agent_id: Option<&str>,
        role: Option<&str>,
        capability: CapabilityClass,
        resource: &str,
    ) -> PermissionDecision {
        let domain = Self::infer_domain(resource);
        let repository = repository::PolicyRepository::new(&self.pool);
        for lookup in
            evaluation::capability_lookups(agent_id, role, capability, resource, domain.as_deref())
        {
            if let Some(decision) = evaluation::decision(lookup, repository.lookup(lookup).await) {
                return decision;
            }
        }
        if capability == CapabilityClass::Execute {
            let legacy = self.get_mode_decision(agent_id, role, resource).await;
            if legacy.source != PermissionDecisionSource::DefaultPrompt {
                return legacy;
            }
        }
        PermissionDecision::with_reason(
            PermissionMode::Prompt,
            PermissionDecisionSource::DefaultPrompt,
            "Unmapped capability defaults to prompt",
        )
    }

    /// Checks if a cryptographically verified signed capability manifest (SEC-05) is currently active for resource.
    pub async fn is_signed_capability_active(
        &self,
        capability: CapabilityClass,
        resource: &str,
    ) -> bool {
        repository::PolicyRepository::new(&self.pool)
            .signed_active(capability, resource)
            .await
    }

    /// Evaluates permission under granular CapabilityClass returning full decision metadata with SEC-06 clamping.
    #[tracing::instrument(skip(self), fields(agent = ?agent_id, role = ?role, capability = %capability, resource = resource), name = "security::check_capability_decision")]
    pub async fn check_capability_decision(
        &self,
        agent_id: Option<&str>,
        role: Option<&str>,
        capability: CapabilityClass,
        resource: &str,
    ) -> PermissionDecision {
        let cache_key = Self::capability_cache_key(agent_id, role, capability, resource);

        // 1. Try Decision Cache (with TTL enforcement)
        if let Some(decision) = self.get_cached_decision(&cache_key) {
            return decision;
        }

        let raw_decision = self
            .evaluate_raw_capability_decision(agent_id, role, capability, resource)
            .await;
        let floor = capability.mandatory_floor();
        let effective_mode = raw_decision.mode.clamp_to_floor(floor);

        let (final_decision, cacheable) = if effective_mode != raw_decision.mode {
            if self.is_signed_capability_active(capability, resource).await {
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
            self.insert_cached_decision(cache_key, final_decision.clone());
        }
        final_decision
    }

    /// Evaluates permission under granular CapabilityClass returning mode only.
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

        // 1. Try Decision Cache (with TTL enforcement)
        if let Some(decision) = self.get_cached_decision(&cache_key) {
            return decision;
        }

        let repository = repository::PolicyRepository::new(&self.pool);
        for lookup in evaluation::legacy_lookups(agent_id, role, tool_name) {
            if let Some(decision) = evaluation::decision(lookup, repository.lookup(lookup).await) {
                if !matches!(
                    decision.source,
                    PermissionDecisionSource::DatabaseFailure
                        | PermissionDecisionSource::PolicyDataError
                ) {
                    self.insert_cached_decision(cache_key, decision.clone());
                }
                return decision;
            }
        }

        // Default to prompt for unknown tools (Sovereign Safety First)
        let default_decision = PermissionDecision::with_reason(
            PermissionMode::Prompt,
            PermissionDecisionSource::DefaultPrompt,
            "Unknown tool defaults to prompt",
        );
        self.insert_cached_decision(cache_key, default_decision.clone());
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
    pub async fn set_mode(&self, tool_name: &str, mode: PermissionMode) -> Result<()> {
        sqlx::query("INSERT INTO permission_policies (tool_name, mode) VALUES (?, ?) ON CONFLICT(tool_name) DO UPDATE SET mode = excluded.mode")
            .bind(tool_name)
            .bind(mode.as_canonical_str())
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
    pub async fn set_capability_mode(
        &self,
        capability: CapabilityClass,
        resource_pattern: &str,
        mode: PermissionMode,
    ) -> Result<()> {
        let floor = capability.mandatory_floor();
        if mode.clamp_to_floor(floor) != mode {
            return Err(anyhow!(
                "Cannot set permission mode '{:?}' for capability '{}' - violates SEC-06 mandatory security floor '{:?}'",
                mode, capability, floor
            ));
        }

        sqlx::query("INSERT INTO capability_policies (capability_class, resource_pattern, mode) VALUES (?, ?, ?) ON CONFLICT(capability_class, resource_pattern) DO UPDATE SET mode = excluded.mode")
            .bind(capability.as_canonical_str())
            .bind(resource_pattern)
            .bind(mode.as_canonical_str())
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

    /// Sets an agent-specific capability policy with SEC-06 mandatory floor validation.
    pub async fn set_agent_capability_mode(
        &self,
        agent_id: &str,
        capability: CapabilityClass,
        resource_pattern: &str,
        mode: PermissionMode,
    ) -> Result<()> {
        let floor = capability.mandatory_floor();
        if mode.clamp_to_floor(floor) != mode {
            return Err(anyhow!(
                "Cannot set agent permission mode '{:?}' for capability '{}' - violates SEC-06 mandatory security floor '{:?}'",
                mode, capability, floor
            ));
        }

        sqlx::query("INSERT INTO agent_capability_policies (agent_id, capability_class, resource_pattern, mode) VALUES (?, ?, ?, ?) ON CONFLICT(agent_id, capability_class, resource_pattern) DO UPDATE SET mode = excluded.mode")
            .bind(agent_id)
            .bind(capability.as_canonical_str())
            .bind(resource_pattern)
            .bind(mode.as_canonical_str())
            .execute(&self.pool)
            .await?;

        self.decision_cache.retain(|key, _| match key {
            PolicyCacheKey::Capability { agent_id: a, .. } => a.as_deref() != Some(agent_id),
            PolicyCacheKey::Legacy { agent_id: a, .. } => a.as_deref() != Some(agent_id),
        });
        Ok(())
    }

    /// Sets a role-specific capability policy with SEC-06 mandatory floor validation.
    pub async fn set_role_capability_mode(
        &self,
        role: &str,
        capability: CapabilityClass,
        resource_pattern: &str,
        mode: PermissionMode,
    ) -> Result<()> {
        let floor = capability.mandatory_floor();
        if mode.clamp_to_floor(floor) != mode {
            return Err(anyhow!(
                "Cannot set role permission mode '{:?}' for capability '{}' - violates SEC-06 mandatory security floor '{:?}'",
                mode, capability, floor
            ));
        }

        sqlx::query("INSERT INTO role_capability_policies (role, capability_class, resource_pattern, mode) VALUES (?, ?, ?, ?) ON CONFLICT(role, capability_class, resource_pattern) DO UPDATE SET mode = excluded.mode")
            .bind(role)
            .bind(capability.as_canonical_str())
            .bind(resource_pattern)
            .bind(mode.as_canonical_str())
            .execute(&self.pool)
            .await?;

        self.decision_cache.retain(|key, _| match key {
            PolicyCacheKey::Capability { role: r, .. } => r.as_deref() != Some(role),
            PolicyCacheKey::Legacy { role: r, .. } => r.as_deref() != Some(role),
        });
        Ok(())
    }

    /// Bypasses SEC-06 floor check ONLY for cryptographically verified signed capability manifests.
    pub(crate) async fn set_capability_mode_signed(
        &self,
        manifest: &crate::security::signed_capability::SignedCapabilityManifest,
    ) -> Result<()> {
        if manifest.status != crate::security::signed_capability::ManifestStatus::Active
            && manifest.status != crate::security::signed_capability::ManifestStatus::Pending
        {
            return Err(anyhow!(
                "Cannot activate capability manifest with status {:?}",
                manifest.status
            ));
        }

        sqlx::query("INSERT INTO capability_policies (capability_class, resource_pattern, mode) VALUES (?, ?, ?) ON CONFLICT(capability_class, resource_pattern) DO UPDATE SET mode = excluded.mode")
            .bind(manifest.risk_class.as_canonical_str())
            .bind(&manifest.resource_pattern)
            .bind(manifest.mode.as_canonical_str())
            .execute(&self.pool)
            .await?;

        let capability = manifest.risk_class;
        let resource_pattern = manifest.resource_pattern.clone();
        self.decision_cache.retain(|key, _| match key {
            PolicyCacheKey::Capability {
                capability: c,
                resource: r,
                ..
            } => {
                if *c != capability {
                    return true;
                }
                if *r == resource_pattern {
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
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
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
