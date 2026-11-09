//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Atomic decision engine for business enquiry runs with two-phase connector outbox persistence.
//! ### 🔍 Debugging & Observability
//! Traces record decision latencies and ticket dispatch status.

use super::connector::ConnectorFailure;
use super::pii::detect_sensitive_pii;
use super::service::{encode, fact_sheet, fresh, source_evidence, BusinessService};
use super::sops;
use super::types::*;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};
use uuid::Uuid;

const CONNECTOR_TIMEOUT: Duration = Duration::from_secs(5);

fn sanitize_operator_note(note: Option<&str>, env: &str) -> Result<Option<String>, AppError> {
    let Some(trimmed) = note.map(str::trim) else {
        return Ok(None);
    };
    if trimmed.is_empty() {
        return Ok(None);
    }
    let valid_chars = trimmed.len() <= 1000
        && !trimmed
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t');
    if !valid_chars {
        return Err(AppError::BadRequest(
            "Operator note must be at most 1000 bytes without control characters".into(),
        ));
    }
    if env == "mock" {
        if let Some(pii_kind) = detect_sensitive_pii(trimmed) {
            return Err(AppError::BadRequest(format!(
                "Sensitive PII ({pii_kind}) is forbidden in mock practice operator notes (FR-11)."
            )));
        }
    }
    Ok(Some(trimmed.to_string()))
}

fn sanitize_edited_response(resp: Option<&str>, env: &str) -> Result<Option<String>, AppError> {
    let Some(trimmed) = resp.map(str::trim) else {
        return Ok(None);
    };
    let valid = !trimmed.is_empty()
        && trimmed.len() <= 4000
        && !trimmed
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t');
    if !valid {
        return Err(AppError::BadRequest(
            "Edited response must be a nonempty string of at most 4000 bytes".into(),
        ));
    }
    if env == "mock" {
        if let Some(pii_kind) = detect_sensitive_pii(trimmed) {
            return Err(AppError::BadRequest(format!(
                "Sensitive PII ({pii_kind}) is forbidden in mock practice responses (FR-11)."
            )));
        }
    }
    Ok(Some(trimmed.to_string()))
}

fn rebase_in_flight_offering(
    run: &mut Run,
    company: &mut Company,
    current: &Company,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    if !fresh(&company.offering, now)
        && fresh(&current.offering, now)
        && current.offering.display_reference == company.offering.display_reference
    {
        tracing::info!(target: "business", company_key = run.company_key, run_id = run.run_id, "[Business] In-flight run rebased to renewed offering freshness window");
        company.offering.source_updated_at = current.offering.source_updated_at.clone();
        company.offering.fresh_until = current.offering.fresh_until.clone();
        run.evidence = source_evidence(company)?;
    }
    Ok(())
}

impl BusinessService {
    async fn record_outbox_ticket_dispatch(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        company: &Company,
        run: &mut Run,
        outbox_id: &str,
        now_str: &str,
    ) -> Result<(), AppError> {
        let result = tokio::time::timeout(
            CONNECTOR_TIMEOUT,
            self.connector.record_ticket(tx, company, run),
        )
        .await
        .unwrap_or(Err(ConnectorFailure::Timeout));

        match result {
            Ok((ticket, receipt)) => {
                let receipt_payload = encode(&receipt)?;
                sqlx::query(
                    "UPDATE business_outbox SET status=?,payload=?,updated_at=? WHERE id=?",
                )
                .bind("committed")
                .bind(receipt_payload)
                .bind(now_str)
                .bind(outbox_id)
                .execute(&mut **tx)
                .await?;
                run.ticket = Some(ticket);
                run.receipt = Some(receipt);
                run.status = RunStatus::Completed;
            }
            Err(error) => {
                sqlx::query("ROLLBACK TO SAVEPOINT business_ticket_write")
                    .execute(&mut **tx)
                    .await?;
                sqlx::query(
                    "UPDATE business_outbox SET status=?,payload=?,updated_at=? WHERE id=?",
                )
                .bind("rolled_back")
                .bind(error.message())
                .bind(now_str)
                .bind(outbox_id)
                .execute(&mut **tx)
                .await?;
                run.status = RunStatus::Failed;
                run.blocked_reason = Some(error.message().into());
            }
        }
        Ok(())
    }

