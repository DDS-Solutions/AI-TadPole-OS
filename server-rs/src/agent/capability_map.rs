//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / capability_map
//! - **Primary Entrypoints**: `CapabilityMap`, `SharedCapabilityMap`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::agent::types::EngineAgent;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// `CapabilityMap` provides an inverted index over the agent registry for $O(1)$ specialist discovery.
/// Maps roles, word-boundary stems, skills, workflows, and names to candidate agent IDs.
#[derive(Debug, Default, Clone)]
pub struct CapabilityMap {
    /// Exact role (lowercase) -> Vec<AgentId>
    role_index: HashMap<String, Vec<String>>,
    /// Word-boundary tokens extracted from role and name -> Vec<AgentId>
    word_index: HashMap<String, Vec<String>>,
    /// Skills and workflows (lowercase) -> Vec<AgentId>
    skill_index: HashMap<String, Vec<String>>,
    /// Exact name (lowercase) -> Vec<AgentId>
    name_index: HashMap<String, Vec<String>>,
}

impl CapabilityMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rebuilds the entire inverted index from an iterator of agents.
    pub fn rebuild<'a, I>(&mut self, agents: I)
    where
        I: IntoIterator<Item = (&'a String, &'a EngineAgent)>,
    {
        self.clear();
        for (_, agent) in agents {
            self.index_agent(agent);
        }
    }

    /// Clears all inverted indices.
    pub fn clear(&mut self) {
        self.role_index.clear();
        self.word_index.clear();
        self.skill_index.clear();
        self.name_index.clear();
    }

    /// Adds or updates an agent in the index.
    pub fn update_agent(&mut self, agent: &EngineAgent) {
        self.remove_agent(&agent.identity.id);
        self.index_agent(agent);
    }

    /// Removes an agent from all inverted index buckets.
    pub fn remove_agent(&mut self, agent_id: &str) {
        let prune = |map: &mut HashMap<String, Vec<String>>| {
            map.retain(|_, ids| {
                ids.retain(|id| id != agent_id);
                !ids.is_empty()
            });
        };
        prune(&mut self.role_index);
        prune(&mut self.word_index);
        prune(&mut self.skill_index);
        prune(&mut self.name_index);
    }

    fn index_agent(&mut self, agent: &EngineAgent) {
        let id = &agent.identity.id;

        // 1. Exact Name
        let name_lower = agent.identity.name.trim().to_lowercase();
        if !name_lower.is_empty() {
            self.name_index
                .entry(name_lower)
                .or_default()
                .push(id.clone());
        }

        // 2. Exact Role
        let role_lower = agent.identity.role.trim().to_lowercase();
        if !role_lower.is_empty() {
            self.role_index
                .entry(role_lower.clone())
                .or_default()
                .push(id.clone());

            // 3. Word-boundary stems in role (>= 3 chars)
            for word in role_lower.split(|c: char| !c.is_alphanumeric()) {
                let trimmed = word.trim();
                if trimmed.len() >= 3 {
                    let entry = self.word_index.entry(trimmed.to_string()).or_default();
                    if !entry.contains(id) {
                        entry.push(id.clone());
                    }
                }
            }
        }

        // 4. Word-boundary stems in name (>= 3 chars)
        for word in agent
            .identity
            .name
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
        {
            let trimmed = word.trim();
            if trimmed.len() >= 3 {
                let entry = self.word_index.entry(trimmed.to_string()).or_default();
                if !entry.contains(id) {
                    entry.push(id.clone());
                }
            }
        }

        // 5. Skills & Workflows
        for skill in &agent.capabilities.skills {
            let skill_lower = skill.trim().to_lowercase();
            if !skill_lower.is_empty() {
                let entry = self.skill_index.entry(skill_lower).or_default();
                if !entry.contains(id) {
                    entry.push(id.clone());
                }
            }
        }
        for wf in &agent.capabilities.workflows {
            let wf_lower = wf.trim().to_lowercase();
            if !wf_lower.is_empty() {
                let entry = self.skill_index.entry(wf_lower).or_default();
                if !entry.contains(id) {
                    entry.push(id.clone());
                }
            }
        }
    }

    /// Finds candidate agent IDs for a given token in O(1) time.
    /// Checks exact role, exact name, word stems, and skills/workflows.
    pub fn find_candidates(&self, token: &str) -> HashSet<String> {
        let mut candidates = HashSet::new();
        let token_lower = token.trim().to_lowercase();
        if token_lower.is_empty() {
            return candidates;
        }

        // Check exact role
        if let Some(ids) = self.role_index.get(&token_lower) {
            candidates.extend(ids.iter().cloned());
        }

        // Check exact name
        if let Some(ids) = self.name_index.get(&token_lower) {
            candidates.extend(ids.iter().cloned());
        }

        // Check word-boundary token
        if let Some(ids) = self.word_index.get(&token_lower) {
            candidates.extend(ids.iter().cloned());
        }

        // Check skills/workflows
        if let Some(ids) = self.skill_index.get(&token_lower) {
            candidates.extend(ids.iter().cloned());
        }

        candidates
    }
}

