//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / outward_gateway
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::outward::outward_gateway::tests::*`

use serde::{Deserialize, Serialize};
use tracing::debug;

use super::{ModelProfile, DEFAULT_A2A_PROTOCOL_VERSION, DEFAULT_MODEL_PROFILE};
use crate::error::AppError;

/// Comprehensive business facts configuration for an SMB.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BusinessProfile {
    pub name: String,
    pub description: String,
    pub address: Option<String>,
    pub hours: Option<String>,
    pub support_email: Option<String>,
    pub support_phone: Option<String>,
    pub return_policy: Option<String>,
}

impl Default for BusinessProfile {
    fn default() -> Self {
        Self {
            name: "Sovereign SMB".to_string(),
            description: "Sovereign Customer Service & Catalog Agent powered by Tadpole OS."
                .to_string(),
            address: None,
            hours: None,
            support_email: None,
            support_phone: None,
            return_policy: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2aSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Standardized Agent-to-Agent (A2A) Public Discovery Card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2aAgentCard {
    pub protocol_version: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub default_input_modes: Vec<String>,
    pub default_output_modes: Vec<String>,
    pub capabilities: Vec<String>,
    pub skills: Vec<A2aSkill>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_schemes: Option<Vec<String>>,
}

/// Outward Gateway managing business identity, advertised capabilities, and A2A card versioning
#[derive(Debug, Clone)]
pub struct OutwardGateway {
    profile: BusinessProfile,
    agent_card: A2aAgentCard,
    model_profile: String,
    custom_skills: Vec<A2aSkill>,
    version_counter: u64,
}

impl OutwardGateway {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        let profile = BusinessProfile {
            name: name.into(),
            description: "Sovereign Customer Service & Catalog Agent powered by Tadpole OS."
                .to_string(),
            ..Default::default()
        };
        Self::with_profile(profile, url)
    }

    pub fn with_profile(profile: BusinessProfile, url: impl Into<String>) -> Self {
        let agent_url = url.into();
        let mut gateway = Self {
            profile: profile.clone(),
            agent_card: A2aAgentCard {
                protocol_version: DEFAULT_A2A_PROTOCOL_VERSION.to_string(),
                name: profile.name.clone(),
                version: "1.0.0".to_string(),
                description: profile.description.clone(),
                url: agent_url,
                default_input_modes: vec!["text/plain".to_string()],
                default_output_modes: vec!["application/json".to_string()],
                capabilities: vec!["streaming".to_string(), "catalog_search".to_string()],
                skills: Vec::new(),
                security_schemes: Some(vec!["bearer".to_string()]),
            },
            model_profile: DEFAULT_MODEL_PROFILE.to_string(),
            custom_skills: Vec::new(),
            version_counter: 0,
        };

        gateway.regenerate_skills();
        gateway
    }

    pub fn profile(&self) -> &BusinessProfile {
        &self.profile
    }

    pub fn get_agent_card(&self) -> &A2aAgentCard {
        debug!(
            "[OutwardGateway] Returning public A2A Agent Card for {}",
            self.agent_card.name
        );
        &self.agent_card
    }

    pub fn get_model_profile(&self) -> &str {
        &self.model_profile
    }

    pub fn version(&self) -> &str {
        &self.agent_card.version
    }

    pub fn set_security_schemes(&mut self, schemes: Option<Vec<String>>) {
        self.agent_card.security_schemes = schemes;
        self.bump_version();
    }

    pub fn set_capabilities(&mut self, capabilities: Vec<String>) {
        self.agent_card.capabilities = capabilities;
        self.bump_version();
    }

    fn bump_version(&mut self) {
        self.version_counter = self.version_counter.saturating_add(1);
        self.agent_card.version = format!("1.0.{}", self.version_counter);
    }

    /// Regenerates dynamic skills from business profile and preserves custom registered skills
    fn regenerate_skills(&mut self) {
        let mut skills = vec![A2aSkill {
            id: "catalog_search".to_string(),
            name: "Customer Catalog Search".to_string(),
            description: "Search product and service catalog for business inquiries.".to_string(),
            tags: vec![
                "catalog".to_string(),
                "products".to_string(),
                "smb".to_string(),
            ],
        }];

        if self.profile.address.is_some() || self.profile.hours.is_some() {
            let mut desc = String::new();
            if let Some(ref addr) = self.profile.address {
                desc.push_str(&format!("Location: {}. ", addr));
            }
            if let Some(ref hrs) = self.profile.hours {
                desc.push_str(&format!("Hours: {}. ", hrs));
            }
            skills.push(A2aSkill {
                id: "info-hours-loc".to_string(),
                name: "Store Hours & Location".to_string(),
                description: desc.trim().to_string(),
                tags: vec![
                    "hours".to_string(),
                    "location".to_string(),
                    "contact".to_string(),
                ],
            });
        }

        if let Some(ref policy) = self.profile.return_policy {
            skills.push(A2aSkill {
                id: "info-faq-returns".to_string(),
                name: "Return & Exchange Policy".to_string(),
                description: policy.clone(),
                tags: vec![
                    "returns".to_string(),
                    "refunds".to_string(),
                    "guarantee".to_string(),
                ],
            });
        }

        if self.profile.support_email.is_some() || self.profile.support_phone.is_some() {
            let mut desc = String::from("Customer Support: ");
            if let Some(ref email) = self.profile.support_email {
                desc.push_str(&format!("Email: {}. ", email));
            }
            if let Some(ref phone) = self.profile.support_phone {
                desc.push_str(&format!("Phone: {}. ", phone));
            }
            skills.push(A2aSkill {
                id: "info-faq-support".to_string(),
                name: "Customer Support Contact".to_string(),
                description: desc.trim().to_string(),
                tags: vec!["faq".to_string(), "support".to_string(), "help".to_string()],
            });
        }

        // Merge non-clobbering custom skills
        for custom in &self.custom_skills {
            if !skills.iter().any(|s| s.id == custom.id) {
                skills.push(custom.clone());
            }
        }

        self.agent_card.skills = skills;
    }

    /// Dynamically update custom exposed skills on the Agent Card without losing default capabilities
    pub fn update_skills(&mut self, new_skills: Vec<A2aSkill>) {
        self.custom_skills = new_skills
            .into_iter()
            .filter(|s| s.id != "catalog_search")
            .collect();
        self.regenerate_skills();
        self.bump_version();
    }

    /// Change outward local model profile with allow-list validation
    pub fn set_model_profile(&mut self, profile: impl AsRef<str>) -> Result<(), AppError> {
        let validated = ModelProfile::parse(profile.as_ref())?;
        self.model_profile = validated.as_str().to_string();
        Ok(())
    }

    /// Update business profile metadata (Name & Description)
    pub fn update_business_profile(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) {
        let n = name.into();
        let d = description.into();
        self.profile.name = n.clone();
        self.profile.description = d.clone();
        self.agent_card.name = n;
        self.agent_card.description = d;
        self.bump_version();
    }

    /// Set full business profile and regenerate skills accordingly
    pub fn set_full_profile(&mut self, profile: BusinessProfile) {
        self.profile = profile.clone();
        self.agent_card.name = profile.name.clone();
        self.agent_card.description = profile.description.clone();
        self.regenerate_skills();
        self.bump_version();
    }

    /// Update store hours and location
    pub fn update_hours_and_location(&mut self, address: Option<String>, hours: Option<String>) {
        self.profile.address = address;
        self.profile.hours = hours;
        self.regenerate_skills();
        self.bump_version();
    }

    /// Update customer support contact details
    pub fn update_support_contact(&mut self, email: Option<String>, phone: Option<String>) {
        self.profile.support_email = email;
        self.profile.support_phone = phone;
        self.regenerate_skills();
        self.bump_version();
    }

    /// Update return and refund policy
    pub fn update_return_policy(&mut self, policy: Option<String>) {
        self.profile.return_policy = policy;
        self.regenerate_skills();
        self.bump_version();
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_outward_gateway_agent_card() {
        let mut gateway = OutwardGateway::new(
            "SMB Auto Repair",
            "http://localhost:8000/a2a/v1/company-agent-card.json",
        );
        gateway.update_hours_and_location(
            Some("123 Auto Way".to_string()),
            Some("Mon-Fri 8am-5pm".to_string()),
        );

        let card = gateway.get_agent_card();
        assert_eq!(card.name, "SMB Auto Repair");
        assert_eq!(card.protocol_version, DEFAULT_A2A_PROTOCOL_VERSION);
        assert_eq!(gateway.get_model_profile(), "gemma4:e4b");
        assert!(card.skills.iter().any(|s| s.id == "catalog_search"));
        assert!(card.skills.iter().any(|s| s.id == "info-hours-loc"));
    }

    #[test]
    fn test_outward_gateway_granular_profile_updates() {
        let mut gateway = OutwardGateway::new("Acme Hardware", "http://localhost/a2a");

        gateway.update_support_contact(
            Some("help@acmehardware.com".to_string()),
            Some("555-0199".to_string()),
        );
        gateway.update_return_policy(Some("60-day money-back guarantee".to_string()));

        let card = gateway.get_agent_card();
        assert_eq!(card.version, "1.0.2");

        let support_skill = card
            .skills
            .iter()
            .find(|s| s.id == "info-faq-support")
            .unwrap();
        assert!(support_skill.description.contains("help@acmehardware.com"));
        assert!(support_skill.description.contains("555-0199"));

        let returns_skill = card
            .skills
            .iter()
            .find(|s| s.id == "info-faq-returns")
            .unwrap();
        assert_eq!(returns_skill.description, "60-day money-back guarantee");
    }

    #[test]
    fn test_outward_gateway_serde_roundtrip() {
        let gateway = OutwardGateway::new("Acme Hardware", "http://localhost/a2a");
        let card = gateway.get_agent_card();
        let json_str = serde_json::to_string(card).unwrap();
        let deserialized: A2aAgentCard = serde_json::from_str(&json_str).unwrap();
        assert_eq!(card, &deserialized);
    }

    #[test]
    fn test_skill_composition_does_not_clobber_custom_skills() {
        let mut gateway = OutwardGateway::new("Skill Co", "http://localhost/a2a");
        let custom_skill = A2aSkill {
            id: "custom_booking".to_string(),
            name: "Appointment Booking".to_string(),
            description: "Book an appointment online.".to_string(),
            tags: vec!["booking".to_string()],
        };

        // 1. Add custom skills
        gateway.update_skills(vec![custom_skill.clone()]);
        assert!(gateway
            .get_agent_card()
            .skills
            .iter()
            .any(|s| s.id == "custom_booking"));
        assert!(gateway
            .get_agent_card()
            .skills
            .iter()
            .any(|s| s.id == "catalog_search"));

        // 2. Now update support contact (which triggers regenerate_skills)
        gateway.update_support_contact(Some("support@skillco.com".into()), None);

        // Custom skill and catalog_search must BOTH survive!
        let card = gateway.get_agent_card();
        assert!(card.skills.iter().any(|s| s.id == "custom_booking"));
        assert!(card.skills.iter().any(|s| s.id == "catalog_search"));
        assert!(card.skills.iter().any(|s| s.id == "info-faq-support"));
    }

    #[test]
    fn test_model_profile_validation() {
        let mut gateway = OutwardGateway::new("Model Co", "http://localhost/a2a");
        assert!(gateway.set_model_profile("gemma4:e8b").is_ok());
        assert_eq!(gateway.get_model_profile(), "gemma4:e8b");

        // Invalid profile rejected with AppError::BadRequest
        assert!(gateway.set_model_profile("invalid-model:xyz").is_err());
    }
}