    async fn handle_approval_action(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        company: &Company,
        run: &mut Run,
        edited_response: Option<String>,
        now: DateTime<Utc>,
    ) -> Result<bool, AppError> {
        if !fresh(&company.offering, now) {
            run.status = RunStatus::Blocked;
            run.blocked_reason = Some(
                "Source freshness expired while approval was pending; no ticket was written."
                    .into(),
            );
            return Ok(false);
        }
        if run.connector_mode == "read_only" {
            run.status = RunStatus::Blocked;
            run.blocked_reason = Some("Stage 2 read_only shadow mode active: external ticket writes are blocked until Stage 3 approved_write promotion.".into());
            return Ok(false);
        }
        if run.draft_response != fact_sheet(company) || run.evidence != source_evidence(company)? {
            return Err(AppError::Conflict(
                "Captured draft and source evidence no longer agree".into(),
            ));
        }

        let mut operator_edited = false;
        if let Some(custom_draft) = edited_response {
            if !sops::verify_grounded_citations(&custom_draft, company) {
                return Err(AppError::BadRequest("Edited response must retain grounded citations to the company offering and verified facts (FR-12).".into()));
            }
            if custom_draft != run.draft_response {
                run.draft_response = custom_draft;
                operator_edited = true;
            }
        }

        let outbox_id = Uuid::new_v4().to_string();
        let now_str = now.to_rfc3339();
        sqlx::query("INSERT INTO business_outbox(id,run_id,company_key,connector_mode,status,payload,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?)")
            .bind(&outbox_id).bind(&run.run_id).bind(&run.company_key).bind(&run.connector_mode).bind("pending_dispatch").bind(&run.draft_response).bind(&now_str).bind(&now_str).execute(&mut **tx).await?;
        sqlx::query("SAVEPOINT business_ticket_write")
            .execute(&mut **tx)
            .await?;

        self.record_outbox_ticket_dispatch(tx, company, run, &outbox_id, &now_str)
            .await?;
        sqlx::query("RELEASE SAVEPOINT business_ticket_write")
            .execute(&mut **tx)
            .await?;
        Ok(operator_edited)
    }

    pub async fn decide_run(
        &self,
        key: &str,
        id: &str,
        decision: DecideRun,
    ) -> Result<Run, AppError> {
        let started = Instant::now();
        let current_company = self.company(key)?;
        Self::valid_run_id(id)?;
        if decision.expected_version < 1 {
            return Err(AppError::BadRequest(
                "A positive expected_version is required".into(),
            ));
        }

        let mut tx = self
            .pool(&current_company)
            .await?
            .begin_with("BEGIN IMMEDIATE")
            .await?;
        let (mut run, mut company) = Self::load_run(&mut tx, key, id).await?;

        if run.version != decision.expected_version || run.status != RunStatus::PendingApproval {
            return Err(AppError::Conflict(
                "Run changed or is no longer awaiting approval; reload before deciding".into(),
            ));
        }
        if !matches!(run.connector_mode.as_str(), "mock" | "read_only")
            || self.connector.mode() != run.connector_mode
        {
            return Err(AppError::NotImplemented(
                "The captured connector is unavailable; live writes remain disabled".into(),
            ));
        }
        if !sops::matches(&run, &company.sop)? {
            return Err(AppError::Conflict(
                "The captured SOP identity or version no longer agrees with this run".into(),
            ));
        }

        let now = (self.clock)();
        let operator_note =
            sanitize_operator_note(decision.operator_note.as_deref(), &company.environment)?;
        let edited_response =
            sanitize_edited_response(decision.edited_response.as_deref(), &company.environment)?;
        rebase_in_flight_offering(&mut run, &mut company, &current_company, now)?;

        run.version += 1;
        run.updated_at = now.to_rfc3339();

        if decision.decision == Decision::Reject && decision.edited_response.is_some() {
            return Err(AppError::BadRequest(
                "An edited response cannot be provided when rejecting a run".into(),
            ));
        }

        let operator_edited = match decision.decision {
            Decision::Reject => {
                run.status = RunStatus::Rejected;
                false
            }
            Decision::Approve => {
                self.handle_approval_action(&mut tx, &company, &mut run, edited_response, now)
                    .await?
            }
        };

        run.approval = Some(Approval {
            decision: decision.decision,
            decided_at: now.to_rfc3339(),
            operator_note,
            operator_edited,
        });
        sqlx::query("INSERT INTO business_decisions(run_id,company_key,payload) VALUES(?,?,?)")
            .bind(id)
            .bind(key)
            .bind(encode(&run.approval)?)
            .execute(&mut *tx)
            .await?;

        run.latency_ms = run
            .latency_ms
            .saturating_add(started.elapsed().as_millis().min(i64::MAX as u128) as i64);
        let update = sqlx::query("UPDATE business_runs SET company_snapshot=?,run_json=?,version=?,updated_at=? WHERE id=? AND company_key=? AND version=?")
            .bind(encode(&company)?).bind(encode(&run)?).bind(run.version).bind(&run.updated_at).bind(id).bind(key).bind(decision.expected_version).execute(&mut *tx).await?;

        if update.rows_affected() != 1 {
            return Err(AppError::Conflict(
                "Business run changed during decision".into(),
            ));
        }
        tx.commit().await?;
        tracing::info!(target: "business", company_key = key, workspace_id = %run.workspace_id, run_id = id, status = ?run.status, "[Business] Enquiry decision persisted");
        Ok(run)
    }
}
