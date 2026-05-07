//! @docs ARCHITECTURE:EnvironmentSecurity
//!
//! ### AI Assist Note
//! **Startup Environment Validation (Gatekeeper)**: Orchestrates the
//! reading and verification of the `.env.schema` file to ensure
//! systemic requirements are met before engine boot. Features
//! **Schema-based Variable Verification**: supports decorators like
//! `@required`, `@sensitive`, and `@type` to manage data
//! sovereignty and security. Implements **Strict Fatal Halt**: in
//! production, the engine will `anyhow::bail!` if required variables
//! (e.g., `NEURAL_TOKEN`) are missing, preventing accidental
//! unauthenticated outages. AI agents should use the "Validation
//! Report" banner in logs to confirm environment readiness (ENV-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Missing `.env.schema` file at the root,
//!   malformed schema decorators causing parse errors, or
//!   unauthorized access to sensitive variables in debug logs.
//! - **Telemetry Link**: Search for `🔐 [EnvSchema]` in `tracing`
//!   logs for the startup "Validation Report" status.
//! - **Trace Scope**: `server-rs::env_schema`

use std::path::Path;

/// A single entry parsed from the `.env.schema` file.
#[derive(Debug, Clone)]
pub struct EnvSchemaEntry {
    /// The environment variable name (e.g. `NEURAL_TOKEN`).
    pub name: String,
    /// Whether the engine fails to start if this variable is missing.
    pub required: bool,
    /// Whether the value should be redacted in logs and audits.
    pub sensitive: bool,
    /// Technical description appearing in tooltips/logs.
    pub description: String,
    /// Optional default value if not provided by the environment.
    pub default: Option<String>,
    /// Primitive type for validation (e.g. `int`, `url`, `string`).
    pub var_type: Option<String>,
}

/// Parsed schema result.
#[derive(Debug, Clone)]
pub struct EnvSchema {
    pub entries: Vec<EnvSchemaEntry>,
}

/// Validation result for a single variable.
#[derive(Debug)]
pub struct ValidationResult {
    pub name: String,
    pub is_set: bool,
    pub required: bool,
    pub sensitive: bool,
    #[allow(dead_code)]
    pub description: String,
}

impl EnvSchema {
    /// Parse the `.env.schema` file using a lightweight line-based parser.
    /// Understands `@required`, `@sensitive`, `@type=...`, `@default=...` decorators.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();

        let mut current_decorators: Vec<String> = Vec::new();
        let mut current_comments: Vec<String> = Vec::new();
        let mut default_sensitive = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Parse header decorators
            if trimmed.starts_with("# @defaultSensitive=true") {
                default_sensitive = true;
                continue;
            }
            if trimmed.starts_with("# ---") || trimmed.starts_with("# ==") {
                // Divider — reset block comments
                current_decorators.clear();
                current_comments.clear();
                continue;
            }

            // Comment lines
            if trimmed.starts_with('#') {
                let comment_text = trimmed.trim_start_matches('#').trim();
                if comment_text.starts_with('@') {
                    current_decorators.push(comment_text.to_string());
                } else if !comment_text.is_empty() {
                    current_comments.push(comment_text.to_string());
                }
                continue;
            }

            // Empty lines reset the comment accumulator
            if trimmed.is_empty() {
                current_decorators.clear();
                current_comments.clear();
                continue;
            }

            // Variable assignment line: NAME=value
            if let Some(eq_pos) = trimmed.find('=') {
                let var_name = &trimmed[..eq_pos];

                // Parse decorators
                let mut required = false;
                let mut sensitive = default_sensitive;
                let mut var_type = None;
                let mut default = None;

                for dec in &current_decorators {
                    for token in dec.split_whitespace() {
                        match token {
                            "@required" => required = true,
                            "@sensitive" => sensitive = true,
                            _ if token.starts_with("@type=") => {
                                var_type = Some(token.trim_start_matches("@type=").to_string());
                            }
                            _ if token.starts_with("@default=") => {
                                default = Some(token.trim_start_matches("@default=").to_string());
                            }
                            _ if token.starts_with("@sensitive=false") => {
                                sensitive = false;
                            }
                            _ => {}
                        }
                    }
                }

                let description = current_comments.join(" ");

                entries.push(EnvSchemaEntry {
                    name: var_name.to_string(),
                    required,
                    sensitive,
                    description,
                    default,
                    var_type,
                });

                current_decorators.clear();
                current_comments.clear();
            }
        }

        Ok(Self { entries })
    }

    /// Validate all schema entries against the current environment.
    /// Returns a list of validation results.
    pub fn validate(&self) -> Vec<ValidationResult> {
        self.entries
            .iter()
            .map(|entry| {
                let is_set = std::env::var(&entry.name)
                    .map(|v| !v.trim().is_empty())
                    .unwrap_or(false);

                ValidationResult {
                    name: entry.name.clone(),
                    is_set,
                    required: entry.required,
                    sensitive: entry.sensitive,
                    description: entry.description.clone(),
                }
            })
            .collect()
    }

    /// Returns entries as a safe metadata list (for the schema API).
    /// Never includes actual secret values.
    pub fn to_safe_metadata(&self) -> Vec<serde_json::Value> {
        self.entries
            .iter()
            .map(|entry| {
                let is_set = std::env::var(&entry.name)
                    .map(|v| !v.trim().is_empty())
                    .unwrap_or(false);

                serde_json::json!({
                    "name": entry.name,
                    "required": entry.required,
                    "sensitive": entry.sensitive,
                    "description": entry.description,
                    "type": entry.var_type,
                    "hasDefault": entry.default.is_some(),
                    "isSet": is_set,
                })
            })
            .collect()
    }
}

/// Run startup validation. Logs a clear banner and returns any fatal errors.
pub fn validate_and_report(schema_path: &Path) -> anyhow::Result<()> {
    if !schema_path.exists() {
        tracing::warn!(
            "⚠️  [EnvSchema] No .env.schema found at {:?} — skipping validation.",
            schema_path
        );
        return Ok(());
    }

    let schema = EnvSchema::load(schema_path)?;
    let results = schema.validate();

    tracing::info!("╔══════════════════════════════════════════════════════╗");
    tracing::info!("║           🔐 Environment Validation Report          ║");
    tracing::info!("╠══════════════════════════════════════════════════════╣");

    let mut missing_required = Vec::new();

    for result in &results {
        let status = if result.is_set {
            "✅"
        } else if result.required {
            "🚨"
        } else {
            "⚠️ "
        };
        let sensitivity = if result.sensitive { "🔒" } else { "  " };
        let set_label = if result.is_set { "set" } else { "missing" };

        tracing::info!(
            "║ {} {} {:<30} ({})",
            status,
            sensitivity,
            result.name,
            set_label
        );

        if result.required && !result.is_set {
            missing_required.push(result.name.clone());
        }
    }

    tracing::info!("╚══════════════════════════════════════════════════════╝");

    if !missing_required.is_empty() {
        let names = missing_required.join(", ");
        if cfg!(debug_assertions) {
            tracing::warn!(
                "⚠️  [EnvSchema] Missing required variables (dev mode — continuing): {}",
                names
            );
        } else {
            // NEURAL_TOKEN is now strictly required. No default injection.

            anyhow::bail!(
                "🚨 FATAL: Missing required environment variables: {}. Set them in .env or your secret manager.",
                names
            );
        }
    }

    tracing::info!(
        "🔐 [EnvSchema] Validation complete — {} variables checked.",
        results.len()
    );
    Ok(())
}

// Metadata: [env_schema]

// Metadata: [env_schema]
