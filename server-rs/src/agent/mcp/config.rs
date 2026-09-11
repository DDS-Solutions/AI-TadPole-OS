//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Config
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Shell commands and environment variables must strictly pass validation before spawning.
//! - `[Structural]` Environment placeholders must fail-closed if unconfigured without leaking secrets in errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `config::tests::*`

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_MCP_PROTOCOL_VERSION: &str = "2026-07-28";
pub const GEV_MCP_URL: &str = "http://127.0.0.1:3000/mcp";
pub const GEV_MCP_AUTHORIZATION_PLACEHOLDER: &str = "${GEV_MCP_AUTHORIZATION}";
pub const GEV_STDIO_CWD: &str = "G:/AI-TadPole-Eye-View";

pub fn is_valid_header_token(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderKind {
    EnvironmentVariable,
    Header,
}

impl PlaceholderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlaceholderKind::EnvironmentVariable => "environment variable",
            PlaceholderKind::Header => "header",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum McpMode {
    #[default]
    Auto,
    PreferHttp,
    Http,
    Stdio,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct McpHttpConfig {
    pub url: String,
    #[serde(default)]
    pub protocol_versions: Vec<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub discovery_timeout_ms: Option<u64>,
    #[serde(default)]
    pub tool_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct McpStdioConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpServerConfig {
    #[serde(default)]
    pub mode: McpMode,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub protocol_version: Option<String>,
    #[serde(default)]
    pub http: Option<McpHttpConfig>,
    #[serde(default)]
    pub stdio_fallback: Option<McpStdioConfig>,
}

impl McpServerConfig {
    pub fn stdio(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            mode: McpMode::Stdio,
            command: Some(command.into()),
            args,
            cwd: None,
            env: None,
            url: None,
            headers: None,
            protocol_version: None,
            http: None,
            stdio_fallback: None,
        }
    }

    pub fn http(url: impl Into<String>) -> Self {
        Self {
            mode: McpMode::Http,
            command: None,
            args: Vec::new(),
            cwd: None,
            env: None,
            url: Some(url.into()),
            headers: None,
            protocol_version: None,
            http: None,
            stdio_fallback: None,
        }
    }

    pub fn prefer_http(http_config: McpHttpConfig, stdio_fallback: Option<McpStdioConfig>) -> Self {
        Self {
            mode: McpMode::PreferHttp,
            command: None,
            args: Vec::new(),
            cwd: None,
            env: None,
            url: Some(http_config.url.clone()),
            headers: http_config.headers.clone(),
            protocol_version: http_config.protocol_versions.first().cloned(),
            http: Some(http_config),
            stdio_fallback,
        }
    }

    pub fn effective_mode(&self) -> McpMode {
        match self.mode {
            McpMode::PreferHttp => McpMode::PreferHttp,
            McpMode::Http => McpMode::Http,
            McpMode::Stdio => McpMode::Stdio,
            McpMode::Auto => {
                if (self.http.is_some() || self.url.is_some())
                    && (self.stdio_fallback.is_some() || self.command.is_some())
                {
                    McpMode::PreferHttp
                } else if self.http.is_some() || self.url.is_some() {
                    McpMode::Http
                } else if self.stdio_fallback.is_some() || self.command.is_some() {
                    McpMode::Stdio
                } else {
                    McpMode::Auto
                }
            }
        }
    }

    pub fn resolved_http_config(&self) -> Option<McpHttpConfig> {
        if let Some(ref h) = self.http {
            let mut resolved = h.clone();
            if resolved.protocol_versions.is_empty() {
                resolved.protocol_versions = vec![DEFAULT_MCP_PROTOCOL_VERSION.to_string()];
            }
            if resolved.resource.is_none() {
                resolved.resource = Some(resolved.url.clone());
            }
            Some(resolved)
        } else if let Some(ref u) = self.url {
            let proto = self
                .protocol_version
                .clone()
                .unwrap_or_else(|| DEFAULT_MCP_PROTOCOL_VERSION.to_string());
            Some(McpHttpConfig {
                url: u.clone(),
                protocol_versions: vec![proto],
                resource: Some(u.clone()),
                headers: self.headers.clone(),
                discovery_timeout_ms: None,
                tool_timeout_ms: None,
            })
        } else {
            None
        }
    }

    pub fn resolved_stdio_config(&self) -> Option<McpStdioConfig> {
        if let Some(ref s) = self.stdio_fallback {
            Some(s.clone())
        } else {
            self.command.as_ref().map(|cmd| McpStdioConfig {
                command: cmd.clone(),
                args: self.args.clone(),
                cwd: self.cwd.clone(),
                env: self.env.clone(),
            })
        }
    }
}

pub fn validate_mcp_server_config(
    server_name: &str,
    config: &McpServerConfig,
) -> Result<(), AppError> {
    if server_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "MCP server name cannot be empty".to_string(),
        ));
    }

    if server_name.contains("__") {
        return Err(AppError::BadRequest(format!(
            "MCP server name '{}' cannot contain double underscores '__'",
            server_name
        )));
    }

    let http_cfg = config.resolved_http_config();
    let stdio_cfg = config.resolved_stdio_config();

    if http_cfg.is_none() && stdio_cfg.is_none() {
        return Err(AppError::BadRequest(format!(
            "MCP server '{}' must specify either 'command', 'url', or nested 'http'/'stdio_fallback'",
            server_name
        )));
    }

    if let Some(stdio) = &stdio_cfg {
        if stdio.command.trim().is_empty() {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' has an empty 'command'",
                server_name
            )));
        }
        let command_line = if stdio.args.is_empty() {
            stdio.command.clone()
        } else {
            format!("{} {}", stdio.command, stdio.args.join(" "))
        };
        crate::utils::security::validate_shell_command(&command_line).map_err(|error| {
            AppError::Forbidden(format!(
                "MCP server '{}' has an unsafe launcher: {}",
                server_name, error
            ))
        })?;
        for arg in &stdio.args {
            if arg.contains('\0') {
                return Err(AppError::BadRequest(format!(
                    "MCP server '{}' argument contains forbidden null byte",
                    server_name
                )));
            }
        }
        if let Some(environment) = &stdio.env {
            validate_environment_map(server_name, environment)?;
        }
    }

    if let Some(http) = &http_cfg {
        if http.url.trim().is_empty() {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' has an empty 'url'",
                server_name
            )));
        }
        let parsed = reqwest::Url::parse(&http.url).map_err(|e| {
            AppError::BadRequest(format!(
                "MCP server '{}' has invalid URL '{}': {}",
                server_name, http.url, e
            ))
        })?;
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' URL scheme must be http or https, got '{}'",
                server_name,
                parsed.scheme()
            )));
        }
        if let Some(ref resource) = http.resource {
            if resource.trim().is_empty() {
                return Err(AppError::BadRequest(format!(
                    "MCP server '{}' has an empty 'resource'",
                    server_name
                )));
            }
            let parsed_res = reqwest::Url::parse(resource).map_err(|e| {
                AppError::BadRequest(format!(
                    "MCP server '{}' has invalid resource URL '{}': {}",
                    server_name, resource, e
                ))
            })?;
            if parsed_res.scheme() != "http" && parsed_res.scheme() != "https" {
                return Err(AppError::BadRequest(format!(
                    "MCP server '{}' resource URL scheme must be http or https, got '{}'",
                    server_name,
                    parsed_res.scheme()
                )));
            }
        }
        if let Some(headers) = &http.headers {
            validate_headers_map(server_name, headers)?;
        }
        for (name, timeout) in [
            ("discovery_timeout_ms", http.discovery_timeout_ms),
            ("tool_timeout_ms", http.tool_timeout_ms),
        ] {
            if timeout.is_some_and(|value| value == 0 || value > 300_000) {
                return Err(AppError::BadRequest(format!(
                    "MCP server '{}' {} must be between 1 and 300000",
                    server_name, name
                )));
            }
        }
    }

    if let Some(environment) = &config.env {
        validate_environment_map(server_name, environment)?;
    }
    if let Some(headers) = &config.headers {
        validate_headers_map(server_name, headers)?;
    }

    if server_name == "gev" {
        validate_gev_profile(config)?;
    }

    Ok(())
}

