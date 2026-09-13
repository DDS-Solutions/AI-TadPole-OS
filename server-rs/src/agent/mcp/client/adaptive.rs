//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Adaptive Dual Transport Client
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Explicit state machine: HTTP primary, stdio fallback only on approved pre-tool discovery triggers.
//! - `[Structural]` Fail-closed: 401, 403, 404, 429, 5xx, timeouts, and committed tool execution never fall back.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InfrastructureError`, `AppError::BadRequest`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `client::adaptive::tests::*`

use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use serde_json::Value;
use tracing::{info, warn};

use super::http::{HttpDiscoveryFailureKind, McpHttpClient};
use super::stdio::McpStdioClient;
use crate::agent::mcp::config::{McpHttpConfig, McpStdioConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveState {
    Configured,
    HttpDiscovering,
    HttpReady,
    HttpCommitted,
    StdioDiscovering,
    StdioReady,
    FailedClosed,
}

pub struct AdaptiveMcpClient {
    pub server_name: String,
    pub state: AdaptiveState,
    pub http_client: McpHttpClient,
    pub stdio_config: Option<McpStdioConfig>,
    pub stdio_client: Option<McpStdioClient>,
}

impl std::fmt::Debug for AdaptiveMcpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdaptiveMcpClient")
            .field("server_name", &self.server_name)
            .field("state", &self.state)
            .field("protocol_version", &self.protocol_version())
            .finish()
    }
}

impl AdaptiveMcpClient {
    pub fn new(
        server_name: &str,
        http_config: McpHttpConfig,
        stdio_fallback: Option<McpStdioConfig>,
    ) -> Result<Self, AppError> {
        let proto = http_config
            .protocol_versions
            .first()
            .map(|s| s.as_str())
            .unwrap_or("2026-07-28");

        let http_client = McpHttpClient::new_with_timeouts(
            server_name,
            &http_config.url,
            http_config.headers.as_ref(),
            Some(proto),
            std::time::Duration::from_millis(http_config.discovery_timeout_ms.unwrap_or(15_000)),
            std::time::Duration::from_millis(http_config.tool_timeout_ms.unwrap_or(30_000)),
        )?;

        Ok(Self {
            server_name: server_name.to_string(),
            state: AdaptiveState::Configured,
            http_client,
            stdio_config: stdio_fallback,
            stdio_client: None,
        })
    }

