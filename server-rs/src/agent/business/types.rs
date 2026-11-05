//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Company-bound customer enquiry contracts shared by mock and future certified connectors.
//! ### 🔍 Debugging & Observability
//! Business events carry company_key, workspace_id and run_id; no credentials are represented here.

use serde::{Deserialize, Serialize};

pub const WORKFLOW_DEFINITION_ID: &str = "customer_enquiry";
pub const WORKFLOW_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BusinessProfile {
    pub description: String,
    pub hours: String,
    pub support_email: String,
    pub policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Offering {
    pub id: String,
    pub display_reference: String,
    pub title: String,
    pub description: String,
    pub price_minor: i64,
    pub currency: String,
    pub source_id: String,
    pub source_updated_at: String,
    pub fresh_until: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Customer {
    pub id: String,
    pub display_reference: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SopRole {
    pub name: String,
    pub execution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SopStep {
    pub id: String,
    pub title: String,
    pub owner_role: String,
    pub check: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sop {
    pub id: String,
    pub version: String,
    pub title: String,
    pub document_reference: String,
    #[serde(default)]
    pub document: String,
    pub roles: Vec<SopRole>,
    pub steps: Vec<SopStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Company {
    pub company_key: String,
    pub workspace_id: String,
    pub environment: String,
    pub display_name: String,
    pub company_kind: String,
    pub fixture_version: String,
    pub configuration_version: String,
    pub workflow_definition_id: String,
    pub workflow_version: String,
    pub connector_mode: String,
    pub agent_display_name: String,
    pub workflow_display_name: String,
    pub profile: BusinessProfile,
    pub offering: Offering,
    pub customer: Customer,
    pub sop: Sop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CreateRun {
    pub idempotency_key: String,
    pub customer_reference: String,
    pub enquiry: String,
    pub offering_reference: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Approve,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecideRun {
    pub expected_version: i64,
    pub decision: Decision,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    PendingApproval,
    Completed,
    Rejected,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    pub source_id: String,
    pub source_updated_at: String,
    pub fresh_until: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Approval {
    pub decision: Decision,
    pub decided_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ticket {
    pub id: String,
    pub display_reference: String,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub id: String,
    pub display_reference: String,
    pub operation: String,
    pub status: String,
    pub external_record_reference: String,
    pub evidence_location: String,
    pub company_key: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Run {
    pub run_id: String,
    pub display_reference: String,
    pub company_key: String,
    pub workspace_id: String,
    pub environment: String,
    pub configuration_version: String,
    pub fixture_version: String,
    pub workflow_definition_id: String,
    pub workflow_version: String,
    pub sop_id: String,
    pub sop_version: String,
    pub sop_digest: String,
    pub connector_mode: String,
    pub version: i64,
    pub status: RunStatus,
    pub customer_reference: String,
    pub offering_reference: String,
    pub enquiry: String,
    pub draft_response: String,
    pub evidence: Vec<Evidence>,
    pub approval: Option<Approval>,
    pub ticket: Option<Ticket>,
    pub receipt: Option<Receipt>,
    pub created_at: String,
    pub updated_at: String,
    pub blocked_reason: Option<String>,
    pub latency_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub verified: bool,
    pub checks: Vec<VerificationCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    pub provider_charges_usd: f64,
    pub estimated_usd: f64,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopStepEvaluation {
    pub step_id: String,
    pub title: String,
    pub owner_role: String,
    pub performed_by: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopEvaluation {
    pub sop_id: String,
    pub sop_version: String,
    pub sop_digest: String,
    pub status: String,
    pub scope: String,
    pub agent_execution_evaluated: bool,
    pub semantic_response_quality_evaluated: bool,
    pub steps: Vec<SopStepEvaluation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub report_display_reference: String,
    pub run: Run,
    pub verification: Verification,
    pub sop_evaluation: SopEvaluation,
    pub latency_ms: i64,
    pub cost: Cost,
}