fn validate_gev_profile(config: &McpServerConfig) -> Result<(), AppError> {
    let fail = |detail: &str| {
        AppError::BadRequest(format!(
            "MCP server 'gev' must use the fixed Port 3000 profile: {}",
            detail
        ))
    };

    if config.mode != McpMode::PreferHttp {
        return Err(fail("mode must be explicit 'prefer_http'"));
    }
    if config.command.is_some()
        || config.url.is_some()
        || config.headers.is_some()
        || config.protocol_version.is_some()
    {
        return Err(fail("use only nested 'http' and 'stdio_fallback' fields"));
    }

    let http = config
        .http
        .as_ref()
        .ok_or_else(|| fail("nested 'http' configuration is required"))?;
    if http.url != GEV_MCP_URL {
        return Err(fail("HTTP URL must be http://127.0.0.1:3000/mcp"));
    }
    if http.resource.as_deref() != Some(GEV_MCP_URL) {
        return Err(fail("resource must be http://127.0.0.1:3000/mcp"));
    }
    if http.protocol_versions != [DEFAULT_MCP_PROTOCOL_VERSION] {
        return Err(fail("protocol_versions must contain only 2026-07-28"));
    }
    let expected_headers = HashMap::from([(
        "Authorization".to_string(),
        GEV_MCP_AUTHORIZATION_PLACEHOLDER.to_string(),
    )]);
    if http.headers.as_ref() != Some(&expected_headers) {
        return Err(fail(
            "headers must contain only Authorization=${GEV_MCP_AUTHORIZATION}",
        ));
    }

    let stdio = config
        .stdio_fallback
        .as_ref()
        .ok_or_else(|| fail("stdio_fallback is required"))?;
    if stdio.command != "pnpm"
        || stdio.args != ["--filter", "@gev/ops-mcp", "start"]
        || stdio.cwd.as_deref() != Some(GEV_STDIO_CWD)
        || stdio.env.is_some()
    {
        return Err(fail(
            "stdio fallback must be 'pnpm --filter @gev/ops-mcp start' from G:/AI-TadPole-Eye-View",
        ));
    }

    Ok(())
}

