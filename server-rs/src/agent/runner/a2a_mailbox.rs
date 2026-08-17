//! @docs ARCHITECTURE:Registry
//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **A2A Mailbox Implementation**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[a2a_mailbox]` in tracing logs.

use crate::error::AppError;
use sqlx::SqlitePool;

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
}

pub struct A2AMailbox {
    pool: SqlitePool,
}

impl A2AMailbox {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn send_envelope(&self, envelope: &MailboxEnvelope) -> Result<(), AppError> {
        if envelope.target_agent_id.starts_with("http://")
            || envelope.target_agent_id.starts_with("https://")
        {
            // Remote dispatch (A2A RemoteA2aAgent Client Proxy pattern)
            let client = reqwest::Client::new();

            // Serialize payload to verify signature
            let payload_str = serde_json::to_string(envelope)
                .map_err(|e| AppError::BadRequest(format!("Failed to serialize payload: {}", e)))?;

            // Generate A2A signature
            let signature =
                crate::agent::runner::tools::capability::sign_a2a_envelope(&payload_str);

            // Post to the remote inbox gateway
            let url = if envelope.target_agent_id.ends_with("/send") {
                envelope.target_agent_id.clone()
            } else {
                format!(
                    "{}/v1/engine/a2a/mailbox/send",
                    envelope.target_agent_id.trim_end_matches('/')
                )
            };

            let response = client
                .post(&url)
                .header("X-A2A-Signature", signature)
                .json(envelope)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(AppError::Forbidden(format!(
                    "Remote A2A delivery failed: {} - {}",
                    status, text
                )));
            }
            return Ok(());
        }

        let artifacts_str = envelope.artifacts.as_ref().map(|v| v.join(","));
        sqlx::query::<sqlx::Sqlite>(
            "INSERT OR REPLACE INTO agent_directives (id, mission_id, source_agent_id, target_agent_id, instruction, status, result, reasoning_trace, artifacts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(&envelope.id)
        .bind(&envelope.mission_id)
        .bind(&envelope.source_agent_id)
        .bind(&envelope.target_agent_id)
        .bind(&envelope.instruction)
        .bind(&envelope.status)
        .bind(&envelope.result)
        .bind(&envelope.reasoning_trace)
        .bind(artifacts_str)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)]
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

        // Order by created_at DESC or id to ensure stable paging
        query_str.push_str(" ORDER BY id DESC");

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
                artifacts: r.artifacts.map(|s| {
                    if s.is_empty() {
                        vec![]
                    } else {
                        s.split(',').map(|item| item.to_string()).collect()
                    }
                }),
            })
            .collect())
    }
}

#[allow(dead_code)]
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

// Metadata: [a2a_mailbox]
