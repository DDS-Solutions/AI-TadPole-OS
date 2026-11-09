//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Port 3000 Conformance - HTTP Gates
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Exact wire contract conformance for McpHttpClient to Port 3000 (URL path, Host authority, Accept, MCP headers).
//! - `[Structural]` Header redaction at storage time, origin prohibition, session header prohibition, and single-origin scoping.
//! - `[Structural]` Strict HTTP 2026-07-28 protocol enforcement and fail-closed responses for invalid frames or oversized bodies.
//! - `[Structural]` Conformance Gate Map: Gates 1 (config) in `gates_config.rs`; Gates 2-5, 9, 11, 14, 16-17, 19-20 in `gates_http.rs`; Gates 12, 15 in `gates_schema.rs`; Gates 6-8, 10, 13, 18 in `gates_adaptive.rs`.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: assertion failures identify violated HTTP gate.
//! - **Telemetry Targets**: deterministic local loopback TCP fixtures.
//! - **Witness Tests**: `port3000_conformance::tests::gates_http::test_gate_*`

use super::harness::{read_http_request, spawn_one_response, write_json_rpc, ParsedHttpRequest};
use crate::agent::mcp::client::http::{
    encode_header_value_if_needed, extract_and_validate_tool_headers, McpHttpClient,
    MAX_RESPONSE_BODY_BYTES,
};
use crate::agent::mcp::client::jsonrpc::{make_meta, JSONRPC_HEADER_MISMATCH};
use crate::agent::mcp::config::DEFAULT_MCP_PROTOCOL_VERSION;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// ---------------------------------------------------------------------------
// Gate 2: Discovery sends exact URL, Host, Accept, Mcp-Method, version & no Origin
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_2_discovery_request_headers_and_body_spec() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}/mcp", port);

    let server_task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let req_str = read_http_request(&mut socket).await.unwrap();

        // Return valid discovery
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "resultType": "complete",
                "supportedVersions": [DEFAULT_MCP_PROTOCOL_VERSION],
                "capabilities": {}
            }
        });
        write_json_rpc(&mut socket, 200, &body).await.unwrap();
        req_str
    });

    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        "Bearer test-token-xyz".to_string(),
    );

    // Note: "gev-test" is the local host registry name; the MCP client identity on the wire
    // is strictly pinned to "ai-tadpole-os" per the Port 3000 specification.
    let mut client = McpHttpClient::new(
        "gev-test",
        &url,
        Some(&headers),
        Some(DEFAULT_MCP_PROTOCOL_VERSION),
    )
    .unwrap();
    client.initialize().await.unwrap();

    let req = server_task.await.unwrap();

    // Positive controls: prove has_header correctly detects present headers
    assert!(
        req.has_header("host"),
        "Positive control: host header must be detected"
    );
    assert!(
        req.has_header("accept"),
        "Positive control: accept header must be detected"
    );
    assert!(
        req.has_header("mcp-protocol-version"),
        "Positive control: mcp-protocol-version must be detected"
    );
    assert!(
        req.has_header("authorization"),
        "Positive control: authorization header must be detected"
    );

    // 1. HTTP method & path
    assert_eq!(req.method, "POST");
    assert_eq!(req.path, "/mcp");

    // 2. Exact Host header
    let expected_host = format!("127.0.0.1:{}", port);
    assert_eq!(req.header("host"), Some(expected_host.as_str()));

    // 3. Accept header
    assert_eq!(
        req.header("accept"),
        Some("application/json, text/event-stream")
    );

    // 4. MCP-Protocol-Version
    assert_eq!(
        req.header("mcp-protocol-version"),
        Some(DEFAULT_MCP_PROTOCOL_VERSION)
    );

    // 5. Mcp-Method: server/discover
    assert_eq!(req.header("mcp-method"), Some("server/discover"));

    // 5b. Authorization header must reach the wire
    assert_eq!(req.header("authorization"), Some("Bearer test-token-xyz"));

    // 6. Mcp-Name must be omitted on discover
    assert!(!req.has_header("mcp-name"));

    // 7. Origin must be omitted!
    assert!(!req.has_header("origin"));

    // 7b. Session and event headers must be omitted!
    assert!(!req.has_header("last-event-id"));
    assert!(!req.has_header("mcp-session-id"));

    // 8. Body contains clientInfo name "ai-tadpole-os" per specification (both top-level and _meta)
    let body_json = req.json_body().unwrap();
    assert_eq!(body_json["params"]["clientInfo"]["name"], "ai-tadpole-os");
    assert_eq!(
        body_json["params"]["_meta"]["io.modelcontextprotocol/clientInfo"]["name"],
        "ai-tadpole-os"
    );
    assert_eq!(
        body_json["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"],
        DEFAULT_MCP_PROTOCOL_VERSION
    );
}

