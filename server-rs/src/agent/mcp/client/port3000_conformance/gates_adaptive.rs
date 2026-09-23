//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Port 3000 Conformance - Adaptive Gates
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Adaptive fallback and commit state machine invariants.
//! - `[Structural]` Fail-closed transitions on unapproved errors, TLS mismatch, or timeouts (never fallback to stdio).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: assertions on `AdaptiveState` transitions.
//! - **Telemetry Targets**: local loopback TCP fixtures and mock stdio processes.
//! - **Witness Tests**: `port3000_conformance::tests::gates_adaptive::test_gate_*`

use super::harness::{get_dummy_stdio_cfg, get_test_python_cmd, read_http_request, write_json_rpc};
use crate::agent::mcp::client::adaptive::{AdaptiveMcpClient, AdaptiveState};
use crate::agent::mcp::client::http::{
    HttpDiscoveryFailureKind, HttpTransportFailureKind, McpHttpClient,
};
use crate::agent::mcp::client::jsonrpc::JSONRPC_UNSUPPORTED_PROTOCOL_VERSION;
use crate::agent::mcp::config::{McpHttpConfig, DEFAULT_MCP_PROTOCOL_VERSION};
use crate::error::AppError;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

// ---------------------------------------------------------------------------
// Gate 6: Connection-refused and host-unreachable select stdio fallback
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_6_connection_refused_selects_stdio() {
    // Bind to allocate an ephemeral local port, then drop listener immediately to cause connection refusal (RST).
    // (Small TOCTOU port race is standard for deterministic local connection refusal fixtures).
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let http_cfg = McpHttpConfig {
        url: format!("http://127.0.0.1:{}/mcp", port),
        protocol_versions: vec!["2026-07-28".to_string()],
        resource: None,
        headers: None,
        discovery_timeout_ms: None,
        tool_timeout_ms: None,
    };

    let stdio_cfg = crate::agent::mcp::config::McpStdioConfig {
        command: get_test_python_cmd(),
        args: vec![
            "-c".to_string(),
            "import sys; sys.stdin.readline(); sys.stdout.write('{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"resultType\":\"complete\",\"supportedVersions\":[\"2026-07-28\"],\"capabilities\":{}}}\\n'); sys.stdout.flush()".to_string(),
        ],
        cwd: None,
        env: None,
    };

    let mut adaptive = AdaptiveMcpClient::new("test-refusal", http_cfg, Some(stdio_cfg)).unwrap();

    assert_eq!(adaptive.state, AdaptiveState::Configured);
    let init_res = adaptive.initialize().await;
    assert!(
        init_res.is_ok(),
        "Adaptive client must succeed by falling back to stdio"
    );
    assert_eq!(adaptive.state, AdaptiveState::StdioReady);
}

// ---------------------------------------------------------------------------
// Gate 7: Structured -32022 is preserved; mutual vs absent intersections
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_7_structured_32022_error_preservation() {
    // Case A: Absent intersection (supported: ["2024-11-05"]) -> selects stdio fallback
    let listener_a = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port_a = listener_a.local_addr().unwrap().port();

    tokio::spawn(async move {
        let (mut socket, _) = listener_a.accept().await.unwrap();
        let _ = read_http_request(&mut socket).await;
        let err_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": JSONRPC_UNSUPPORTED_PROTOCOL_VERSION,
                "message": "Unsupported protocol version",
                "data": {
                    "requested": DEFAULT_MCP_PROTOCOL_VERSION,
                    "supported": ["2024-11-05"]
                }
            }
        });
        let _ = write_json_rpc(&mut socket, 400, &err_body).await;
    });

    let http_cfg_a = McpHttpConfig {
        url: format!("http://127.0.0.1:{}/mcp", port_a),
        protocol_versions: vec![DEFAULT_MCP_PROTOCOL_VERSION.to_string()],
        resource: None,
        headers: None,
        discovery_timeout_ms: None,
        tool_timeout_ms: None,
    };

    let stdio_cfg_a = crate::agent::mcp::config::McpStdioConfig {
        command: get_test_python_cmd(),
        args: vec![
            "-c".to_string(),
            "import sys; sys.stdin.readline(); sys.stdout.write('{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"resultType\":\"complete\",\"supportedVersions\":[\"2026-07-28\"],\"capabilities\":{}}}\\n'); sys.stdout.flush()".to_string(),
        ],
        cwd: None,
        env: None,
    };

    let mut adaptive_a =
        AdaptiveMcpClient::new("test-absent", http_cfg_a, Some(stdio_cfg_a)).unwrap();
    let init_res = adaptive_a.initialize().await;
    assert!(
        init_res.is_ok(),
        "Expected stdio fallback to succeed, got: {:?}",
        init_res
    );
    assert_eq!(adaptive_a.state, AdaptiveState::StdioReady);

    // Case B: Protocol inconsistency (-32022 rejecting 2026-07-28 while advertising it)
    let listener_b = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port_b = listener_b.local_addr().unwrap().port();

    tokio::spawn(async move {
        let (mut socket, _) = listener_b.accept().await.unwrap();
        let _ = read_http_request(&mut socket).await;
        let err_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": JSONRPC_UNSUPPORTED_PROTOCOL_VERSION,
                "message": "Protocol error",
                "data": {
                    "requested": DEFAULT_MCP_PROTOCOL_VERSION,
                    "supported": [DEFAULT_MCP_PROTOCOL_VERSION]
                }
            }
        });
        let _ = write_json_rpc(&mut socket, 400, &err_body).await;
    });

    let http_cfg_b = McpHttpConfig {
        url: format!("http://127.0.0.1:{}/mcp", port_b),
        protocol_versions: vec![DEFAULT_MCP_PROTOCOL_VERSION.to_string()],
        resource: None,
        headers: None,
        discovery_timeout_ms: None,
        tool_timeout_ms: None,
    };

    let stdio_cfg_b = get_dummy_stdio_cfg();

    let mut adaptive_b =
        AdaptiveMcpClient::new("test-inconsistency", http_cfg_b, Some(stdio_cfg_b)).unwrap();
    let res_b = adaptive_b.initialize().await;
    assert!(res_b.is_err(), "Protocol inconsistency must fail closed");
    assert_eq!(adaptive_b.state, AdaptiveState::FailedClosed);
}

