//! @docs ARCHITECTURE:Contracts
//! 
//! ### AI Assist Note
//! **Contract Bridge**: Authoritative type exporter from Rust to TypeScript. 
//! Uses **Specta** to ensure that frontend interfaces exactly match backend 
//! data structures, preventing serialization drift.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Failed binding export due to missing `#[derive(Type)]` on new Rust structs or file permission errors on Windows.
//! - **Telemetry Link**: Search `[Bridge]` in server traces.
//!


use crate::agent::types::*;
use crate::agent::merge::*;
use crate::agent::skill_manifest::*;

#[allow(dead_code)]
pub fn export_bindings() {
    let config = specta_typescript::Typescript::default();
    let mut output = String::new();
    
    output.push_str("/**\n");
    output.push_str(" * @docs ARCHITECTURE:Contracts\n");
    output.push_str(" * \n");
    output.push_str(" * ### AI Assist Note\n");
    output.push_str(" * **Auto-Generated Contracts**: Authoritative TypeScript types mirrored from Rust structs via Specta.\n");
    output.push_str(" * This file ensures frontend-backend type parity. Do not edit manually.\n");
    output.push_str(" * \n");
    output.push_str(" * ### 🔍 Debugging & Observability\n");
    output.push_str(" * - **Failure Path**: Type mismatch if Rust structs drift without triggering a re-generation.\n");
    output.push_str(" * - **Telemetry Link**: Not tracked (Static generated types).\n");
    output.push_str(" */\n\n");
    
    // Individual exports for core roots (recursively includes sub-types if possible, but Specta export::<T> is unit-based)
    output.push_str(&specta_typescript::export::<EngineAgent>(&config)
        .expect("Failed to export EngineAgent"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<AgentIdentity>(&config)
        .expect("Failed to export AgentIdentity"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<AgentModels>(&config)
        .expect("Failed to export AgentModels"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<AgentEconomics>(&config)
        .expect("Failed to export AgentEconomics"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<AgentHealth>(&config)
        .expect("Failed to export AgentHealth"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<AgentCapabilities>(&config)
        .expect("Failed to export AgentCapabilities"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<AgentState>(&config)
        .expect("Failed to export AgentState"));
    output.push_str("\n\n");
    
    output.push_str(&specta_typescript::export::<ModelConfig>(&config)
        .expect("Failed to export ModelConfig"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<ConnectorConfig>(&config)
        .expect("Failed to export ConnectorConfig"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<TokenUsage>(&config)
        .expect("Failed to export TokenUsage"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<ModelProvider>(&config)
        .expect("Failed to export ModelProvider"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<RoleBlueprint>(&config)
        .expect("Failed to export RoleBlueprint"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<AgentConfigUpdate>(&config)
        .expect("Failed to export AgentConfigUpdate"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<SkillManifest>(&config)
        .expect("Failed to export SkillManifest"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<DangerLevel>(&config)
        .expect("Failed to export DangerLevel"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<Permission>(&config)
        .expect("Failed to export Permission"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<SkillParameter>(&config)
        .expect("Failed to export SkillParameter"));
    output.push_str("\n\n");

    output.push_str(&specta_typescript::export::<SkillHooks>(&config)
        .expect("Failed to export SkillHooks"));
    output.push_str("\n\n");

    output.push_str("export type JsonValue = string | number | boolean | null | { [key: string]: JsonValue } | JsonValue[];\n");

    let export_path = "../src/contracts/generated.ts";
    std::fs::write(export_path, output)
        .expect("Failed to write TypeScript bindings to file");
        
    tracing::info!("✅ [Bridge] TypeScript bindings exported to: {}", export_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_bindings() {
        // Trigger the binding export to verify the type tree is valid
        // and doesn't contain any incompatible specta types.
        export_bindings();
    }
}

// Metadata: [bridge]

// Metadata: [bridge]