// ---------------------------------------------------------------------------
// Gate 3: Discovery selects only 2026-07-28 and rejects arbitrary first advertised
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_3_discovery_selects_only_2026_07_28() {
    // Case A: Supported set contains 2026-07-28
    let listener_a = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port_a = listener_a.local_addr().unwrap().port();
    let server_task_a = tokio::spawn(async move {
        let (mut socket, _) = listener_a.accept().await.unwrap();
        let _ = read_http_request(&mut socket).await;
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "resultType": "complete",
                "supportedVersions": [DEFAULT_MCP_PROTOCOL_VERSION, "2024-11-05"],
                "capabilities": {}
            }
        });
        write_json_rpc(&mut socket, 200, &body).await.unwrap();
    });

    let mut client_a = McpHttpClient::new(
        "test-a",
        &format!("http://127.0.0.1:{}/mcp", port_a),
        None,
        None,
    )
    .unwrap();
    assert!(client_a.initialize().await.is_ok());
    assert_eq!(
        client_a.protocol_version.as_deref(),
        Some(DEFAULT_MCP_PROTOCOL_VERSION)
    );
    server_task_a.await.unwrap();

    // Case B: Supported set does NOT contain 2026-07-28 (e.g. only legacy 2025-11-25)
    let listener_b = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port_b = listener_b.local_addr().unwrap().port();
    let server_task_b = tokio::spawn(async move {
        let (mut socket, _) = listener_b.accept().await.unwrap();
        let _ = read_http_request(&mut socket).await;
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "resultType": "complete",
                "supportedVersions": ["2025-11-25"],
                "capabilities": {}
            }
        });
        write_json_rpc(&mut socket, 200, &body).await.unwrap();
    });

    let mut client_b = McpHttpClient::new(
        "test-b",
        &format!("http://127.0.0.1:{}/mcp", port_b),
        None,
        None,
    )
    .unwrap();
    let res_b = client_b.initialize().await;
    // Must fail rather than blindly selecting "2025-11-25"
    assert!(
        res_b.is_err(),
        "Unsupported version must fail closed, got: {:?}",
        res_b
    );
    server_task_b.await.unwrap();
}

