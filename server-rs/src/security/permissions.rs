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
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Permission denial for unrecognized tools, 
//!   UI-blocking during waiting-for-approval states, or 
//!   misconfiguration of the internal tool whitelist.
//! - **Trace Scope**: `server-rs::security::permissions`

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum PermissionMode {
    /// Tool is always allowed (e.g., read-only safe commands).
    Allow,
    /// Tool is always denied (e.g., restricted system access).
    Deny,
    /// Tool execution is paused until the user provides explicit approval.
    Prompt,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PermissionDecision {
    pub mode: PermissionMode,
    pub reason: Option<String>,
}

pub trait PermissionPrompter: Send + Sync {
    /// Prompts the user for a decision on a pending tool execution.
    /// This may be implemented via a Tauri modal or a CLI prompt.
    fn prompt_user(&self, tool_name: &str, arguments: &str) -> anyhow::Result<PermissionMode>;
}

pub struct PermissionPolicy {
    pool: sqlx::SqlitePool,
    cache: dashmap::DashMap<String, PermissionMode>,
}

impl PermissionPolicy {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self {
            pool,
            cache: dashmap::DashMap::new(),
        }
    }

    /// Reloads the policy cache from the database.
    pub async fn refresh_cache(&self) -> anyhow::Result<()> {
        let rows: Vec<(String, String)> = sqlx::query_as("SELECT tool_name, mode FROM permission_policies")
            .fetch_all(&self.pool)
            .await?;

        self.cache.clear();
        for (name, mode_str) in rows {
            if let Ok(mode) = mode_str.parse::<PermissionMode>() {
                self.cache.insert(name, mode);
            }
        }

        tracing::info!("✅ [Security] Permission cache refreshed ({} entries).", self.cache.len());
        Ok(())
    }

    /// Determines the default permission mode for a tool.
    /// Checks the cache first, falling back to database query if missing.
    #[tracing::instrument(skip(self), fields(tool = tool_name), name = "security::get_permission_mode")]
    pub async fn get_mode(&self, tool_name: &str) -> PermissionMode {
        // 1. Try Cache
        if let Some(mode) = self.cache.get(tool_name) {
            return *mode;
        }

        // 2. Try DB
        let res: Result<(String,), sqlx::Error> = sqlx::query_as("SELECT mode FROM permission_policies WHERE tool_name = ?")
            .bind(tool_name)
            .fetch_one(&self.pool)
            .await;

        if let Ok((mode_str,)) = res {
            if let Ok(mode) = mode_str.parse::<PermissionMode>() {
                self.cache.insert(tool_name.to_string(), mode);
                return mode;
            }
        }

        // Default to prompt for unknown tools (Sovereign Safety First)
        PermissionMode::Prompt
    }
}
