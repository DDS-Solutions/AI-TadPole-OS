//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Generates audit reports and verifies evidentiary integrity against captured company profiles and SOPs.
//! ### 🔍 Debugging & Observability
//! Traces identify failed verification assertions and SOP compliance mismatches.

use super::service::{fact_sheet, fresh, source_evidence, timestamp, BusinessService};
use super::sops;
use super::types::*;
use crate::error::AppError;

fn check_ticket_match(
    ticket: Option<&Ticket>,
    run: &Run,
    key: &str,
    ticket_row: Option<&(String, String)>,
) -> bool {
    let matches_criteria = ticket.as_ref().is_some_and(|value| {
        value.response == run.draft_response
            && value.display_reference == format!("{key}-Ticket-{}", value.id)
            && ticket_row.as_ref().is_some_and(|(id, _)| id == &value.id)
    });
    matches_criteria && ticket == run.ticket.as_ref()
}

fn check_receipt_match(
    receipt: Option<&Receipt>,
    ticket: Option<&Ticket>,
    run: &Run,
    snapshot: &Company,
    receipt_row: Option<&(String, String)>,
) -> bool {
    let key = &snapshot.company_key;
    let matches_criteria = receipt.as_ref().zip(ticket).is_some_and(|(value, t)| {
        value.company_key == *key
            && value.run_id == run.run_id
            && value.status == "completed"
            && value.operation == "record_customer_enquiry"
            && value.external_record_reference == t.display_reference
            && value.evidence_location
                == format!("business://{}/tickets/{}", snapshot.workspace_id, t.id)
            && value.display_reference == format!("{key}-Receipt-{}", value.id)
            && receipt_row.as_ref().is_some_and(|(id, _)| id == &value.id)
    });
    matches_criteria && receipt == run.receipt.as_ref()
}

fn verify_facts_and_evidence(
    run: &Run,
    snapshot: &Company,
    approval_matches: bool,
) -> Result<bool, AppError> {
    let citations_ok = sops::verify_grounded_citations(&run.draft_response, snapshot);
    let edited_valid = approval_matches
        && run.approval.as_ref().is_some_and(|a| a.operator_edited)
        && !run.draft_response.trim().is_empty()
        && run.draft_response.len() <= 4000
        && citations_ok;
    let facts_match = (run.draft_response == fact_sheet(snapshot) || edited_valid)
        && run.evidence == source_evidence(snapshot)?
        && citations_ok;
    Ok(facts_match)
}

fn build_sop_evaluation(
    p: &VerificationParams<'_>,
    approval: Option<&Approval>,
    evidence: &RunEvidenceRows,
    verified: bool,
) -> SopEvaluation {
    sops::evaluate(
        p.run,
        &p.snapshot.sop,
        sops::EvaluationEvidence {
            context_matches: p.sop_matches
                && p.run.company_key == p.snapshot.company_key
                && p.run.workspace_id == p.snapshot.workspace_id,
            facts_match: p.facts_match,
            approved: p.approval_matches,
            decision_consistent: approval == p.run.approval.as_ref(),
            rejected: approval.is_some_and(|v| v.decision == Decision::Reject)
                && approval == p.run.approval.as_ref(),
            no_output_records: evidence.ticket_row.is_none()
                && evidence.receipt_row.is_none()
                && p.run.ticket.is_none()
                && p.run.receipt.is_none(),
            verified_outcome: verified,
        },
    )
}

struct VerificationParams<'a> {
    run: &'a Run,
    snapshot: &'a Company,
    approval_matches: bool,
    ticket_matches: bool,
    receipt_matches: bool,
    approved_fresh: bool,
    sop_matches: bool,
    facts_match: bool,
}