// ---------------------------------------------------------------------------
// Gate 4: JSON response and fragmented SSE response both complete correctly
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_4_json_and_fragmented_sse_responses() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}/mcp", port);

    let server_task = tokio::spawn(async move {
        // Request 1: server/discover (returns JSON)
        if let Ok((mut socket, _)) = listener.accept().await {
            let request = read_http_request(&mut socket)
                .await
                .expect("Request 1 read error");
            let request_json: Value = request.json_body().expect("Request 1 invalid JSON body");
            let request_id = request_json.get("id").cloned().unwrap_or(json!(1));
            let body = json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "resultType": "complete",
                    "supportedVersions": [DEFAULT_MCP_PROTOCOL_VERSION],
                    "capabilities": {}
                }
            });
            write_json_rpc(&mut socket, 200, &body).await.unwrap();
        }

        // Request 2: tools/list (returns JSON)
        if let Ok((mut socket, _)) = listener.accept().await {
            let req1 = read_http_request(&mut socket)
                .await
                .expect("Request 2 read error");
            let req1_json: Value = req1.json_body().expect("Request 2 invalid JSON body");
            let req1_id = req1_json.get("id").cloned().unwrap_or(json!(1));

            let body = json!({
                "jsonrpc": "2.0",
                "id": req1_id,
                "result": {
                    "resultType": "complete",
                    "tools": [{
                        "name": "get_budget",
                        "description": "fixture tool",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    }]
                }
            });
            write_json_rpc(&mut socket, 200, &body).await.unwrap();
        }

        // Request 3: tools/call (returns fragmented SSE text/event-stream)
        if let Ok((mut socket, _)) = listener.accept().await {
            let req2 = read_http_request(&mut socket)
                .await
                .expect("Request 3 read error");
            let req2_json: Value = req2.json_body().expect("Request 3 invalid JSON body");
            let req2_id = req2_json.get("id").cloned().unwrap_or(json!(2));

            let headers = "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n";
            let _ = socket.write_all(headers.as_bytes()).await;

            // Chunk 1: comment/keep-alive line
            let chunk1 = ": ping keepalive\n\n";
            let _ = socket
                .write_all(format!("{:X}\r\n{}\r\n", chunk1.len(), chunk1).as_bytes())
                .await;
            tokio::time::sleep(Duration::from_millis(10)).await;

            // Chunk 2: Fragmented data echoing dynamic request id
            let frag1 = format!("data: {{\"jsonrpc\":\"2.0\",\"id\":{},\"res", req2_id);
            let _ = socket
                .write_all(format!("{:X}\r\n{}\r\n", frag1.len(), frag1).as_bytes())
                .await;
            tokio::time::sleep(Duration::from_millis(10)).await;

            // Chunk 3: Final data and dispatch event
            let frag2 = "ult\":{\"resultType\":\"complete\",\"structuredContent\":{\"budget\":1000},\"content\":[{\"type\":\"text\",\"text\":\"Budget 1000\"}]}}\n\n";
            let _ = socket
                .write_all(format!("{:X}\r\n{}\r\n", frag2.len(), frag2).as_bytes())
                .await;

            // End chunked stream
            let _ = socket.write_all(b"0\r\n\r\n").await;
            let _ = socket.shutdown().await;
        }
    });

    let mut client = McpHttpClient::new("test-stream", &url, None, None).unwrap();

    client.initialize().await.unwrap();

    // JSON tools/list
    let tools = client.list_tools().await.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "get_budget");

    // 2. Fragmented SSE tools/call
    let tool_result = client.call_tool("get_budget", json!({})).await.unwrap();
    assert_eq!(tool_result["structuredContent"]["budget"], 1000);
    server_task.await.unwrap();
}

