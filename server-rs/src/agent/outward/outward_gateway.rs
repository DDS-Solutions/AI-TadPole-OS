//! @docs ARCHITECTURE:Agent
//! @docs OPERATIONS_MANUAL:OutwardGateway
//!
//! ### AI Assist Note
//! Outward A2A Agent Gateway for SMB customer interaction.
//! Generates well-known A2A Agent Cards (`agent-card.json`) and handles
//! outward customer inquiries using local model profiles (default: `gemma4:e4b`).
//! Supports dynamic skill updates, model profile switching, and card mutations.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Outward A2A schema errors, rate-limiting on local inference.
//! - **Trace Scope**: `server-rs::agent::outward` (Search for `[OutwardGateway]` in logs)

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2aSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2aAgentCard {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub default_input_modes: Vec<String>,
    pub default_output_modes: Vec<String>,
    pub skills: Vec<A2aSkill>,
    pub model_profile: String,
}

pub struct OutwardGateway {
    pub agent_card: A2aAgentCard,
}

impl OutwardGateway {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        let agent_name = name.into();
        let agent_url = url.into();
        info!("[OutwardGateway] Initializing A2A Outward Gateway for: {}", agent_name);

        let card = A2aAgentCard {
            name: agent_name,
            version: "1.0.0".to_string(),
            description: "Sovereign SMB Customer Service & Catalog Agent powered by Tadpole OS.".to_string(),
            url: agent_url,
            default_input_modes: vec!["text/plain".to_string()],
            default_output_modes: vec!["application/json".to_string()],
            skills: vec![
                A2aSkill {
                    id: "catalog_search".to_string(),
                    name: "Customer Catalog Search".to_string(),
                    description: "Search product and service catalog for business inquiries.".to_string(),
                    tags: vec!["catalog".to_string(), "products".to_string(), "smb".to_string()],
                },
                A2aSkill {
                    id: "business_qa".to_string(),
                    name: "Business FAQ & Operating Info".to_string(),
                    description: "Answer store hours, location, and service policy questions.".to_string(),
                    tags: vec!["faq".to_string(), "info".to_string()],
                },
            ],
            model_profile: "gemma4:e4b".to_string(),
        };

        Self { agent_card: card }
    }

    /// Return the public A2A Agent Card JSON structure
    pub fn get_agent_card(&self) -> &A2aAgentCard {
        info!("[OutwardGateway] Returning public A2A Agent Card for {}", self.agent_card.name);
        &self.agent_card
    }

    /// Dynamically update the exposed skills vector on the Agent Card
    pub fn update_skills(&mut self, new_skills: Vec<A2aSkill>) {
        info!(
            "[OutwardGateway] Updating skills for {}: {} skills registered",
            self.agent_card.name,
            new_skills.len()
        );
        self.agent_card.skills = new_skills;
    }

    /// Change the outward local model profile (e.g. gemma4:e4b, gemma4:e8b, gemma4:full)
    pub fn set_model_profile(&mut self, profile: impl Into<String>) {
        let p = profile.into();
        info!("[OutwardGateway] Switching model profile for {} to: {}", self.agent_card.name, p);
        self.agent_card.model_profile = p;
    }

    /// Update business profile metadata (Name & Description)
    pub fn update_business_profile(&mut self, name: impl Into<String>, description: impl Into<String>) {
        let n = name.into();
        let d = description.into();
        info!("[OutwardGateway] Updating business profile for {}", n);
        self.agent_card.name = n;
        self.agent_card.description = d;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outward_gateway_agent_card() {
        let gateway = OutwardGateway::new(
            "SMB Auto Repair",
            "http://localhost:26453/a2a/v1/agent-card.json",
        );
        let card = gateway.get_agent_card();
        assert_eq!(card.name, "SMB Auto Repair");
        assert_eq!(card.model_profile, "gemma4:e4b");
        assert_eq!(card.version, "1.0.0");
        assert_eq!(card.skills.len(), 2);
        assert_eq!(card.skills[0].id, "catalog_search");
    }

    #[test]
    fn test_outward_gateway_mutations() {
        let mut gateway = OutwardGateway::new("Acme Hardware", "http://localhost/a2a/card.json");

        // Test model profile change
        gateway.set_model_profile("gemma4:e8b");
        assert_eq!(gateway.get_agent_card().model_profile, "gemma4:e8b");

        // Test business profile update
        gateway.update_business_profile("Acme Hardware & Tools", "Custom hardware store agent");
        assert_eq!(gateway.get_agent_card().name, "Acme Hardware & Tools");
        assert_eq!(gateway.get_agent_card().description, "Custom hardware store agent");

        // Test dynamic skills update
        let new_skill = A2aSkill {
            id: "inventory_check".to_string(),
            name: "Live Inventory Check".to_string(),
            description: "Check stock availability".to_string(),
            tags: vec!["inventory".to_string()],
        };
        gateway.update_skills(vec![new_skill.clone()]);
        assert_eq!(gateway.get_agent_card().skills.len(), 1);
        assert_eq!(gateway.get_agent_card().skills[0], new_skill);
    }
}
