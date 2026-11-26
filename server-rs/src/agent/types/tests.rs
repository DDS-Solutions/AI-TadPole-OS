//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / tests
//!
//! ### AI Assist Note
//! - Exercise wire compatibility and reject ambiguous economics input.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::*;
use serde_json::json;

#[test]
fn test_economics_alias_round_trips_and_conflicts() {
    let mut agent = EngineAgent {
        economics: AgentEconomics {
            tokens_used: 125000,
            cost_usd: 5.25,
            budget_usd: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    for slot in ["planning", "execution", "default"] {
        agent.models.active_model_slot = Some(slot.to_string());
        let serialized = serde_json::to_value(&agent).unwrap();
        for omit in [
            None,
            Some(["tokensUsed", "costUsd", "budgetUsd"]),
            Some(["tokens_used", "cost_usd", "budget_usd"]),
        ] {
            let mut input = serialized.clone();
            if let Some(keys) = omit {
                for key in keys {
                    input.as_object_mut().unwrap().remove(key);
                }
            }
            let decoded: EngineAgent = serde_json::from_value(input).unwrap();
            assert_eq!(decoded.models.active_model_slot.as_deref(), Some(slot));
            assert_eq!(decoded.economics.tokens_used, 125000);
            assert_eq!(decoded.economics.cost_usd, 5.25);
            assert_eq!(decoded.economics.budget_usd, 100.0);
        }
        for key in ["tokens_used", "cost_usd", "budget_usd"] {
            let mut conflicting = serialized.clone();
            conflicting[key] = json!(999);
            let error = serde_json::from_value::<EngineAgent>(conflicting).unwrap_err();
            assert!(error.to_string().contains("Conflicting economics aliases"));
        }
    }
}

#[test]
fn test_engine_agent_deserialization_defaults() {
    let agent_json = json!({
        "id": "test-agent",
        "name": "Test Agent",
        "role": "Tester",
        "department": "QA",
        "description": "Tests things",
        "status": "active",
        "model": "gpt-4",
        "modelConfig": {
            "provider": "openai",
            "modelId": "gpt-4"
        },
        "tokensUsed": 0,
        "budgetUsd": 10.0,
        "costUsd": 0.0
    });

    let agent_str = agent_json.to_string();
    let agent: EngineAgent =
        serde_json::from_str(&agent_str).expect("Failed to deserialize agent with missing fields");

    assert_eq!(agent.identity.id, "test-agent");
    assert_eq!(agent.economics.token_usage.total_tokens, 0);
    assert!(agent.capabilities.skills.is_empty());
    assert!(agent.capabilities.workflows.is_empty());
    assert!(agent.metadata.is_empty());
    assert_eq!(agent.economics.budget_usd, 10.0);
}

#[test]
fn test_engine_agent_deserialization_full() {
    let agent_json = json!({
        "id": "full-agent",
        "name": "Full Agent",
        "role": "Lead",
        "department": "Engineering",
        "description": "Full description",
        "status": "active",
        "model": "gpt-4o",
        "modelConfig": {
            "provider": "openai",
            "modelId": "gpt-4o"
        },
        "tokensUsed": 100,
        "tokenUsage": {
            "inputTokens": 40,
            "outputTokens": 60,
            "totalTokens": 100
        },
        "skills": ["coding"],
        "workflows": ["deploy"],
        "metadata": {"key": "value"},
        "budgetUsd": 100.0,
        "costUsd": 0.5
    });

    let agent_str = agent_json.to_string();
    let agent: EngineAgent =
        serde_json::from_str(&agent_str).expect("Failed to deserialize full agent");

    assert_eq!(agent.capabilities.skills, vec!["coding"]);
    assert_eq!(agent.capabilities.workflows, vec!["deploy"]);
    assert!(agent.capabilities.mcp_tools.is_empty());
    assert_eq!(agent.metadata.get("key").unwrap(), &json!("value"));
}

#[test]
fn test_model_config_merge() {
    let mut base_extras = std::collections::HashMap::new();
    base_extras.insert("json_mode".to_string(), json!(true));
    base_extras.insert("seed".to_string(), json!(42));

    let base = ModelConfig {
        provider: ModelProvider::Openai,
        model_id: "gpt-4".to_string(),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        extra_parameters: Some(base_extras),
        ..Default::default()
    };

    let mut override_extras = std::collections::HashMap::new();
    override_extras.insert("seed".to_string(), json!(123)); // Should take precedence
    override_extras.insert("thinking".to_string(), json!(true)); // Should be added

    let overrides = ModelConfig {
        provider: ModelProvider::Openai,
        model_id: "gpt-4".to_string(),
        temperature: Some(0.0), // Should take precedence
        extra_parameters: Some(override_extras),
        ..Default::default()
    };

    let merged = overrides.merge(&base);

    assert_eq!(merged.temperature, Some(0.0));
    assert_eq!(merged.max_tokens, Some(1000)); // From base

    let extras = merged.extra_parameters.unwrap();
    assert_eq!(extras.get("json_mode").unwrap(), &json!(true)); // From base
    assert_eq!(extras.get("seed").unwrap(), &json!(123)); // From overrides
    assert_eq!(extras.get("thinking").unwrap(), &json!(true)); // From overrides
}

#[test]
fn test_agent_config_update_serialization_parity() {
    let update = crate::agent::merge::AgentConfigUpdate {
        name: Some("Test Agent".to_string()),
        role: Some("Analyst".to_string()),
        department: Some("QA".to_string()),
        budget_usd: Some(500.0),
        metadata: Some(std::collections::HashMap::from([(
            "key".to_string(),
            json!("value"),
        )])),
        input_tokens: Some(100),
        ..Default::default()
    };

    let serialized = serde_json::to_value(&update).unwrap();

    // Assert camelCase serialization (as required by frontend mappers)
    assert_eq!(serialized["name"], "Test Agent");
    assert_eq!(serialized["budgetUsd"], 500.0);
    assert_eq!(serialized["inputTokens"], 100);
    assert!(serialized.get("metadata").is_some());
}

#[test]
fn test_engine_agent_serialization_parity() {
    let agent = EngineAgent {
        identity: AgentIdentity {
            id: "agent-1".to_string(),
            name: "Test Agent".to_string(),
            ..AgentIdentity::default()
        },
        economics: AgentEconomics {
            budget_usd: 100.0,
            cost_usd: 5.25,
            tokens_used: 125000,
            ..AgentEconomics::default()
        },
        ..EngineAgent::default()
    };

    let serialized = serde_json::to_value(&agent).unwrap();

    // Assert camelCase serialization (frontend DTO compatibility)
    assert_eq!(serialized["id"], "agent-1");
    assert_eq!(serialized["name"], "Test Agent");
    assert_eq!(serialized["budgetUsd"], 100.0);
    assert_eq!(serialized["costUsd"], 5.25);
    assert_eq!(serialized["tokensUsed"], 125000);

    // Assert snake_case serialization (backend/SQLite telemetry compatibility)
    assert_eq!(serialized["budget_usd"], 100.0);
    assert_eq!(serialized["cost_usd"], 5.25);
    assert_eq!(serialized["tokens_used"], 125000);
}

#[test]
fn test_engine_agent_active_model_slot_round_trip() {
    let mut agent = EngineAgent {
        identity: AgentIdentity {
            id: "agent-slot".to_string(),
            name: "Slot Tester".to_string(),
            role: "Developer".to_string(),
            ..AgentIdentity::default()
        },
        ..EngineAgent::default()
    };
    agent.models.active_model_slot = Some("planning".to_string());

    // 1. String representation emitted by Serialize round-trips into EngineAgent
    let serialized_str = serde_json::to_string(&agent).expect("serialize");
    assert!(serialized_str.contains("\"activeModelSlot\":\"planning\""));

    let deserialized: EngineAgent =
        serde_json::from_str(&serialized_str).expect("deserialize round trip");
    assert_eq!(
        deserialized.models.active_model_slot.as_deref(),
        Some("planning")
    );

    // 2. Legacy integer indices (1 = planning, 2 = execution) are supported
    let legacy_json = serde_json::json!({
        "id": "agent-legacy",
        "name": "Legacy Agent",
        "role": "Engineer",
        "department": "Core",
        "description": "Legacy",
        "status": "active",
        "activeModelSlot": 2
    });
    let legacy_agent: EngineAgent =
        serde_json::from_value(legacy_json).expect("deserialize legacy int slot");
    assert_eq!(
        legacy_agent.models.active_model_slot.as_deref(),
        Some("execution")
    );
}

#[test]
fn test_model_config_merge_extended_and_credential_binding() {
    let base = ModelConfig {
        provider: ModelProvider::Openai,
        model_id: "gpt-4o".to_string(),
        api_key: Some("base-secret-key".to_string()),
        base_url: Some("https://api.openai.com/v1".to_string()),
        max_turns: Some(42),
        skills: Some(vec!["skill-a".to_string()]),
        workflows: Some(vec!["workflow-a".to_string()]),
        mcp_tools: Some(vec!["mcp-a".to_string()]),
        connector_configs: Some(vec![ConnectorConfig {
            r#type: "postgres".to_string(),
            uri: "postgres://localhost/db".to_string(),
        }]),
        ..Default::default()
    };

    // Override preserves matching base_url -> inherits api_key and all missing fields
    let override_same_host = ModelConfig {
        provider: ModelProvider::Openai,
        model_id: "gpt-4o-mini".to_string(),
        base_url: Some("https://api.openai.com/v1".to_string()),
        ..Default::default()
    };
    let merged_same = override_same_host.merge(&base);
    assert_eq!(merged_same.api_key.as_deref(), Some("base-secret-key"));
    assert_eq!(merged_same.max_turns, Some(42));
    assert_eq!(merged_same.skills.unwrap(), vec!["skill-a"]);
    assert_eq!(merged_same.workflows.unwrap(), vec!["workflow-a"]);
    assert_eq!(merged_same.mcp_tools.unwrap(), vec!["mcp-a"]);
    assert_eq!(merged_same.connector_configs.unwrap().len(), 1);

    // Override specifying a different endpoint MUST NOT inherit the base api_key
    let override_diff_host = ModelConfig {
        provider: ModelProvider::Openai,
        model_id: "custom-gateway".to_string(),
        base_url: Some("https://my-proxy.company.internal/v1".to_string()),
        ..Default::default()
    };
    let merged_diff = override_diff_host.merge(&base);
    assert_eq!(merged_diff.api_key, None);
    assert_eq!(merged_diff.max_turns, Some(42));
}