fn validate_environment_map(
    server_name: &str,
    environment: &HashMap<String, String>,
) -> Result<(), AppError> {
    for (name, value) in environment {
        if !is_valid_environment_name(name) {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' has invalid environment variable name '{}'",
                server_name, name
            )));
        }
        if value.starts_with("${") && value.ends_with('}') {
            let placeholder = &value[2..value.len() - 1];
            if !is_valid_environment_name(placeholder) {
                return Err(AppError::BadRequest(format!(
                    "MCP server '{}' has invalid environment placeholder '{}'",
                    server_name, value
                )));
            }
        }
    }
    Ok(())
}

fn validate_headers_map(
    server_name: &str,
    headers: &HashMap<String, String>,
) -> Result<(), AppError> {
    let forbidden = |c: char| matches!(c, '\r' | '\n' | '\0');
    for (name, value) in headers {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' has empty header name",
                server_name
            )));
        }
        if name.chars().any(forbidden) || name.starts_with(' ') || name.ends_with(' ') {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' header name '{}' contains invalid characters or whitespace",
                server_name, name
            )));
        }
        if !is_valid_header_token(name) {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' header name '{}' is not a valid HTTP header token",
                server_name, name
            )));
        }
        let lower_name = name.to_ascii_lowercase();
        if matches!(
            lower_name.as_str(),
            "host"
                | "content-type"
                | "accept"
                | "mcp-protocol-version"
                | "mcp-method"
                | "mcp-name"
                | "origin"
                | "mcp-session-id"
                | "last-event-id"
        ) || lower_name.starts_with("mcp-param-")
        {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' may not override transport-owned header '{}'",
                server_name, name
            )));
        }
        let starts_with_whitespace = value.chars().next().is_some_and(char::is_whitespace);
        let ends_with_whitespace = value.chars().next_back().is_some_and(char::is_whitespace);
        if value.chars().any(forbidden) || starts_with_whitespace || ends_with_whitespace {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' header '{}' value contains forbidden control or edge whitespace",
                server_name, name
            )));
        }
        if value.contains("${") && !(value.starts_with("${") && value.ends_with('}')) {
            return Err(AppError::BadRequest(format!(
                "MCP server '{}' header '{}' must use an exact ${{NAME}} placeholder",
                server_name, name
            )));
        }
        if value.starts_with("${") && value.ends_with('}') {
            let placeholder = &value[2..value.len() - 1];
            if !is_valid_environment_name(placeholder) {
                return Err(AppError::BadRequest(format!(
                    "MCP server '{}' has invalid header placeholder '{}'",
                    server_name, value
                )));
            }
        }
    }
    Ok(())
}

