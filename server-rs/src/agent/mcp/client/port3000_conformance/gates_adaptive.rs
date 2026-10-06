//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Port 3000 Conformance - Adaptive Gates
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Adaptive fallback and commit state machine invariants.
//! - `[Structural]` Modern 2026-07-28 HTTP primary; stdio fallback only on approved pre-tool triggers (connection refused, host unreachable, unsupported version without 2026-07-28 intersection).
//! - `[Structural]` Fail-closed transitions on unapproved errors (4xx, 5xx), timeouts, protocol inconsistencies, and committed tool execution (never fallback to stdio).
//! - `[Structural]` Conformance Gate Map: Gates 1 (config) in `gates_config.rs`; Gates 2-5, 9, 11, 14, 16-17, 19-20 in `gates_http.rs`; Gates 12, 15 in `gates_schema.rs`; Gates 6-8, 10, 13, 18 in `gates_adaptive.rs`.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: assertions on `AdaptiveState` transitions and failure classifications.
//! - **Telemetry Targets**: local loopback TCP fixtures and mock stdio processes.
//! - **Witness Tests**: `port3000_conformance::tests::gates_adaptive::test_gate_*`

use super::harness::{get_dummy_stdio_cfg, get_test_python_cmd, read_http_request, write_json_rpc};
use crate::agent::mcp::client::adaptive::{AdaptiveMcpClient, AdaptiveState};
use crate::agent::mcp::client::http::{
    HttpDiscoveryFailureKind, HttpTransportFailureKind, McpHttpClient,
};
use crate::agent::mcp::client::jsonrpc::JSONRPC_UNSUPPORTED_PROTOCOL_VERSION;
use crate::agent::mcp::client::MCP_PROTOCOL_2024_11_05;
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
        protocol_versions: vec![DEFAULT_MCP_PROTOCOL_VERSION.to_string()],
        resource: None,
        headers: None,
        discovery_timeout_ms: None,
        tool_timeout_ms: None,
    };

    let stdio_cfg = crate::agent::mcp::config::McpStdioConfig {
        command: get_test_python_cmd(),
        args: vec![
            "-c".to_string(),
            format!(
                "import sys; sys.stdin.readline(); sys.stdout.write('{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"resultType\":\"complete\",\"supportedVersions\":[\"{}\"],\"capabilities\":{{}}}}}}\\n'); sys.stdout.flush()",
                DEFAULT_MCP_PROTOCOL_VERSION
            ),
        ],
        cwd: None,
        env: None,
    };

    let mut adaptive = AdaptiveMcpClient::new("test-refusal", http_cfg, Some(stdio_cfg)).unwrap();

    assert_eq!(adaptive.state, AdaptiveState::Configured);
    let init_res = adaptive.initialize().await;
    assert!(
        init_res.is_ok(),
        "Adaptive client must succeed by falling back to stdio, got: {:?}",
        init_res
    );
    assert_eq!(adaptive.state, AdaptiveState::StdioReady);
}