fn assemble_verification_checks(p: &VerificationParams<'_>) -> Vec<VerificationCheck> {
    let check = |name: &str, passed: bool, detail: &str| VerificationCheck {
        name: name.into(),
        passed,
        detail: detail.into(),
    };
    vec![
        check("completed", p.run.status == RunStatus::Completed, "A completed workflow is required for verified completion."),
        check("company_scope", p.run.company_key == p.snapshot.company_key && p.run.workspace_id == p.snapshot.workspace_id && p.run.environment == p.snapshot.environment, "Saved records retain the captured company workspace."),
        check("workflow_context", p.run.workflow_definition_id == WORKFLOW_DEFINITION_ID && p.run.workflow_version == p.snapshot.workflow_version && p.run.configuration_version == p.snapshot.configuration_version && p.run.fixture_version == p.snapshot.fixture_version && matches!(p.run.connector_mode.as_str(), "mock" | "read_only"), "Captured workflow, fixture and connector versions agree."),
        check("sop_context", p.sop_matches, "The exact SOP content, role assignments and version are bound to this run."),
        check("approval_persisted", p.approval_matches, "An independent saved approval must authorize the recorded response."),
        check("ticket_persisted", p.ticket_matches, "An independent saved ticket must match the approved draft."),
        check("receipt_persisted", p.receipt_matches, "An independent receipt must identify the ticket, company and run."),
        check("fact_backed_response", p.facts_match, "The fact sheet and evidence must exactly match the captured company facts; semantic enquiry quality requires owner review."),
        check("source_fresh_at_approval", p.approved_fresh, "The source must have been current when the approval decision was made."),
    ]
}

struct RunEvidenceRows {
    ticket_row: Option<(String, String)>,
    receipt_row: Option<(String, String)>,
    approval_json: Option<String>,
}

async fn fetch_run_evidence(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    key: &str,
    id: &str,
) -> Result<RunEvidenceRows, AppError> {
    let ticket_row =
        sqlx::query_as("SELECT id,payload FROM business_tickets WHERE run_id=? AND company_key=?")
            .bind(id)
            .bind(key)
            .fetch_optional(&mut **tx)
            .await?;
    let receipt_row =
        sqlx::query_as("SELECT id,payload FROM business_receipts WHERE run_id=? AND company_key=?")
            .bind(id)
            .bind(key)
            .fetch_optional(&mut **tx)
            .await?;
    let approval_json = sqlx::query_scalar(
        "SELECT payload FROM business_decisions WHERE run_id=? AND company_key=?",
    )
    .bind(id)
    .bind(key)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(RunEvidenceRows {
        ticket_row,
        receipt_row,
        approval_json,
    })
}

impl BusinessService {
    pub async fn report(&self, key: &str, id: &str) -> Result<Report, AppError> {
        let company = self.company(key)?;
        Self::valid_run_id(id)?;
        let mut tx = self.pool(&company).await?.begin().await?;
        let (run, snapshot) = Self::load_run(&mut tx, key, id).await?;
        let evidence = fetch_run_evidence(&mut tx, key, id).await?;
        tx.commit().await?;

        let ticket: Option<Ticket> = evidence
            .ticket_row
            .as_ref()
            .and_then(|(_, v)| serde_json::from_str(v).ok());
        let receipt: Option<Receipt> = evidence
            .receipt_row
            .as_ref()
            .and_then(|(_, v)| serde_json::from_str(v).ok());
        let approval: Option<Approval> = evidence
            .approval_json
            .as_deref()
            .and_then(|v| serde_json::from_str(v).ok());

        let approval_matches = approval
            .as_ref()
            .is_some_and(|v| v.decision == Decision::Approve)
            && approval == run.approval;
        let ticket_matches =
            check_ticket_match(ticket.as_ref(), &run, key, evidence.ticket_row.as_ref());
        let receipt_matches = check_receipt_match(
            receipt.as_ref(),
            ticket.as_ref(),
            &run,
            &snapshot,
            evidence.receipt_row.as_ref(),
        );
        let approved_fresh = approval
            .as_ref()
            .and_then(|v| timestamp(&v.decided_at))
            .is_some_and(|t| fresh(&snapshot.offering, t));

        let sop_matches = sops::matches(&run, &snapshot.sop)?;
        let facts_match = verify_facts_and_evidence(&run, &snapshot, approval_matches)?;

        let v_params = VerificationParams {
            run: &run,
            snapshot: &snapshot,
            approval_matches,
            ticket_matches,
            receipt_matches,
            approved_fresh,
            sop_matches,
            facts_match,
        };
        let checks = assemble_verification_checks(&v_params);
        let verified = checks.iter().all(|c| c.passed);
        let sop_evaluation =
            build_sop_evaluation(&v_params, approval.as_ref(), &evidence, verified);

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
