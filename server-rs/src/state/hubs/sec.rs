//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / sec
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::secret_redactor::SecretRedactor;
use crate::security::audit::MerkleAuditTrail;
use crate::security::metering::BudgetGuard;
use crate::security::monitoring::SecurityMonitor;
use crate::security::permissions::PermissionPolicy;
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
    /// Dynamic tool permission and governance policy engine.
    #[allow(dead_code)]
    pub permission_policy: Arc<PermissionPolicy>,
    /// Authentication token for administrative/deploy requests.
    #[allow(dead_code)]
    pub deploy_token: String,
    /// Authentication token for admin/governance write operations.
    pub admin_token: String,
    /// C-03: Pinned oversight operator public key (hex-encoded ed25519).
    /// Loaded from `OVERSIGHT_PUBLIC_KEY` at startup. When `Some`, only this
    /// key is accepted for oversight decision signatures. In production,
    /// this MUST be set — the system will refuse to start otherwise.
    pub oversight_public_key: Option<String>,
}
