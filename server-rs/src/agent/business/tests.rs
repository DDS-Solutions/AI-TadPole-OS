//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Exercise real file-backed company stores, connector failures and independent evidence checks.
//! ### 🔍 Debugging & Observability
//! Deterministic test clocks keep explicit fixture freshness repeatable without changing runtime time.

use super::{
    connector::{BusinessConnector, ConnectorFailure, MockConnector},
    service::BusinessService,
    types::*,
};
use crate::error::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqliteConnectOptions, SqliteConnection, SqlitePool};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-10-23T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}
fn service(path: &Path) -> BusinessService {
    BusinessService::with_dependencies(path.into(), Arc::new(now), Arc::new(MockConnector))
}
fn request(company: &Company, key: &str) -> CreateRun {
    CreateRun {
        idempotency_key: key.into(),
        customer_reference: company.customer.display_reference.clone(),
        enquiry: "What is the listed price and company policy?".into(),
        offering_reference: company.offering.display_reference.clone(),
    }
}
async fn raw_pool(root: &Path, company: &Company) -> SqlitePool {
    let path = root.join("data/business/workspaces").join(format!(
        "{}-{}.sqlite3",
        company.company_key, company.workspace_id
    ));
    SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(path)
            .foreign_keys(true),
    )
    .await
    .unwrap()
}
async fn counts(pool: &SqlitePool) -> (i64, i64, i64) {
    let tickets = sqlx::query_scalar("SELECT COUNT(*) FROM business_tickets")
        .fetch_one(pool)
        .await
        .unwrap();
    let receipts = sqlx::query_scalar("SELECT COUNT(*) FROM business_receipts")
        .fetch_one(pool)
        .await
        .unwrap();
    let decisions = sqlx::query_scalar("SELECT COUNT(*) FROM business_decisions")
        .fetch_one(pool)
        .await
        .unwrap();
    (tickets, receipts, decisions)
}

#[tokio::test]
async fn both_companies_complete_same_workflow_with_independent_records() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let companies = svc.companies().unwrap();
    assert_eq!(companies.len(), 2);
    for company in &companies {
        let (run, created) = svc
            .create_run(
                &company.company_key,
                request(company, "shared-operation-key"),
            )
            .await
            .unwrap();
        assert!(created);
        assert_eq!(run.status, RunStatus::PendingApproval);
        assert!(run.ticket.is_none());
        assert!(run.draft_response.contains(&company.profile.policy));
        assert!(run.evidence[0]
            .content
            .contains(&company.profile.support_email));
        assert!(
            !svc.report(&company.company_key, &run.run_id)
                .await
                .unwrap()
                .verification
                .verified
        );
        let completed = svc
            .decide_run(
                &company.company_key,
                &run.run_id,
                DecideRun {
                    expected_version: 1,
                    decision: Decision::Approve,
                    edited_response: None,
                    operator_note: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(completed.status, RunStatus::Completed);
        let report = svc.report(&company.company_key, &run.run_id).await.unwrap();
        assert!(
            report.verification.verified,
            "{:?}",
            report.verification.checks
        );
        assert_eq!(report.sop_evaluation.status, "passed");
        assert_eq!(report.sop_evaluation.steps.len(), 6);
        assert!(!report.sop_evaluation.agent_execution_evaluated);
        assert!(!report.sop_evaluation.semantic_response_quality_evaluated);
        assert_eq!(report.run.sop_version, company.sop.version);
        assert!(company.sop.document.contains(&company.company_key));
        assert_eq!(report.cost.provenance, "deterministic_local_no_model");
        assert_eq!(report.cost.provider_charges_usd, 0.0);
        assert!(completed
            .ticket
            .unwrap()
            .display_reference
            .starts_with(&company.company_key));
        assert_eq!(
            counts(&raw_pool(temp.path(), company).await).await,
            (1, 1, 1)
        );
        assert_eq!(svc.list_runs(&company.company_key).await.unwrap().len(), 1);
    }
    assert_eq!(
        companies[0].workflow_definition_id,
        companies[1].workflow_definition_id
    );
    assert_ne!(companies[0].workspace_id, companies[1].workspace_id);
}

#[tokio::test]
async fn concurrent_idempotency_is_payload_bound_and_company_scoped() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let company = svc.company("Mock-1-Product").unwrap();
    let req = request(&company, "retry-key");
    let (first, second) = tokio::join!(
        svc.create_run(&company.company_key, req.clone()),
        svc.create_run(&company.company_key, req.clone())
    );
    let first = first.unwrap();
    let second = second.unwrap();
    assert_eq!(first.0.run_id, second.0.run_id);
    assert_ne!(first.1, second.1);
    let mut changed = req;
    changed.enquiry = "Different question".into();
    assert!(matches!(
        svc.create_run(&company.company_key, changed).await,
        Err(AppError::Conflict(_))
    ));
    let other = svc.company("Mock-2-Service").unwrap();
    let (other_run, _) = svc
        .create_run(&other.company_key, request(&other, "retry-key"))
        .await
        .unwrap();
    assert_ne!(first.0.run_id, other_run.run_id);
}

