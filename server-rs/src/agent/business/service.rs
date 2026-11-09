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

pub(super) const FIXTURES: [(&str, &str); 2] = [
    (
        "Mock-1-Product",
        include_str!("../../../../starter_kits/mock_smb/companies/Mock-1-Product.json"),
    ),
    (
        "Mock-2-Service",
        include_str!("../../../../starter_kits/mock_smb/companies/Mock-2-Service.json"),
    ),
];
pub(super) const CONNECTOR_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

#[derive(Clone)]
pub struct BusinessService {
    pub(super) root: PathBuf,
    pools: Arc<[OnceCell<SqlitePool>; 2]>,
    pub(super) connector: Arc<dyn BusinessConnector>,
    pub(super) clock: Clock,
}

pub(super) fn encode<T: Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value)
        .map_err(|_| AppError::InternalServerError("Business evidence serialization failed".into()))
}
pub(super) fn decode<T: DeserializeOwned>(value: &str) -> Result<T, AppError> {
    serde_json::from_str(value).map_err(|err| {
        tracing::error!(
            target: "business",
            error = %err,
            payload_sample = &value.chars().take(200).collect::<String>(),
            "[Business] Stored business evidence JSON deserialization failed"
        );
        AppError::InternalServerError("Stored business evidence is invalid".into())
    })
}
pub(super) fn timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}
pub(super) fn fresh(offering: &Offering, now: DateTime<Utc>) -> bool {
    match (
        timestamp(&offering.source_updated_at),
        timestamp(&offering.fresh_until),
    ) {
        (Some(updated), Some(until)) => updated <= now && now < until,
        _ => false,
    }
}

pub(super) use super::pii::{detect_sensitive_pii, luhn_valid};

pub(super) fn fact_sheet(company: &Company) -> String {
    let offering = &company.offering;
    format!(
        "Company fact sheet for review\n{}\n{}\nListed base price: {} {}.{:02}\nHours: {}\nPolicy: {}\nSupport: {}\nConfirm these facts address the customer enquiry before approving this response.",
        offering.title, offering.description, offering.currency, offering.price_minor / 100,
        offering.price_minor % 100, company.profile.hours, company.profile.policy, company.profile.support_email,
    )
}

pub(super) fn source_evidence(company: &Company) -> Result<Vec<Evidence>, AppError> {
    Ok(vec![Evidence {
        source_id: company.offering.source_id.clone(),
        source_updated_at: company.offering.source_updated_at.clone(),
        fresh_until: company.offering.fresh_until.clone(),
        content: encode(company)?,
    }])
}

fn is_valid_idempotency_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 128
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_: .".contains(&b))
}

fn is_valid_enquiry(enquiry: &str) -> bool {
    !enquiry.is_empty()
        && enquiry.len() <= 4000
        && !enquiry
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
}

fn validate_request_enquiry(company: &Company, request: &CreateRun) -> Result<(), AppError> {
    if !is_valid_idempotency_key(&request.idempotency_key) || !is_valid_enquiry(&request.enquiry) {
        return Err(AppError::BadRequest(
            "Provide a bounded idempotency key and a nonempty enquiry of at most 4000 bytes".into(),
        ));
    }
    if company.environment == "mock" {
        if let Some(pii) = detect_sensitive_pii(&request.enquiry) {
            return Err(AppError::BadRequest(format!(
                "Sensitive PII ({pii}) is forbidden in mock practice workspaces (FR-11). Redact personal identifiers before submitting."
            )));
        }
    }
    Ok(())
}

fn validate_request_references(
    mode: &str,
    company: &Company,
    req: &CreateRun,
) -> Result<(), AppError> {
    if req.customer_reference != company.customer.display_reference {
        return Err(AppError::BadRequest(
            "Customer reference does not belong to this company".into(),
        ));
    }
    let valid_prefix = format!("{}-", company.company_key);
    if req.offering_reference.len() > 200 || !req.offering_reference.starts_with(&valid_prefix) {
        return Err(AppError::BadRequest(
            "Offering reference does not belong to this company".into(),
        ));
    }
    if !matches!(mode, "mock" | "read_only") {
        return Err(AppError::NotImplemented(
            "Only the registered mock or read_only shadow connector is available; live access is disabled".into(),
        ));
    }
    Ok(())
}