pub fn resolve_mcp_environment(
    configured: Option<&HashMap<String, String>>,
) -> Result<HashMap<String, String>, AppError> {
    resolve_placeholders_internal(configured, PlaceholderKind::EnvironmentVariable)
}

pub fn resolve_mcp_headers(
    configured: Option<&HashMap<String, String>>,
) -> Result<HashMap<String, String>, AppError> {
    resolve_placeholders_internal(configured, PlaceholderKind::Header)
}

fn resolve_placeholders_internal(
    configured: Option<&HashMap<String, String>>,
    kind: PlaceholderKind,
) -> Result<HashMap<String, String>, AppError> {
    let mut resolved = HashMap::new();
    let Some(configured) = configured else {
        return Ok(resolved);
    };

    for (key, value) in configured {
        match kind {
            PlaceholderKind::EnvironmentVariable => {
                if !is_valid_environment_name(key) {
                    return Err(AppError::BadRequest(format!(
                        "MCP environment variable name '{}' is invalid",
                        key
                    )));
                }
            }
            PlaceholderKind::Header => {
                if !is_valid_header_token(key) {
                    return Err(AppError::BadRequest(format!(
                        "MCP header name '{}' is invalid",
                        key
                    )));
                }
            }
        }

        let resolved_value = if value.starts_with("${") && value.ends_with('}') {
            let variable_name = &value[2..value.len() - 1];
            if !is_valid_environment_name(variable_name) {
                return Err(AppError::BadRequest(format!(
                    "MCP {} placeholder '{}' is invalid",
                    kind.as_str(),
                    value
                )));
            }
            std::env::var(variable_name).map_err(|_| {
                AppError::BadRequest(format!(
                    "MCP {} '{}' is required by placeholder '{}' but is not configured",
                    kind.as_str(),
                    variable_name,
                    value
                ))
            })?
        } else if value == "CONFIGURE_LOCALLY" {
            return Err(AppError::BadRequest(format!(
                "MCP {} '{}' is set to placeholder sentinel 'CONFIGURE_LOCALLY'",
                kind.as_str(),
                key
            )));
        } else {
            if kind == PlaceholderKind::Header
                && value.chars().any(|c| matches!(c, '\r' | '\n' | '\0'))
            {
                return Err(AppError::BadRequest(format!(
                    "MCP header '{}' value contains forbidden control or CRLF characters",
                    key
                )));
            }
            value.clone()
        };

        resolved.insert(key.clone(), resolved_value);
    }

    Ok(resolved)
}

