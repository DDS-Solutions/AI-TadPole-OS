//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Versioned company SOPs bind role expectations to saved evidence, independently of agent narratives.
//! ### 🔍 Debugging & Observability
//! SOP reports distinguish pending, blocked and compliant rejection from completed customer outcomes.

use super::types::*;
use crate::error::AppError;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

const CHECKS: [&str; 6] = [
    "company_scope",
    "source_facts",
    "prepare_response",
    "approval_gate",
    "record_once",
    "review_evidence",
];

pub(super) fn attach_document(company: &mut Company) -> Result<(), AppError> {
    company.sop.document = match company.company_key.as_str() {
        "Mock-1-Product" => include_str!(
            "../../../../starter_kits/mock_smb/sops/Mock-1-Product-Customer-Enquiry-SOP-v1.md"
        ),
        "Mock-2-Service" => include_str!(
            "../../../../starter_kits/mock_smb/sops/Mock-2-Service-Customer-Enquiry-SOP-v1.md"
        ),
        _ => return Err(AppError::NotFound("Business SOP not found".into())),
    }
    .into();
    let key = &company.company_key;
    let expected_id = format!("{key}-Customer-Enquiry-SOP");
    let checks: HashSet<_> = company
        .sop
        .steps
        .iter()
        .map(|step| step.check.as_str())
        .collect();
    let ids: HashSet<_> = company
        .sop
        .steps
        .iter()
        .map(|step| step.id.as_str())
        .collect();
    let roles: HashSet<_> = company
        .sop
        .roles
        .iter()
        .map(|role| role.name.as_str())
        .collect();
    let valid_roles = company.sop.roles.len() == 4
        && roles.len() == 4
        && company.sop.roles.iter().all(|role| {
            role.name.starts_with(&format!("{key}-"))
                && role.execution
                    == if role.name == format!("{key}-Owner") {
                        "human_operator"
                    } else {
                        "local_rules"
                    }
        });
    if company.sop.id != expected_id
        || company.sop.version != "1"
        || company.sop.title != expected_id
        || company.sop.document_reference != format!("{expected_id}-v1.md")
        || company.sop.steps.len() != CHECKS.len()
        || ids.len() != CHECKS.len()
        || checks != CHECKS.into_iter().collect::<HashSet<_>>()
        || !valid_roles
        || !company
            .sop
            .steps
            .iter()
            .any(|step| step.check == "approval_gate" && step.owner_role == format!("{key}-Owner"))
        || company.sop.steps.iter().any(|step| {
            !step.id.starts_with(&format!("{key}-SOP-"))
                || !roles.contains(step.owner_role.as_str())
        })
    {
        return Err(AppError::InternalServerError(
            "Business SOP configuration is invalid".into(),
        ));
    }
    Ok(())
}

pub(super) fn digest(sop: &Sop) -> Result<String, AppError> {
    let encoded = serde_json::to_vec(sop)
        .map_err(|_| AppError::InternalServerError("SOP serialization failed".into()))?;
    Ok(hex::encode(Sha256::digest(encoded)))
}

pub(super) fn matches(run: &Run, sop: &Sop) -> Result<bool, AppError> {
    Ok(run.sop_id == sop.id && run.sop_version == sop.version && run.sop_digest == digest(sop)?)
}

pub(super) struct EvaluationEvidence {
    pub context_matches: bool,
    pub facts_match: bool,
    pub approved: bool,
    pub rejected: bool,
    pub decision_consistent: bool,
    pub no_output_records: bool,
    pub verified_outcome: bool,
}

pub(super) fn evaluate(run: &Run, sop: &Sop, evidence: EvaluationEvidence) -> SopEvaluation {
    let pending = run.status == RunStatus::PendingApproval;
    let stopped = matches!(run.status, RunStatus::Blocked | RunStatus::Failed);
    let safe_stop = stopped
        && evidence.no_output_records
        && run
            .blocked_reason
            .as_ref()
            .is_some_and(|reason| !reason.is_empty());
    let gate_passed = if run.status == RunStatus::Completed {
        evidence.approved
    } else if run.status == RunStatus::Rejected {
        evidence.rejected && evidence.no_output_records
    } else {
        evidence.no_output_records && evidence.decision_consistent
    };
    let review_passed = evidence.verified_outcome
        || (run.status == RunStatus::Rejected && gate_passed)
        || safe_stop;
    let steps: Vec<_> = sop
        .steps
        .iter()
        .map(|step| {
            let status = match step.check.as_str() {
                "company_scope" => {
                    if evidence.context_matches {
                        "passed"
                    } else {
                        "failed"
                    }
                }
                "source_facts" | "prepare_response" => {
                    if evidence.facts_match {
                        "passed"
                    } else if safe_stop {
                        "blocked"
                    } else {
                        "failed"
                    }
                }
                "approval_gate" => {
                    if !gate_passed {
                        "failed"
                    } else if pending {
                        "pending"
                    } else if stopped && run.approval.is_none() {
                        "not_applicable"
                    } else {
                        "passed"
                    }
                }
                "record_once" => {
                    if run.status == RunStatus::Completed {
                        if evidence.verified_outcome {
                            "passed"
                        } else {
                            "failed"
                        }
                    } else if !evidence.no_output_records {
                        "failed"
                    } else if pending {
                        "pending"
                    } else {
                        "not_applicable"
                    }
                }
                "review_evidence" => {
                    if pending {
                        "pending"
                    } else if review_passed {
                        "passed"
                    } else {
                        "failed"
                    }
                }
                _ => "failed",
            };
            let assigned_execution = sop
                .roles
                .iter()
                .find(|role| role.name == step.owner_role)
                .map(|role| role.execution.as_str())
                .unwrap_or("unassigned");
            let performed_by = if matches!(status, "pending" | "not_applicable") {
                "not_performed"
            } else {
                assigned_execution
            };
            SopStepEvaluation {
                step_id: step.id.clone(),
                title: step.title.clone(),
                owner_role: step.owner_role.clone(),
                performed_by: performed_by.into(),
                status: status.into(),
            }
        })
        .collect();
    let status = if steps.iter().any(|step| step.status == "failed") {
        "failed"
    } else if pending {
        "pending"
    } else if stopped {
        "blocked"
    } else {
        "passed"
    };
    SopEvaluation {
        sop_id: run.sop_id.clone(),
        sop_version: run.sop_version.clone(),
        sop_digest: run.sop_digest.clone(),
        status: status.into(),
        scope: "local_workflow_and_human_approval".into(),
        agent_execution_evaluated: false,
        semantic_response_quality_evaluated: false,
        steps,
    }
}