#[tokio::test]
async fn company_switch_never_retargets_reads_decisions_or_references() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let product = svc.company("Mock-1-Product").unwrap();
    let other = svc.company("Mock-2-Service").unwrap();
    let (run, _) = svc
        .create_run(&product.company_key, request(&product, "switch"))
        .await
        .unwrap();
    assert!(matches!(
        svc.run(&other.company_key, &run.run_id).await,
        Err(AppError::NotFound(_))
    ));
    assert!(matches!(
        svc.report(&other.company_key, &run.run_id).await,
        Err(AppError::NotFound(_))
    ));
    assert!(matches!(
        svc.decide_run(
            &other.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: None,
                operator_note: None,
            }
        )
        .await,
        Err(AppError::NotFound(_))
    ));
    let mut cross = request(&product, "cross");
    cross.offering_reference = other.offering.display_reference;
    assert!(matches!(
        svc.create_run(&product.company_key, cross).await,
        Err(AppError::BadRequest(_))
    ));
    assert!(matches!(
        svc.company("../Mock-1-Product"),
        Err(AppError::NotFound(_))
    ));
    assert_eq!(
        svc.run(&product.company_key, &run.run_id)
            .await
            .unwrap()
            .status,
        RunStatus::PendingApproval
    );
}

#[tokio::test]
async fn denied_and_competing_decisions_never_duplicate_writes() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let company = svc.company("Mock-1-Product").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "deny"))
        .await
        .unwrap();
    let rejected = svc
        .decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Reject,
                edited_response: None,
                operator_note: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(rejected.status, RunStatus::Rejected);
    assert!(
        !svc.report(&company.company_key, &run.run_id)
            .await
            .unwrap()
            .verification
            .verified
    );
    assert_eq!(
        svc.report(&company.company_key, &run.run_id)
            .await
            .unwrap()
            .sop_evaluation
            .status,
        "passed"
    );
    let pool = raw_pool(temp.path(), &company).await;
    assert_eq!(counts(&pool).await, (0, 0, 1));
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "race"))
        .await
        .unwrap();
    let (approved, rejected) = tokio::join!(
        svc.decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: None,
                operator_note: None,
            }
        ),
        svc.decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Reject,
                edited_response: None,
                operator_note: None,
            }
        )
    );
    assert_ne!(approved.is_ok(), rejected.is_ok());
    assert!(matches!(
        approved.as_ref().err().or(rejected.as_ref().err()),
        Some(AppError::Conflict(_))
    ));
    let saved = svc.run(&company.company_key, &run.run_id).await.unwrap();
    let expected_tickets = i64::from(saved.status == RunStatus::Completed);
    assert_eq!(counts(&pool).await, (expected_tickets, expected_tickets, 2));
}

#[tokio::test]
async fn file_backed_pending_approval_survives_service_restart() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let company = svc.company("Mock-2-Service").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "restart"))
        .await
        .unwrap();
    drop(svc);
    let restarted = service(temp.path());
    assert_eq!(
        restarted
            .run(&company.company_key, &run.run_id)
            .await
            .unwrap(),
        run
    );
    restarted
        .decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: None,
                operator_note: None,
            },
        )
        .await
        .unwrap();
    assert!(
        restarted
            .report(&company.company_key, &run.run_id)
            .await
            .unwrap()
            .verification
            .verified
    );
    assert_eq!(
        counts(&raw_pool(temp.path(), &company).await).await,
        (1, 1, 1)
    );
}

