//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / a2a_mailbox
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use sqlx::SqlitePool;
use std::time::Duration;

pub const MAX_ENVELOPE_ID_LEN: usize = 128;
pub const MAX_AGENT_ID_LEN: usize = 128;
pub const MAX_TARGET_ID_LEN: usize = 512;
pub const MAX_INSTRUCTION_BYTES: usize = 128 * 1024; // 128 KB
pub const MAX_REASONING_TRACE_BYTES: usize = 128 * 1024; // 128 KB
pub const MAX_RESULT_BYTES: usize = 128 * 1024; // 128 KB
pub const MAX_ARTIFACTS_COUNT: usize = 50;
pub const MAX_ARTIFACT_ITEM_BYTES: usize = 4 * 1024; // 4 KB per item
pub const MAX_ARTIFACTS_TOTAL_BYTES: usize = 512 * 1024; // 512 KB total

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailboxEnvelope {
    pub id: String,
    pub mission_id: String,
    pub source_agent_id: String,
    pub target_agent_id: String,
    pub instruction: String,
    pub reasoning_trace: Option<String>,
    pub status: String,
    pub result: Option<String>,
    pub artifacts: Option<Vec<String>>,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub nonce: Option<String>,
}

pub struct A2AMailbox {
    pool: SqlitePool,
    http_client: reqwest::Client,
}