// ---------------------------------------------------------------------------
// Gate 8: HTTP 401, 403, 404, 429, 5xx, timeouts fail closed without stdio
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_8_fail_closed_on_unapproved_errors() {
    let statuses = [400, 401, 403, 404, 405, 409, 429, 500, 503];

    for status_code in statuses {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let conn_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cc = conn_counter.clone();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let _ = read_http_request(&mut socket).await;
            let empty_json = json!({});
            let _ = write_json_rpc(&mut socket, status_code, &empty_json).await;
        });

        let http_cfg = McpHttpConfig {
            url: format!("http://127.0.0.1:{}/mcp", port),
            protocol_versions: vec![DEFAULT_MCP_PROTOCOL_VERSION.to_string()],
            resource: None,
            headers: None,
            discovery_timeout_ms: None,
            tool_timeout_ms: None,
        };

        let stdio_cfg = get_dummy_stdio_cfg();

        let mut adaptive = AdaptiveMcpClient::new(
            &format!("test-status-{}", status_code),
            http_cfg,
            Some(stdio_cfg),
        )
        .unwrap();

        let init_res = adaptive.initialize().await;
        assert!(
            init_res.is_err(),
            "Status {} MUST fail closed and not invoke stdio",
            status_code
        );
        assert_eq!(
            adaptive.state,
            AdaptiveState::FailedClosed,
            "Status {} must transition to FailedClosed",
            status_code
        );
        assert_eq!(
            conn_counter.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "Status {} must not trigger retries or secondary connections",
            status_code
        );
    }
}

// ---------------------------------------------------------------------------
// Gate 10: Tool is never dispatched twice or across both transports
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_10_single_tool_dispatch_and_no_replay() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let dispatch_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let dc = dispatch_counter.clone();

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let request = read_http_request(&mut socket).await.unwrap_or_default();
            let request_json: Value = request.json_body().unwrap_or(json!({"id": 1}));
            let request_id = request_json.get("id").cloned().unwrap_or(json!(1));
            let count = dc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            if count == 0 {
                // Discovery
                let body = json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "result": {
                        "resultType": "complete",
                        "supportedVersions": [DEFAULT_MCP_PROTOCOL_VERSION],
                        "capabilities": {}
                    }
                });
                let _ = write_json_rpc(&mut socket, 200, &body).await;
            } else if count == 1 {
                let body = json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "result": {
                        "resultType": "complete",
                        "tools": [{
                            "name": "mutate_state",
                            "description": "fixture mutation",
                            "inputSchema": {"type": "object"}
                        }]
                    }
                });
                let _ = write_json_rpc(&mut socket, 200, &body).await;
            } else {
                // Tool call fails with 500
                let resp =
                    "HTTP/1.1 500 Internal Error\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
                let _ = socket.write_all(resp.as_bytes()).await;
                let _ = socket.shutdown().await;
            }
        }
    });

    let http_cfg = McpHttpConfig {
        url: format!("http://127.0.0.1:{}/mcp", port),
        protocol_versions: vec!["2026-07-28".to_string()],
        resource: None,
        headers: None,
        discovery_timeout_ms: None,
        tool_timeout_ms: None,
    };

    let stdio_cfg = get_dummy_stdio_cfg();

    let mut adaptive = AdaptiveMcpClient::new("test-commit", http_cfg, Some(stdio_cfg)).unwrap();
    assert!(adaptive.initialize().await.is_ok());
    assert_eq!(adaptive.state, AdaptiveState::HttpReady);
    assert_eq!(adaptive.list_tools().await.unwrap().len(), 1);

    let call_res = adaptive.call_tool("mutate_state", json!({})).await;
    assert!(call_res.is_err());
    // State must remain HttpCommitted; never fallen back to stdio!
    assert_eq!(adaptive.state, AdaptiveState::HttpCommitted);
    assert!(adaptive.stdio_client.is_none());
    assert_eq!(
        dispatch_counter.load(std::sync::atomic::Ordering::SeqCst),
        3,
        "exactly one discovery, one catalog request, and one tool dispatch"
    );
}

