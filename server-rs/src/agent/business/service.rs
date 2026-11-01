//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Business contexts resolve from server-owned profiles into separate SQLite stores.
//! Runs retain their profile snapshot; switching the operator view cannot retarget work.
//! ### 🔍 Debugging & Observability
//! [Business] creation and decision traces contain company_key, workspace_id and run_id.

use super::connector::{BusinessConnector, ConnectorFailure, MockConnector};
use super::sops;
use super::types::*;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqliteConnection, SqlitePool,
};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::OnceCell;
use uuid::Uuid;

const FIXTURES: [(&str, &str); 2] = [
    (
        "Mock-1-Product",
        include_str!("../../../../starter_kits/mock_smb/companies/Mock-1-Product.json"),
    ),
    (
        "Mock-2-Service",
        include_str!("../../../../starter_kits/mock_smb/companies/Mock-2-Service.json"),
    ),
];
const CONNECTOR_TIMEOUT: Duration = Duration::from_secs(5);
type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

#[derive(Clone)]
pub struct BusinessService {
    root: PathBuf,
    pools: Arc<[OnceCell<SqlitePool>; 2]>,
    connector: Arc<dyn BusinessConnector>,
    clock: Clock,
}

fn encode<T: Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value)
        .map_err(|_| AppError::InternalServerError("Business evidence serialization failed".into()))
}
fn decode<T: DeserializeOwned>(value: &str) -> Result<T, AppError> {
    serde_json::from_str(value)
        .map_err(|_| AppError::InternalServerError("Stored business evidence is invalid".into()))
}
fn timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}
fn fresh(offering: &Offering, now: DateTime<Utc>) -> bool {
    match (
        timestamp(&offering.source_updated_at),
        timestamp(&offering.fresh_until),
    ) {
        (Some(updated), Some(until)) => updated <= now && now < until,
        _ => false,
    }
}

pub(super) fn fact_sheet(company: &Company) -> String {
    let offering = &company.offering;
    format!(
        "Company fact sheet for review\n{}\n{}\nListed base price: {} {}.{:02}\nHours: {}\nPolicy: {}\nSupport: {}\nConfirm these facts address the customer enquiry before approving this response.",
        offering.title, offering.description, offering.currency, offering.price_minor / 100,
        offering.price_minor % 100, company.profile.hours, company.profile.policy, company.profile.support_email,
    )
}

fn source_evidence(company: &Company) -> Result<Vec<Evidence>, AppError> {
    Ok(vec![Evidence {
        source_id: company.offering.source_id.clone(),
        source_updated_at: company.offering.source_updated_at.clone(),
        fresh_until: company.offering.fresh_until.clone(),
        content: encode(company)?,
    }])
}

