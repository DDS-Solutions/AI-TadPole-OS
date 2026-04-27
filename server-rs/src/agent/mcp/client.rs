use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tracing::{debug, error, info};

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

pub struct McpClient {
    #[allow(dead_code)]
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpClient {
    pub async fn spawn(command_line: &str) -> Result<Self> {
        info!("🚀 [MCP] Spawning server: {}", command_line);
        
        let mut parts = command_line.split_whitespace();
        let program = parts.next().ok_or_else(|| anyhow::anyhow!("Empty command"))?;
        let args: Vec<&str> = parts.collect();

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Log stderr to the console
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to open stdout"))?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(id),
            method: method.to_string(),
            params,
        };

        let request_str = serde_json::to_string(&request)? + "\n";
        debug!(">> [MCP] Sending: {}", request_str.trim());
        self.stdin.write_all(request_str.as_bytes()).await?;
        self.stdin.flush().await?;

        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line).await?;
        debug!("<< [MCP] Received: {}", response_line.trim());

        if response_line.is_empty() {
            return Err(anyhow::anyhow!("MCP server closed connection"));
        }

        let response: JsonRpcResponse = serde_json::from_str(&response_line)?;
        
        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("MCP Error: {}", error));
        }

        response.result.ok_or_else(|| anyhow::anyhow!("Missing result in MCP response"))
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "tadpole-os-engine",
                "version": "1.1.0"
            }
        });

        self.call("initialize", params).await?;
        
        // Notify initialized
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let notif_str = serde_json::to_string(&notification)? + "\n";
        self.stdin.write_all(notif_str.as_bytes()).await?;
        self.stdin.flush().await?;

        Ok(())
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Value>> {
        let result = self.call("tools/list", json!({})).await?;
        let tools = result.get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(tools)
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        let params = json!({
            "name": name,
            "arguments": arguments
        });
        self.call("tools/call", params).await
    }
}
