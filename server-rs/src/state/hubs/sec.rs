//! Security Hub — Tamper-evident auditing and preventative safety checks
//!
//! Manages the Merkle-hash audit trail, budget metering, 
//! shell safety scanning, and runtime secret redaction.
//!
//! @docs ARCHITECTURE:State

use std::sync::Arc;
use crate::security::audit::MerkleAuditTrail;
use crate::security::metering::BudgetGuard;
use crate::security::scanner::ShellScanner;
use crate::secret_redactor::SecretRedactor;
use crate::security::monitoring::SecurityMonitor;

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
