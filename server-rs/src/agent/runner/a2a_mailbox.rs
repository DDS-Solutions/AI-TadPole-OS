//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / a2a_mailbox
//!
//! ### AI Assist Note
//! - Keep remote endpoint validation, signature verification, nonce persistence, and mailbox delivery aligned.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::error::AppError;
use sqlx::SqlitePool;
use std::time::Duration;

pub const MAX_ENVELOPE_ID_LEN: usize = 128;
pub const MAX_AGENT_ID_LEN: usize = 128;
pub const MAX_TARGET_ID_LEN: usize = 512;
pub const MAX_ENVELOPE_BYTES: usize = 512 * 1024 - 1024;
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_agent_id: Option<String>,
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
}

impl A2AMailbox {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub(crate) fn validate_remote_endpoint(target: &str) -> Result<(), AppError> {
        if !target
            .get(..8)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
        {
            return Err(AppError::Forbidden(
                "Remote A2A delivery requires an absolute HTTPS endpoint".to_string(),
            ));
        }

        let url = reqwest::Url::parse(target)
            .map_err(|error| AppError::BadRequest(format!("Invalid remote endpoint: {}", error)))?;
        if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
            return Err(AppError::Forbidden(
                "Remote A2A endpoints cannot contain credentials or fragments".to_string(),
            ));
        }
        if url.host_str().is_none() {
            return Err(AppError::BadRequest(
                "Remote A2A endpoint must include a host".to_string(),
            ));
        }

