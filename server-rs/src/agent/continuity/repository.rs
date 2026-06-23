//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[repository]` in tracing logs.

use crate::error::AppError;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use super::workflow::{Workflow, WorkflowStep};
use chacha20::ChaCha20;
use rand::RngExt;
use sha2::Digest;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use std::env;
use serde_json;

pub struct WorkflowRepository {
    pool: SqlitePool,
}

impl WorkflowRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Verifies if a workflow exists and belongs to the given tenant.
    pub async fn verify_workflow_owner(&self, workflow_id: &str, tenant_id: &str) -> Result<(), AppError> {
        let exists: Option<String> = sqlx::query_scalar("SELECT id FROM workflows WHERE id = ?1 AND tenant_id = ?2")
            .bind(workflow_id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

        if exists.is_none() {
            return Err(AppError::Forbidden("Workflow not found or access denied".to_string()));
        }
        Ok(())
    }

    /// Verifies if a workflow run exists and belongs to the given tenant.
    #[allow(dead_code)]
    pub async fn verify_run_owner(&self, run_id: &str, tenant_id: &str) -> Result<(), AppError> {
        let exists: Option<String> = sqlx::query_scalar("SELECT id FROM workflow_runs WHERE id = ?1 AND tenant_id = ?2")
            .bind(run_id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

        if exists.is_none() {
            return Err(AppError::Forbidden("Workflow run not found or access denied".to_string()));
        }
        Ok(())
    }

    pub async fn list_workflows(&self, tenant_id: &str) -> Result<Vec<Workflow>, AppError> {
        let rows = sqlx::query("SELECT * FROM workflows WHERE tenant_id = ?1 ORDER BY created_at DESC")
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        let mut workflows = Vec::new();
        for row in rows {
            workflows.push(Workflow {
                id: row.get::<String, _>("id"),
                name: row.get::<String, _>("name"),
                description: row.get::<Option<String>, _>("description"),
                enabled: row.get::<i64, _>("enabled") != 0,
                created_at: row.get::<chrono::DateTime<Utc>, _>("created_at"),
                updated_at: row.get::<chrono::DateTime<Utc>, _>("updated_at"),
                version: row.get::<i32, _>("version"),
            });
        }
        Ok(workflows)
    }

    pub async fn get_workflow(&self, id: &str, tenant_id: &str) -> Result<Workflow, AppError> {
        let row = sqlx::query("SELECT * FROM workflows WHERE id = ?1 AND tenant_id = ?2")
            .bind(id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Workflow not found".to_string()))?;

        Ok(Workflow {
            id: row.get::<String, _>("id"),
            name: row.get::<String, _>("name"),
            description: row.get::<Option<String>, _>("description"),
            enabled: row.get::<i64, _>("enabled") != 0,
            created_at: row.get::<chrono::DateTime<Utc>, _>("created_at"),
            updated_at: row.get::<chrono::DateTime<Utc>, _>("updated_at"),
            version: row.get::<i32, _>("version"),
        })
    }

    pub async fn create_workflow(
        &self,
        tenant_id: &str,
        name: String,
        description: Option<String>,
    ) -> Result<Workflow, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query("INSERT INTO workflows (id, name, description, enabled, created_at, updated_at, tenant_id, version) VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, 1)")
            .bind(&id)
            .bind(&name)
            .bind(&description)
            .bind(now)
            .bind(now)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        Ok(Workflow {
            id,
            name,
            description,
            enabled: true,
            created_at: now,
            updated_at: now,
            version: 1,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_step(
        &self,
        tenant_id: &str,
        workflow_id: &str,
        agent_id: &str,
        name: String,
        prompt_template: String,
        step_order: i32,
        max_retries: Option<i32>,
        backoff_factor_secs: Option<i64>,
        depends_on: Option<Vec<String>>,
    ) -> Result<WorkflowStep, AppError> {
        self.verify_workflow_owner(workflow_id, tenant_id).await?;

        let id = uuid::Uuid::new_v4().to_string();
        let max_r = max_retries.unwrap_or(3);
        let backoff = backoff_factor_secs.unwrap_or(2);
        let deps_vec = depends_on.unwrap_or_default();
        let deps_str = serde_json::to_string(&deps_vec).unwrap_or_else(|_| "[]".to_string());

        sqlx::query("INSERT INTO workflow_steps (id, workflow_id, agent_id, step_order, name, prompt_template, max_retries, backoff_factor_secs, depends_on) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")
            .bind(&id)
            .bind(workflow_id)
            .bind(agent_id)
            .bind(step_order)
            .bind(&name)
            .bind(&prompt_template)
            .bind(max_r)
            .bind(backoff)
            .bind(&deps_str)
            .execute(&self.pool)
            .await?;

        // Increment workflow version
        let now = Utc::now();
        sqlx::query("UPDATE workflows SET version = version + 1, updated_at = ?1 WHERE id = ?2")
            .bind(now)
            .bind(workflow_id)
            .execute(&self.pool)
            .await?;

        Ok(WorkflowStep {
            id,
            workflow_id: workflow_id.to_string(),
            agent_id: agent_id.to_string(),
            step_order,
            name,
            prompt_template,
            config: None,
            max_retries: max_r,
            backoff_factor_secs: backoff,
            depends_on: deps_vec,
        })
    }

    pub async fn delete_workflow(&self, id: &str, tenant_id: &str) -> Result<(), AppError> {
        self.verify_workflow_owner(id, tenant_id).await?;

        sqlx::query("DELETE FROM workflows WHERE id = ?1 AND tenant_id = ?2")
            .bind(id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_workflow_steps(&self, workflow_id: &str, tenant_id: &str) -> Result<Vec<WorkflowStep>, AppError> {
        self.verify_workflow_owner(workflow_id, tenant_id).await?;

        let rows = sqlx::query(
            "SELECT ws.* FROM workflow_steps ws \
             INNER JOIN workflows w ON ws.workflow_id = w.id \
             WHERE ws.workflow_id = ?1 AND w.tenant_id = ?2 \
             ORDER BY ws.step_order ASC, ws.id ASC",
        )
        .bind(workflow_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let mut steps = Vec::new();
        for row in rows {
            steps.push(WorkflowStep {
                id: row.get::<String, _>("id"),
                workflow_id: row.get::<String, _>("workflow_id"),
                agent_id: row.get::<String, _>("agent_id"),
                step_order: row.get::<i32, _>("step_order"),
                name: row.get::<String, _>("name"),
                prompt_template: row.get::<String, _>("prompt_template"),
                config: match row.get::<Option<String>, _>("config") {
                    Some(s) if !s.trim().is_empty() => Some(serde_json::from_str(&s).map_err(|e| {
                        AppError::InternalServerError(format!("Invalid workflow step config JSON: {}", e))
                    })?),
                    _ => None,
                },
                max_retries: row.get::<Option<i32>, _>("max_retries").unwrap_or(3),
                backoff_factor_secs: row.get::<Option<i64>, _>("backoff_factor_secs").unwrap_or(2),
                depends_on: match row.get::<Option<String>, _>("depends_on") {
                    Some(s) if !s.trim().is_empty() => serde_json::from_str(&s).map_err(|e| {
                        AppError::InternalServerError(format!("Invalid workflow step depends_on JSON: {}", e))
                    })?,
                    _ => vec![],
                },
            });
        }
        Ok(steps)
    }

    pub async fn create_workflow_run(
        &self,
        run_id: &str,
        workflow_id: &str,
        tenant_id: &str,
        context_str: String,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let encrypted = encrypt_context(&context_str)?;

        sqlx::query("INSERT INTO workflow_runs (id, workflow_id, tenant_id, started_at, status, current_step, context) VALUES (?1, ?2, ?3, ?4, 'running', 0, ?5)")
            .bind(run_id)
            .bind(workflow_id)
            .bind(tenant_id)
            .bind(now)
            .bind(encrypted)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_workflow_run_success(
        &self,
        run_id: &str,
        tenant_id: &str,
        context_str: String,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let encrypted = encrypt_context(&context_str)?;

        sqlx::query("UPDATE workflow_runs SET completed_at = ?1, status = 'completed', context = ?2 WHERE id = ?3 AND tenant_id = ?4")
            .bind(now)
            .bind(encrypted)
            .bind(run_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn update_workflow_run_failed(&self, run_id: &str, tenant_id: &str) -> Result<(), AppError> {
        let now = Utc::now();
        sqlx::query("UPDATE workflow_runs SET completed_at = ?1, status = 'failed' WHERE id = ?2 AND tenant_id = ?3")
            .bind(now)
            .bind(run_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_step_run(&self, step_run_id: &str, run_id: &str, step_id: &str) -> Result<(), AppError> {
        let now = Utc::now();
        sqlx::query("INSERT INTO workflow_step_runs (id, run_id, step_id, started_at, status) VALUES (?1, ?2, ?3, ?4, 'running')")
            .bind(step_run_id)
            .bind(run_id)
            .bind(step_id)
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_step_run_success(
        &self,
        step_run_id: &str,
        output_text: &str,
        metadata_str: String,
        cost_usd: f64,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        sqlx::query("UPDATE workflow_step_runs SET completed_at = ?1, status = 'completed', output_text = ?2, metadata = ?3, cost_usd = ?4 WHERE id = ?5")
            .bind(now)
            .bind(output_text)
            .bind(metadata_str)
            .bind(cost_usd)
            .bind(step_run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_step_run_failed(
        &self,
        step_run_id: &str,
        err_msg: &str,
        metadata_str: String,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        sqlx::query("UPDATE workflow_step_runs SET completed_at = ?1, status = 'failed', output_text = ?2, metadata = ?3 WHERE id = ?4")
            .bind(now)
            .bind(err_msg)
            .bind(metadata_str)
            .bind(step_run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn cleanup_step_runs_on_cancel(&self, run_id: &str, tenant_id: &str) -> Result<(), AppError> {
        let now = Utc::now();
        sqlx::query("UPDATE workflow_step_runs SET completed_at = ?1, status = 'failed', output_text = 'Cancelled due to execution termination' WHERE run_id = ?2 AND status = 'running' AND EXISTS (SELECT 1 FROM workflow_runs WHERE id = ?2 AND tenant_id = ?3)")
            .bind(now)
            .bind(run_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_step_run_cost(&self, step_run_id: &str) -> Result<f64, AppError> {
        let cost = sqlx::query_scalar::<_, f64>(
            "SELECT COALESCE(SUM(cost_usd), 0.0) FROM mission_history WHERE id LIKE ?1",
        )
        .bind(format!("{}%", step_run_id))
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0.0);
        Ok(cost)
    }

    pub async fn get_workflow_run_tenant_and_status(&self, run_id: &str) -> Result<Option<(String, String)>, AppError> {
        let row = sqlx::query("SELECT tenant_id, status FROM workflow_runs WHERE id = ?1")
            .bind(run_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            let tenant_id: String = r.get("tenant_id");
            let status: String = r.get("status");
            Ok(Some((tenant_id, status)))
        } else {
            Ok(None)
        }
    }

    pub async fn set_run_status_cancelling(&self, run_id: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE workflow_runs SET status = 'cancelling' WHERE id = ?1 AND status = 'running'")
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_workflow_runs(
        &self,
        workflow_id: &str,
        tenant_id: &str,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        self.verify_workflow_owner(workflow_id, tenant_id).await?;
        let rows = sqlx::query(
            "SELECT id, workflow_id, started_at, completed_at, status, current_step \
             FROM workflow_runs WHERE workflow_id = ?1 AND tenant_id = ?2 \
             ORDER BY started_at DESC LIMIT ?3",
        )
        .bind(workflow_id)
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let runs: Vec<serde_json::Value> = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.get::<String, _>("id"),
                    "workflow_id": r.get::<String, _>("workflow_id"),
                    "started_at": r.get::<String, _>("started_at"),
                    "completed_at": r.get::<Option<String>, _>("completed_at"),
                    "status": r.get::<String, _>("status"),
                    "current_step": r.get::<i32, _>("current_step"),
                })
            })
            .collect();

        Ok(runs)
    }

    /// Recovers interrupted workflow runs on server startup.
    /// Marks all 'running' workflow runs and step runs as 'failed' to prevent
    /// orphaned records from accumulating after crashes.
    pub async fn recover_interrupted_workflow_runs(pool: &SqlitePool) -> Result<(), AppError> {
        let now = Utc::now();
        let run_result = sqlx::query(
            "UPDATE workflow_runs SET completed_at = ?1, status = 'failed' WHERE status = 'running'"
        )
        .bind(now)
        .execute(pool)
        .await?;

        let step_result = sqlx::query(
            "UPDATE workflow_step_runs SET completed_at = ?1, status = 'failed', \
             output_text = 'Interrupted by server restart' WHERE status = 'running'"
        )
        .bind(now)
        .execute(pool)
        .await?;

        if run_result.rows_affected() > 0 || step_result.rows_affected() > 0 {
            tracing::warn!(
                "🔄 [Workflow] Recovered {} interrupted workflow runs and {} step runs",
                run_result.rows_affected(),
                step_result.rows_affected()
            );
        }
        Ok(())
    }
}

pub fn encrypt_context(plaintext: &str) -> Result<String, AppError> {
    use chacha20::cipher::KeyIvInit;
    use chacha20::cipher::StreamCipher;

    let key_str = env::var("WORKFLOW_ENCRYPTION_KEY").map_err(|_| {
        AppError::InternalServerError(
            "WORKFLOW_ENCRYPTION_KEY environment variable is not set".to_string(),
        )
    })?;

    let hashed = sha2::Sha512::digest(key_str.as_bytes());
    let (key_bytes, mac_key) = hashed.split_at(32);

    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(key_bytes);

    let mut nonce = [0u8; 12];
    rand::rng().fill(&mut nonce);

    let mut data = plaintext.as_bytes().to_vec();
    let mut cipher = ChaCha20::new(&key_arr.into(), &nonce.into());
    cipher.apply_keystream(&mut data);

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&data);

    let mut mac_hasher = sha2::Sha256::new();
    mac_hasher.update(mac_key);
    mac_hasher.update(&combined);
    let signature = mac_hasher.finalize();

    let mut final_payload = combined;
    final_payload.extend_from_slice(&signature);

    Ok(BASE64_STANDARD.encode(&final_payload))
}

#[allow(dead_code)]
pub fn decrypt_context(encoded: &str) -> Result<String, AppError> {
    use chacha20::cipher::KeyIvInit;
    use chacha20::cipher::StreamCipher;
    use subtle::ConstantTimeEq;

    let key_str = env::var("WORKFLOW_ENCRYPTION_KEY").map_err(|_| {
        AppError::InternalServerError(
            "WORKFLOW_ENCRYPTION_KEY environment variable is not set".to_string(),
        )
    })?;

    let hashed = sha2::Sha512::digest(key_str.as_bytes());
    let (key_bytes, mac_key) = hashed.split_at(32);

    let decoded = BASE64_STANDARD.decode(encoded)
        .map_err(|e| AppError::InternalServerError(format!("Base64 decode error: {}", e)))?;

    if decoded.len() < 44 {
        return Err(AppError::InternalServerError("Encrypted context too short".to_string()));
    }

    let (combined, signature) = decoded.split_at(decoded.len() - 32);

    let mut mac_hasher = sha2::Sha256::new();
    mac_hasher.update(mac_key);
    mac_hasher.update(combined);
    let expected_signature = mac_hasher.finalize();

    if signature.ct_eq(&expected_signature).unwrap_u8() != 1 {
        return Err(AppError::InternalServerError("Encrypted context integrity check failed (MAC mismatch)".to_string()));
    }

    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(key_bytes);

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&combined[..12]);
    let mut ciphertext = combined[12..].to_vec();

    let mut cipher = ChaCha20::new(&key_arr.into(), &nonce.into());
    cipher.apply_keystream(&mut ciphertext);

    String::from_utf8(ciphertext)
        .map_err(|e| AppError::InternalServerError(format!("Utf8 conversion error: {}", e)))
}

// Metadata: [repository]