pub fn is_valid_environment_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn resolves_environment_placeholders_without_exposing_secret_values() {
        let expected_path = std::env::var_os("PATH")
            .expect("PATH must exist in the test environment")
            .into_string()
            .expect("PATH must be valid Unicode in the test environment");
        let configured = HashMap::from([
            ("CHILD_PATH".to_string(), "${PATH}".to_string()),
            ("LITERAL_SETTING".to_string(), "enabled".to_string()),
        ]);

        let resolved = resolve_mcp_environment(Some(&configured)).unwrap();

        assert_eq!(resolved.get("CHILD_PATH"), Some(&expected_path));
        assert_eq!(
            resolved.get("LITERAL_SETTING").map(String::as_str),
            Some("enabled")
        );
    }

    #[test]
    fn rejects_placeholder_sentinel_configure_locally() {
        let configured =
            HashMap::from([("API_TOKEN".to_string(), "CONFIGURE_LOCALLY".to_string())]);

        let error = resolve_mcp_environment(Some(&configured)).unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
        assert!(!error.to_string().contains("API_TOKEN="));
    }

    #[test]
    fn rejects_unset_environment_variable_placeholder() {
        let configured = HashMap::from([(
            "API_TOKEN".to_string(),
            "${GLM_DEFINITELY_UNSET_VAR_7Q3}".to_string(),
        )]);

        let error = resolve_mcp_environment(Some(&configured)).unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
        assert!(error.to_string().contains("GLM_DEFINITELY_UNSET_VAR_7Q3"));
    }

    #[test]
    fn rejects_header_crlf_injection() {
        let bad_header_value = HashMap::from([(
            "Authorization".to_string(),
            "Bearer evil\r\nInjected: true".to_string(),
        )]);
        let err = resolve_mcp_headers(Some(&bad_header_value)).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));

        let bad_header_name = HashMap::from([("Bad\r\nName".to_string(), "valid_val".to_string())]);
        let err2 = resolve_mcp_headers(Some(&bad_header_name)).unwrap_err();
        assert!(matches!(err2, AppError::BadRequest(_)));
    }

    #[test]
    fn rejects_empty_url_configuration() {
        let config = McpServerConfig {
            command: None,
            args: Vec::new(),
            env: None,
            url: Some("   ".to_string()),
            headers: None,
            protocol_version: None,
            ..Default::default()
        };
        let err = validate_mcp_server_config("empty_url_srv", &config).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn rejects_empty_command_configuration() {
        let config = McpServerConfig {
            command: Some("   ".to_string()),
            args: Vec::new(),
            env: None,
            url: None,
            headers: None,
            protocol_version: None,
            ..Default::default()
        };
        let err = validate_mcp_server_config("empty_cmd_srv", &config).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn rejects_invalid_environment_names() {
        let configured = HashMap::from([("BAD-NAME".to_string(), "value".to_string())]);

        let error = resolve_mcp_environment(Some(&configured)).unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn validates_safe_mcp_server_configuration() {
        let config = McpServerConfig {
            command: Some("python".to_string()),
            args: vec!["execution/server.py".to_string()],
            env: Some(HashMap::from([(
                "API_TOKEN".to_string(),
                "${LOCAL_API_TOKEN}".to_string(),
            )])),
            url: None,
            headers: None,
            protocol_version: None,
            ..Default::default()
        };

        validate_mcp_server_config("example", &config).unwrap();
    }

    #[test]
    fn validates_safe_mcp_http_server_configuration() {
        let config = McpServerConfig {
            command: None,
            args: Vec::new(),
            env: None,
            url: Some("https://mcp.example.com/api".to_string()),
            headers: Some(HashMap::from([(
                "Authorization".to_string(),
                "${AUTH_TOKEN}".to_string(),
            )])),
            protocol_version: Some("2026-07-28".to_string()),
            ..Default::default()
        };

        validate_mcp_server_config("remote_example", &config).unwrap();
    }

    #[test]
    fn rejects_mcp_server_with_double_underscore_in_name() {
        let config = McpServerConfig::http("https://mcp.example.com/api");
        let err = validate_mcp_server_config("server__name", &config).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn rejects_mcp_server_with_invalid_url_scheme() {
        let config = McpServerConfig {
            command: None,
            args: Vec::new(),
            env: None,
            url: Some("ftp://mcp.example.com/api".to_string()),
            headers: None,
            protocol_version: None,
            ..Default::default()
        };

        let err = validate_mcp_server_config("bad_scheme", &config).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn rejects_mcp_server_with_neither_command_nor_url() {
        let config = McpServerConfig::default();
        let err = validate_mcp_server_config("empty_config", &config).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn rejects_mcp_server_with_invalid_placeholder() {
        let config = McpServerConfig {
            command: Some("python".to_string()),
            args: vec!["execution/server.py".to_string()],
            env: Some(HashMap::from([(
                "API_TOKEN".to_string(),
                "${BAD-NAME}".to_string(),
            )])),
            url: None,
            headers: None,
            protocol_version: None,
            ..Default::default()
        };

        let error = validate_mcp_server_config("example", &config).unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn parses_and_validates_coexisting_prefer_http_configuration() {
        let json_data = serde_json::json!({
            "mode": "prefer_http",
            "http": {
                "url": "http://127.0.0.1:3000/mcp",
                "protocol_versions": ["2026-07-28"],
                "resource": "http://127.0.0.1:3000/mcp",
                "headers": {
                    "Authorization": "${GEV_MCP_AUTHORIZATION}"
                }
            },
            "stdio_fallback": {
                "command": "pnpm",
                "args": ["--filter", "@gev/ops-mcp", "start"],
                "cwd": "G:/AI-TadPole-Eye-View"
            }
        });

        let config: McpServerConfig = serde_json::from_value(json_data).unwrap();
        assert_eq!(config.effective_mode(), McpMode::PreferHttp);

        let http = config.resolved_http_config().unwrap();
        assert_eq!(http.url, "http://127.0.0.1:3000/mcp");
        assert_eq!(http.protocol_versions, vec!["2026-07-28"]);

        let stdio = config.resolved_stdio_config().unwrap();
        assert_eq!(stdio.command, "pnpm");
        assert_eq!(stdio.args, vec!["--filter", "@gev/ops-mcp", "start"]);
        assert_eq!(stdio.cwd.as_deref(), Some("G:/AI-TadPole-Eye-View"));

        validate_mcp_server_config("gev", &config).unwrap();
    }

    #[test]
    fn rejects_every_near_miss_gev_profile() {
        let exact = serde_json::json!({
            "mode": "prefer_http",
            "http": {
                "url": GEV_MCP_URL,
                "protocol_versions": [DEFAULT_MCP_PROTOCOL_VERSION],
                "resource": GEV_MCP_URL,
                "headers": { "Authorization": GEV_MCP_AUTHORIZATION_PLACEHOLDER }
            },
            "stdio_fallback": {
                "command": "pnpm",
                "args": ["--filter", "@gev/ops-mcp", "start"],
                "cwd": GEV_STDIO_CWD
            }
        });

        for (label, mutation) in [
            ("auto mode", ("/mode", serde_json::json!("auto"))),
            (
                "wrong host",
                ("/http/url", serde_json::json!("http://localhost:3000/mcp")),
            ),
            (
                "wrong resource",
                (
                    "/http/resource",
                    serde_json::json!("http://127.0.0.1:3000/"),
                ),
            ),
            (
                "extra version",
                (
                    "/http/protocol_versions",
                    serde_json::json!(["2026-07-28", "2024-11-05"]),
                ),
            ),
            (
                "literal token",
                (
                    "/http/headers/Authorization",
                    serde_json::json!("Bearer secret"),
                ),
            ),
            (
                "wrong launcher",
                ("/stdio_fallback/command", serde_json::json!("npm")),
            ),
        ] {
            let mut value = exact.clone();
            *value.pointer_mut(mutation.0).expect(label) = mutation.1;
            let config: McpServerConfig = serde_json::from_value(value).unwrap();
            assert!(
                validate_mcp_server_config("gev", &config).is_err(),
                "near-miss profile must fail: {label}"
            );
        }

        let mut unsupported_secret_field = exact;
        unsupported_secret_field["http"]["bearer_token_source"] = serde_json::json!("injected");
        assert!(serde_json::from_value::<McpServerConfig>(unsupported_secret_field).is_err());
    }

    #[test]
    fn rejects_composite_header_placeholder_and_transport_owned_headers() {
        for headers in [
            HashMap::from([(
                "Authorization".to_string(),
                "Bearer ${AUTH_TOKEN}".to_string(),
            )]),
            HashMap::from([("Mcp-Method".to_string(), "tools/list".to_string())]),
            HashMap::from([("Origin".to_string(), "http://example.test".to_string())]),
        ] {
            assert!(validate_headers_map("example", &headers).is_err());
        }
    }

    #[test]
    fn tracked_mcp_config_contains_the_exact_secret_free_gev_profile() {
        let content = include_str!("../../../../.agent/mcp_config.json");
        assert!(!content.contains("Bearer "));
        let config: McpConfig = serde_json::from_str(content).unwrap();
        let gev = config.mcp_servers.get("gev").expect("tracked gev entry");
        validate_mcp_server_config("gev", gev).unwrap();
    }
}