        Ok(())
    }

    pub(crate) async fn persist_local_envelope_in_transaction(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        envelope: &MailboxEnvelope,
    ) -> Result<bool, AppError> {
        Self::validate_envelope_bounds(envelope)?;
        if is_remote_target(&envelope.target_agent_id) || envelope.recipient_agent_id.is_some() {
            return Err(AppError::BadRequest(
                "A local mailbox transaction requires a local target_agent_id".to_string(),
            ));
        }

        let artifacts_json = match envelope.artifacts.as_ref() {
            Some(artifacts) => Some(serde_json::to_string(artifacts).map_err(|e| {
                AppError::BadRequest(format!("Failed to serialize A2A artifacts: {}", e))
            })?),
            None => None,
        };
        let status = if envelope.status.trim().is_empty() {
            "pending"
        } else {
            envelope.status.as_str()
        };
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
        .bind(status)
        .bind(&envelope.result)
        .bind(&envelope.reasoning_trace)
        .bind(artifacts_json)
        .execute(&mut **tx)
        .await?;

        let queued = res.rows_affected() > 0;
        if !queued {
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

        Ok(queued)
    }

    pub(crate) async fn persist_received_envelope(
        pool: &SqlitePool,
        envelope: &MailboxEnvelope,
        nonce: &str,
        timestamp: i64,
    ) -> Result<bool, AppError> {
        let mut tx = pool.begin().await?;
        let prune_threshold = chrono::Utc::now()
            .timestamp_millis()
            .max(0)
            .saturating_sub(600_000);
        sqlx::query("DELETE FROM used_nonces WHERE timestamp < ?")
            .bind(prune_threshold)
            .execute(&mut *tx)
            .await?;

        if let Err(error) = sqlx::query("INSERT INTO used_nonces (nonce, timestamp) VALUES (?, ?)")
            .bind(nonce)
            .bind(timestamp)
            .execute(&mut *tx)
            .await
        {
            let is_unique_violation = if let Some(db_err) = error.as_database_error() {
                db_err
                    .code()
                    .map(|code| code == "2067" || code == "1555" || code == "23000")
                    .unwrap_or(false)
            } else {
                false
            } || error.to_string().contains("UNIQUE constraint failed");

            if is_unique_violation {
                return Err(AppError::Forbidden(
                    "Replay attack detected: nonce already used".to_string(),
                ));
            }
            return Err(AppError::Sqlx(error));
        }

        let queued = Self::persist_local_envelope_in_transaction(&mut tx, envelope).await?;
        tx.commit().await?;
        Ok(queued)
    }

    pub(crate) async fn record_dispatch_audit(
        envelope: &MailboxEnvelope,
        is_remote: bool,
        audit_trail: Option<&crate::security::audit::MerkleAuditTrail>,
    ) {
        if let Some(audit) = audit_trail {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(envelope.instruction.as_bytes());
            let instruction_digest = hex::encode(hasher.finalize());

            let audit_params = serde_json::json!({
                "envelope_id": &envelope.id,
                "target_agent_id": &envelope.target_agent_id,
                "recipient_agent_id": &envelope.recipient_agent_id,
                "instruction_digest": instruction_digest,
                "artifacts_count": envelope.artifacts.as_ref().map(|a| a.len()).unwrap_or(0),
                "is_remote": is_remote,
            })
            .to_string();

            if let Err(error) = audit
                .record(
                    &envelope.source_agent_id,
                    Some(&envelope.mission_id),
                    None,
                    "[A2A_DISPATCH]",
                    &audit_params,
                )
                .await
            {
                tracing::error!(
                    "🚨 Failed to record [A2A_DISPATCH] in audit trail: {:?}",
                    error
                );
            }
        }
    }

    /// Builds the remote mailbox endpoint while preserving any configured base path and query.
    fn remote_send_url(url_str: &str) -> Result<reqwest::Url, AppError> {
        let mut url = reqwest::Url::parse(url_str).map_err(|e| {
            AppError::BadRequest(format!("Invalid remote agent endpoint URL: {}", e))
        })?;

        if !url.path().trim_end_matches('/').ends_with("/send") {
            let mut segments = url.path_segments_mut().map_err(|_| {
                AppError::BadRequest("Remote agent endpoint URL cannot be a base URL".to_string())
            })?;
            segments.pop_if_empty();
            segments.extend(["v1", "engine", "a2a", "mailbox", "send"]);
        }

        Ok(url)
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
        if envelope.mission_id.trim().is_empty() || envelope.mission_id.len() > MAX_ENVELOPE_ID_LEN
        {
            return Err(AppError::BadRequest(format!(
                "Mission ID must be non-empty and at most {} characters",
                MAX_ENVELOPE_ID_LEN
            )));
        }
        if let Some(recipient_agent_id) = envelope.recipient_agent_id.as_deref() {
            if recipient_agent_id.trim().is_empty() || recipient_agent_id.len() > MAX_AGENT_ID_LEN {
                return Err(AppError::BadRequest(format!(
                    "Remote recipient agent ID must be non-empty and at most {} characters",
                    MAX_AGENT_ID_LEN
                )));
            }
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
        let serialized_len = serde_json::to_vec(envelope)
            .map_err(|e| AppError::BadRequest(format!("Invalid A2A envelope: {}", e)))?
            .len();
        if serialized_len > MAX_ENVELOPE_BYTES {
            return Err(AppError::BadRequest(format!(
                "Serialized A2A envelope exceeds maximum allowed size of {} bytes",
                MAX_ENVELOPE_BYTES
            )));
        }
        Ok(())
    }

    pub async fn send_envelope(
        &self,
        envelope: &MailboxEnvelope,
        audit_trail: Option<&crate::security::audit::MerkleAuditTrail>,
    ) -> Result<(), AppError> {
        // Enforce chokepoint payload validation
        Self::validate_envelope_bounds(envelope)?;

        let is_remote = is_remote_target(&envelope.target_agent_id);

        if is_remote {
            if envelope.recipient_agent_id.is_none() {
                return Err(AppError::BadRequest(
                    "Remote A2A delivery requires recipient_agent_id".to_string(),
                ));
            }
            Self::validate_remote_endpoint(&envelope.target_agent_id)?;
            let mut remote_envelope = envelope.clone();
            let ts = chrono::Utc::now().timestamp_millis().max(0);
            let nonce = uuid::Uuid::new_v4().to_string();

            remote_envelope.timestamp = Some(ts);
            remote_envelope.nonce = Some(nonce.clone());
            remote_envelope.status = "pending".to_string();
            Self::validate_envelope_bounds(&remote_envelope)?;

            // Sign the serialized envelope so recipient, artifacts, trace, and other payload
            // fields cannot be changed independently of the authenticated instruction.
            let payload_str = serde_json::to_string(&remote_envelope)
                .map_err(|e| AppError::BadRequest(format!("Failed to serialize payload: {}", e)))?;
            let signature =
                crate::agent::runner::tools::capability::sign_a2a_envelope(&payload_str);

            // Validate the final endpoint after route construction. The shared guard resolves all
            // DNS answers and rejects private, local, link-local, and reserved destinations.
            let endpoint = Self::remote_send_url(&remote_envelope.target_agent_id)?;
            if endpoint.scheme() != "https" {
                return Err(AppError::Forbidden(
                    "Remote A2A delivery requires an HTTPS endpoint".to_string(),
                ));
            }
            let validated =
                crate::security::ssrf_guard::validate_public_http_url(endpoint.as_str())
                    .await
                    .map_err(|error| match error {
                        AppError::Forbidden(message) => {
                            AppError::Forbidden(format!("SSRF Security Gate: {}", message))
                        }
                        other => other,
                    })?;

            // Pin the connection to the address checked above and reject redirects, which could
            // otherwise move a public endpoint request into a private network.
            let http_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(4))
                .redirect(reqwest::redirect::Policy::none())
                .resolve_to_addrs(
                    &validated.host,
                    &[std::net::SocketAddr::new(validated.ip, validated.port)],
                )
                .build()?;

            tracing::info!(
                "📨 [a2a_mailbox] Dispatching remote envelope {} to host {}",
                remote_envelope.id,
                validated.host
            );

            let mut response = http_client
                .post(validated.url)
                .header("X-A2A-Signature", signature)
                .header("Content-Type", "application/json")
                .body(payload_str)
                .send()
                .await?;

            if response.status() != reqwest::StatusCode::ACCEPTED {
                let status = response.status();
                const MAX_ERROR_BODY_BYTES: usize = 4096;
                let mut body = Vec::with_capacity(MAX_ERROR_BODY_BYTES);
                while body.len() < MAX_ERROR_BODY_BYTES {
                    match response.chunk().await {
                        Ok(Some(chunk)) => {
                            let remaining = MAX_ERROR_BODY_BYTES - body.len();
                            body.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
                        }
                        Ok(None) | Err(_) => break,
                    }
                }
                let text = String::from_utf8_lossy(&body);
                return Err(AppError::InternalServerError(format!(
                    "Remote A2A delivery failed: HTTP {} - {}",
                    status, text
                )));
            }

            const MAX_CONFIRMATION_BYTES: usize = 16 * 1024;
            let mut confirmation_body = Vec::with_capacity(MAX_CONFIRMATION_BYTES);
            while let Some(chunk) = response.chunk().await? {
                if confirmation_body.len() + chunk.len() > MAX_CONFIRMATION_BYTES {
                    return Err(AppError::InternalServerError(
                        "Remote A2A confirmation exceeded the maximum response size".to_string(),
                    ));
                }
                confirmation_body.extend_from_slice(&chunk);
            }
            let confirmation: serde_json::Value = serde_json::from_slice(&confirmation_body)
                .map_err(|error| {
                    AppError::InternalServerError(format!(
                        "Remote A2A endpoint returned an invalid confirmation: {}",
                        error
                    ))
                })?;
            let confirmation_status = confirmation
                .get("status")
                .and_then(serde_json::Value::as_str);
            if !matches!(confirmation_status, Some("queued" | "already_processed"))
                || confirmation
                    .get("envelope_id")
                    .and_then(serde_json::Value::as_str)
                    != Some(remote_envelope.id.as_str())
            {
                return Err(AppError::InternalServerError(
                    "Remote A2A endpoint did not confirm this envelope was queued".to_string(),
                ));
            }
        } else {
            let mut tx = self.pool.begin().await?;
            Self::persist_local_envelope_in_transaction(&mut tx, envelope).await?;
            tx.commit().await?;
        }

        // Chain into Merkle Audit Trail
        Self::record_dispatch_audit(envelope, is_remote, audit_trail).await;

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
                recipient_agent_id: None,
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

pub(crate) fn is_remote_target(target_agent_id: &str) -> bool {
    target_agent_id
        .trim()
        .split_once(':')
        .is_some_and(|(scheme, _)| {
            scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
        })
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