// ---------------------------------------------------------------------------
// Gate 5: Closing SSE response cancels only its originating request and leaves client isolated
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_5_sse_stream_cancellation_and_isolation() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}/mcp", port);

    let (headers_sent_tx, headers_sent_rx) = tokio::sync::oneshot::channel::<()>();
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    let server_task = tokio::spawn(async move {
        // Request 1: established SSE stream that client will cancel
        if let Ok((mut socket, _)) = listener.accept().await {
            let _ = read_http_request(&mut socket).await;

            let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n";
            let _ = socket.write_all(headers.as_bytes()).await;

            // Send a keepalive comment
            let ping = ": keepalive\n\n";
            let _ = socket
                .write_all(format!("{:X}\r\n{}\r\n", ping.len(), ping).as_bytes())
                .await;

            // Notify client that headers and keepalive were sent to the wire
            let _ = headers_sent_tx.send(());

            // Wait to see if client drops connection
            let mut read_buf = [0u8; 64];
            let read_res = socket.read(&mut read_buf).await;
            let client_closed = match read_res {
                Ok(0) => true, // EOF: client closed socket cleanly
                Err(_) => true,
                _ => false,
            };
            let _ = tx.send(client_closed);
        }

        // Request 2: subsequent request on the same client to prove isolation
        if let Ok((mut socket, _)) = listener.accept().await {
            let req = read_http_request(&mut socket).await.unwrap();
            let req_id = req
                .json_body()
                .unwrap()
                .get("id")
                .cloned()
                .unwrap_or(json!(1));
            let body = json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "resultType": "complete",
                    "supportedVersions": [DEFAULT_MCP_PROTOCOL_VERSION],
                    "capabilities": {}
                }
            });
            write_json_rpc(&mut socket, 200, &body).await.unwrap();
        }
    });

    let mut client = McpHttpClient::new("test-cancel", &url, None, None).unwrap();

    // Cancel in-flight stream future while keeping client instance intact
    tokio::select! {
        _ = headers_sent_rx => {
            // Drop the in-flight call_internal future when headers confirmation arrives
        }
        _ = client.call_internal(
            "tools/call",
            Some("long_running"),
            json!({
                "name": "long_running",
                "arguments": {},
                "_meta": make_meta(DEFAULT_MCP_PROTOCOL_VERSION)
            }),
            None,
        ) => {}
    }

    let server_detected_close = tokio::time::timeout(Duration::from_secs(3), rx)
        .await
        .expect("Server must receive client close signal within 3s")
        .expect("Channel must not be dropped");
    assert!(
        server_detected_close,
        "Cancelling stream future must close SSE connection"
    );

    // Verify isolation: A subsequent request on the SAME client must succeed cleanly
    let init_res = client.initialize().await;
    assert!(
        init_res.is_ok(),
        "Client must remain fully functional after prior stream cancellation, got: {:?}",
        init_res
    );
    server_task.await.unwrap();
}

// ---------------------------------------------------------------------------
// Gate 9: No legacy initialize, GET, DELETE, or session ID reaches /mcp
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_9_no_legacy_initialize_sent_to_http() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let recorded_methods = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let recorded_methods_clone = recorded_methods.clone();

    let server_task = tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            if let Ok(req_str) = read_http_request(&mut socket).await {
                if !req_str.method.is_empty() {
                    recorded_methods_clone.lock().unwrap().push(req_str);
                    let resp =
                        "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
                    let _ = socket.write_all(resp.as_bytes()).await;
                    let _ = socket.shutdown().await;
                }
            }
        }
    });

    let mut client = McpHttpClient::new(
        "test-no-legacy",
        &format!("http://127.0.0.1:{}/mcp", port),
        None,
        None,
    )
    .unwrap();

    let _ = client.initialize().await;

    let requests = recorded_methods.lock().unwrap();
    assert!(
        !requests.is_empty(),
        "Server must have received at least one discovery request"
    );
    for req in requests.iter() {
        if let Ok(json_body) = req.json_body() {
            assert_ne!(
                json_body.get("method").and_then(Value::as_str),
                Some("initialize"),
                "Client must NEVER send legacy 'initialize' to /mcp"
            );
        }
        assert_ne!(req.method, "GET", "Client must NEVER send GET to /mcp");
        assert_ne!(
            req.method, "DELETE",
            "Client must NEVER send DELETE to /mcp"
        );
        assert!(
            !req.has_header("mcp-session-id"),
            "Client must NEVER send Mcp-Session-Id"
        );
        assert!(
            !req.has_header("last-event-id"),
            "Client must NEVER send Last-Event-ID"
        );
    }
    server_task.abort();
}

