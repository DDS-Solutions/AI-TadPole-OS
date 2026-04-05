//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **Defense Core**: Manages tamper-evident auditing and preventative safety
//! checks. Orchestrates the **Merkle Hash-Chain** audit trail,
//! **Budget Metering**, **Shell Safety Scanning**, and **Runtime Secret
//! Redaction**. Ensures operational integrity by enforcing **Hard-Coded
//! Security Boundaries** across all agentic activities.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Merkle chain integrity breach (tampered logs), budget
//!   quota exhaustion blocking valid missions, or shell scanner
//!   false-positives on sanitized command strings.
//! - **Trace Scope**: `server-rs::state::hubs::sec`

use crate::secret_redactor::SecretRedactor;
use crate::security::audit::MerkleAuditTrail;
use crate::security::metering::BudgetGuard;
use crate::security::monitoring::SecurityMonitor;
use crate::security::scanner::ShellScanner;
use std::sync::Arc;

/// Hub for tamper-evident auditing and preventative security checks.
pub struct SecurityHub {
    /// Tamper-evident audit trail engine (Merkle Hash Chain).
    #[allow(dead_code)]
    pub audit_trail: Arc<MerkleAuditTrail>,
    /// Persistent budget governance and metering engine.
    #[allow(dead_code)]
    pub budget_guard: Arc<BudgetGuard>,
    /// Proactive shell safety scanner (API key leak protection).
    #[allow(dead_code)]
    pub shell_scanner: Arc<ShellScanner>,
    /// Runtime secret redactor for logs and telemetry.
    #[allow(dead_code)]
    pub secret_redactor: Arc<SecretRedactor>,
    /// System resource and environment monitor.
    #[allow(dead_code)]
    pub system_monitor: Arc<SecurityMonitor>,
    /// Authentication token for administrative/deploy requests.
    #[allow(dead_code)]
    pub deploy_token: String,
}
