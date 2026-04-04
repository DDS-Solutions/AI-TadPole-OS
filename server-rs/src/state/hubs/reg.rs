//! Registry Hub — Centralized directory for identities, models, and skills
//!
//! Maintains the "Brain" of the swarm, including LLM provider configurations,
//! agent role definitions, and the tool discovery catalog.
//!
//! @docs ARCHITECTURE:State

use std::sync::Arc;
use dashmap::DashMap;
use crate::agent::types::{EngineAgent, ModelEntry, ProviderConfig, SwarmNode};
use crate::agent::script_skills::ScriptSkillsRegistry;
use crate::agent::skill_manifest::SkillRegistry;
use crate::agent::mcp::McpHost;
use crate::agent::hooks::HooksManager;

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