pub type SharedCapabilityMap = Arc<RwLock<CapabilityMap>>;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_agent(
        id: &str,
        name: &str,
        role: &str,
        skills: Vec<&str>,
        workflows: Vec<&str>,
    ) -> EngineAgent {
        let mut agent = EngineAgent::default();
        agent.identity.id = id.to_string();
        agent.identity.name = name.to_string();
        agent.identity.role = role.to_string();
        agent.capabilities.skills = skills.into_iter().map(String::from).collect();
        agent.capabilities.workflows = workflows.into_iter().map(String::from).collect();
        agent
    }

    #[test]
    fn test_exact_role_and_name_lookup() {
        let mut map = CapabilityMap::new();
        let elon = make_test_agent(
            "3",
            "Elon",
            "Chief Technology Officer",
            vec!["rust-pro"],
            vec!["/status"],
        );
        let coder = make_test_agent(
            "coder-1",
            "Code Specialist",
            "Senior Developer",
            vec!["clean-code"],
            vec![],
        );

        map.update_agent(&elon);
        map.update_agent(&coder);

        // Exact name
        let by_name = map.find_candidates("elon");
        assert!(by_name.contains("3"));

        // Exact role
        let by_role = map.find_candidates("senior developer");
        assert!(by_role.contains("coder-1"));
    }

    #[test]
    fn test_word_stem_lookup() {
        let mut map = CapabilityMap::new();
        let sec = make_test_agent(
            "sec-1",
            "Checkmate",
            "Application Security Auditor",
            vec!["vulnerability-scanner"],
            vec![],
        );
        map.update_agent(&sec);

        // Word boundary match inside role
        let by_word = map.find_candidates("security");
        assert!(by_word.contains("sec-1"));

        let by_auditor = map.find_candidates("auditor");
        assert!(by_auditor.contains("sec-1"));
    }

    #[test]
    fn test_skill_and_workflow_lookup() {
        let mut map = CapabilityMap::new();
        let scout = make_test_agent(
            "scout-1",
            "Explorer",
            "Research Scout",
            vec!["explorer-scout"],
            vec!["/github_scout"],
        );
        map.update_agent(&scout);

        // Skill
        let by_skill = map.find_candidates("explorer-scout");
        assert!(by_skill.contains("scout-1"));

        // Workflow
        let by_wf = map.find_candidates("/github_scout");
        assert!(by_wf.contains("scout-1"));
    }

    #[test]
    fn test_agent_removal() {
        let mut map = CapabilityMap::new();
        let agent = make_test_agent(
            "temp-1",
            "Temp Agent",
            "Temporary Tester",
            vec!["testing"],
            vec![],
        );
        map.update_agent(&agent);
        assert!(map.find_candidates("tester").contains("temp-1"));

        map.remove_agent("temp-1");
        assert!(map.find_candidates("tester").is_empty());
        assert!(map.find_candidates("temp agent").is_empty());
    }
}