// ---------------------------------------------------------------------------
// Gate 11: Bearer tokens sent only in Authorization and redacted from logs
// ---------------------------------------------------------------------------
#[test]
fn test_gate_11_authorization_bearer_token_redaction() {
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        "Bearer secret-token-12345".to_string(),
    );

    let client = McpHttpClient::new(
        "test-auth",
        "http://127.0.0.1:3000/mcp",
        Some(&headers),
        None,
    )
    .unwrap();

    let debug_str = format!("{:?}", client);
    // Debug representations must NEVER expose the raw secret token
    assert!(
        !debug_str.contains("secret-token-12345"),
        "Bearer token must not leak in Debug representation"
    );
    // Debug representation must explicitly format sanitized token
    assert!(
        debug_str.contains("Bearer [REDACTED]"),
        "Bearer token must be sanitized to Bearer [REDACTED] in Debug representation"
    );
}

#[tokio::test]
async fn test_gate_11b_bearer_token_is_redacted_from_peer_error_logs() {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32001,
            "message": "rejected Bearer secret-token-12345",
            "data": {"echo": "secret-token-12345"}
        }
    })
    .to_string();
    let url = spawn_one_response(Some("application/json"), body).await;
    let headers = HashMap::from([(
        "Authorization".to_string(),
        "Bearer secret-token-12345".to_string(),
    )]);
    let mut client = McpHttpClient::new("test-auth-error", &url, Some(&headers), None).unwrap();
    let error = client.initialize().await.unwrap_err();
    assert!(!error.to_string().contains("secret-token-12345"));
    assert!(!format!("{:?}", client).contains("secret-token-12345"));
    assert_eq!(
        client.last_raw_error.as_ref().map(|error| error.code),
        Some(-32001)
    );
    let raw_err = client.last_raw_error.as_ref().unwrap();
    assert!(
        !raw_err.message.contains("secret-token-12345"),
        "Raw error message must have secrets redacted at storage time"
    );
    assert!(raw_err.message.contains("[REDACTED]"));
    let data_str = serde_json::to_string(&raw_err.data).unwrap();
    assert!(
        !data_str.contains("secret-token-12345"),
        "Raw error data must have secrets redacted at storage time"
    );
    assert!(data_str.contains("[REDACTED]"));
}

