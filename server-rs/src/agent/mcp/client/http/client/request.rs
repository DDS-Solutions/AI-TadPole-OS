//! @docs ARCHITECTURE:Registry:Mcp
//! ### AI Assist Note
//! Low-level JSON-RPC HTTP transport execution, bounded payload reception, and SSE parsing.
//! ### 🔍 Debugging & Observability
//! Dispatches HTTP POST requests with Content-Type-driven SSE and JSON stream decoders.

use crate::agent::mcp::client::jsonrpc::JsonRpcResponse;
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

use super::super::body::read_bounded_body;
use super::super::classify::{classify_reqwest_error, HttpTransportFailureKind};
use super::super::sse::parse_sse_stream;

pub struct TransportParams<'a> {
    pub server_name: &'a str,
    pub url: &'a str,
    pub host_authority: &'a str,
    pub proto: &'a str,
    pub headers: &'a reqwest::header::HeaderMap,
    pub timeout: std::time::Duration,
    pub client: &'a reqwest::Client,
}

pub async fn execute_http_post(
    params: &TransportParams<'_>,
    method: &str,
    tool_name: Option<&str>,
    req_body: Vec<u8>,
    extra_headers: Option<HashMap<String, String>>,
) -> Result<reqwest::Response, (AppError, Option<HttpTransportFailureKind>)> {
    let mut req_builder = params
        .client
        .post(params.url)
        .timeout(params.timeout)
        .headers(params.headers.clone())
        .header("Host", params.host_authority)
        .header("MCP-Protocol-Version", params.proto)
        .header("Mcp-Method", method);

    if let Some(name) = tool_name {
        req_builder = req_builder.header("Mcp-Name", name);
    }
    if let Some(extra) = extra_headers {
        for (k, v) in extra {
            req_builder = req_builder.header(k, v);
        }
    }

    debug!(
        "[MCP HTTP] Sending {method} for '{}' to {}",
        params.server_name, params.url
    );

    req_builder.body(req_body).send().await.map_err(|e| {
        let failure = classify_reqwest_error(&e);
        let kind = match failure {
            HttpTransportFailureKind::Timeout => InfrastructureErrorKind::Timeout,
            HttpTransportFailureKind::ConnectionRefused
            | HttpTransportFailureKind::HostUnreachable => InfrastructureErrorKind::NetworkError,
            _ => InfrastructureErrorKind::Other,
        };
        let err = AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind,
            detail: format!(
                "Failed to send request to MCP HTTP server '{}': {e}",
                params.server_name
            ),
            help_link: None,
        };
        (err, Some(failure))
    })
}

pub async fn parse_rpc_response(
    response: reqwest::Response,
    server_name: &str,
    expected_id: &Value,
    content_type: &str,
) -> Result<JsonRpcResponse, AppError> {
    if content_type == "text/event-stream" {
        return parse_sse_stream(response, server_name, expected_id).await;
    }
    if content_type == "application/json" {
        let body = read_bounded_body(response, server_name).await?;
        return serde_json::from_slice::<JsonRpcResponse>(&body).map_err(|_| {
            AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!("MCP server '{server_name}' returned malformed JSON-RPC"),
                help_link: None,
            }
        });
    }

    Err(AppError::InfrastructureError {
        provider_id: ProviderId::Mcp,
        kind: InfrastructureErrorKind::ApiError,
        detail: format!(
            "MCP server '{server_name}' returned unsupported content type '{content_type}'"
        ),
        help_link: None,
    })
}
