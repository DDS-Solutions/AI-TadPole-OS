//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **The Brain Registry**: Centralized directory for swarm identities,
//! models, and skills. Maintains **Provider Configurations** (OpenAI,
//! Ollama), **Agent Role Definitions**, and the **Tool Discovery Catalog**.
//! Features **Dynamic Skill Loading** (JSON manifests) and **MCP Tool
//! Aggregation** for a high-fidelity capability surface.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Registry cache staling due to persistence sync
//!   delays, duplicate skill IDs in the `DashMap`, or `McpHost` tool
//!   enumeration timeouts.
//! - **Trace Scope**: `server-rs::state::hubs::reg`

use crate::agent::hooks::HooksManager;
use crate::agent::mcp::McpHost;
use crate::agent::script_skills::ScriptSkillsRegistry;
use crate::agent::skill_manifest::SkillRegistry;
use crate::agent::types::{EngineAgent, ModelEntry, ProviderConfig, SwarmNode};
use dashmap::DashMap;
use std::sync::Arc;

/// Hub for agent identities, provider configs, and skill discovery.
pub struct RegistryHub {
    /// The live agent registry, synced with persistence.
    pub agents: DashMap<String, EngineAgent>,
    /// Configured LLM providers (e.g., OpenAI, Ollama).
    pub providers: DashMap<String, ProviderConfig>,
    /// Available LLM models catalog.
    pub models: DashMap<String, ModelEntry>,
    /// Discovery registry for infrastructure nodes in the swarm.
    #[allow(dead_code)]
    pub nodes: DashMap<String, SwarmNode>,
    /// Registry for dynamic file-based Skills and Workflows.
    #[allow(dead_code)]
    pub skills: Arc<ScriptSkillsRegistry>,
    /// Manager for dynamic Skill Manifests (skill.json).
    #[allow(dead_code)]
    pub skill_registry: Arc<SkillRegistry>,
    /// Host for Model Context Protocol (MCP) tool aggregation.
    #[allow(dead_code)]
    pub mcp_host: Arc<McpHost>,
    /// Manager for Lifecycle Hooks (Pre/Post tool execution).
    #[allow(dead_code)]
    pub hooks: Arc<HooksManager>,
}

impl RegistryHub {
    /// Returns a summarized list of specialized agents available in the cluster.
    /// Filters out generic or internal nodes to provide a clean recruitment directory.
    pub fn list_active_specialists(&self) -> Vec<String> {
        self.agents
            .iter()
            .filter_map(|kv| {
                let agent = kv.value();
                let id = &agent.identity.id;
 
                // Skip root/orchestrator nodes for the specialist directory
                if id == "1" || id == "2" || id == "alpha" || id == "general" {
                    return None;
                }
 
                Some(format!(
                    "- {} (Role: {}, Dept: {}): {}",
                    id, agent.identity.role, agent.identity.department, agent.identity.description
                ))
            })
            .collect()
    }
}

// Metadata: [reg]

// Metadata: [reg]