// ---------------------------------------------------------------------------
// Gate 14: Invalid identity, content type, and response limits fail closed
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_14_invalid_identity_content_type_and_response_limits_fail_closed() {
    let valid_result = json!({
        "resultType": "complete",
        "supportedVersions": [DEFAULT_MCP_PROTOCOL_VERSION],
        "capabilities": {}
    });
    let cases = [
        (
            Some("application/json"),
            json!({"jsonrpc": "1.0", "id": 1, "result": valid_result.clone()}).to_string(),
            "Invalid JSON-RPC response",
        ),
        (
            Some("application/json"),
            json!({"jsonrpc": "2.0", "id": 99, "result": valid_result.clone()}).to_string(),
            "Invalid JSON-RPC response",
        ),
        (
            Some("application/json"),
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": valid_result.clone(),
                "error": {"code": -32600, "message": "both"}
            })
            .to_string(),
            "Invalid JSON-RPC response",
        ),
        (
            Some("application/json"),
            "{malformed".to_string(),
            "malformed JSON-RPC",
        ),
        (
            Some("text/plain"),
            json!({"jsonrpc": "2.0", "id": 1, "result": valid_result.clone()}).to_string(),
            "unsupported content type 'text/plain'",
        ),
        (
            None,
            json!({"jsonrpc": "2.0", "id": 1, "result": valid_result.clone()}).to_string(),
            "unsupported content type ''",
        ),
    ];

    for (content_type, body, expected_err_substring) in cases {
        let url = spawn_one_response(content_type, body).await;
        let mut client = McpHttpClient::new("invalid-response", &url, None, None).unwrap();
        let init_res = client.initialize().await;
        assert!(
            init_res.is_err(),
            "Expected failure for case {:?}, got: {:?}",
            content_type,
            init_res
        );
        let err_msg = init_res.unwrap_err().to_string();
        assert!(
            err_msg.contains(expected_err_substring),
            "Expected error containing {:?}, got {:?}",
            expected_err_substring,
            err_msg
        );
    }

    // Provable oversized response body with valid JSON syntax
    let prefix = r#"{"jsonrpc":"2.0","id":1,"result":{"resultType":"complete","supportedVersions":["2026-07-28"],"pad":""#;
    let suffix = r#""}}"#;
    let pad_len = (MAX_RESPONSE_BODY_BYTES + 1).saturating_sub(prefix.len() + suffix.len());
    let oversized = format!("{}{}{}", prefix, "A".repeat(pad_len), suffix);
    assert_eq!(oversized.len(), MAX_RESPONSE_BODY_BYTES + 1);

    let url = spawn_one_response(Some("application/json"), oversized).await;
    let mut client = McpHttpClient::new("oversized-response", &url, None, None).unwrap();
    let err = client.initialize().await.unwrap_err();
    assert!(
        err.to_string().contains("exceeded the response body limit"),
        "Oversized response must trigger response body limit error, got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Gate 16: Catalog headers and stable operation IDs
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_16_catalog_headers_and_stable_operation_ids() {
    tokio::time::timeout(Duration::from_secs(5), async {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let requests = std::sync::Arc::new(std::sync::Mutex::new(Vec::<ParsedHttpRequest>::new()));
        let captured = requests.clone();
        let server = tokio::spawn(async move {
            for index in 0..5 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let request = read_http_request(&mut socket).await.unwrap();
                let request_json: Value = request.json_body().unwrap();
                let request_id = request_json.get("id").cloned().unwrap();
                captured.lock().unwrap().push(request);
                let result = match index {
                    0 => json!({
                        "resultType": "complete",
                        "supportedVersions": [DEFAULT_MCP_PROTOCOL_VERSION],
                        "capabilities": {}
                    }),
                    1 => json!({
                        "resultType": "complete",
                        "tools": [{
                            "name": "save_scene",
                            "description": "fixture",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "scene": {
                                        "type": "object",
                                        "properties": {
                                            "id": {
                                                "type": "string",
                                                "x-mcp-header": "Scene-Id"
                                            }
                                        }
                                    }
                                }
                            }
                        }]
                    }),
                    _ => json!({
                        "resultType": "complete",
                        "content": [{"type": "text", "text": "saved"}],
                        "structuredContent": {"saved": true},
                        "isError": false,
                        "_meta": {"execution": {"retryable": index == 2}}
                    }),
                };
                let body = json!({"jsonrpc": "2.0", "id": request_id, "result": result});
                write_json_rpc(&mut socket, 200, &body).await.unwrap();
            }
        });

        let url = format!("http://127.0.0.1:{}/mcp", port);
        let mut client = McpHttpClient::new("catalog", &url, None, None).unwrap();
        client.initialize().await.unwrap();
        assert_eq!(client.list_tools().await.unwrap().len(), 1);

        let operation_id = "00000000-0000-4000-8000-000000000001";
        let arguments = json!({"scene": {"id": "=?sentinel?="}});
        client
            .call_tool_with_operation_id("save_scene", arguments.clone(), operation_id)
            .await
            .unwrap();
        client
            .call_tool_with_operation_id("save_scene", arguments, operation_id)
            .await
            .unwrap();

        assert!(client
            .call_tool_with_operation_id(
                "save_scene",
                json!({"scene": {"id": "=?sentinel?="}}),
                operation_id,
            )
            .await
            .is_err());

        assert!(client
            .call_tool_with_operation_id(
                "save_scene",
                json!({"scene": {"id": "different"}}),
                operation_id,
            )
            .await
            .is_err());
        assert!(client.call_tool("not_in_catalog", json!({})).await.is_err());

        client
            .call_tool_with_operation_id(
                "save_scene",
                json!({"scene": {"id": null}}),
                "00000000-0000-4000-8000-000000000002",
            )
            .await
            .unwrap();
        server.await.unwrap();

        let captured = requests.lock().unwrap();
        assert_eq!(captured.len(), 5);
        assert!(
            captured[0].has_header("host"),
            "Positive control: host header present"
        );
        let expected_header = encode_header_value_if_needed("=?sentinel?=");
        for request in [&captured[2], &captured[3]] {
            assert_eq!(request.header("mcp-method"), Some("tools/call"));
            assert_eq!(request.header("mcp-name"), Some("save_scene"));
            assert_eq!(
                request.header("mcp-param-scene-id"),
                Some(expected_header.as_str())
            );
            assert!(request.body.contains(operation_id));
        }
        assert!(!captured[4].has_header("mcp-param-scene-id"));

        // Defense-in-depth: verify reserved header name in x-mcp-header is rejected at extraction time
        let bad_schema = json!({
            "type": "object",
            "properties": {
                "token": {
                    "type": "string",
                    "x-mcp-header": "Authorization"
                }
            }
        });
        assert!(extract_and_validate_tool_headers("malicious_tool", &bad_schema).is_err());
    })
    .await
    .expect("Gate 16 must complete within 5 seconds");
}

