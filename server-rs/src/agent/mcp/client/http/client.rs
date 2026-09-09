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
use super::classify::{classify_reqwest_error, HttpDiscoveryFailureKind, HttpTransportFailureKind};
use super::headers::{
    encode_header_value_if_needed, extract_and_validate_tool_headers, hash_operation_binding,
    primitive_header_value, value_at_property_path,
};
use super::limits::{MAX_OPERATION_BINDINGS, MAX_REQUEST_BODY_BYTES};
use super::sse::parse_sse_stream;

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
                &self.last_raw_error.as_ref().map(|error| error.code),
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

impl McpHttpClient {
    fn sanitize_raw_error(&self, error: &JsonRpcError) -> JsonRpcError {
        let mut sanitized = error.clone();
        if let Some(secret) = self
            .headers
            .get(reqwest::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        {
            sanitized.message = sanitized.message.replace(secret, "[REDACTED]");
            if let Some(token) = secret.strip_prefix("Bearer ") {
                sanitized.message = sanitized.message.replace(token, "[REDACTED]");
            }
            if let Some(ref mut data) = sanitized.data {
                Self::redact_json_value(data, secret);
                if let Some(token) = secret.strip_prefix("Bearer ") {
                    Self::redact_json_value(data, token);
                }
            }
        }
        sanitized
    }

    fn redact_json_value(value: &mut Value, secret: &str) {
        match value {
            Value::String(s) => {
                if s.contains(secret) {
                    *s = s.replace(secret, "[REDACTED]");
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    Self::redact_json_value(item, secret);
                }
            }
            Value::Object(obj) => {
                for (_, val) in obj.iter_mut() {
                    Self::redact_json_value(val, secret);
                }
            }
            _ => {}
        }
    }

    fn safe_rpc_error(&self, error: &JsonRpcError) -> String {
        let sanitized = self.sanitize_raw_error(error);
        format!("Code {}: {}", sanitized.code, sanitized.message)
    }

    pub fn new(
        server_name: &str,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        protocol_version: Option<&str>,
    ) -> Result<Self, AppError> {
        Self::new_with_timeouts(
            server_name,
            url,
            headers,
            protocol_version,
            DEFAULT_MCP_DISCOVERY_TIMEOUT,
            DEFAULT_MCP_CALL_TIMEOUT,
        )
    }

    pub fn new_with_timeouts(
        server_name: &str,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        protocol_version: Option<&str>,
        discovery_timeout: std::time::Duration,
        tool_timeout: std::time::Duration,
    ) -> Result<Self, AppError> {
        if discovery_timeout.is_zero() || tool_timeout.is_zero() {
            return Err(AppError::BadRequest(
                "MCP HTTP timeouts must be greater than zero".to_string(),
            ));
        }
        let parsed_url = reqwest::Url::parse(url).map_err(|e| {
            AppError::BadRequest(format!("Invalid MCP server URL '{}': {}", url, e))
        })?;

        // Bracket IPv6 authorities per RFC 9110 / RFC 3986
        let host_authority = {
            let raw_host = parsed_url.host_str().unwrap_or("");
            let formatted_host = if raw_host.contains(':') && !raw_host.starts_with('[') {
                format!("[{}]", raw_host)
            } else {
                raw_host.to_string()
            };
            if let Some(port) = parsed_url.port() {
                format!("{}:{}", formatted_host, port)
            } else {
                formatted_host
            }
        };

        let mut header_map = reqwest::header::HeaderMap::new();

        // Spec REQUIRED Accept header: "application/json, text/event-stream"
        header_map.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json, text/event-stream"),
        );
        header_map.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        if let Some(h) = headers {
            for (key, val) in h {
                let k_lower = key.to_ascii_lowercase();
                // Architectural Invariant: Origin, Mcp-Session-Id, and Last-Event-ID are strictly
                // forbidden by the Port 3000 Sovereign IPC specification to prevent CORS/origin
                // spoofing vectors in native binary process communication.
                if matches!(
                    k_lower.as_str(),
                    "host"
                        | "content-type"
                        | "accept"
                        | "mcp-protocol-version"
                        | "mcp-method"
                        | "mcp-name"
                        | "origin"
                        | "mcp-session-id"
                        | "last-event-id"
                ) || k_lower.starts_with("mcp-param-")
                {
                    return Err(AppError::BadRequest(format!(
                        "MCP server '{}' may not configure transport-owned header '{}'",
                        server_name, key
                    )));
                }

                let name =
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                        AppError::BadRequest(format!(
                            "Invalid header name '{}' for MCP server '{}': {}",
                            key, server_name, e
                        ))
                    })?;
                let val_parsed = reqwest::header::HeaderValue::from_str(val).map_err(|e| {
                    AppError::BadRequest(format!(
                        "Invalid header value for '{}' on MCP server '{}': {}",
                        key, server_name, e
                    ))
                })?;
                header_map.insert(name, val_parsed);
            }
        }

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!(
                    "Failed to build HTTP client for MCP server '{}': {}",
                    server_name, e
                ),
                help_link: None,
            })?;

        let proto = protocol_version.unwrap_or(MCP_PROTOCOL_2026_07_28);
        if proto != MCP_PROTOCOL_2026_07_28 {
            return Err(AppError::BadRequest(format!(
                "MCP HTTP server '{}' only supports protocol '{}'",
                server_name, MCP_PROTOCOL_2026_07_28
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
            discovery_timeout,
            tool_timeout,
            client,
            headers: header_map,
            next_id: 1,
        })
    }

    pub fn classify_discovery_error(&self, err: &AppError) -> HttpDiscoveryFailureKind {
        if let Some(ref rpc_err) = self.last_raw_error {
            if rpc_err.is_unsupported_protocol_version() {
                let supported = rpc_err.supported_versions();
                let requested = rpc_err.requested_version();
                if supported.contains(&MCP_PROTOCOL_2026_07_28.to_string()) {
                    return HttpDiscoveryFailureKind::ProtocolInconsistency(format!(
                        "server rejected exact '{}' request while advertising it (requested={:?})",
                        MCP_PROTOCOL_2026_07_28, requested
                    ));
                } else {
                    return HttpDiscoveryFailureKind::UnsupportedVersionNoIntersection(supported);
                }
            }
        }

        // Invariant: Timeout, TLS errors, and unspecified failures must fail closed to prevent
        // spawning a competing stdio subprocess while an HTTP service is running or lagged.
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
        uuid::Uuid::parse_str(operation_id)
            .map_err(|_| AppError::BadRequest("MCP operation_id must be a UUID".to_string()))?;

        let binding = hash_operation_binding(name, arguments);

        if let Some(existing) = self.operation_bindings.get(operation_id) {
            if existing != &binding {
                return Err(AppError::Conflict(
                    "MCP operation_id cannot be reused with different tool arguments".to_string(),
                ));
            }
            if !self.retryable_operations.remove(operation_id) {
                return Err(AppError::Conflict(
                    "MCP operation_id may be retried only after an explicit retryable outcome"
                        .to_string(),
                ));
            }
        } else {
            // Eviction strategy: prioritize evicting settled non-retryable operations
            if self.operation_bindings.len() >= MAX_OPERATION_BINDINGS {
                let non_retryable_idx = self
                    .operation_order
                    .iter()
                    .position(|id| !self.retryable_operations.contains(id));

                let evicted_id = if let Some(idx) = non_retryable_idx {
                    self.operation_order.remove(idx)
                } else {
                    // Fall back to oldest entry if all entries are retryable
                    self.operation_order.pop_front()
                };

                if let Some(ref id) = evicted_id {
                    self.operation_bindings.remove(id);
                    self.retryable_operations.remove(id);
                }
            }

            self.operation_bindings
                .insert(operation_id.to_string(), binding);
            self.operation_order.push_back(operation_id.to_string());
        }

        Ok(())
    }

    pub async fn call_internal(
        &mut self,
        method: &str,
        tool_name: Option<&str>,
        params: Value,
        extra_headers: Option<HashMap<String, String>>,
    ) -> Result<Value, AppError> {
        let proto = self
            .protocol_version
            .as_deref()
            .unwrap_or(MCP_PROTOCOL_2026_07_28);

        let id = self.next_id;
        self.next_id += 1;
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(id),
            method: method.to_string(),
            params,
        };
        let req_body = serde_json::to_vec(&req).map_err(|e| {
            AppError::InternalServerError(format!("Failed to serialize JSON-RPC request: {}", e))
        })?;

        if req_body.len() > MAX_REQUEST_BODY_BYTES {
            return Err(AppError::BadRequest(format!(
                "MCP request body exceeds limit of {} bytes",
                MAX_REQUEST_BODY_BYTES
            )));
        }

        let mut req_builder = self
            .client
            .post(&self.url)
            .timeout(if method == "server/discover" {
                self.discovery_timeout
            } else {
                self.tool_timeout
            })
            .headers(self.headers.clone())
            .header("Host", &self.host_authority)
            .header("MCP-Protocol-Version", proto)
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
            "[MCP HTTP] Sending {} for '{}' (protocol={}) to {}",
            method, self.server_name, proto, self.url
        );

        self.last_transport_failure = None;
        let response = match req_builder.body(req_body).send().await {
            Ok(resp) => resp,
            Err(e) => {
                let failure = classify_reqwest_error(&e);
                self.last_transport_failure = Some(failure);
                let kind = if failure == HttpTransportFailureKind::Timeout {
                    InfrastructureErrorKind::Timeout
                } else if matches!(
                    failure,
                    HttpTransportFailureKind::ConnectionRefused
                        | HttpTransportFailureKind::HostUnreachable
                ) {
                    InfrastructureErrorKind::NetworkError
                } else {
                    InfrastructureErrorKind::Other
                };
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind,
                    detail: format!(
                        "Failed to send request to MCP HTTP server '{}': {}",
                        self.server_name, e
                    ),
                    help_link: None,
                });
            }
        };

        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let expected_id = json!(id);

        if !status.is_success() {
            let body = read_bounded_body(response, &self.server_name).await?;
            if content_type == "application/json" {
                if let Ok(json_resp) = serde_json::from_slice::<JsonRpcResponse>(&body) {
                    if json_resp.validate_for(&expected_id).is_ok() {
                        if let Some(error) = json_resp.error {
                            self.last_raw_error = Some(self.sanitize_raw_error(&error));
                            return Err(AppError::InfrastructureError {
                                provider_id: ProviderId::Mcp,
                                kind: InfrastructureErrorKind::ApiError,
                                detail: format!(
                                    "MCP HTTP {}: {}",
                                    status,
                                    self.safe_rpc_error(&error)
                                ),
                                help_link: None,
                            });
                        }
                    }
                }
            }
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP server '{}' returned HTTP {} with content type '{}'",
                    self.server_name, status, content_type
                ),
                help_link: None,
            });
        }

        let json_resp = match content_type.as_str() {
            "text/event-stream" => {
                parse_sse_stream(response, &self.server_name, &expected_id).await?
            }
            "application/json" => {
                let body = read_bounded_body(response, &self.server_name).await?;
                serde_json::from_slice::<JsonRpcResponse>(&body).map_err(|_| {
                    AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: format!(
                            "MCP server '{}' returned malformed JSON-RPC",
                            self.server_name
                        ),
                        help_link: None,
                    }
                })?
            }
            _ => {
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!(
                        "MCP server '{}' returned unsupported content type '{}'",
                        self.server_name, content_type
                    ),
                    help_link: None,
                });
            }
        };

        json_resp
            .validate_for(&expected_id)
            .map_err(|detail| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "Invalid JSON-RPC response from '{}': {}",
                    self.server_name, detail
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
                if result.get("resultType").and_then(Value::as_str) != Some("complete") {
                    return Err(AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: format!(
                            "MCP server '{}' returned invalid server/discover resultType",
                            self.server_name
                        ),
                        help_link: None,
                    });
                }
                let versions = result
                    .get("supportedVersions")
                    .and_then(Value::as_array)
                    .ok_or_else(|| AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: format!(
                            "MCP server '{}' omitted supportedVersions",
                            self.server_name
                        ),
                        help_link: None,
                    })?;
                if versions.iter().any(|version| !version.is_string())
                    || !versions
                        .iter()
                        .any(|version| version.as_str() == Some(MCP_PROTOCOL_2026_07_28))
                {
                    return Err(AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: format!(
                            "MCP server '{}' did not offer required protocol version '{}'",
                            self.server_name, MCP_PROTOCOL_2026_07_28
                        ),
                        help_link: None,
                    });
                }
                let capabilities = result
                    .get("capabilities")
                    .filter(|value| value.is_object())
                    .ok_or_else(|| AppError::InfrastructureError {
                        provider_id: ProviderId::Mcp,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: format!(
                            "MCP server '{}' omitted valid capabilities",
                            self.server_name
                        ),
                        help_link: None,
                    })?;
                self.protocol_version = Some(MCP_PROTOCOL_2026_07_28.to_string());
                self.capabilities = Some(capabilities.clone());
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
                    "[MCP HTTP] server/discover failed for '{}' ({:?}): {}",
                    self.server_name, failure, e
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

        let params = json!({
            "_meta": make_meta(proto)
        });

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

        let mut valid_tools = Vec::new();
        let mut excluded_count = 0usize;
        self.tool_param_headers.clear();
        self.listed_tools.clear();

        for tool in raw_tools {
            // Protocol violation: A tool without a name is structurally unidentifiable.
            let Some(name) = tool.get("name").and_then(|n| n.as_str()) else {
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!(
                        "MCP server '{}' returned a tool without a name",
                        self.server_name
                    ),
                    help_link: None,
                });
            };

            // Quarantine policy: Malformed inputSchema or invalid x-mcp-header excludes the tool
            // while preserving valid tools in the catalog (SEP-2243 conformance).
            let Some(schema) = tool.get("inputSchema").filter(|schema| schema.is_object()) else {
                warn!(
                    "[MCP HTTP] Excluded tool '{}' because inputSchema is not an object",
                    name
                );
                excluded_count += 1;
                continue;
            };
            match extract_and_validate_tool_headers(name, schema) {
                Ok(headers_map) => {
                    if !headers_map.is_empty() {
                        self.tool_param_headers
                            .insert(name.to_string(), headers_map);
                    }
                    self.listed_tools.insert(name.to_string());
                    valid_tools.push(tool);
                }
                Err(err_msg) => {
                    warn!(
                        "[MCP HTTP] Excluded tool '{}' due to invalid x-mcp-header: {}",
                        name, err_msg
                    );
                    excluded_count += 1;
                }
            }
        }

        if excluded_count > 0 {
            warn!(
                "[MCP HTTP] Catalog update for '{}': {} tools active, {} tools quarantined",
                self.server_name,
                valid_tools.len(),
                excluded_count
            );
        }

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
                "MCP tool '{}' is absent from the last successful tools/list catalog",
                name
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

        let mut extra_headers = HashMap::new();
        if let Some(param_map) = self.tool_param_headers.get(name) {
            for (argument_path, header_name) in param_map {
                if let Some(value) = value_at_property_path(&arguments, argument_path) {
                    if !value.is_null() {
                        let value_string = primitive_header_value(value).ok_or_else(|| {
                            AppError::BadRequest(format!(
                                "MCP header-bound argument '{}' must be a string, boolean, or safe integer",
                                argument_path
                            ))
                        })?;
                        let encoded = encode_header_value_if_needed(&value_string);
                        let header_key = format!("Mcp-Param-{}", header_name);
                        reqwest::header::HeaderName::from_bytes(header_key.as_bytes()).map_err(
                            |e| {
                                AppError::BadRequest(format!(
                                    "Invalid projected header name '{}': {}",
                                    header_key, e
                                ))
                            },
                        )?;
                        reqwest::header::HeaderValue::from_str(&encoded).map_err(|e| {
                            AppError::BadRequest(format!(
                                "Invalid projected header value for '{}': {}",
                                header_key, e
                            ))
                        })?;
                        extra_headers.insert(header_key, encoded);
                    }
                }
            }
        }

        let call_result = self
            .call_internal(
                "tools/call",
                Some(name),
                params,
                if extra_headers.is_empty() {
                    None
                } else {
                    Some(extra_headers)
                },
            )
            .await;

        if let Some(error) = self.last_raw_error.as_ref() {
            if error.code == JSONRPC_HEADER_MISMATCH {
                self.catalog_refresh_required = true;
            }
            if error
                .data
                .as_ref()
                .and_then(|data| data.get("retryable"))
                .and_then(Value::as_bool)
                == Some(true)
            {
                self.retryable_operations.insert(operation_id.to_string());
            }
        }

        let result = call_result?;
        if result.get("resultType").and_then(Value::as_str) != Some("complete") {
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP server '{}' returned invalid tools/call resultType",
                    self.server_name
                ),
                help_link: None,
            });
        }
        if !result.get("content").is_some_and(Value::is_array) {
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP server '{}' returned tools/call without content array",
                    self.server_name
                ),
                help_link: None,
            });
        }
        if result
            .get("structuredContent")
            .is_some_and(|value| !value.is_object())
        {
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP server '{}' returned non-object structuredContent",
                    self.server_name
                ),
                help_link: None,
            });
        }
        if result
            .get("isError")
            .is_some_and(|value| !value.is_boolean())
        {
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP server '{}' returned non-boolean isError",
                    self.server_name
                ),
                help_link: None,
            });
        }
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_http_client_creation_and_header_population() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer test-key".to_string());
        headers.insert("X-Custom-Header".to_string(), "CustomValue".to_string());

        let client = McpHttpClient::new(
            "remote-docs",
            "http://127.0.0.1:3000/mcp",
            Some(&headers),
            Some(MCP_PROTOCOL_2026_07_28),
        )
        .unwrap();

        assert_eq!(client.server_name, "remote-docs");
        assert_eq!(client.url, "http://127.0.0.1:3000/mcp");
        assert_eq!(client.host_authority, "127.0.0.1:3000");

        assert_eq!(
            client
                .headers
                .get("Authorization")
                .unwrap()
                .to_str()
                .unwrap(),
            "Bearer test-key"
        );
        assert_eq!(
            client
                .headers
                .get("X-Custom-Header")
                .unwrap()
                .to_str()
                .unwrap(),
            "CustomValue"
        );
        assert_eq!(
            client
                .headers
                .get(reqwest::header::ACCEPT)
                .unwrap()
                .to_str()
                .unwrap(),
            "application/json, text/event-stream"
        );
        assert_eq!(
            client
                .headers
                .get(reqwest::header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_mcp_http_client_rejects_transport_owned_headers() {
        for name in [
            "Host",
            "Content-Type",
            "Accept",
            "MCP-Protocol-Version",
            "Mcp-Method",
            "Mcp-Name",
            "Mcp-Param-Test",
            "Origin",
            "Mcp-Session-Id",
            "Last-Event-ID",
        ] {
            let headers = HashMap::from([(name.to_string(), "unsafe".to_string())]);
            assert!(McpHttpClient::new(
                "remote-docs",
                "http://127.0.0.1:3000/mcp",
                Some(&headers),
                Some(MCP_PROTOCOL_2026_07_28),
            )
            .is_err());
        }
    }

    #[test]
    fn test_mcp_http_client_rejects_invalid_header_name() {
        let mut headers = HashMap::new();
        headers.insert("Invalid Header Name!".to_string(), "value".to_string());

        let err = McpHttpClient::new(
            "bad-header",
            "https://mcp.example.com/api",
            Some(&headers),
            None,
        )
        .unwrap_err();

        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn test_ipv6_host_authority_is_bracketed() {
        let client_port =
            McpHttpClient::new("ipv6-test", "http://[::1]:3000/mcp", None, None).unwrap();
        assert_eq!(client_port.host_authority, "[::1]:3000");

        let client_no_port =
            McpHttpClient::new("ipv6-test", "http://[::1]/mcp", None, None).unwrap();
        assert_eq!(client_no_port.host_authority, "[::1]");

        let client_v4 =
            McpHttpClient::new("ipv4-test", "http://127.0.0.1:3000/mcp", None, None).unwrap();
        assert_eq!(client_v4.host_authority, "127.0.0.1:3000");
    }

    #[test]
    fn test_operation_binding_eviction_is_bounded() {
        let mut client =
            McpHttpClient::new("bound-test", "http://127.0.0.1:3000/mcp", None, None).unwrap();

        // Exercise production path directly through remember_operation
        for i in 0..(MAX_OPERATION_BINDINGS + 10) {
            let op_id = format!("00000000-0000-4000-8000-{:012x}", i);
            assert!(client
                .remember_operation(&op_id, "tool", &json!({"i": i}))
                .is_ok());
        }
        assert_eq!(client.operation_bindings.len(), MAX_OPERATION_BINDINGS);
    }

    #[test]
    fn test_sanitize_raw_error_redacts_tokens() {
        let headers = HashMap::from([(
            "Authorization".to_string(),
            "Bearer super-secret-token".to_string(),
        )]);
        let client = McpHttpClient::new(
            "sec-test",
            "http://127.0.0.1:3000/mcp",
            Some(&headers),
            None,
        )
        .unwrap();
        let raw = crate::agent::mcp::client::jsonrpc::JsonRpcError {
            code: -32001,
            message: "invalid request for Bearer super-secret-token".to_string(),
            data: Some(json!({
                "echo": "super-secret-token",
                "nested": ["Bearer super-secret-token"]
            })),
        };
        let sanitized = client.sanitize_raw_error(&raw);
        assert!(!sanitized.message.contains("super-secret-token"));
        assert!(sanitized.message.contains("[REDACTED]"));
        let data_str = serde_json::to_string(&sanitized.data).unwrap();
        assert!(!data_str.contains("super-secret-token"));
        assert!(data_str.contains("[REDACTED]"));
    }
}
