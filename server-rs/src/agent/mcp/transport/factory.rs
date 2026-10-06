//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Transport Client Factory
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Shell command validation and environment sanitization prior to stdio process creation.
//! - `[Structural]` Deterministic mode dispatch across Auto, PreferHttp, Http, and Stdio.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::Forbidden`, `AppError::BadRequest`, `AppError::InfrastructureError`
//! - **Telemetry Targets**: none declared

use super::super::client::McpClient;
use super::super::config::{
    resolve_mcp_environment, resolve_mcp_headers, validate_mcp_server_config, McpMode,
    McpServerConfig,
};
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};

/// Spawns or connects and initializes an MCP client for the given server configuration.
pub async fn spawn_mcp_client(
    server_name: &str,
    server_config: &McpServerConfig,
) -> Result<McpClient, AppError> {
    validate_mcp_server_config(server_name, server_config)?;

    let mode = server_config.effective_mode();
    let http_opt = server_config.resolved_http_config();
    let stdio_opt = server_config.resolved_stdio_config();

    let mut client = match mode {
        McpMode::PreferHttp => {
            let http_cfg = http_opt.ok_or_else(|| {
                AppError::BadRequest(format!(
                    "MCP server '{}' in prefer_http mode must have http configuration",
                    server_name
                ))
            })?;
            let mut resolved_http = http_cfg.clone();
            let resolved_headers = resolve_mcp_headers(resolved_http.headers.as_ref())?;
            resolved_http.headers = if resolved_headers.is_empty() {
                None
            } else {
                Some(resolved_headers)
            };

            let resolved_stdio = if let Some(stdio_cfg) = stdio_opt {
                let cmd_to_test = if stdio_cfg.args.is_empty() {
                    stdio_cfg.command.clone()
                } else {
                    format!("{} {}", stdio_cfg.command, stdio_cfg.args.join(" "))
                };
                crate::utils::security::validate_shell_command(&cmd_to_test).map_err(|e| {
                    AppError::Forbidden(format!(
                        "Security boundary: Refusing to spawn untrusted MCP server '{}' with hazardous command '{}': {}",
                        server_name, cmd_to_test, e
                    ))
                })?;
                let resolved_env = resolve_mcp_environment(stdio_cfg.env.as_ref())?;
                let mut s = stdio_cfg.clone();
                s.env = if resolved_env.is_empty() {
                    None
                } else {
                    Some(resolved_env)
                };
                Some(s)
            } else {
                None
            };

            McpClient::connect_adaptive(server_name, resolved_http, resolved_stdio)?
        }
        McpMode::Http => {
            let http_cfg = http_opt.ok_or_else(|| {
                AppError::BadRequest(format!(
                    "MCP server '{}' in http mode must have url configuration",
                    server_name
                ))
            })?;
            let resolved_headers = resolve_mcp_headers(http_cfg.headers.as_ref())?;
            McpClient::connect_http(
                server_name,
                &http_cfg.url,
                if resolved_headers.is_empty() {
                    None
                } else {
                    Some(&resolved_headers)
                },
                http_cfg.protocol_versions.first().map(|s| s.as_str()),
            )?
        }
        McpMode::Stdio => {
            let stdio_cfg = stdio_opt.ok_or_else(|| {
                AppError::BadRequest(format!(
                    "MCP server '{}' in stdio mode must have command configuration",
                    server_name
                ))
            })?;
            let cmd_to_test = if stdio_cfg.args.is_empty() {
                stdio_cfg.command.clone()
            } else {
                format!("{} {}", stdio_cfg.command, stdio_cfg.args.join(" "))
            };
            crate::utils::security::validate_shell_command(&cmd_to_test).map_err(|e| {
                AppError::Forbidden(format!(
                    "Security boundary: Refusing to spawn untrusted MCP server '{}' with hazardous command '{}': {}",
                    server_name, cmd_to_test, e
                ))
            })?;

            let resolved_env = resolve_mcp_environment(stdio_cfg.env.as_ref())?;
            McpClient::spawn_stdio_with_cwd(
                server_name,
                &stdio_cfg.command,
                &stdio_cfg.args,
                if resolved_env.is_empty() {
                    None
                } else {
                    Some(&resolved_env)
                },
                stdio_cfg.cwd.as_deref(),
            )
            .await
            .map_err(|e| AppError::InfrastructureError {
                provider_id: ProviderId::Mcp,
                kind: InfrastructureErrorKind::Other,
                detail: format!("Failed to spawn MCP server '{}': {}", server_name, e),
                help_link: None,
            })?
        }
        McpMode::Auto => {
            if let (Some(http_cfg), Some(stdio_cfg)) = (http_opt.as_ref(), stdio_opt.as_ref()) {
                let mut resolved_http = http_cfg.clone();
                let resolved_headers = resolve_mcp_headers(resolved_http.headers.as_ref())?;
                resolved_http.headers = if resolved_headers.is_empty() {
                    None
                } else {
                    Some(resolved_headers)
                };

                let cmd_to_test = if stdio_cfg.args.is_empty() {
                    stdio_cfg.command.clone()
                } else {
                    format!("{} {}", stdio_cfg.command, stdio_cfg.args.join(" "))
                };
                crate::utils::security::validate_shell_command(&cmd_to_test).map_err(|e| {
                    AppError::Forbidden(format!(
                        "Security boundary: Refusing to spawn untrusted MCP server '{}' with hazardous command '{}': {}",
                        server_name, cmd_to_test, e
                    ))
                })?;
                let resolved_env = resolve_mcp_environment(stdio_cfg.env.as_ref())?;
                let mut s = stdio_cfg.clone();
                s.env = if resolved_env.is_empty() {
                    None
                } else {
                    Some(resolved_env)
                };

                McpClient::connect_adaptive(server_name, resolved_http, Some(s))?
            } else if let Some(http_cfg) = http_opt.as_ref() {
                let resolved_headers = resolve_mcp_headers(http_cfg.headers.as_ref())?;
                McpClient::connect_http(
                    server_name,
                    &http_cfg.url,
                    if resolved_headers.is_empty() {
                        None
                    } else {
                        Some(&resolved_headers)
                    },
                    http_cfg.protocol_versions.first().map(|s| s.as_str()),
                )?
            } else if let Some(stdio_cfg) = stdio_opt.as_ref() {
                let cmd_to_test = if stdio_cfg.args.is_empty() {
                    stdio_cfg.command.clone()
                } else {
                    format!("{} {}", stdio_cfg.command, stdio_cfg.args.join(" "))
                };
                crate::utils::security::validate_shell_command(&cmd_to_test).map_err(|e| {
                    AppError::Forbidden(format!(
                        "Security boundary: Refusing to spawn untrusted MCP server '{}' with hazardous command '{}': {}",
                        server_name, cmd_to_test, e
                    ))
                })?;
                let resolved_env = resolve_mcp_environment(stdio_cfg.env.as_ref())?;
                McpClient::spawn_stdio_with_cwd(
                    server_name,
                    &stdio_cfg.command,
                    &stdio_cfg.args,
                    if resolved_env.is_empty() {
                        None
                    } else {
                        Some(&resolved_env)
                    },
                    stdio_cfg.cwd.as_deref(),
                )
                .await
                .map_err(|e| AppError::InfrastructureError {
                    provider_id: ProviderId::Mcp,
                    kind: InfrastructureErrorKind::Other,
                    detail: format!("Failed to spawn MCP server '{}': {}", server_name, e),
                    help_link: None,
                })?
            } else {
                return Err(AppError::BadRequest(format!(
                    "MCP server '{}' in auto mode has neither HTTP nor stdio configured",
                    server_name
                )));
            }
        }
    };

    client
        .initialize()
        .await
        .map_err(|e| AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("Failed to initialize MCP client '{}': {}", server_name, e),
            help_link: None,
        })?;

    Ok(client)
}