// ---------------------------------------------------------------------------
// Gate 17: Concurrent SSE responses remain strictly request-scoped across connections
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_17_concurrent_sse_responses_remain_request_scoped() {
    tokio::time::timeout(Duration::from_secs(5), async {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let mut handlers = Vec::new();
            for _ in 0..2 {
                let (mut socket, _) = listener.accept().await.unwrap();
                handlers.push(tokio::spawn(async move {
                    let request = read_http_request(&mut socket).await.unwrap();
                    let request_json: Value = request.json_body().unwrap();
                    let request_id = request_json.get("id").cloned().unwrap();
                    let marker = request_json["params"]["marker"]
                        .as_str()
                        .unwrap()
                        .to_string();
                    if marker == "slow" {
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                    let event = format!(
                        "data: {}\n\n",
                        json!({
                            "jsonrpc": "2.0",
                            "id": request_id,
                            "result": {"marker": marker}
                        })
                    );
                    socket
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n",
                        )
                        .await
                        .unwrap();
                    socket
                        .write_all(format!("{:X}\r\n{}\r\n0\r\n\r\n", event.len(), event).as_bytes())
                        .await
                        .unwrap();
                }));
            }
            for handler in handlers {
                handler.await.unwrap();
            }
        });

        let url = format!("http://127.0.0.1:{}/mcp", port);
        let mut slow = McpHttpClient::new("slow", &url, None, None).unwrap();
        let mut fast = McpHttpClient::new("fast", &url, None, None).unwrap();
        let (slow_result, fast_result) = tokio::join!(
            slow.call_internal("test/echo", None, json!({"marker": "slow"}), None),
            fast.call_internal("test/echo", None, json!({"marker": "fast"}), None),
        );
        assert_eq!(slow_result.unwrap()["marker"], "slow");
        assert_eq!(fast_result.unwrap()["marker"], "fast");
        server.await.unwrap();
    })
    .await
    .expect("Gate 17 must complete within 5 seconds");
}