#[tokio::test]
async fn source_expiry_blocks_creation_and_pending_approval() {
    let temp = tempfile::tempdir().unwrap();
    let ticks = Arc::new(AtomicI64::new(now().timestamp()));
    let clock_ticks = Arc::clone(&ticks);
    let svc = BusinessService::with_dependencies(
        temp.path().into(),
        Arc::new(move || DateTime::from_timestamp(clock_ticks.load(Ordering::SeqCst), 0).unwrap()),
        Arc::new(MockConnector),
    );
    let company = svc.company("Mock-1-Product").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "expires-pending"))
        .await
        .unwrap();
    ticks.store(
        DateTime::parse_from_rfc3339(&company.offering.fresh_until)
            .unwrap()
            .timestamp(),
        Ordering::SeqCst,
    );
    let blocked = svc
        .decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: None,
                operator_note: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(blocked.status, RunStatus::Blocked);
    assert!(blocked.ticket.is_none());
    assert!(
        !svc.report(&company.company_key, &run.run_id)
            .await
            .unwrap()
            .verification
            .verified
    );
    let (stale, _) = svc
        .create_run(&company.company_key, request(&company, "already-stale"))
        .await
        .unwrap();
    assert_eq!(stale.status, RunStatus::Blocked);
    assert!(stale.draft_response.is_empty());
    assert_eq!(
        counts(&raw_pool(temp.path(), &company).await).await,
        (0, 0, 1)
    );
}

#[derive(Clone, Copy)]
enum Fault {
    Missing,
    ReadTimeout,
    PartialWriteTimeout,
    Unavailable,
    HangingRead,
    HangingWrite,
    HangingBetweenInserts,
}
struct FaultConnector(Fault);
#[async_trait]
impl BusinessConnector for FaultConnector {
    fn mode(&self) -> &'static str {
        if matches!(self.0, Fault::Unavailable) {
            "live"
        } else {
            "mock"
        }
    }
    async fn retrieve_offering(
        &self,
        company: &Company,
        reference: &str,
    ) -> Result<Offering, ConnectorFailure> {
        match self.0 {
            Fault::Missing => Err(ConnectorFailure::MissingFacts),
            Fault::ReadTimeout => Err(ConnectorFailure::Timeout),
            Fault::Unavailable => Err(ConnectorFailure::Unavailable),
            Fault::HangingRead => std::future::pending().await,
            _ => MockConnector.retrieve_offering(company, reference).await,
        }
    }
    async fn record_ticket(
        &self,
        connection: &mut SqliteConnection,
        company: &Company,
        run: &Run,
    ) -> Result<(Ticket, Receipt), ConnectorFailure> {
        if matches!(self.0, Fault::HangingBetweenInserts) {
            let ticket_id = uuid::Uuid::new_v4().to_string();
            let ticket = Ticket {
                display_reference: format!("{}-Ticket-{}", company.company_key, ticket_id),
                id: ticket_id,
                response: run.draft_response.clone(),
            };
            let ticket_json =
                serde_json::to_string(&ticket).map_err(|_| ConnectorFailure::Persistence)?;
            sqlx::query(
                "INSERT INTO business_tickets(id,run_id,company_key,payload) VALUES(?,?,?,?)",
            )
            .bind(&ticket.id)
            .bind(&run.run_id)
            .bind(&company.company_key)
            .bind(ticket_json)
            .execute(&mut *connection)
            .await
            .map_err(|_| ConnectorFailure::Persistence)?;
            // Terminate mid-write before receipt insert to test savepoint rollback
            return Err(ConnectorFailure::Timeout);
        }
        let result = MockConnector
            .record_ticket(connection, company, run)
            .await?;
        if matches!(self.0, Fault::HangingWrite) {
            return std::future::pending().await;
        }
        if matches!(self.0, Fault::PartialWriteTimeout) {
            Err(ConnectorFailure::Timeout)
        } else {
            Ok(result)
        }
    }
}

