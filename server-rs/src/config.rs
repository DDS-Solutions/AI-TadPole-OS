//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / config
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Config]`
//! - **Witness Tests**: none declared

use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub port: u16,
    pub bind_address: String,
    pub socket_addr: SocketAddr,
    pub workspace_root: Option<PathBuf>,
    pub disable_telemetry: bool,
    pub heartbeat_interval_secs: u64,
    pub mirror_mode: bool,
}

impl Config {
    /// Loads environment variables, validates them against the `.env.schema`,
    /// and parses configuration into a validated `Config` struct.
    pub fn load() -> Result<Self, anyhow::Error> {
        // 1. Initialize environment variables from .env if present
        if dotenvy::dotenv().is_err() {
            println!("⚠️  [Config] No .env file found. Relying on system environment variables.");
        }

        // 2. Validate environment variables against schema
        let schema_path = std::path::Path::new(".env.schema");
        if let Err(e) = crate::env_schema::validate_and_report(schema_path) {
            let err_msg = format!("🚨 [EnvSchema] Validation failed: {}", e);
            eprintln!("{}", err_msg);
            if !cfg!(debug_assertions) {
                anyhow::bail!("{}", err_msg);
            }
        }

        Self::load_internal(|key| std::env::var(key).ok())
    }

    fn load_internal<F>(get_env: F) -> Result<Self, anyhow::Error>
    where
        F: Fn(&str) -> Option<String>,
    {
        // 3. Resolve and validate PORT
        let port_str = get_env("PORT").unwrap_or_else(|| "8000".to_string());
        let port: u16 = port_str
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid PORT value '{}': {:?}", port_str, e))?;

        if port < 1024 {
            println!(
                "⚠️  [Config] WARNING: PORT {} is in the privileged range (< 1024)",
                port
            );
        }

        // 4. Resolve and validate BIND_ADDRESS
        let bind_address = get_env("BIND_ADDRESS").unwrap_or_else(|| "127.0.0.1".to_string());
        let socket_addr: SocketAddr =
            format!("{}:{}", bind_address, port).parse().map_err(|e| {
                anyhow::anyhow!(
                    "Invalid BIND_ADDRESS '{}' or PORT '{}': {:?}",
                    bind_address,
                    port,
                    e
                )
            })?;

        // 5. Resolve and validate WORKSPACE_ROOT
        let workspace_root = if let Some(root) = get_env("WORKSPACE_ROOT") {
            let root_path = PathBuf::from(root);
            if root_path.exists() && root_path.is_dir() {
                let canonical = root_path.canonicalize().map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to canonicalize WORKSPACE_ROOT {:?}: {:?}",
                        root_path,
                        e
                    )
                })?;
                Some(canonical)
            } else {
                anyhow::bail!(
                    "WORKSPACE_ROOT path does not exist or is not a directory: {:?}",
                    root_path
                );
            }
        } else {
            None
        };

        // 6. Resolve optional features
        let disable_telemetry = get_env("DISABLE_TELEMETRY").as_deref() == Some("true");

        let heartbeat_interval_secs: u64 = get_env("HEARTBEAT_INTERVAL_SECS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        let mirror_mode = get_env("MIRROR_MODE").as_deref() == Some("true");

        Ok(Self {
            port,
            bind_address,
            socket_addr,
            workspace_root,
            disable_telemetry,
            heartbeat_interval_secs,
            mirror_mode,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_invalid_port_parsing() {
        let mut envs = HashMap::new();
        envs.insert("PORT".to_string(), "abc".to_string());

        let result = Config::load_internal(|k| envs.get(k).cloned());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_bind_address() {
        let mut envs = HashMap::new();
        envs.insert("BIND_ADDRESS".to_string(), "10.0.0.1".to_string());
        envs.insert("PORT".to_string(), "8000".to_string());

        let result = Config::load_internal(|k| envs.get(k).cloned());
        assert!(result.is_err());
    }

    #[test]
    fn test_nonexistent_workspace_root_fails() {
        let mut envs = HashMap::new();
        envs.insert(
            "WORKSPACE_ROOT".to_string(),
            "/nonexistent/directory/path/here".to_string(),
        );

        let result = Config::load_internal(|k| envs.get(k).cloned());
        assert!(result.is_err());
    }

    #[test]
    fn test_existing_workspace_root_canonicalizes() {
        let dir = tempdir().unwrap();
        let path_str = dir.path().to_str().unwrap().to_string();

        let mut envs = HashMap::new();
        envs.insert("WORKSPACE_ROOT".to_string(), path_str);

        let result = Config::load_internal(|k| envs.get(k).cloned());
        assert!(result.is_ok(), "Config load failed: {:?}", result.err());
        let config = result.unwrap();
        assert_eq!(
            config.workspace_root.unwrap(),
            dir.path().canonicalize().unwrap()
        );
    }
}
