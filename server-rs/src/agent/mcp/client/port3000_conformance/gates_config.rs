//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Port 3000 Conformance - Config Gates
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Coexistence of HTTP transport and stdio fallback in MCP config schema.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: config validation failures.
//! - **Telemetry Targets**: none (pure config validation).
//! - **Witness Tests**: `port3000_conformance::tests::gates_config::test_gate_1_*`

use crate::agent::mcp::config::{
    validate_mcp_server_config, McpMode, McpServerConfig, DEFAULT_MCP_PROTOCOL_VERSION,
};
use serde_json::json;

// ---------------------------------------------------------------------------
// Gate 1: Both transports coexist in one configuration with explicit prefer_http
// ---------------------------------------------------------------------------
#[test]
fn test_gate_1_coexisting_prefer_http_configuration() {
    let json_config = json!({
        "mode": "prefer_http",
        "http": {
            "url": "http://127.0.0.1:3000/mcp",
            "protocol_versions": [DEFAULT_MCP_PROTOCOL_VERSION],
            "resource": "http://127.0.0.1:3000/mcp",
            "headers": {
                "Authorization": "${GEV_MCP_AUTHORIZATION}"
            }
        },
        "stdio_fallback": {
            "command": "pnpm",
            "args": ["--filter", "@gev/ops-mcp", "start"],
            "cwd": "G:/AI-TadPole-Eye-View"
        }
    });

    let config: McpServerConfig = serde_json::from_value(json_config).unwrap();
    assert_eq!(config.effective_mode(), McpMode::PreferHttp);

    let http = config
        .resolved_http_config()
        .expect("HTTP config must be present");
    assert_eq!(http.url, "http://127.0.0.1:3000/mcp");
    assert_eq!(http.protocol_versions, vec![DEFAULT_MCP_PROTOCOL_VERSION]);
    assert_eq!(http.resource.as_deref(), Some("http://127.0.0.1:3000/mcp"));

    let stdio = config
        .resolved_stdio_config()
        .expect("Stdio fallback config must be present");
    assert_eq!(stdio.command, "pnpm");
    assert_eq!(stdio.args, vec!["--filter", "@gev/ops-mcp", "start"]);
    assert_eq!(stdio.cwd.as_deref(), Some("G:/AI-TadPole-Eye-View"));

    validate_mcp_server_config("gev", &config).expect("Config must be valid");
}