impl A2AMailbox {
    pub fn new(pool: SqlitePool) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(4))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { pool, http_client }
    }

    /// Validates remote endpoint destination against SSRF and metadata attacks.
    fn validate_remote_target(url_str: &str) -> Result<String, AppError> {
        let parsed = url::Url::parse(url_str).map_err(|e| {
            AppError::BadRequest(format!(
                "Invalid remote agent endpoint URL '{}': {}",
                url_str, e
            ))
        })?;

        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(AppError::BadRequest(
                "Remote agent URL scheme must be HTTP or HTTPS".to_string(),
            ));
        }

        if let Some(host) = parsed.host_str() {
            let host_lower = host.to_lowercase();
            // Block cloud metadata and link-local address targets
            if host_lower == "10.0.0.1"
                || host_lower == "metadata.google.internal"
                || host_lower == "instance-data"
                || host_lower.starts_with("169.254.")
            {
                return Err(AppError::Forbidden(
                    "SSRF Security Gate: Access to cloud metadata or link-local endpoints is forbidden"
                        .to_string(),
                ));
            }
        }

        Ok(url_str.to_string())
    }

    /// Validates all envelope payload fields against the system bounds contract.
    pub fn validate_envelope_bounds(envelope: &MailboxEnvelope) -> Result<(), AppError> {
        if envelope.id.trim().is_empty() || envelope.id.len() > MAX_ENVELOPE_ID_LEN {
            return Err(AppError::BadRequest(format!(
                "Envelope ID must be non-empty and at most {} characters",
                MAX_ENVELOPE_ID_LEN
            )));
        }
        if envelope.source_agent_id.trim().is_empty()
            || envelope.source_agent_id.len() > MAX_AGENT_ID_LEN
        {
            return Err(AppError::BadRequest(format!(
                "Source agent ID must be non-empty and at most {} characters",
                MAX_AGENT_ID_LEN
            )));
        }
        if envelope.target_agent_id.trim().is_empty()
            || envelope.target_agent_id.len() > MAX_TARGET_ID_LEN
        {
            return Err(AppError::BadRequest(format!(
                "Target agent ID must be non-empty and at most {} characters",
                MAX_TARGET_ID_LEN
            )));
        }
        if envelope.instruction.len() > MAX_INSTRUCTION_BYTES {
            return Err(AppError::BadRequest(format!(
                "Instruction exceeds maximum allowed size of {} bytes",
                MAX_INSTRUCTION_BYTES
            )));
        }
        if let Some(ref trace) = envelope.reasoning_trace {
            if trace.len() > MAX_REASONING_TRACE_BYTES {
                return Err(AppError::BadRequest(format!(
                    "Reasoning trace exceeds maximum allowed size of {} bytes",
                    MAX_REASONING_TRACE_BYTES
                )));
            }
        }
        if let Some(ref result) = envelope.result {
            if result.len() > MAX_RESULT_BYTES {
                return Err(AppError::BadRequest(format!(
                    "Result exceeds maximum allowed size of {} bytes",
                    MAX_RESULT_BYTES
                )));
            }
        }
        if let Some(ref artifacts) = envelope.artifacts {
            if artifacts.len() > MAX_ARTIFACTS_COUNT {
                return Err(AppError::BadRequest(format!(
                    "Artifacts count exceeds maximum allowed of {} items",
                    MAX_ARTIFACTS_COUNT
                )));
            }
            let mut total_bytes = 0;
            for (idx, item) in artifacts.iter().enumerate() {
                if item.len() > MAX_ARTIFACT_ITEM_BYTES {
                    return Err(AppError::BadRequest(format!(
                        "Artifact item [{}] exceeds maximum allowed size of {} bytes",
                        idx, MAX_ARTIFACT_ITEM_BYTES
                    )));
                }
                total_bytes += item.len();
            }
            if total_bytes > MAX_ARTIFACTS_TOTAL_BYTES {
                return Err(AppError::BadRequest(format!(
                    "Total artifacts size exceeds maximum allowed of {} bytes",
                    MAX_ARTIFACTS_TOTAL_BYTES
                )));
            }
        }
        Ok(())
    }

    pub async fn send_envelope(&self, envelope: &MailboxEnvelope) -> Result<(), AppError> {
        // Enforce chokepoint payload validation
        Self::validate_envelope_bounds(envelope)?;

        if envelope.target_agent_id.starts_with("http://")
            || envelope.target_agent_id.starts_with("https://")
        {
            // SSRF Validation
            Self::validate_remote_target(&envelope.target_agent_id)?;

            // Serialize payload to exact string bytes to ensure signature byte parity
            let payload_str = serde_json::to_string(envelope)
                .map_err(|e| AppError::BadRequest(format!("Failed to serialize payload: {}", e)))?;

            // Generate A2A signature over exact serialized body
            let signature =
                crate::agent::runner::tools::capability::sign_a2a_envelope(&payload_str);

            // Determine target endpoint URL
            let url = if envelope.target_agent_id.ends_with("/send") {
                envelope.target_agent_id.clone()
            } else {
                format!(
                    "{}/v1/engine/a2a/mailbox/send",
                    envelope.target_agent_id.trim_end_matches('/')
                )
            };

            tracing::info!(
                "📨 [a2a_mailbox] Dispatching remote envelope {} to {}",
                envelope.id,
                url
            );

            let response = self
                .http_client
                .post(&url)
                .header("X-A2A-Signature", signature)
                .header("Content-Type", "application/json")
                .body(payload_str)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(AppError::InternalServerError(format!(
                    "Remote A2A delivery failed: HTTP {} - {}",
                    status, text
                )));
            }
            return Ok(());
        }

        let artifacts_json = envelope
            .artifacts
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        let res = sqlx::query::<sqlx::Sqlite>(
            "INSERT INTO agent_directives (id, mission_id, source_agent_id, target_agent_id, instruction, status, result, reasoning_trace, artifacts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                instruction = excluded.instruction,
                artifacts = excluded.artifacts,
                reasoning_trace = excluded.reasoning_trace
             WHERE agent_directives.status = 'pending'",
        )
        .bind(&envelope.id)
        .bind(&envelope.mission_id)
        .bind(&envelope.source_agent_id)
        .bind(&envelope.target_agent_id)
        .bind(&envelope.instruction)
        .bind(&envelope.status)
        .bind(&envelope.result)
        .bind(&envelope.reasoning_trace)
        .bind(artifacts_json)
        .execute(&self.pool)
        .await?;

        if res.rows_affected() == 0 {
            tracing::warn!(
                "⚠️ [a2a_mailbox] Directive {} update/replay ignored: directive already active or finalized",
                envelope.id
            );
        } else {
            tracing::info!(
                "📥 [a2a_mailbox] Stored directive {} from {} to {}",
                envelope.id,
                envelope.source_agent_id,
                envelope.target_agent_id
            );
        }

        Ok(())
    }

    pub async fn fetch_mailbox(
        &self,
        agent_id: &str,
        status: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<MailboxEnvelope>, AppError> {
        let mut query_str = "SELECT id, mission_id, source_agent_id, target_agent_id, instruction, status, result, reasoning_trace, artifacts \
                             FROM agent_directives \
                             WHERE target_agent_id = ?".to_string();

        if status.is_some() {
            query_str.push_str(" AND status = ?");
        }

        // Stable deterministic pagination order
        query_str.push_str(" ORDER BY created_at DESC, id DESC");

        if limit.is_some() {
            query_str.push_str(" LIMIT ?");
        }
        if offset.is_some() {
            query_str.push_str(" OFFSET ?");
        }

        let mut q = sqlx::query_as::<_, MailboxRow>(sqlx::AssertSqlSafe(query_str)).bind(agent_id);

        if let Some(s) = status {
            q = q.bind(s.to_string());
        }
        if let Some(l) = limit {
            q = q.bind(l as i64);
        }
        if let Some(o) = offset {
            q = q.bind(o as i64);
        }

        let rows = q.fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|r| MailboxEnvelope {
                id: r.id,
                mission_id: r.mission_id,
                source_agent_id: r.source_agent_id,
                target_agent_id: r.target_agent_id,
                instruction: r.instruction,
                reasoning_trace: r.reasoning_trace,
                status: r.status,
                result: r.result,
                artifacts: r.artifacts.and_then(|s| {
                    if s.trim().is_empty() {
                        None
                    } else if s.starts_with('[') {
                        serde_json::from_str::<Vec<String>>(&s).ok()
                    } else {
                        // Legacy comma-delimited fallback
                        Some(
                            s.split(',')
                                .filter(|item| !item.is_empty())
                                .map(|item| item.to_string())
                                .collect(),
                        )
                    }
                }),
                timestamp: None,
                nonce: None,
            })
            .collect())
    }
}

#[derive(sqlx::FromRow)]
struct MailboxRow {
    id: String,
    mission_id: String,
    source_agent_id: String,
    target_agent_id: String,
    instruction: String,
    status: String,
    result: Option<String>,
    reasoning_trace: Option<String>,
    artifacts: Option<String>,
}