#[tokio::test]
async fn missing_facts_timeouts_and_unavailable_live_connectors_fail_closed() {
    for fault in [
        Fault::Missing,
        Fault::ReadTimeout,
        Fault::PartialWriteTimeout,
        Fault::Unavailable,
    ] {
        let temp = tempfile::tempdir().unwrap();
        let svc = BusinessService::with_dependencies(
            temp.path().into(),
            Arc::new(now),
            Arc::new(FaultConnector(fault)),
        );
        let company = svc.company("Mock-2-Service").unwrap();
        let result = svc
            .create_run(&company.company_key, request(&company, "failure"))
            .await;
        if matches!(fault, Fault::Unavailable) {
            assert!(matches!(result, Err(AppError::NotImplemented(_))));
            continue;
        }
        let (mut run, _) = result.unwrap();
        if matches!(fault, Fault::PartialWriteTimeout) {
            run = svc
                .decide_run(
                    &company.company_key,
                    &run.run_id,
                    DecideRun {
                        expected_version: 1,
                        decision: Decision::Approve,
                        edited_response: None,
                        operator_note: None,
                    },
                )
                .await
                .unwrap();
        }
        assert!(matches!(run.status, RunStatus::Blocked | RunStatus::Failed));
        if matches!(fault, Fault::Missing | Fault::ReadTimeout) {
            let report = svc.report(&company.company_key, &run.run_id).await.unwrap();
            let owner = report
                .sop_evaluation
                .steps
                .iter()
                .find(|step| step.owner_role.ends_with("-Owner"))
                .unwrap();
            assert_eq!(owner.status, "not_applicable");
            assert_eq!(owner.performed_by, "not_performed");
        }
        assert!(
            !svc.report(&company.company_key, &run.run_id)
                .await
                .unwrap()
                .verification
                .verified
        );
        assert_eq!(counts(&raw_pool(temp.path(), &company).await).await.0, 0);
        assert_eq!(counts(&raw_pool(temp.path(), &company).await).await.1, 0);
    }
}

#[tokio::test]
async fn completed_narrative_cannot_replace_missing_or_altered_receipt() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let company = svc.company("Mock-1-Product").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "tamper"))
        .await
        .unwrap();
    svc.decide_run(
        &company.company_key,
        &run.run_id,
        DecideRun {
            expected_version: 1,
            decision: Decision::Approve,
            edited_response: None,
            operator_note: None,
        },
    )
    .await
    .unwrap();
    let pool = raw_pool(temp.path(), &company).await;
    sqlx::query("UPDATE business_receipts SET payload='{}' WHERE run_id=?")
        .bind(&run.run_id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        !svc.report(&company.company_key, &run.run_id)
            .await
            .unwrap()
            .verification
            .verified
    );
    sqlx::query("DELETE FROM business_receipts WHERE run_id=?")
        .bind(&run.run_id)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        svc.run(&company.company_key, &run.run_id)
            .await
            .unwrap()
            .status,
        RunStatus::Completed
    );
    assert!(
        !svc.report(&company.company_key, &run.run_id)
            .await
            .unwrap()
            .verification
            .verified
    );
}

#[test]
fn request_contract_rejects_unknown_fault_injection_fields() {
    let body = serde_json::json!({"idempotency_key":"x","customer_reference":"x","enquiry":"x","offering_reference":"x","scenario":"timeout"});
    assert!(serde_json::from_value::<CreateRun>(body).is_err());
}

#[tokio::test]
async fn actual_connector_deadlines_cancel_hangs_and_roll_back_partial_writes() {
    for fault in [Fault::HangingRead, Fault::HangingWrite] {
        let temp = tempfile::tempdir().unwrap();
        let svc = BusinessService::with_dependencies(
            temp.path().into(),
            Arc::new(now),
            Arc::new(FaultConnector(fault)),
        );
        let company = svc.company("Mock-1-Product").unwrap();
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(12), async {
            let (run, _) = svc
                .create_run(&company.company_key, request(&company, "deadline"))
                .await
                .unwrap();
            if matches!(fault, Fault::HangingWrite) {
                svc.decide_run(
                    &company.company_key,
                    &run.run_id,
                    DecideRun {
                        expected_version: 1,
                        decision: Decision::Approve,
                        edited_response: None,
                        operator_note: None,
                    },
                )
                .await
                .unwrap()
            } else {
                run
            }
        })
        .await
        .expect("service must enforce its connector deadline");
        assert_eq!(outcome.status, RunStatus::Failed);
        assert_eq!(counts(&raw_pool(temp.path(), &company).await).await.0, 0);
        assert_eq!(counts(&raw_pool(temp.path(), &company).await).await.1, 0);
        assert!(
            !svc.report(&company.company_key, &outcome.run_id)
                .await
                .unwrap()
                .verification
                .verified
        );
    }
}