    pub async fn initialize(&mut self) -> Result<(), AppError> {
        match self.state {
            AdaptiveState::Configured => {}
            AdaptiveState::HttpReady | AdaptiveState::HttpCommitted | AdaptiveState::StdioReady => {
                return Ok(())
            }
            _ => {
                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!(
                        "MCP client '{}' cannot initialize in state {:?}",
                        self.server_name, self.state
                    ),
                    help_link: None,
                });
            }
        }

        self.state = AdaptiveState::HttpDiscovering;
        match self.http_client.initialize().await {
            Ok(()) => {
                self.state = AdaptiveState::HttpReady;
                info!(
                    "✅ [MCP Adaptive] Server '{}' successfully negotiated HTTP 2026-07-28 transport",
                    self.server_name
                );
                Ok(())
            }
            Err(err) => {
                let classification = self.http_client.classify_discovery_error(&err);
                match classification {
                    HttpDiscoveryFailureKind::ConnectionRefused
                    | HttpDiscoveryFailureKind::HostUnreachable
                    | HttpDiscoveryFailureKind::UnsupportedVersionNoIntersection(_) => {
                        // Approved pre-tool discovery fallback trigger!
                        if let Some(ref stdio_cfg) = self.stdio_config {
                            info!(
                                "⚠️ [MCP Adaptive] Approved HTTP discovery fallback triggered ({:?}) for server '{}'; starting configured stdio transport",
                                classification, self.server_name
                            );
                            self.state = AdaptiveState::StdioDiscovering;

                            let mut stdio = match McpStdioClient::spawn_with_cwd(
                                &self.server_name,
                                &stdio_cfg.command,
                                &stdio_cfg.args,
                                stdio_cfg.env.as_ref(),
                                stdio_cfg.cwd.as_deref(),
                            )
                            .await
                            {
                                Ok(client) => client,
                                Err(error) => {
                                    self.state = AdaptiveState::FailedClosed;
                                    return Err(error);
                                }
                            };

                            if let Err(error) = stdio.initialize().await {
                                let _ = stdio.shutdown().await;
                                self.state = AdaptiveState::FailedClosed;
                                return Err(error);
                            }
                            self.stdio_client = Some(stdio);
                            self.state = AdaptiveState::StdioReady;
                            Ok(())
                        } else {
                            self.state = AdaptiveState::FailedClosed;
                            Err(err)
                        }
                    }
                    HttpDiscoveryFailureKind::ProtocolInconsistency(msg) => {
                        warn!(
                            "❌ [MCP Adaptive] Protocol inconsistency detected for '{}': {}",
                            self.server_name, msg
                        );
                        self.state = AdaptiveState::FailedClosed;
                        Err(AppError::InfrastructureError {
                            provider_id: ProviderId::Mcp,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: msg,
                            help_link: None,
                        })
                    }
                    HttpDiscoveryFailureKind::FailClosed(detail) => {
                        warn!(
                            "🛑 [MCP Adaptive] Discovery failed closed (no stdio fallback permitted) for '{}': {}",
                            self.server_name, detail
                        );
                        self.state = AdaptiveState::FailedClosed;
                        Err(err)
                    }
                }
            }
        }
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Value>, AppError> {
        match self.state {
            AdaptiveState::HttpReady => {
                self.state = AdaptiveState::HttpCommitted;
                self.http_client.list_tools().await
            }
            AdaptiveState::HttpCommitted => self.http_client.list_tools().await,
            AdaptiveState::StdioReady => {
                let stdio =
                    self.stdio_client
                        .as_mut()
                        .ok_or_else(|| AppError::InfrastructureError {
                            provider_id: ProviderId::Mcp,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: "Stdio client missing in StdioReady state".to_string(),
                            help_link: None,
                        })?;
                stdio.list_tools().await
            }
            _ => Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP client for '{}' cannot list tools in state {:?}",
                    self.server_name, self.state
                ),
                help_link: None,
            }),
        }
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, AppError> {
        match self.state {
            AdaptiveState::HttpReady => {
                self.state = AdaptiveState::HttpCommitted;
                self.http_client.call_tool(name, arguments).await
            }
            AdaptiveState::HttpCommitted => self.http_client.call_tool(name, arguments).await,
            AdaptiveState::StdioReady => {
                let stdio =
                    self.stdio_client
                        .as_mut()
                        .ok_or_else(|| AppError::InfrastructureError {
                            provider_id: ProviderId::Mcp,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: "Stdio client missing in StdioReady state".to_string(),
                            help_link: None,
                        })?;
                stdio.call_tool(name, arguments).await
            }
            _ => Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP client for '{}' cannot call tool in state {:?}",
                    self.server_name, self.state
                ),
                help_link: None,
            }),
        }
    }

    pub async fn call_tool_with_operation_id(
        &mut self,
        name: &str,
        arguments: Value,
        operation_id: &str,
    ) -> Result<Value, AppError> {
        match self.state {
            AdaptiveState::HttpReady => {
                self.state = AdaptiveState::HttpCommitted;
                self.http_client
                    .call_tool_with_operation_id(name, arguments, operation_id)
                    .await
            }
            AdaptiveState::HttpCommitted => {
                self.http_client
                    .call_tool_with_operation_id(name, arguments, operation_id)
                    .await
            }
            AdaptiveState::StdioReady => {
                let stdio =
                    self.stdio_client
                        .as_mut()
                        .ok_or_else(|| AppError::InfrastructureError {
                            provider_id: ProviderId::Mcp,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: "Stdio client missing in StdioReady state".to_string(),
                            help_link: None,
                        })?;
                stdio
                    .call_tool_with_operation_id(name, arguments, operation_id)
                    .await
            }
            _ => Err(AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "MCP client for '{}' cannot call tool in state {:?}",
                    self.server_name, self.state
                ),
                help_link: None,
            }),
        }
    }

    pub async fn shutdown(&mut self) -> Result<(), AppError> {
        let _ = self.http_client.shutdown().await;
        if let Some(ref mut stdio) = self.stdio_client {
            let _ = stdio.shutdown().await;
        }
        self.state = AdaptiveState::FailedClosed;
        Ok(())
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    pub fn protocol_version(&self) -> Option<&str> {
        match self.state {
            AdaptiveState::HttpReady | AdaptiveState::HttpCommitted => {
                self.http_client.protocol_version.as_deref()
            }
            AdaptiveState::StdioReady => self
                .stdio_client
                .as_ref()
                .and_then(|s| s.protocol_version.as_deref()),
            _ => None,
        }
    }

    pub fn capabilities(&self) -> Option<&Value> {
        match self.state {
            AdaptiveState::HttpReady | AdaptiveState::HttpCommitted => {
                self.http_client.capabilities.as_ref()
            }
            AdaptiveState::StdioReady => self
                .stdio_client
                .as_ref()
                .and_then(|s| s.capabilities.as_ref()),
            _ => None,
        }
    }
}
