//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP HTTP Client
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` MCP 2026-07-28 Stateless Streamable HTTP specification conformance.
//! - `[Structural]` Fail-closed: Discovery timeout, TLS errors, and 4xx/5xx never fall back to stdio.
//! - `[Structural]` Pinned client identity "ai-tadpole-os" over wire; zero Authorization token leakage.
//! - `[Structural]` Operation ID binding cache bounded to MAX_OPERATION_BINDINGS (1,024 entries).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InfrastructureError`, `AppError::Conflict`, `AppError::Forbidden`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::mcp::client::http::client::tests::*`

pub mod config;
pub mod operations;
pub mod redact;
pub mod request;
#[cfg(test)]
pub(crate) mod tests;
pub mod validation;

pub use config::ClientTimeouts;

use crate::agent::mcp::client::jsonrpc::{
    make_meta, make_tool_call_meta, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    JSONRPC_HEADER_MISMATCH,
};
use crate::agent::mcp::client::{
    DEFAULT_MCP_CALL_TIMEOUT, DEFAULT_MCP_DISCOVERY_TIMEOUT, MCP_PROTOCOL_2026_07_28,
};
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, warn};

use super::body::read_bounded_body;
use super::classify::{HttpDiscoveryFailureKind, HttpTransportFailureKind};
use super::limits::MAX_REQUEST_BODY_BYTES;

pub struct McpHttpClient {
    pub server_name: String,
    pub url: String,
    pub host_authority: String,
    pub protocol_version: Option<String>,
    pub capabilities: Option<Value>,
    pub tool_param_headers: HashMap<String, HashMap<String, String>>,
    pub last_raw_error: Option<JsonRpcError>,
    pub last_transport_failure: Option<HttpTransportFailureKind>,
    pub(crate) discovered: bool,
    pub(crate) listed_tools: HashSet<String>,
    pub(crate) operation_bindings: HashMap<String, String>,
    pub(crate) retryable_operations: HashSet<String>,
    pub(crate) operation_order: VecDeque<String>,
    pub(crate) catalog_refresh_required: bool,
    discovery_timeout: std::time::Duration,
    tool_timeout: std::time::Duration,
    client: reqwest::Client,
    headers: reqwest::header::HeaderMap,
    next_id: u64,
}

impl std::fmt::Debug for McpHttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sanitized_headers = std::collections::BTreeMap::new();
        for (k, v) in &self.headers {
            let k_str = k.as_str();
            if k_str.eq_ignore_ascii_case("authorization") {
                sanitized_headers.insert(k_str.to_string(), "Bearer [REDACTED]".to_string());
            } else if let Ok(val_str) = v.to_str() {
                sanitized_headers.insert(k_str.to_string(), val_str.to_string());
            }
        }

        f.debug_struct("McpHttpClient")
            .field("server_name", &self.server_name)
            .field("url", &self.url)
            .field("host_authority", &self.host_authority)
            .field("protocol_version", &self.protocol_version)
            .field("capabilities", &self.capabilities)
            .field("tool_param_headers", &self.tool_param_headers)
            .field(
                "last_raw_error_code",
                &self.last_raw_error.as_ref().map(|e| e.code),
            )
            .field("last_transport_failure", &self.last_transport_failure)
            .field("discovered", &self.discovered)
            .field("catalog_refresh_required", &self.catalog_refresh_required)
            .field("headers", &sanitized_headers)
            .field("next_id", &self.next_id)
            .field("discovery_timeout", &self.discovery_timeout)
            .field("tool_timeout", &self.tool_timeout)
            .finish()
    }
}

fn classify_version_error(rpc_err: &JsonRpcError) -> Option<HttpDiscoveryFailureKind> {
    if !rpc_err.is_unsupported_protocol_version() {
        return None;
    }
    let supported = rpc_err.supported_versions();
    if supported.contains(&MCP_PROTOCOL_2026_07_28.to_string()) {
        let requested = rpc_err.requested_version();
        let msg = format!(
            "server rejected exact '{MCP_PROTOCOL_2026_07_28}' request while advertising it (requested={requested:?})"
        );
        return Some(HttpDiscoveryFailureKind::ProtocolInconsistency(msg));
    }
    Some(HttpDiscoveryFailureKind::UnsupportedVersionNoIntersection(
        supported,
    ))
}

impl McpHttpClient {
    fn auth_header(&self) -> Option<&str> {
        self.headers
            .get(reqwest::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
    }

    pub(crate) fn sanitize_raw_error(&self, error: &JsonRpcError) -> JsonRpcError {
        redact::sanitize_raw_error(error, self.auth_header())
    }

    pub(crate) fn safe_rpc_error(&self, error: &JsonRpcError) -> String {
        redact::safe_rpc_error(error, self.auth_header())
    }

    pub fn new(
        server_name: &str,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        protocol_version: Option<&str>,
    ) -> Result<Self, AppError> {
        let timeouts = ClientTimeouts {
            discovery: DEFAULT_MCP_DISCOVERY_TIMEOUT,
            tool: DEFAULT_MCP_CALL_TIMEOUT,
        };
        Self::new_with_timeouts(server_name, url, headers, protocol_version, timeouts)
    }

    pub fn new_with_timeouts(
        server_name: &str,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        protocol_version: Option<&str>,
        timeouts: ClientTimeouts,
    ) -> Result<Self, AppError> {
        if timeouts.discovery.is_zero() || timeouts.tool.is_zero() {
            return Err(AppError::BadRequest(
                "MCP HTTP timeouts must be greater than zero".to_string(),
            ));
        }
        let parsed_url = reqwest::Url::parse(url)
            .map_err(|e| AppError::BadRequest(format!("Invalid MCP server URL '{url}': {e}")))?;

        let host_authority = config::validate_and_format_authority(&parsed_url);
        let header_map = config::build_initial_headers(headers, server_name)?;

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!("Failed to build HTTP client for MCP server '{server_name}': {e}"),
                help_link: None,
            })?;

        let proto = protocol_version.unwrap_or(MCP_PROTOCOL_2026_07_28);
        if proto != MCP_PROTOCOL_2026_07_28 {
            return Err(AppError::BadRequest(format!(
                "MCP HTTP server '{server_name}' only supports protocol '{MCP_PROTOCOL_2026_07_28}'"
            )));
        }

        Ok(Self {
            server_name: server_name.to_string(),
            url: url.to_string(),
            host_authority,
            protocol_version: Some(proto.to_string()),
            capabilities: None,
            tool_param_headers: HashMap::new(),
            last_raw_error: None,
            last_transport_failure: None,
            discovered: false,
            listed_tools: HashSet::new(),
            operation_bindings: HashMap::new(),
            retryable_operations: HashSet::new(),
            operation_order: VecDeque::new(),
            catalog_refresh_required: false,
            discovery_timeout: timeouts.discovery,
            tool_timeout: timeouts.tool,
            client,
            headers: header_map,
            next_id: 1,
        })
    }

    pub fn classify_discovery_error(&self, err: &AppError) -> HttpDiscoveryFailureKind {
        if let Some(ref rpc_err) = self.last_raw_error {
            if let Some(kind) = classify_version_error(rpc_err) {
                return kind;
            }
        }

        match self.last_transport_failure {
            Some(HttpTransportFailureKind::ConnectionRefused) => {
                HttpDiscoveryFailureKind::ConnectionRefused
            }
            Some(HttpTransportFailureKind::HostUnreachable) => {
                HttpDiscoveryFailureKind::HostUnreachable
            }
            _ => HttpDiscoveryFailureKind::FailClosed(err.to_string()),
        }
    }

    pub(crate) fn remember_operation(
        &mut self,
        operation_id: &str,
        name: &str,
        arguments: &Value,
    ) -> Result<(), AppError> {
        let mut state = operations::OperationState {
            bindings: &mut self.operation_bindings,
            retryable: &mut self.retryable_operations,
            order: &mut self.operation_order,
        };
        operations::remember_operation(&mut state, operation_id, name, arguments)
    }

    fn serialize_rpc_request(
        &self,
        id: u64,
        method: &str,
        params: Value,
    ) -> Result<Vec<u8>, AppError> {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(id),
            method: method.to_string(),
            params,
        };
        let req_body = serde_json::to_vec(&req).map_err(|e| {
            AppError::InternalServerError(format!("Failed to serialize JSON-RPC request: {e}"))
        })?;

        if req_body.len() > MAX_REQUEST_BODY_BYTES {
            return Err(AppError::BadRequest(format!(
                "MCP request body exceeds limit of {MAX_REQUEST_BODY_BYTES} bytes"
            )));
        }
        Ok(req_body)
    }

    async fn handle_error_response(
        &mut self,
        response: reqwest::Response,
        expected_id: &Value,
        content_type: &str,
    ) -> Result<Value, AppError> {
        let status = response.status();
        let body = read_bounded_body(response, &self.server_name).await?;

        if content_type == "application/json" {
            if let Some(error) = redact::extract_rpc_error_from_body(&body, expected_id) {
                self.last_raw_error = Some(self.sanitize_raw_error(&error));
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!("MCP HTTP {status}: {}", self.safe_rpc_error(&error)),
                    help_link: None,
                });
            }
        }

        Err(AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!(
                "MCP server '{}' returned HTTP {status} with content type '{content_type}'",
                self.server_name
            ),
            help_link: None,
        })
    }

    fn validate_and_extract_result(
        &mut self,
        json_resp: JsonRpcResponse,
        expected_id: &Value,
    ) -> Result<Value, AppError> {
        json_resp
            .validate_for(expected_id)
            .map_err(|detail| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "Invalid JSON-RPC response from '{}': {detail}",
                    self.server_name
                ),
                help_link: None,
            })?;

        if let Some(error) = json_resp.error {
            self.last_raw_error = Some(self.sanitize_raw_error(&error));
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP Error from '{}': {}",
                    self.server_name,
                    self.safe_rpc_error(&error)
                ),
                help_link: None,
            });
        }

        json_resp
            .result
            .ok_or_else(|| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP response from '{}' contained neither result nor error",
                    self.server_name
                ),
                help_link: None,
            })
    }

    async fn process_raw_response(
        &mut self,
        response: reqwest::Response,
        expected_id: Value,
    ) -> Result<Value, AppError> {
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();

        if !response.status().is_success() {
            return self
                .handle_error_response(response, &expected_id, &content_type)
                .await;
        }

        let json_resp =
            request::parse_rpc_response(response, &self.server_name, &expected_id, &content_type)
                .await?;
        self.validate_and_extract_result(json_resp, &expected_id)
    }

    pub async fn call_internal(
        &mut self,
        method: &str,
        tool_name: Option<&str>,
        params: Value,
        extra_headers: Option<HashMap<String, String>>,
    ) -> Result<Value, AppError> {
        let id = self.next_id;
        self.next_id += 1;
        let req_body = self.serialize_rpc_request(id, method, params)?;

        let timeout = if method == "server/discover" {
            self.discovery_timeout
        } else {
            self.tool_timeout
        };
        let proto = self
            .protocol_version
            .as_deref()
            .unwrap_or(MCP_PROTOCOL_2026_07_28);
        let transport_params = request::TransportParams {
            server_name: &self.server_name,
            url: &self.url,
            host_authority: &self.host_authority,
            proto,
            headers: &self.headers,
            timeout,
            client: &self.client,
        };

        let response = match request::execute_http_post(
            &transport_params,
            method,
            tool_name,
            req_body,
            extra_headers,
        )
        .await
        {
            Ok(resp) => resp,
            Err((err, failure)) => {
                self.last_transport_failure = failure;
                return Err(err);
            }
        };

        self.process_raw_response(response, json!(id)).await
    }

    pub async fn initialize(&mut self) -> Result<(), AppError> {
        let proto = self
            .protocol_version
            .as_deref()
            .unwrap_or(MCP_PROTOCOL_2026_07_28);
        let params = json!({
            "protocolVersion": proto,
            "capabilities": {},
            "clientInfo": {
                "name": "ai-tadpole-os",
                "version": env!("CARGO_PKG_VERSION")
            },
            "_meta": make_meta(proto)
        });

        let discover_res = self
            .call_internal("server/discover", None, params, None)
            .await;
        match discover_res {
            Ok(result) => {
                let capabilities =
                    validation::validate_discovery_response(&result, &self.server_name)?;
                self.protocol_version = Some(MCP_PROTOCOL_2026_07_28.to_string());
                self.capabilities = Some(capabilities);
                self.discovered = true;
                debug!(
                    "[MCP HTTP] Discovered capabilities for '{}': {:?}",
                    self.server_name, self.capabilities
                );
                Ok(())
            }
            Err(e) => {
                let failure = self.classify_discovery_error(&e);
                warn!(
                    "[MCP HTTP] server/discover failed for '{}' ({failure:?}): {e}",
                    self.server_name
                );
                Err(e)
            }
        }
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Value>, AppError> {
        if !self.discovered {
            return Err(AppError::Forbidden(
                "MCP HTTP server/discover must complete before tools/list".to_string(),
            ));
        }
        let proto = self
            .protocol_version
            .as_deref()
            .unwrap_or(MCP_PROTOCOL_2026_07_28);
        let params = json!({ "_meta": make_meta(proto) });

        let result = self.call_internal("tools/list", None, params, None).await?;
        if result.get("resultType").and_then(Value::as_str) != Some("complete") {
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP server '{}' returned invalid tools/list resultType",
                    self.server_name
                ),
                help_link: None,
            });
        }
        let raw_tools = result
            .get("tools")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!("MCP server '{}' omitted tools array", self.server_name),
                help_link: None,
            })?;

        let valid_tools = validation::filter_and_quarantine_tools(
            raw_tools,
            &self.server_name,
            &mut self.tool_param_headers,
            &mut self.listed_tools,
        )?;

        self.catalog_refresh_required = false;
        Ok(valid_tools)
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, AppError> {
        let operation_id = uuid::Uuid::new_v4().to_string();
        self.call_tool_with_operation_id(name, arguments, &operation_id)
            .await
    }

    pub async fn call_tool_with_operation_id(
        &mut self,
        name: &str,
        arguments: Value,
        operation_id: &str,
    ) -> Result<Value, AppError> {
        if !self.discovered {
            return Err(AppError::Forbidden(
                "MCP HTTP server/discover must complete before tools/call".to_string(),
            ));
        }
        if self.catalog_refresh_required {
            return Err(AppError::Conflict(
                "MCP tools/list must refresh the catalog before retrying a header mismatch"
                    .to_string(),
            ));
        }
        if !self.listed_tools.contains(name) {
            return Err(AppError::Forbidden(format!(
                "MCP tool '{name}' is absent from the last successful tools/list catalog"
            )));
        }

        self.remember_operation(operation_id, name, &arguments)?;

        let proto = self
            .protocol_version
            .as_deref()
            .unwrap_or(MCP_PROTOCOL_2026_07_28);
        let meta = make_tool_call_meta(proto, Some(operation_id));
        let params = json!({
            "name": name,
            "arguments": arguments,
            "_meta": meta
        });

        let extra_headers =
            validation::build_projected_tool_headers(&self.tool_param_headers, name, &arguments)?;
        let extra_opt = if extra_headers.is_empty() {
            None
        } else {
            Some(extra_headers)
        };

        let call_result = self
            .call_internal("tools/call", Some(name), params, extra_opt)
            .await;

        if let Some(error) = self.last_raw_error.as_ref() {
            if error.code == JSONRPC_HEADER_MISMATCH {
                self.catalog_refresh_required = true;
            }
            if error
                .data
                .as_ref()
                .and_then(|d| d.get("retryable"))
                .and_then(Value::as_bool)
                == Some(true)
            {
                self.retryable_operations.insert(operation_id.to_string());
            }
        }

        let result = call_result?;
        validation::validate_tool_call_result(&result, &self.server_name)?;
        if result
            .pointer("/_meta/execution/retryable")
            .and_then(Value::as_bool)
            == Some(true)
        {
            self.retryable_operations.insert(operation_id.to_string());
        }
        Ok(result)
    }

    pub async fn shutdown(&mut self) -> Result<(), AppError> {
        debug!(
            "[MCP HTTP] Closing session for stateless server '{}'",
            self.server_name
        );
        Ok(())
    }
}