// ---------------------------------------------------------------------------
// Gate 13: Only typed refused and unreachable failures allow fallback
// ---------------------------------------------------------------------------
#[test]
fn test_gate_13_only_typed_refused_and_unreachable_failures_allow_fallback() {
    let mut client =
        McpHttpClient::new("typed-failure", "http://127.0.0.1:3000/mcp", None, None).unwrap();
    let error = AppError::BadRequest("transport failure".to_string());

    client.last_transport_failure = Some(HttpTransportFailureKind::ConnectionRefused);
    assert_eq!(
        client.classify_discovery_error(&error),
        HttpDiscoveryFailureKind::ConnectionRefused
    );
    client.last_transport_failure = Some(HttpTransportFailureKind::HostUnreachable);
    assert_eq!(
        client.classify_discovery_error(&error),
        HttpDiscoveryFailureKind::HostUnreachable
    );
    for failure in [
        Some(HttpTransportFailureKind::Timeout),
        Some(HttpTransportFailureKind::Other),
        None,
    ] {
        client.last_transport_failure = failure;
        assert!(matches!(
            client.classify_discovery_error(&error),
            HttpDiscoveryFailureKind::FailClosed(_)
        ));
    }
}

// ---------------------------------------------------------------------------
// Gate 18: Timeout reset and TLS never start stdio
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_18_timeout_reset_and_tls_never_start_stdio() {
    let stdio = get_dummy_stdio_cfg;

    let timeout_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let timeout_port = timeout_listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (_socket, _) = timeout_listener.accept().await.unwrap();
        tokio::time::sleep(Duration::from_millis(250)).await;
    });
    let mut timeout_client = AdaptiveMcpClient::new(
        "timeout",
        McpHttpConfig {
            url: format!("http://127.0.0.1:{}/mcp", timeout_port),
            protocol_versions: vec![DEFAULT_MCP_PROTOCOL_VERSION.to_string()],
            resource: None,
            headers: None,
            discovery_timeout_ms: Some(30),
            tool_timeout_ms: Some(30),
        },
        Some(stdio()),
    )
    .unwrap();
    assert!(timeout_client.initialize().await.is_err());
    assert_eq!(timeout_client.state, AdaptiveState::FailedClosed);
    assert!(timeout_client.stdio_client.is_none());

    let reset_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let reset_port = reset_listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (mut socket, _) = reset_listener.accept().await.unwrap();
        let _ = read_http_request(&mut socket).await;
        drop(socket);
    });
    let mut reset_client = AdaptiveMcpClient::new(
        "reset",
        McpHttpConfig {
            url: format!("http://127.0.0.1:{}/mcp", reset_port),
            protocol_versions: vec![DEFAULT_MCP_PROTOCOL_VERSION.to_string()],
            resource: None,
            headers: None,
            discovery_timeout_ms: Some(500),
            tool_timeout_ms: Some(500),
        },
        Some(stdio()),
    )
    .unwrap();
    assert!(reset_client.initialize().await.is_err());
    assert_eq!(reset_client.state, AdaptiveState::FailedClosed);
    assert!(reset_client.stdio_client.is_none());

    let tls_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let tls_port = tls_listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (mut socket, _) = tls_listener.accept().await.unwrap();
        let _ = socket.write_all(b"not tls").await;
        let _ = socket.shutdown().await;
    });
    let mut tls_client = AdaptiveMcpClient::new(
        "tls",
        McpHttpConfig {
            url: format!("https://127.0.0.1:{}/mcp", tls_port),
            protocol_versions: vec![DEFAULT_MCP_PROTOCOL_VERSION.to_string()],
            resource: None,
            headers: None,
            discovery_timeout_ms: Some(500),
            tool_timeout_ms: Some(500),
        },
        Some(stdio()),
    )
    .unwrap();
    assert!(tls_client.initialize().await.is_err());
    assert_eq!(tls_client.state, AdaptiveState::FailedClosed);
    assert!(tls_client.stdio_client.is_none());
}
