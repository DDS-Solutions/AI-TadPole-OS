//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Port 3000 Conformance Harness
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic local HTTP and stdio test harness for Port 3000 conformance witnesses.
//! - `[Structural]` No secret token emission in test assertions or logging.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `std::io::Error` on socket timeouts or parsing violations.
//! - **Telemetry Targets**: local loopback TCP fixtures only.
//! - **Witness Tests**: `port3000_conformance::tests::*`

use crate::agent::mcp::config::McpStdioConfig;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Resolved python executable for hermetic testing across Linux (python3) and Windows (python) environments.
pub fn get_test_python_cmd() -> String {
    if let Ok(output) = std::process::Command::new("python3")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            return "python3".to_string();
        }
    }
    "python".to_string()
}

/// Dummy stdio config that fails immediately if invoked.
/// Used in fail-closed tests to guarantee stdio is never spawned.
pub fn get_dummy_stdio_cfg() -> McpStdioConfig {
    McpStdioConfig {
        command: "invalid-stdio-command-must-never-run".to_string(),
        args: vec![],
        cwd: None,
        env: None,
    }
}

/// Structured HTTP request parser providing exact header lookups and body extractions.
#[derive(Debug, Clone, Default)]
pub struct ParsedHttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub raw: String,
}

impl ParsedHttpRequest {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    pub fn has_header(&self, name: &str) -> bool {
        self.headers.contains_key(&name.to_ascii_lowercase())
    }

    pub fn json_body(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_str(&self.body)
    }
}

/// Helper: Read full HTTP request with a 5-second timeout and 2 MiB cap to prevent hanging and TCP fragmentation flakes.
pub async fn read_http_request(socket: &mut TcpStream) -> std::io::Result<ParsedHttpRequest> {
    let read_future = async {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        let max_read_bytes = 2 * 1024 * 1024; // 2 MiB test harness cap
        loop {
            let n = socket.read(&mut chunk).await?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if buf.len() > max_read_bytes {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "HTTP request exceeded maximum allowed test harness limit (2 MiB)",
                ));
            }
            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let header_str = String::from_utf8_lossy(&buf[..pos]);
                let mut content_length: Option<usize> = None;
                for line in header_str.lines() {
                    let lower = line.to_ascii_lowercase();
                    if let Some(val_str) = lower.strip_prefix("content-length:") {
                        if let Ok(len) = val_str.trim().parse::<usize>() {
                            content_length = Some(len);
                        }
                    }
                }
                if let Some(cl) = content_length {
                    let body_read = buf.len() - (pos + 4);
                    if body_read >= cl {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        let raw = String::from_utf8_lossy(&buf).into_owned();
        let (header_part, body_part) = match raw.split_once("\r\n\r\n") {
            Some((h, b)) => (h, b),
            None => (raw.as_str(), ""),
        };

        let mut lines = header_part.lines();
        let req_line = lines.next().unwrap_or("");
        let parts: Vec<&str> = req_line.split_whitespace().collect();
        let method = parts.first().copied().unwrap_or("").to_string();
        let path = parts.get(1).copied().unwrap_or("").to_string();
        let version = parts.get(2).copied().unwrap_or("").to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if let Some((name, val)) = line.split_once(':') {
                headers.insert(name.trim().to_ascii_lowercase(), val.trim().to_string());
            }
        }

        Ok(ParsedHttpRequest {
            method,
            path,
            version,
            headers,
            body: body_part.to_string(),
            raw,
        })
    };

    match tokio::time::timeout(Duration::from_secs(5), read_future).await {
        Ok(res) => res,
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Timeout reading HTTP request in test harness",
        )),
    }
}

/// Helper to write a complete JSON-RPC HTTP response with Content-Length and close the connection.
pub async fn write_json_rpc(
    socket: &mut TcpStream,
    status: u16,
    body: &Value,
) -> std::io::Result<()> {
    let body_str = body.to_string();
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Status",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status,
        status_text,
        body_str.len(),
        body_str
    );
    socket.write_all(response.as_bytes()).await?;
    socket.shutdown().await
}

/// Spawn a one-shot TCP server responding with the given status/body and return its URL.
pub async fn spawn_one_response(content_type: Option<&str>, body: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let content_type = content_type.map(str::to_string);
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let _ = read_http_request(&mut socket).await;
        let content_type_header = content_type
            .map(|value| format!("Content-Type: {}\r\n", value))
            .unwrap_or_default();
        let response = format!(
            "HTTP/1.1 200 OK\r\nConnection: close\r\n{}Content-Length: {}\r\n\r\n{}",
            content_type_header,
            body.len(),
            body
        );
        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.shutdown().await;
    });
    format!("http://127.0.0.1:{}/mcp", port)
}