// ---------------------------------------------------------------------------
// Gate 19: Header mismatch requires catalog refresh before safe retry
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_19_header_mismatch_requires_catalog_refresh_before_safe_retry() {
    tokio::time::timeout(Duration::from_secs(5), async {
        assert_eq!(
            JSONRPC_HEADER_MISMATCH, -32020,
            "JSONRPC_HEADER_MISMATCH literal pin"
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let dispatches = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = dispatches.clone();
        let server = tokio::spawn(async move {
            for index in 0..5 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let request = read_http_request(&mut socket).await.unwrap();
                observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let request_json: Value = request.json_body().unwrap_or(json!({}));
                let id = request_json["id"].clone();
                let envelope = match index {
                    0 => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "resultType": "complete",
                            "supportedVersions": [DEFAULT_MCP_PROTOCOL_VERSION],
                            "capabilities": {}
                        }
                    }),
                    1 | 3 => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "resultType": "complete",
                            "tools": [{
                                "name": "save_scene",
                                "description": "fixture",
                                "inputSchema": {"type": "object"}
                            }]
                        }
                    }),
                    2 => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": JSONRPC_HEADER_MISMATCH,
                            "message": "catalog changed",
                            "data": {"retryable": true}
                        }
                    }),
                    _ => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "resultType": "complete",
                            "content": [{"type": "text", "text": "saved"}],
                            "isError": false,
                            "_meta": {"execution": {"retryable": false}}
                        }
                    }),
                };
                write_json_rpc(&mut socket, 200, &envelope).await.unwrap();
            }
        });

        let mut client = McpHttpClient::new(
            "header-refresh",
            &format!("http://127.0.0.1:{}/mcp", port),
            None,
            None,
        )
        .unwrap();
        client.initialize().await.unwrap();
        client.list_tools().await.unwrap();
        let operation_id = "00000000-0000-4000-8000-000000000019";
        assert!(client
            .call_tool_with_operation_id("save_scene", json!({}), operation_id)
            .await
            .is_err());
        assert!(client
            .call_tool_with_operation_id("save_scene", json!({}), operation_id)
            .await
            .is_err());
        assert_eq!(
            dispatches.load(std::sync::atomic::Ordering::SeqCst),
            3,
            "retry must be blocked locally before catalog refresh"
        );
        client.list_tools().await.unwrap();
        client
            .call_tool_with_operation_id("save_scene", json!({}), operation_id)
            .await
            .unwrap();
        server.await.unwrap();
        assert_eq!(dispatches.load(std::sync::atomic::Ordering::SeqCst), 5);
    })
    .await
    .expect("Gate 19 must complete within 5 seconds");
}

// ---------------------------------------------------------------------------
// Gate 20: SSE unterminated final data line flushes successfully
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_gate_20_sse_unterminated_final_data_line_flushes_successfully() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let _ = read_http_request(&mut socket).await;
        // Deliberately omit trailing newline before connection shutdown
        let event = format!(
            "data: {}",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"status": "unterminated-flush-ok"}
            })
        );
        let headers =
            "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: text/event-stream\r\n\r\n";
        let _ = socket.write_all(headers.as_bytes()).await;
        let _ = socket.write_all(event.as_bytes()).await;
        let _ = socket.shutdown().await;
    });

    let url = format!("http://127.0.0.1:{}/mcp", port);
    let mut client = McpHttpClient::new("sse-flush", &url, None, None).unwrap();
    let res = client
        .call_internal("test/unterminated", None, json!({}), None)
        .await
        .unwrap();
    assert_eq!(res["status"], "unterminated-flush-ok");
    server_task.await.unwrap();

    // Negative control: Truncated JSON-RPC frame in SSE stream fails closed
    let trunc_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let trunc_port = trunc_listener.local_addr().unwrap().port();
    let trunc_task = tokio::spawn(async move {
        let (mut socket, _) = trunc_listener.accept().await.unwrap();
        let _ = read_http_request(&mut socket).await;
        let truncated_event = "data: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"status\":"; // truncated
        let headers =
            "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: text/event-stream\r\n\r\n";
        let _ = socket.write_all(headers.as_bytes()).await;
        let _ = socket.write_all(truncated_event.as_bytes()).await;
        let _ = socket.shutdown().await;
    });

    let trunc_url = format!("http://127.0.0.1:{}/mcp", trunc_port);
    let mut trunc_client = McpHttpClient::new("sse-trunc", &trunc_url, None, None).unwrap();
    let trunc_res = trunc_client
        .call_internal("test/truncated", None, json!({}), None)
        .await;
    assert!(
        trunc_res.is_err(),
        "Truncated JSON in SSE stream must fail closed, got: {:?}",
        trunc_res
    );
    trunc_task.await.unwrap();
}
