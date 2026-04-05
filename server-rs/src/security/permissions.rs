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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionMode {
    /// Tool is always allowed (e.g., read-only safe commands).
    Allow,
    /// Tool is always denied (e.g., restricted system access).
    Deny,
    /// Tool execution is paused until the user provides explicit approval.
    Prompt,
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
    /// A set of tools that are considered safe and bypass prompts.
    whitelist: Vec<String>,
    /// A set of tools that are considered high-risk and always prompt.
    guardrail_list: Vec<String>,
}

impl PermissionPolicy {
    pub fn new() -> Self {
        Self {
            whitelist: vec![
                "ls".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "cat".to_string(),
                "read_file".to_string(),
                "list_dir".to_string(),
                "get_project_status".to_string(),
            ],
            guardrail_list: vec![
                "bash".to_string(),
                "write_to_file".to_string(),
                "delete_file".to_string(),
                "git_push".to_string(),
                "deploy_application".to_string(),
            ],
        }
    }

    /// Determines the default permission mode for a tool.
    pub fn get_mode(&self, tool_name: &str) -> PermissionMode {
        if self.whitelist.contains(&tool_name.to_string()) {
            PermissionMode::Allow
        } else if self.guardrail_list.contains(&tool_name.to_string()) {
            PermissionMode::Prompt
        } else {
            // Default to prompt for unknown tools (Sovereign Safety First)
            PermissionMode::Prompt
        }
    }
}

impl Default for PermissionPolicy {
    fn default() -> Self {
        Self::new()
    }
}