#[tokio::test]
async fn independent_record_primary_ids_and_sop_versions_cannot_be_forged() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let company = svc.company("Mock-1-Product").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "identity"))
        .await
        .unwrap();
    let done = svc
        .decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: None,
                operator_note: None,
            },
        )
        .await
        .unwrap();
    let pool = raw_pool(temp.path(), &company).await;
    let ticket_id = &done.ticket.as_ref().unwrap().id;
    sqlx::query("UPDATE business_tickets SET id='altered-id' WHERE run_id=?")
        .bind(&run.run_id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        !svc.report(&company.company_key, &run.run_id)
            .await
            .unwrap()
            .verification
            .verified
    );
    sqlx::query("UPDATE business_tickets SET id=? WHERE run_id=?")
        .bind(ticket_id)
        .bind(&run.run_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE business_receipts SET id='altered-id' WHERE run_id=?")
        .bind(&run.run_id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        !svc.report(&company.company_key, &run.run_id)
            .await
            .unwrap()
            .verification
            .verified
    );
    let (mut pending, _) = svc
        .create_run(&company.company_key, request(&company, "sop-change"))
        .await
        .unwrap();
    pending.sop_version = "2".into();
    sqlx::query("UPDATE business_runs SET run_json=? WHERE id=?")
        .bind(serde_json::to_string(&pending).unwrap())
        .bind(&pending.run_id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        svc.decide_run(
            &company.company_key,
            &pending.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: None,
                operator_note: None,
            }
        )
        .await,
        Err(AppError::Conflict(_))
    ));
    let report = svc
        .report(&company.company_key, &pending.run_id)
        .await
        .unwrap();
    assert!(!report.verification.verified);
    assert_eq!(report.sop_evaluation.status, "failed");
}

#[tokio::test]
async fn operator_edited_response_and_decision_note_persist_and_verify() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let company = svc.company("Mock-1-Product").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "operator-edit"))
        .await
        .unwrap();
    let custom_reply = format!(
        "{}\n\nOwner note: Shipping to Canada requires a separate quote.",
        run.draft_response
    );
    let completed = svc
        .decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: Some(custom_reply.clone()),
                operator_note: Some("Added Canada shipping clarification".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(completed.status, RunStatus::Completed);
    assert_eq!(completed.draft_response, custom_reply);
    assert_eq!(completed.ticket.as_ref().unwrap().response, custom_reply);
    let approval = completed.approval.as_ref().unwrap();
    assert!(approval.operator_edited);
    assert_eq!(
        approval.operator_note.as_deref(),
        Some("Added Canada shipping clarification")
    );
    let report = svc.report(&company.company_key, &run.run_id).await.unwrap();
    assert!(report.verification.verified);
    assert_eq!(report.sop_evaluation.status, "passed");
}

#[tokio::test]
async fn server_side_pii_guard_rejects_ssn_and_luhn_payment_cards_in_mock_mode() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let company = svc.company("Mock-1-Product").unwrap();
    let mut ssn_req = request(&company, "pii-ssn");
    ssn_req.enquiry = "Please look up my SSN 123-45-6789 for this order.".into();
    assert!(matches!(
        svc.create_run(&company.company_key, ssn_req).await,
        Err(AppError::BadRequest(_))
    ));
    let mut card_req = request(&company, "pii-card");
    card_req.enquiry = "Charge card 4532-0151-1283-0366 for the field bottle.".into();
    assert!(matches!(
        svc.create_run(&company.company_key, card_req).await,
        Err(AppError::BadRequest(_))
    ));
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "clean-run"))
        .await
        .unwrap();
    assert!(matches!(
        svc.decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: Some(format!("{} SSN 123-45-6789", run.draft_response)),
                operator_note: None,
            }
        )
        .await,
        Err(AppError::BadRequest(_))
    ));
}

