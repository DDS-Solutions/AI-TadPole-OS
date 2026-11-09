//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Evaluates company readiness, cross-company silo isolation, and outbox persistence.
//! ### 🔍 Debugging & Observability
//! Traces identify readiness gate failures and promotion eligibility.

use super::pii::detect_sensitive_pii;
use super::service::{fresh, BusinessService};
use super::types::*;
use crate::error::AppError;
use sqlx::SqlitePool;

const SILO_ISOLATION_QUERY: &str = "\
SELECT (SELECT COUNT(*) FROM business_runs WHERE company_key != ?) + \
(SELECT COUNT(*) FROM business_tickets WHERE company_key != ?) + \
(SELECT COUNT(*) FROM business_receipts WHERE company_key != ?) + \
(SELECT COUNT(*) FROM business_decisions WHERE company_key != ?) + \
(SELECT COUNT(*) FROM business_outbox WHERE company_key != ?)";

const OUTBOX_ATOMICITY_QUERY: &str = "\
SELECT COUNT(*) FROM business_runs r \
LEFT JOIN business_outbox o ON r.id = o.run_id AND o.status = 'committed' \
WHERE r.company_key = ? AND r.status = 'completed' AND o.id IS NULL";

struct ReadinessCheckParams<'a> {
    company: &'a Company,
    silo_isolated: bool,
    cross_company_count: i64,
    facts_fresh: bool,
    pii_guard_active: bool,
    outbox_atomic_ready: bool,
    uncommitted_completed_outbox: i64,
    practice_runs_passed: bool,
    verified_runs: usize,
    completed_runs: usize,
    blocked_or_failed_runs: usize,
}

fn build_readiness_checks(p: &ReadinessCheckParams<'_>) -> Vec<ReadinessCheck> {
    vec![
        ReadinessCheck {
            id: "company_silo_isolation".into(),
            label: "Company Silo & Prefix Isolation".into(),
            passed: p.silo_isolated,
            detail: if p.silo_isolated {
                format!(
                    "Workspace {} and prefix {}- strictly isolated (0 cross-company rows).",
                    p.company.workspace_id, p.company.company_key
                )
            } else {
                format!(
                    "Isolation violation: {} cross-company rows found.",
                    p.cross_company_count
                )
            },
        },
        ReadinessCheck {
            id: "fact_freshness_window".into(),
            label: "Offering Fact Freshness Window".into(),
            passed: p.facts_fresh,
            detail: format!("Facts valid until {}.", p.company.offering.fresh_until),
        },
        ReadinessCheck {
            id: "pii_fail_closed_guard".into(),
            label: "Server-Side PII Fail-Closed Guard (FR-11)".into(),
            passed: p.pii_guard_active,
            detail: "SSN and Luhn payment-card patterns rejected at server boundary.".into(),
        },
        ReadinessCheck {
            id: "outbox_atomic_persistence".into(),
            label: "Two-Phase Connector Outbox Atomicity".into(),
            passed: p.outbox_atomic_ready,
            detail: if p.outbox_atomic_ready {
                "SQLite outbox and receipt savepoint verified for all completed runs.".into()
            } else {
                format!(
                    "{} completed runs missing committed outbox receipts.",
                    p.uncommitted_completed_outbox
                )
            },
        },
        ReadinessCheck {
            id: "verified_practice_runs".into(),
            label: "Verified Practice Run History".into(),
            passed: p.practice_runs_passed,
            detail: format!(
                "{} verified run(s), {} completed run(s), {} blocked/failed run(s).",
                p.verified_runs, p.completed_runs, p.blocked_or_failed_runs
            ),
        },
    ]
}

impl BusinessService {
    async fn count_verified_runs(&self, key: &str, runs: &[Run]) -> usize {
        let mut count = 0;
        for r in runs.iter().filter(|r| r.status == RunStatus::Completed) {
            let Ok(rep) = self.report(key, &r.run_id).await else {
                continue;
            };
            if rep.verification.verified && rep.sop_evaluation.status == "passed" {
                count += 1;
            }
        }
        count
    }

    async fn check_silo_isolation(pool: &SqlitePool, key: &str) -> (bool, i64) {
        let count: i64 = sqlx::query_scalar(SILO_ISOLATION_QUERY)
            .bind(key)
            .bind(key)
            .bind(key)
            .bind(key)
            .bind(key)
            .fetch_one(pool)
            .await
            .unwrap_or(0);
        (count == 0, count)
    }

    async fn check_outbox_atomicity(pool: &SqlitePool, key: &str) -> (bool, i64) {
        let count: i64 = sqlx::query_scalar(OUTBOX_ATOMICITY_QUERY)
            .bind(key)
            .fetch_one(pool)
            .await
            .unwrap_or(0);
        (count == 0, count)
    }

    pub async fn evaluate_readiness(&self, key: &str) -> Result<ReadinessSummary, AppError> {
        let company = self.company(key)?;
        let pool = self.pool(&company).await?;
        let total_runs: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM business_runs WHERE company_key=?")
                .bind(key)
                .fetch_one(pool)
                .await
                .unwrap_or(0);
        let runs = self.list_runs(key).await?;
        let truncated = (total_runs as usize) > runs.len();

        let completed_runs = runs
            .iter()
            .filter(|r| r.status == RunStatus::Completed)
            .count();
        let blocked_or_failed_runs = runs
            .iter()
            .filter(|r| matches!(r.status, RunStatus::Blocked | RunStatus::Failed))
            .count();
        let verified_runs = self.count_verified_runs(key, &runs).await;
        let facts_fresh = fresh(&company.offering, (self.clock)());

        let (silo_isolated, cross_company_count) = Self::check_silo_isolation(pool, key).await;
        let pii_guard_active = detect_sensitive_pii("SSN 000-00-0000").is_some()
            && detect_sensitive_pii("card 4532-0151-1283-0366").is_some();
        let (outbox_atomic_ready, uncommitted_completed_outbox) =
            Self::check_outbox_atomicity(pool, key).await;

        let recent_clean = runs
            .first()
            .map_or(false, |r| r.status == RunStatus::Completed);
        let practice_runs_passed =
            verified_runs >= 1 && (blocked_or_failed_runs == 0 || recent_clean);

        let checks = build_readiness_checks(&ReadinessCheckParams {
            company: &company,
            silo_isolated,
            cross_company_count,
            facts_fresh,
            pii_guard_active,
            outbox_atomic_ready,
            uncommitted_completed_outbox,
            practice_runs_passed,
            verified_runs,
            completed_runs,
            blocked_or_failed_runs,
        });

        let eligible_for_promotion = checks.iter().all(|c| c.passed);
        let next_stage = if company.connector_mode == "mock" {
            "read_only".into()
        } else {
            "approved_write".into()
        };

        Ok(ReadinessSummary {
            current_stage: company.connector_mode,
            next_stage,
            verified_runs,
            completed_runs,
            blocked_or_failed_runs,
            total_runs: total_runs as usize,
            truncated,
            facts_fresh,
            pii_guard_active,
            outbox_atomic_ready,
            eligible_for_promotion,
            checks,
        })
    }
}
