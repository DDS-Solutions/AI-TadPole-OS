//! @docs ARCHITECTURE:SecurityModel
//!
//! ### AI Assist Note
//! - **Subsystem**: Agent Runner / Resolved tool policy
//! - Token targets and policy resources retain their existing, distinct meanings.
//! - Each execution boundary still enforces its own identity and declaration checks.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::{capability::Permission, error::ToolExecutionError, policy};
use crate::security::permissions::{CapabilityClass, PermissionDecision, PermissionPolicy};
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ToolEffects {
    pub mutating: bool,
    pub dangerous: bool,
    pub cacheable: bool,
}

impl ToolEffects {
    pub(crate) fn classified(name: &str) -> Self {
        let mutating = policy::is_mutating_tool_name(name);
        Self {
            mutating,
            dangerous: mutating
                || policy::is_dangerous_native_operation(name)
                || policy::is_dangerous_mcp_operation(name),
            cacheable: policy::is_cacheable_tool_name(name),
        }
    }
}

/// The final MCP guard retains its legacy policy lookup during this migration.
/// Moving it to granular policy is a separate change to prompt/floor semantics.
#[derive(Debug, Clone, Copy)]
pub(crate) enum PolicyBoundary {
    Runner,
    Mcp,
}

#[derive(Debug, Clone)]
pub(crate) struct ToolPolicyDescriptor {
    pub name: String,
    pub capability: CapabilityClass,
    pub policy_resource: String,
    pub manifest_alias: Option<&'static str>,
    pub effects: ToolEffects,
}

impl ToolPolicyDescriptor {
    pub(crate) fn for_tool(name: &str) -> Self {
        Self::with_effects(name, ToolEffects::classified(name))
    }

    pub(crate) fn with_effects(name: &str, effects: ToolEffects) -> Self {
        let capability = match name {
            "delete_file" | "drop_table" => CapabilityClass::Delete,
            "write_file" | "create_or_update_file" | "restore_file_version" => {
                CapabilityClass::Modify
            }
            "install_package" | "install_skill" => CapabilityClass::Install,
            "approve_action" | "confirm_asset_transaction" => CapabilityClass::Approve,
            _ => CapabilityClass::Execute,
        };
        let manifest_alias = match name {
            "read_file" | "write_file" | "list_files" | "delete_file" => Some("filesystem"),
            "execute_shell" => Some("shell"),
            _ => None,
        };
        Self {
            name: name.to_string(),
            capability,
            // Resource policies currently address the tool name, while token
            // permissions below address the final path/command/URL.
            policy_resource: name.to_string(),
            manifest_alias,
            effects,
        }
    }

    pub(crate) fn required_permission(
        &self,
        arguments: &serde_json::Value,
        workspace: &Path,
    ) -> Result<Permission, ToolExecutionError> {
        policy::resolve_required_permission(&self.name, arguments, workspace, self.effects.mutating)
    }

    pub(crate) async fn decision(
        &self,
        policy: &PermissionPolicy,
        agent: Option<&str>,
        role: Option<&str>,
        boundary: PolicyBoundary,
    ) -> PermissionDecision {
        match boundary {
            PolicyBoundary::Runner => {
                policy
                    .check_capability_decision(agent, role, self.capability, &self.policy_resource)
                    .await
            }
            PolicyBoundary::Mcp => {
                policy
                    .get_mode_decision(agent, role, &self.policy_resource)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_preserve_capabilities_aliases_and_mcp_effects() {
        for (name, class, alias) in [
            ("write_file", CapabilityClass::Modify, Some("filesystem")),
            ("delete_file", CapabilityClass::Delete, Some("filesystem")),
            ("execute_shell", CapabilityClass::Execute, Some("shell")),
            ("confirm_asset_transaction", CapabilityClass::Approve, None),
            ("unknown_read", CapabilityClass::Execute, None),
        ] {
            let descriptor = ToolPolicyDescriptor::for_tool(name);
            assert_eq!(descriptor.capability, class);
            assert_eq!(descriptor.manifest_alias, alias);
            assert_eq!(descriptor.policy_resource, name);
        }
        let mcp = ToolPolicyDescriptor::for_tool("mcp_my_fs_server_write_file");
        assert!(mcp.effects.mutating && mcp.effects.dangerous);
        assert!(!mcp.effects.cacheable);
        assert_eq!(mcp.capability, CapabilityClass::Execute);
    }

    #[test]
    fn token_target_uses_normalized_path_without_changing_policy_resource() {
        let dir = tempfile::tempdir().unwrap();
        let descriptor = ToolPolicyDescriptor::for_tool("write_file");
        let permission = descriptor
            .required_permission(&serde_json::json!({"filename": "result.md"}), dir.path())
            .unwrap();
        assert!(matches!(permission, Permission::FileWrite(path) if path.ends_with("result.md")));
        assert_eq!(descriptor.policy_resource, "write_file");
        let unknown = ToolPolicyDescriptor::with_effects(
            "unknown_mutator",
            ToolEffects {
                mutating: true,
                ..Default::default()
            },
        );
        assert!(unknown
            .required_permission(&serde_json::json!({}), dir.path())
            .is_err());
    }
}