#[tokio::test]
async fn promotion_readiness_outbox_atomicity_fact_renewal_and_grounded_citations_verify() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let initial = svc.company_with_readiness("Mock-1-Product").await.unwrap();
    let initial_readiness = initial.readiness.as_ref().unwrap();
    assert!(!initial_readiness.eligible_for_promotion);
    assert_eq!(initial_readiness.verified_runs, 0);

    let (run, _) = svc
        .create_run(&initial.company_key, request(&initial, "readiness-run"))
        .await
        .unwrap();
    // Ungrounded edit (stripping company and fact citations) must be rejected by FR-12
    assert!(matches!(
        svc.decide_run(
            &initial.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: Some(
                    "Completely ungrounded reply without any company facts.".into()
                ),
                operator_note: None,
            }
        )
        .await,
        Err(AppError::BadRequest(_))
    ));

    // Approve with grounded draft
    svc.decide_run(
        &initial.company_key,
        &run.run_id,
        DecideRun {
            expected_version: 1,
            decision: Decision::Approve,
            edited_response: None,
            operator_note: Some("Verified for FR-10 promotion check".into()),
        },
    )
    .await
    .unwrap();

    // Verify two-phase outbox row is committed
    let pool = raw_pool(temp.path(), &initial).await;
    let outbox_status: String =
        sqlx::query_scalar("SELECT status FROM business_outbox WHERE run_id=?")
            .bind(&run.run_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(outbox_status, "committed");

    // Readiness evaluator now reports eligible_for_promotion = true
    let updated = svc.company_with_readiness("Mock-1-Product").await.unwrap();
    let readiness = updated.readiness.unwrap();
    assert_eq!(readiness.verified_runs, 1);
    assert!(readiness.eligible_for_promotion);

    // Renew company facts and verify durability
    let renewed = svc.renew_company_facts("Mock-1-Product").unwrap();
    assert_eq!(
        renewed.offering.source_updated_at,
        "2026-10-23T12:00:00+00:00"
    );
}

#[tokio::test]
async fn read_only_shadow_connector_prepares_drafts_and_blocks_external_ticket_writes() {
    let temp = tempfile::tempdir().unwrap();
    let svc = BusinessService::with_dependencies(
        temp.path().into(),
        Arc::new(now),
        Arc::new(super::connector::ReadOnlyConnector),
    );
    let company = svc.company("Mock-2-Service").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "shadow-mode"))
        .await
        .unwrap();
    assert_eq!(run.status, RunStatus::PendingApproval);
    assert_eq!(run.connector_mode, "read_only");
    let decided = svc
        .decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: None,
                operator_note: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(decided.status, RunStatus::Blocked);
    assert!(decided.ticket.is_none());
    assert!(decided
        .blocked_reason
        .unwrap_or_default()
        .contains("read_only shadow mode"));
    let report = svc.report(&company.company_key, &run.run_id).await.unwrap();
    let wf_check = report
        .verification
        .checks
        .iter()
        .find(|c| c.name == "workflow_context")
        .unwrap();
    assert!(
        wf_check.passed,
        "workflow_context must pass for read_only runs"
    );
}

#[tokio::test]
async fn hanging_between_inserts_rolls_back_partial_ticket() {
    let temp = tempfile::tempdir().unwrap();
    let svc = BusinessService::with_dependencies(
        temp.path().into(),
        Arc::new(now),
        Arc::new(FaultConnector(Fault::HangingBetweenInserts)),
    );
    let company = svc.company("Mock-1-Product").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "mid-hang"))
        .await
        .unwrap();
    let decided = svc
        .decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: None,
                operator_note: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(decided.status, RunStatus::Failed);
    let pool = raw_pool(temp.path(), &company).await;
    let (tickets, receipts, _) = counts(&pool).await;
    assert_eq!(
        tickets, 0,
        "Partial ticket must be rolled back by savepoint"
    );
    assert_eq!(receipts, 0, "Receipt must not exist");
}

#[test]
fn grounded_citations_rejects_empty_profile_strings() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let mut company = svc.company("Mock-1-Product").unwrap();
    company.profile.policy = String::new();
    company.profile.hours = String::new();
    company.profile.support_email = String::new();
    let ungrounded = "Hello from Mock-1-Product with generic text";
    assert!(!super::sops::verify_grounded_citations(
        ungrounded, &company
    ));
    let grounded = format!(
        "Mock-1-Product item costs {:.2}",
        company.offering.price_minor as f64 / 100.0
    );
    assert!(super::sops::verify_grounded_citations(&grounded, &company));
}