impl BusinessService {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            root: base_dir.join("data").join("business").join("workspaces"),
            pools: Arc::new([OnceCell::new(), OnceCell::new()]),
            connector: Arc::new(MockConnector),
            clock: Arc::new(Utc::now),
        }
    }

    #[cfg(test)]
    pub(super) fn with_dependencies(
        base_dir: PathBuf,
        clock: Clock,
        connector: Arc<dyn BusinessConnector>,
    ) -> Self {
        Self {
            clock,
            connector,
            ..Self::new(base_dir)
        }
    }

    pub fn companies(&self) -> Result<Vec<Company>, AppError> {
        FIXTURES.iter().map(|(key, _)| self.company(key)).collect()
    }

    pub fn company(&self, key: &str) -> Result<Company, AppError> {
        let (_, fixture) = FIXTURES
            .iter()
            .find(|(allowed, _)| *allowed == key)
            .ok_or_else(|| AppError::NotFound("Business company not found".into()))?;
        let mut company: Company = decode(fixture)?;
        sops::attach_document(&mut company)?;
        let prefix = format!("{key}-");
        let names = [
            &company.agent_display_name,
            &company.workflow_display_name,
            &company.offering.display_reference,
            &company.offering.title,
            &company.offering.source_id,
            &company.customer.display_reference,
            &company.customer.name,
        ];
        if company.company_key != key
            || company.display_name != key
            || company.environment != "mock"
            || company.connector_mode != "mock"
            || company.workflow_definition_id != WORKFLOW_DEFINITION_ID
            || company.workflow_version != WORKFLOW_VERSION
            || company.offering.price_minor < 0
            || company.offering.currency != "USD"
            || names.iter().any(|name| !name.starts_with(&prefix))
            || Uuid::parse_str(&company.workspace_id).is_err()
            || Uuid::parse_str(&company.offering.id).is_err()
            || Uuid::parse_str(&company.customer.id).is_err()
            || timestamp(&company.offering.source_updated_at).is_none()
            || timestamp(&company.offering.fresh_until).is_none()
        {
            return Err(AppError::InternalServerError(
                "Business fixture configuration is invalid".into(),
            ));
        }
        Ok(company)
    }

    async fn pool(&self, company: &Company) -> Result<&SqlitePool, AppError> {
        let index = FIXTURES
            .iter()
            .position(|(key, _)| *key == company.company_key)
            .ok_or_else(|| AppError::NotFound("Business company not found".into()))?;
        self.pools[index].get_or_try_init(|| async {
            tokio::fs::create_dir_all(&self.root).await?;
            // Both path components originate from the validated server fixture, never caller input.
            let filename = format!("{}-{}.sqlite3", company.company_key, company.workspace_id);
            let options = SqliteConnectOptions::new().filename(self.root.join(filename))
                .create_if_missing(true).foreign_keys(true).journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(Duration::from_secs(5));
            let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await?;
            let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
            let version: i64 = sqlx::query_scalar("PRAGMA user_version").fetch_one(&mut *tx).await?;
            if version > 1 { return Err(AppError::DegradedState("Business workspace schema is newer than this engine".into())); }
            for statement in [
                "CREATE TABLE IF NOT EXISTS business_runs (id TEXT PRIMARY KEY, company_key TEXT NOT NULL, idempotency_key TEXT NOT NULL UNIQUE, request_json TEXT NOT NULL, company_snapshot TEXT NOT NULL, run_json TEXT NOT NULL, version INTEGER NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
                "CREATE TABLE IF NOT EXISTS business_tickets (id TEXT PRIMARY KEY, run_id TEXT NOT NULL UNIQUE REFERENCES business_runs(id), company_key TEXT NOT NULL, payload TEXT NOT NULL)",
                "CREATE TABLE IF NOT EXISTS business_receipts (id TEXT PRIMARY KEY, run_id TEXT NOT NULL UNIQUE REFERENCES business_runs(id), company_key TEXT NOT NULL, payload TEXT NOT NULL)",
                "CREATE TABLE IF NOT EXISTS business_decisions (run_id TEXT PRIMARY KEY REFERENCES business_runs(id), company_key TEXT NOT NULL, payload TEXT NOT NULL)",
                "PRAGMA user_version = 1",
            ] { sqlx::query(statement).execute(&mut *tx).await?; }
            tx.commit().await?;
            Ok::<_, AppError>(pool)
        }).await
    }

    fn validate_request(&self, company: &Company, request: &mut CreateRun) -> Result<(), AppError> {
        request.idempotency_key = request.idempotency_key.trim().into();
        request.customer_reference = request.customer_reference.trim().into();
        request.offering_reference = request.offering_reference.trim().into();
        request.enquiry = request.enquiry.trim().into();
        if request.idempotency_key.is_empty()
            || request.idempotency_key.len() > 128
            || !request
                .idempotency_key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_: .".contains(&byte))
            || request.enquiry.is_empty()
            || request.enquiry.len() > 4000
            || request
                .enquiry
                .chars()
                .any(|character| character.is_control() && character != '\n' && character != '\t')
        {
            return Err(AppError::BadRequest(
                "Provide a bounded idempotency key and a nonempty enquiry of at most 4000 bytes"
                    .into(),
            ));
        }
        if request.customer_reference != company.customer.display_reference {
            return Err(AppError::BadRequest(
                "Customer reference does not belong to this company".into(),
            ));
        }
        if request.offering_reference.len() > 200
            || !request
                .offering_reference
                .starts_with(&format!("{}-", company.company_key))
        {
            return Err(AppError::BadRequest(
                "Offering reference does not belong to this company".into(),
            ));
        }
        if self.connector.mode() != company.connector_mode || self.connector.mode() != "mock" {
            return Err(AppError::NotImplemented(
                "Only the registered mock connector is available; live access is disabled".into(),
            ));
        }
        Ok(())
    }

    pub async fn create_run(
        &self,
        key: &str,
        mut request: CreateRun,
    ) -> Result<(Run, bool), AppError> {
        let started = Instant::now();
        let mut company = self.company(key)?;
        self.validate_request(&company, &mut request)?;
        let pool = self.pool(&company).await?;
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
        let request_json = encode(&request)?;
        let existing: Option<(String, String)> = sqlx::query_as("SELECT request_json,run_json FROM business_runs WHERE company_key=? AND idempotency_key=?")
            .bind(key).bind(&request.idempotency_key).fetch_optional(&mut *tx).await?;
        if let Some((prior_request, run_json)) = existing {
            if prior_request != request_json {
                return Err(AppError::Conflict(
                    "Idempotency key was already used with different enquiry data".into(),
                ));
            }
            let run = decode(&run_json)?;
            tx.commit().await?;
            return Ok((run, false));
        }
        let now = (self.clock)();
        let run_id = Uuid::new_v4().to_string();
        let mut run = Run {
            display_reference: format!("{key}-Run-{run_id}"),
            run_id,
            company_key: key.into(),
            workspace_id: company.workspace_id.clone(),
            environment: company.environment.clone(),
            configuration_version: company.configuration_version.clone(),
            fixture_version: company.fixture_version.clone(),
            workflow_definition_id: company.workflow_definition_id.clone(),
            workflow_version: company.workflow_version.clone(),
            sop_id: company.sop.id.clone(),
            sop_version: company.sop.version.clone(),
            sop_digest: sops::digest(&company.sop)?,
            connector_mode: company.connector_mode.clone(),
            version: 1,
            status: RunStatus::PendingApproval,
            customer_reference: request.customer_reference.clone(),
            offering_reference: request.offering_reference.clone(),
            enquiry: request.enquiry.clone(),
            draft_response: String::new(),
            evidence: Vec::new(),
            approval: None,
            ticket: None,
            receipt: None,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            blocked_reason: None,
            latency_ms: 0,
        };
        let retrieval = tokio::time::timeout(
            CONNECTOR_TIMEOUT,
            self.connector
                .retrieve_offering(&company, &request.offering_reference),
        )
        .await
        .unwrap_or(Err(ConnectorFailure::Timeout));
        match retrieval {
            Ok(offering)
                if offering.display_reference == request.offering_reference
                    && offering.source_id.starts_with(&format!("{key}-")) =>
            {
                company.offering = offering;
                run.evidence = source_evidence(&company)?;
                if fresh(&company.offering, now) {
                    run.draft_response = fact_sheet(&company);
                } else {
                    run.status = RunStatus::Blocked;
                    run.blocked_reason = Some(
                        "Source facts are stale, future-dated, or have invalid freshness bounds."
                            .into(),
                    );
                }
            }
            Ok(_) => {
                run.status = RunStatus::Blocked;
                run.blocked_reason =
                    Some("Connector facts do not match this company and offering.".into());
            }
            Err(error) => {
                run.status = if error == ConnectorFailure::MissingFacts {
                    RunStatus::Blocked
                } else {
                    RunStatus::Failed
                };
                run.blocked_reason = Some(error.message().into());
            }
        }
        run.latency_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
        sqlx::query("INSERT INTO business_runs(id,company_key,idempotency_key,request_json,company_snapshot,run_json,version,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?)")
            .bind(&run.run_id).bind(key).bind(&request.idempotency_key).bind(request_json)
            .bind(encode(&company)?).bind(encode(&run)?).bind(run.version).bind(&run.created_at).bind(&run.updated_at)
            .execute(&mut *tx).await?;
        tx.commit().await?;
        tracing::info!(target: "business", company_key = key, workspace_id = %run.workspace_id, run_id = %run.run_id, status = ?run.status, "[Business] Enquiry run persisted");
        Ok((run, true))
    }

    fn valid_run_id(id: &str) -> Result<(), AppError> {
        Uuid::parse_str(id)
            .map(|_| ())
            .map_err(|_| AppError::BadRequest("Invalid business run identity".into()))
    }

    async fn load_run(
        connection: &mut SqliteConnection,
        key: &str,
        id: &str,
    ) -> Result<(Run, Company), AppError> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT run_json,company_snapshot FROM business_runs WHERE company_key=? AND id=?",
        )
        .bind(key)
        .bind(id)
        .fetch_optional(connection)
        .await?;
        let (run_json, snapshot_json) =
            row.ok_or_else(|| AppError::NotFound("Business run not found in this company".into()))?;
        let run: Run = decode(&run_json)?;
        let snapshot: Company = decode(&snapshot_json)?;
        if run.company_key != key
            || snapshot.company_key != key
            || run.workspace_id != snapshot.workspace_id
            || run.run_id != id
        {
            return Err(AppError::InternalServerError(
                "Stored business run ownership is invalid".into(),
            ));
        }
        Ok((run, snapshot))
    }

    pub async fn run(&self, key: &str, id: &str) -> Result<Run, AppError> {
        let company = self.company(key)?;
        Self::valid_run_id(id)?;
        let mut connection = self.pool(&company).await?.acquire().await?;
        let (run, _) = Self::load_run(&mut connection, key, id).await?;
        Ok(run)
    }

    pub async fn list_runs(&self, key: &str) -> Result<Vec<Run>, AppError> {
        let company = self.company(key)?;
        let rows: Vec<String> = sqlx::query_scalar("SELECT run_json FROM business_runs WHERE company_key=? ORDER BY created_at DESC,id DESC LIMIT 100")
            .bind(key).fetch_all(self.pool(&company).await?).await?;
        let runs: Vec<Run> = rows
            .iter()
            .map(|row| decode(row))
            .collect::<Result<_, _>>()?;
        if runs
            .iter()
            .any(|run| run.company_key != key || run.workspace_id != company.workspace_id)
        {
            return Err(AppError::InternalServerError(
                "Stored business run ownership is invalid".into(),
            ));
        }
        Ok(runs)
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
        let (mut run, company) = Self::load_run(&mut tx, key, id).await?;
        if run.version != decision.expected_version || run.status != RunStatus::PendingApproval {
            return Err(AppError::Conflict(
                "Run changed or is no longer awaiting approval; reload before deciding".into(),
            ));
        }
        if run.connector_mode != "mock" || self.connector.mode() != run.connector_mode {
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
        run.version += 1;
        run.updated_at = now.to_rfc3339();
        run.approval = Some(Approval {
            decision: decision.decision,
            decided_at: now.to_rfc3339(),
        });
        match decision.decision {
            Decision::Reject => {
                run.status = RunStatus::Rejected;
            }
            Decision::Approve if !fresh(&company.offering, now) => {
                run.status = RunStatus::Blocked;
                run.blocked_reason = Some(
                    "Source freshness expired while approval was pending; no ticket was written."
                        .into(),
                );
            }
            Decision::Approve => {
                if run.draft_response != fact_sheet(&company)
                    || run.evidence != source_evidence(&company)?
                {
                    return Err(AppError::Conflict(
                        "Captured draft and source evidence no longer agree".into(),
                    ));
                }
                sqlx::query("SAVEPOINT business_ticket_write")
                    .execute(&mut *tx)
                    .await?;
                let result = tokio::time::timeout(
                    CONNECTOR_TIMEOUT,
                    self.connector.record_ticket(&mut tx, &company, &run),
                )
                .await
                .unwrap_or(Err(ConnectorFailure::Timeout));
                match result {
                    Ok((ticket, receipt)) => {
                        run.ticket = Some(ticket);
                        run.receipt = Some(receipt);
                        run.status = RunStatus::Completed;
                    }
                    Err(error) => {
                        sqlx::query("ROLLBACK TO SAVEPOINT business_ticket_write")
                            .execute(&mut *tx)
                            .await?;
                        run.status = RunStatus::Failed;
                        run.blocked_reason = Some(error.message().into());
                    }
                }
                sqlx::query("RELEASE SAVEPOINT business_ticket_write")
                    .execute(&mut *tx)
                    .await?;
            }
        }
        sqlx::query("INSERT INTO business_decisions(run_id,company_key,payload) VALUES(?,?,?)")
            .bind(id)
            .bind(key)
            .bind(encode(&run.approval)?)
            .execute(&mut *tx)
            .await?;
        run.latency_ms = run
            .latency_ms
            .saturating_add(started.elapsed().as_millis().min(i64::MAX as u128) as i64);
        let update = sqlx::query("UPDATE business_runs SET run_json=?,version=?,updated_at=? WHERE id=? AND company_key=? AND version=?")
            .bind(encode(&run)?).bind(run.version).bind(&run.updated_at).bind(id).bind(key).bind(decision.expected_version)
            .execute(&mut *tx).await?;
        if update.rows_affected() != 1 {
            return Err(AppError::Conflict(
                "Business run changed during decision".into(),
            ));
        }
        tx.commit().await?;
        tracing::info!(target: "business", company_key = key, workspace_id = %run.workspace_id, run_id = id, status = ?run.status, "[Business] Enquiry decision persisted");
        Ok(run)
    }

    pub async fn report(&self, key: &str, id: &str) -> Result<Report, AppError> {
        let company = self.company(key)?;
        Self::valid_run_id(id)?;
        let mut tx = self.pool(&company).await?.begin().await?;
        let (run, snapshot) = Self::load_run(&mut tx, key, id).await?;
        let ticket_row: Option<(String, String)> = sqlx::query_as(
            "SELECT id,payload FROM business_tickets WHERE run_id=? AND company_key=?",
        )
        .bind(id)
        .bind(key)
        .fetch_optional(&mut *tx)
        .await?;
        let receipt_row: Option<(String, String)> = sqlx::query_as(
            "SELECT id,payload FROM business_receipts WHERE run_id=? AND company_key=?",
        )
        .bind(id)
        .bind(key)
        .fetch_optional(&mut *tx)
        .await?;
        let approval_json: Option<String> = sqlx::query_scalar(
            "SELECT payload FROM business_decisions WHERE run_id=? AND company_key=?",
        )
        .bind(id)
        .bind(key)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        let ticket: Option<Ticket> = ticket_row
            .as_ref()
            .and_then(|(_, value)| serde_json::from_str(value).ok());
        let receipt: Option<Receipt> = receipt_row
            .as_ref()
            .and_then(|(_, value)| serde_json::from_str(value).ok());
        let approval: Option<Approval> = approval_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok());
        let approval_matches = approval
            .as_ref()
            .is_some_and(|value| value.decision == Decision::Approve)
            && approval == run.approval;
        let ticket_matches = ticket.as_ref().is_some_and(|value| {
            value.response == run.draft_response
                && value.display_reference == format!("{key}-Ticket-{}", value.id)
                && ticket_row.as_ref().is_some_and(|(id, _)| id == &value.id)
        }) && ticket == run.ticket;
        let receipt_matches =
            receipt
                .as_ref()
                .zip(ticket.as_ref())
                .is_some_and(|(value, ticket)| {
                    value.company_key == key
                        && value.run_id == id
                        && value.status == "completed"
                        && value.operation == "record_customer_enquiry"
                        && value.external_record_reference == ticket.display_reference
                        && value.evidence_location
                            == format!("business://{}/tickets/{}", snapshot.workspace_id, ticket.id)
                        && value.display_reference == format!("{key}-Receipt-{}", value.id)
                        && receipt_row.as_ref().is_some_and(|(id, _)| id == &value.id)
                })
                && receipt == run.receipt;
        let approved_fresh = approval
            .as_ref()
            .and_then(|value| timestamp(&value.decided_at))
            .is_some_and(|time| fresh(&snapshot.offering, time));
        let check = |name: &str, passed: bool, detail: &str| VerificationCheck {
            name: name.into(),
            passed,
            detail: detail.into(),
        };
        let sop_matches = sops::matches(&run, &snapshot.sop)?;
        let facts_match = run.draft_response == fact_sheet(&snapshot)
            && run.evidence == source_evidence(&snapshot)?;
        let checks = vec![
            check("completed", run.status == RunStatus::Completed, "A completed workflow is required for verified completion."),
            check("company_scope", run.company_key == snapshot.company_key && run.workspace_id == snapshot.workspace_id && run.environment == snapshot.environment, "Saved records retain the captured company workspace."),
            check("workflow_context", run.workflow_definition_id == WORKFLOW_DEFINITION_ID && run.workflow_version == snapshot.workflow_version && run.configuration_version == snapshot.configuration_version && run.fixture_version == snapshot.fixture_version && run.connector_mode == "mock", "Captured workflow, fixture and connector versions agree."),
            check("sop_context", sop_matches, "The exact SOP content, role assignments and version are bound to this run."),
            check("approval_persisted", approval_matches, "An independent saved approval must authorize the recorded response."),
            check("ticket_persisted", ticket_matches, "An independent saved ticket must match the approved draft."),
            check("receipt_persisted", receipt_matches, "An independent receipt must identify the ticket, company and run."),
            check("fact_backed_response", facts_match, "The fact sheet and evidence must exactly match the captured company facts; semantic enquiry quality requires owner review."),
            check("source_fresh_at_approval", approved_fresh, "The source must have been current when the approval decision was made."),
        ];
        let verified = checks.iter().all(|check| check.passed);
        let sop_evaluation = sops::evaluate(
            &run,
            &snapshot.sop,
            sops::EvaluationEvidence {
                context_matches: sop_matches
                    && run.company_key == snapshot.company_key
                    && run.workspace_id == snapshot.workspace_id,
                facts_match,
                approved: approval_matches,
                decision_consistent: approval == run.approval,
                rejected: approval
                    .as_ref()
                    .is_some_and(|value| value.decision == Decision::Reject)
                    && approval == run.approval,
                no_output_records: ticket_row.is_none()
                    && receipt_row.is_none()
                    && run.ticket.is_none()
                    && run.receipt.is_none(),
                verified_outcome: verified,
            },
        );
        Ok(Report {
            report_display_reference: format!("{key}-Report-{id}"),
            latency_ms: run.latency_ms,
            verification: Verification { verified, checks },
            sop_evaluation,
            run,
            cost: Cost {
                provider_charges_usd: 0.0,
                estimated_usd: 0.0,
                provenance: "deterministic_local_no_model".into(),
            },
        })
    }
}