fn init_run(
    key: &str,
    company: &Company,
    req: &CreateRun,
    mode: &str,
    now: DateTime<Utc>,
) -> Result<Run, AppError> {
    let run_id = Uuid::new_v4().to_string();
    Ok(Run {
        schema_version: 1,
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
        connector_mode: mode.into(),
        version: 1,
        status: RunStatus::PendingApproval,
        customer_reference: req.customer_reference.clone(),
        offering_reference: req.offering_reference.clone(),
        enquiry: req.enquiry.clone(),
        draft_response: String::new(),
        evidence: Vec::new(),
        approval: None,
        ticket: None,
        receipt: None,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        blocked_reason: None,
        latency_ms: 0,
    })
}

struct RetrievalContext<'a> {
    key: &'a str,
    offering_ref: &'a str,
    now: DateTime<Utc>,
    retrieval: Result<Offering, ConnectorFailure>,
}

fn apply_offering_retrieval(
    ctx: RetrievalContext<'_>,
    company: &mut Company,
    run: &mut Run,
) -> Result<(), AppError> {
    let valid_prefix = format!("{}-", ctx.key);
    match ctx.retrieval {
        Ok(offering)
            if offering.display_reference == ctx.offering_ref
                && offering.source_id.starts_with(&valid_prefix) =>
        {
            company.offering = offering;
            run.evidence = source_evidence(company)?;
            if fresh(&company.offering, ctx.now) {
                run.draft_response = fact_sheet(company);
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
        Err(err) => {
            run.status = if err == ConnectorFailure::MissingFacts {
                RunStatus::Blocked
            } else {
                RunStatus::Failed
            };
            run.blocked_reason = Some(err.message().into());
        }
    }
    Ok(())
}

async fn persist_or_dedup_run(
    pool: &SqlitePool,
    key: &str,
    request: &CreateRun,
    company: &Company,
    run: &Run,
) -> Result<(Run, bool), AppError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let request_json = encode(request)?;
    let existing: Option<(String, String)> = sqlx::query_as(
        "SELECT request_json,run_json FROM business_runs WHERE company_key=? AND idempotency_key=?",
    )
    .bind(key)
    .bind(&request.idempotency_key)
    .fetch_optional(&mut *tx)
    .await?;
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
    sqlx::query("INSERT INTO business_runs(id,company_key,idempotency_key,request_json,company_snapshot,run_json,version,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?)")
        .bind(&run.run_id).bind(key).bind(&request.idempotency_key).bind(request_json)
        .bind(encode(company)?).bind(encode(run)?).bind(run.version).bind(&run.created_at).bind(&run.updated_at)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    tracing::info!(target: "business", company_key = key, workspace_id = %run.workspace_id, run_id = %run.run_id, status = ?run.status, "[Business] Enquiry run persisted");
    Ok((run.clone(), true))
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

    pub(super) async fn pool(&self, company: &Company) -> Result<&SqlitePool, AppError> {
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
                "CREATE TABLE IF NOT EXISTS business_outbox (id TEXT PRIMARY KEY, run_id TEXT NOT NULL UNIQUE REFERENCES business_runs(id), company_key TEXT NOT NULL, connector_mode TEXT NOT NULL, status TEXT NOT NULL, payload TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
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
        validate_request_enquiry(company, request)?;
        validate_request_references(self.connector.mode(), company, request)
    }

    pub async fn create_run(
        &self,
        key: &str,
        mut request: CreateRun,
    ) -> Result<(Run, bool), AppError> {
        let started = Instant::now();
        let mut company = self.company(key)?;
        self.validate_request(&company, &mut request)?;
        let now = (self.clock)();
        let mut run = init_run(key, &company, &request, self.connector.mode(), now)?;
        company.catalog_offerings = self.load_catalog_offerings(&company);
        let retrieval = tokio::time::timeout(
            CONNECTOR_TIMEOUT,
            self.connector
                .retrieve_offering(&company, &request.offering_reference),
        )
        .await
        .unwrap_or(Err(ConnectorFailure::Timeout));
        company.catalog_offerings.clear();
        let ctx = RetrievalContext {
            key,
            offering_ref: &request.offering_reference,
            now,
            retrieval,
        };
        apply_offering_retrieval(ctx, &mut company, &mut run)?;
        run.latency_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
        let pool = self.pool(&company).await?;
        persist_or_dedup_run(pool, key, &request, &company, &run).await
    }

    pub(super) fn valid_run_id(id: &str) -> Result<(), AppError> {
        Uuid::parse_str(id)
            .map(|_| ())
            .map_err(|_| AppError::BadRequest("Invalid business run identity".into()))
    }

    pub(super) async fn load_run(
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
}