#[test]
fn pii_guard_detects_unspaced_card_and_spaced_ssn() {
    assert_eq!(
        super::service::detect_sensitive_pii("Payment for order card4532015112830366 now"),
        Some("Payment Card")
    );
    assert_eq!(
        super::service::detect_sensitive_pii("My SSN is 123 45 6789 please verify"),
        Some("SSN")
    );
    assert_eq!(
        super::service::detect_sensitive_pii("Standard SSN 123-45-6789 works"),
        Some("SSN")
    );
}

#[tokio::test]
async fn reject_with_edited_response_returns_bad_request() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let company = svc.company("Mock-1-Product").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "reject-edit"))
        .await
        .unwrap();
    let err = svc
        .decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Reject,
                edited_response: Some("Edited draft on reject".into()),
                operator_note: None,
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
}

#[test]
fn corrupt_profile_override_falls_back_to_fixture() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let workspaces_dir = temp.path().join("data/business/workspaces");
    std::fs::create_dir_all(&workspaces_dir).unwrap();
    std::fs::write(
        workspaces_dir.join("Mock-1-Product-profile.json"),
        b"{ this is definitely not valid json }",
    )
    .unwrap();
    let company = svc.company("Mock-1-Product").unwrap();
    assert_eq!(company.company_key, "Mock-1-Product");
}

#[tokio::test]
async fn fact_renewal_rebases_stale_in_flight_run() {
    let temp = tempfile::tempdir().unwrap();
    let ticks = Arc::new(AtomicI64::new(now().timestamp()));
    let clock_ticks = Arc::clone(&ticks);
    let svc = BusinessService::with_dependencies(
        temp.path().into(),
        Arc::new(move || DateTime::from_timestamp(clock_ticks.load(Ordering::SeqCst), 0).unwrap()),
        Arc::new(MockConnector),
    );
    let company = svc.company("Mock-1-Product").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "rebase-run"))
        .await
        .unwrap();
    ticks.store(
        DateTime::parse_from_rfc3339(&company.offering.fresh_until)
            .unwrap()
            .timestamp()
            + 100,
        Ordering::SeqCst,
    );
    svc.renew_company_facts("Mock-1-Product").unwrap();
    let completed = svc
        .decide_run(
            &company.company_key,
            &run.run_id,
            DecideRun {
                expected_version: 1,
                decision: Decision::Approve,
                edited_response: None,
                operator_note: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(completed.status, RunStatus::Completed);
    let rep = svc.report(&company.company_key, &run.run_id).await.unwrap();
    assert!(rep.verification.verified);
}

#[tokio::test]
async fn readiness_checks_are_measured_and_detect_silo_leak() {
    let temp = tempfile::tempdir().unwrap();
    let svc = service(temp.path());
    let company = svc.company("Mock-1-Product").unwrap();
    let (run, _) = svc
        .create_run(&company.company_key, request(&company, "silo-run"))
        .await
        .unwrap();
    svc.decide_run(
        &company.company_key,
        &run.run_id,
        DecideRun {
            expected_version: 1,
            decision: Decision::Approve,
            edited_response: None,
            operator_note: None,
        },
    )
    .await
    .unwrap();
    let initial = svc.evaluate_readiness("Mock-1-Product").await.unwrap();
    let silo_check = initial
        .checks
        .iter()
        .find(|c| c.id == "company_silo_isolation")
        .unwrap();
    assert!(silo_check.passed);

    let pool = raw_pool(temp.path(), &company).await;
    sqlx::query(
        "INSERT INTO business_runs(id, company_key, idempotency_key, request_json, company_snapshot, run_json, version, created_at, updated_at) VALUES('foreign-run-id', 'Foreign-Company', 'foreign-idem', '{}', '{}', '{}', 1, '2026-10-23T12:00:00Z', '2026-10-23T12:00:00Z')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let tampered = svc.evaluate_readiness("Mock-1-Product").await.unwrap();
    let silo_check_after = tampered
        .checks
        .iter()
        .find(|c| c.id == "company_silo_isolation")
        .unwrap();
    assert!(
        !silo_check_after.passed,
        "Silo check must fail when foreign company record exists"
    );
    assert!(!tampered.eligible_for_promotion);
}
