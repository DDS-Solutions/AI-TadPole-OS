//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Authorization
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Pure, deterministic encoding, decoding, and capability authorization.
//! - `[Structural]` Default-deny: empty capability declarations grant zero tool/server access.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `authz::tests::*`

/// Encodes an MCP server name and tool name into an unambiguous dispatch identifier.
pub fn encode_mcp_tool_name(server: &str, tool: &str) -> String {
    format!("mcp__{}__{}", server, tool)
}

/// Decodes an MCP tool name into `(server_name, actual_tool_name)`.
pub fn decode_mcp_tool_name(name: &str) -> Option<(&str, &str)> {
    if let Some(rest) = name.strip_prefix("mcp__") {
        if let Some((server, tool)) = rest.split_once("__") {
            return Some((server, tool));
        }
    }
    // Backward-compatibility fallback for single-underscore prefix
    if let Some(rest) = name.strip_prefix("mcp_") {
        if let Some((server, tool)) = rest.split_once('_') {
            return Some((server, tool));
        }
    }
    None
}

/// Returns whether an externally discovered MCP tool is present in an agent's
/// explicit capability declaration. Both the encoded runtime name, the
/// human-authored `server:tool` form, `server:*`, and `mcp__{server}__*` are accepted.
/// An empty declaration grants no external MCP access.
pub fn is_mcp_tool_authorized(declarations: &[String], encoded_tool_name: &str) -> bool {
    let Some((server, tool)) = decode_mcp_tool_name(encoded_tool_name) else {
        return false;
    };
    let qualified = format!("{}:{}", server, tool);
    let server_wildcard = format!("{}:*", server);
    let encoded_wildcard = format!("mcp__{}__*", server);

    declarations.iter().any(|declaration| {
        declaration == encoded_tool_name
            || declaration == &qualified
            || declaration == &server_wildcard
            || declaration == &encoded_wildcard
    })
}

/// Returns whether an agent declaration grants any access to an MCP server.
/// This check is used before discovery so an undeclared external process is
/// never started merely to build the agent's tool list.
pub fn is_mcp_server_authorized(declarations: &[String], server_name: &str) -> bool {
    declarations.iter().any(|declaration| {
        decode_mcp_tool_name(declaration).is_some_and(|(server, _)| server == server_name)
            || declaration
                .split_once(':')
                .is_some_and(|(server, _)| server == server_name)
            || declaration == server_name
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_name_encoding_and_decoding() {
        let encoded = encode_mcp_tool_name("github_tools", "create_issue");
        assert_eq!(encoded, "mcp__github_tools__create_issue");

        let decoded = decode_mcp_tool_name(&encoded).unwrap();
        assert_eq!(decoded.0, "github_tools");
        assert_eq!(decoded.1, "create_issue");

        // Legacy fallback
        let legacy_decoded = decode_mcp_tool_name("mcp_sqlite_query").unwrap();
        assert_eq!(legacy_decoded.0, "sqlite");
        assert_eq!(legacy_decoded.1, "query");
    }

    #[test]
    fn mcp_authorization_accepts_exact_and_qualified_names() {
        let encoded = encode_mcp_tool_name("github", "create_issue");

        assert!(is_mcp_tool_authorized(
            std::slice::from_ref(&encoded),
            &encoded
        ));
        assert!(is_mcp_tool_authorized(
            &["github:create_issue".to_string()],
            &encoded
        ));
        assert!(is_mcp_tool_authorized(&["github:*".to_string()], &encoded));
        assert!(is_mcp_tool_authorized(
            &["mcp__github__*".to_string()],
            &encoded
        ));
        assert!(!is_mcp_tool_authorized(&[], &encoded));
        assert!(!is_mcp_tool_authorized(
            &["github:delete_issue".to_string()],
            &encoded
        ));
    }

    #[test]
    fn mcp_server_authorization_accepts_only_declared_servers() {
        assert!(is_mcp_server_authorized(
            &["github:create_issue".to_string()],
            "github"
        ));
        assert!(is_mcp_server_authorized(
            &["github:*".to_string()],
            "github"
        ));
        assert!(is_mcp_server_authorized(
            &[encode_mcp_tool_name("github", "create_issue")],
            "github"
        ));
        assert!(is_mcp_server_authorized(&["github".to_string()], "github"));
        assert!(!is_mcp_server_authorized(
            &["gitlab:create_issue".to_string()],
            "github"
        ));
        assert!(!is_mcp_server_authorized(&[], "github"));
    }
}
