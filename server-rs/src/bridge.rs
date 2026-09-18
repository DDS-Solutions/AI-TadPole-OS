//! @docs ARCHITECTURE:Contracts
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / bridge
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::merge::*;
use crate::agent::skill_manifest::*;
use crate::agent::types::*;

#[allow(dead_code)]
pub fn export_bindings() {
    let config = specta_typescript::Typescript::default();
    let mut output = String::new();

    output.push_str(
        "/**\n * @docs ARCHITECTURE:Contracts\n *\n * ### AI Context Alignment\n * - **Subsystem**: Generated TypeScript Contracts\n *\n * ### ⚠️ Invariants & Non-Negotiables\n * - `[Structural]` Generated types mirror the Rust engine contract.\n *\n * ### 🔍 Debugging & Observability\n * - **Local Errors**: none\n * - **Telemetry Targets**: none declared\n * - **Witness Tests**: none declared\n */\n\n",
    );

    // Individual exports for core roots (recursively includes sub-types if possible, but Specta export::<T> is unit-based)
    output.push_str(
        &specta_typescript::export::<EngineAgent>(&config).expect("Failed to export EngineAgent"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<AgentIdentity>(&config)
            .expect("Failed to export AgentIdentity"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<AgentModels>(&config).expect("Failed to export AgentModels"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<AgentEconomics>(&config)
            .expect("Failed to export AgentEconomics"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<AgentHealth>(&config).expect("Failed to export AgentHealth"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<AgentCapabilities>(&config)
            .expect("Failed to export AgentCapabilities"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<AgentState>(&config).expect("Failed to export AgentState"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<ModelConfig>(&config).expect("Failed to export ModelConfig"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<ConnectorConfig>(&config)
            .expect("Failed to export ConnectorConfig"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<TokenUsage>(&config).expect("Failed to export TokenUsage"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<ModelProvider>(&config)
            .expect("Failed to export ModelProvider"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<RoleBlueprint>(&config)
            .expect("Failed to export RoleBlueprint"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<AgentConfigUpdate>(&config)
            .expect("Failed to export AgentConfigUpdate"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<SkillManifest>(&config)
            .expect("Failed to export SkillManifest"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<DangerLevel>(&config).expect("Failed to export DangerLevel"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<Permission>(&config).expect("Failed to export Permission"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<SkillParameter>(&config)
            .expect("Failed to export SkillParameter"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<SkillHooks>(&config).expect("Failed to export SkillHooks"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<crate::intelligence::graph::SymbolNode>(&config)
            .expect("Failed to export SymbolNode"),
    );
    output.push_str("\n\n");

    output.push_str(
        &specta_typescript::export::<crate::utils::parser::SymbolRange>(&config)
            .expect("Failed to export SymbolRange"),
    );
    output.push_str("\n\n");

    output.push_str("export type JsonValue = string | number | boolean | null | { [key: string]: JsonValue } | JsonValue[];\n\n// Metadata: [generated]\n\n// Metadata: [generated]\n");

    let export_path = "../src/contracts/generated.ts";
    std::fs::write(export_path, output).expect("Failed to write TypeScript bindings to file");

    tracing::info!(
        "✅ [Bridge] TypeScript bindings exported to: {}",
        export_path
    );
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