// ---------------------------------------------------------------------------
// Gate 7: Structured -32022 is preserved; mutual vs absent intersections
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_7_structured_32022_error_preservation() {
    // Case A: Absent intersection (supported: ["2024-11-05"]) on HTTP 400 -> selects stdio fallback
    let listener_a = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port_a = listener_a.local_addr().unwrap().port();

    let server_task_a = tokio::spawn(async move {
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
                    "supported": [MCP_PROTOCOL_2024_11_05]
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
            format!(
                "import sys; sys.stdin.readline(); sys.stdout.write('{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"resultType\":\"complete\",\"supportedVersions\":[\"{}\"],\"capabilities\":{{}}}}}}\\n'); sys.stdout.flush()",
                DEFAULT_MCP_PROTOCOL_VERSION
            ),
        ],
        cwd: None,
        env: None,
    };

    let mut adaptive_a =
        AdaptiveMcpClient::new("test-absent", http_cfg_a, Some(stdio_cfg_a)).unwrap();
    let init_res_a = adaptive_a.initialize().await;
    assert!(
        init_res_a.is_ok(),
        "Expected stdio fallback to succeed on HTTP 400, got: {:?}",
        init_res_a
    );
    assert_eq!(adaptive_a.state, AdaptiveState::StdioReady);
    let _ = server_task_a.await;

    // Case B: Protocol inconsistency (-32022 rejecting requested 2026-07-28 while advertising it)
    let listener_b = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port_b = listener_b.local_addr().unwrap().port();

    let server_task_b = tokio::spawn(async move {
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
    assert!(
        res_b.is_err(),
        "Protocol inconsistency must fail closed, got: {:?}",
        res_b
    );
    assert_eq!(adaptive_b.state, AdaptiveState::FailedClosed);
    let _ = server_task_b.await;

    // Case C: Absent intersection on HTTP 200 OK with error body -> selects stdio fallback
    let listener_c = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port_c = listener_c.local_addr().unwrap().port();

    let server_task_c = tokio::spawn(async move {
        let (mut socket, _) = listener_c.accept().await.unwrap();
        let _ = read_http_request(&mut socket).await;
        let err_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": JSONRPC_UNSUPPORTED_PROTOCOL_VERSION,
                "message": "Unsupported protocol version",
                "data": {
                    "requested": DEFAULT_MCP_PROTOCOL_VERSION,
                    "supported": [MCP_PROTOCOL_2024_11_05]
                }
            }
        });
        let _ = write_json_rpc(&mut socket, 200, &err_body).await;
    });

    let http_cfg_c = McpHttpConfig {
        url: format!("http://127.0.0.1:{}/mcp", port_c),
        protocol_versions: vec![DEFAULT_MCP_PROTOCOL_VERSION.to_string()],
        resource: None,
        headers: None,
        discovery_timeout_ms: None,
        tool_timeout_ms: None,
    };

    let stdio_cfg_c = crate::agent::mcp::config::McpStdioConfig {
        command: get_test_python_cmd(),
        args: vec![
            "-c".to_string(),
            format!(
                "import sys; sys.stdin.readline(); sys.stdout.write('{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"resultType\":\"complete\",\"supportedVersions\":[\"{}\"],\"capabilities\":{{}}}}}}\\n'); sys.stdout.flush()",
                DEFAULT_MCP_PROTOCOL_VERSION
            ),
        ],
        cwd: None,
        env: None,
    };

    let mut adaptive_c =
        AdaptiveMcpClient::new("test-absent-200", http_cfg_c, Some(stdio_cfg_c)).unwrap();
    let init_res_c = adaptive_c.initialize().await;
    assert!(
        init_res_c.is_ok(),
        "Expected stdio fallback to succeed on HTTP 200 with -32022 error, got: {:?}",
        init_res_c
    );
    assert_eq!(adaptive_c.state, AdaptiveState::StdioReady);
    let _ = server_task_c.await;
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

        // Keep listener open for multiple connections to prove no retries occur
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
        let server_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    conn = listener.accept() => {
                        if let Ok((mut socket, _)) = conn {
                            cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            let _ = read_http_request(&mut socket).await;
                            let empty_json = json!({});
                            let _ = write_json_rpc(&mut socket, status_code, &empty_json).await;
                        }
                    }
                }
            }
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
            "Status {} MUST fail closed and not invoke stdio, got: {:?}",
            status_code,
            init_res
        );
        assert_eq!(
            adaptive.state,
            AdaptiveState::FailedClosed,
            "Status {} must transition to FailedClosed",
            status_code
        );

        // Small wait to prove no rogue retry connects before server stops
        tokio::time::sleep(Duration::from_millis(25)).await;
        let _ = stop_tx.send(());
        let _ = server_task.await;

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

    let discover_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let list_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let call_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let dc = discover_counter.clone();
    let lc = list_counter.clone();
    let cc = call_counter.clone();

    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
    let server_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
                conn = listener.accept() => {
                    if let Ok((mut socket, _)) = conn {
                        let request = read_http_request(&mut socket).await.unwrap_or_default();
                        let request_json: Value = request.json_body().expect("Gate 10 fixture received invalid JSON body");
                        let request_id = request_json.get("id").cloned().unwrap_or(json!(1));
                        let method = request_json.get("method").and_then(Value::as_str).unwrap_or("");

                        match method {
                            "server/discover" => {
                                dc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
                            }
                            "tools/list" => {
                                lc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
                            }
                            "tools/call" => {
                                cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                let resp =
                                    "HTTP/1.1 500 Internal Error\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
                                let _ = socket.write_all(resp.as_bytes()).await;
                                let _ = socket.shutdown().await;
                            }
                            unexpected => {
                                panic!("Gate 10 fixture received unexpected method: {unexpected}");
                            }
                        }
                    }
                }
            }
        }
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

    let mut adaptive = AdaptiveMcpClient::new("test-commit", http_cfg, Some(stdio_cfg)).unwrap();
    assert!(adaptive.initialize().await.is_ok());
    assert_eq!(adaptive.state, AdaptiveState::HttpReady);
    assert_eq!(adaptive.list_tools().await.unwrap().len(), 1);

    let call_res = adaptive.call_tool("mutate_state", json!({})).await;
    assert!(
        call_res.is_err(),
        "Tool call must fail with 500, got: {:?}",
        call_res
    );
    // State must remain HttpCommitted; never fallen back to stdio!
    assert_eq!(adaptive.state, AdaptiveState::HttpCommitted);
    assert!(adaptive.stdio_client.is_none());

    let _ = stop_tx.send(());
    let _ = server_task.await;

    assert_eq!(
        discover_counter.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "exactly one discovery dispatched"
    );
    assert_eq!(
        list_counter.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "exactly one catalog request dispatched"
    );
    assert_eq!(
        call_counter.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "tool dispatched exactly once"
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
    let timeout_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let timeout_port = timeout_listener.local_addr().unwrap().port();
    let timeout_task = tokio::spawn(async move {
        if let Ok((_socket, _)) = timeout_listener.accept().await {
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
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
        Some(get_dummy_stdio_cfg()),
    )
    .unwrap();

    let start = std::time::Instant::now();
    let timeout_res = timeout_client.initialize().await;
    let elapsed = start.elapsed();

    assert!(
        timeout_res.is_err(),
        "Timeout must fail closed, got: {:?}",
        timeout_res
    );
    assert!(
        elapsed < Duration::from_millis(180),
        "Timeout client must abort around discovery_timeout_ms (30ms), took {:?}",
        elapsed
    );
    assert_eq!(timeout_client.state, AdaptiveState::FailedClosed);
    assert!(timeout_client.stdio_client.is_none());
    timeout_task.abort();

    let reset_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let reset_port = reset_listener.local_addr().unwrap().port();
    let reset_task = tokio::spawn(async move {
        if let Ok((mut socket, _)) = reset_listener.accept().await {
            let _ = read_http_request(&mut socket).await;
            drop(socket);
        }
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
        Some(get_dummy_stdio_cfg()),
    )
    .unwrap();
    let reset_res = reset_client.initialize().await;
    assert!(
        reset_res.is_err(),
        "Connection reset must fail closed, got: {:?}",
        reset_res
    );
    assert_eq!(reset_client.state, AdaptiveState::FailedClosed);
    assert!(reset_client.stdio_client.is_none());
    let _ = reset_task.await;

    let tls_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let tls_port = tls_listener.local_addr().unwrap().port();
    let tls_task = tokio::spawn(async move {
        if let Ok((mut socket, _)) = tls_listener.accept().await {
            let _ = socket.write_all(b"not tls").await;
            let _ = socket.shutdown().await;
        }
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
        Some(get_dummy_stdio_cfg()),
    )
    .unwrap();
    let tls_res = tls_client.initialize().await;
    assert!(
        tls_res.is_err(),
        "TLS handshake failure must fail closed, got: {:?}",
        tls_res
    );
    assert_eq!(tls_client.state, AdaptiveState::FailedClosed);
    assert!(tls_client.stdio_client.is_none());
    let _ = tls_task.await;
}
