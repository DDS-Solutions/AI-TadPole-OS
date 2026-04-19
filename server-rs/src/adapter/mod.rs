//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Infrastructure Adapters**: Orchestrates the standardized interfaces
//! for the engine to interact with external resources. Features
//! **Two-Tier Storage Separation**: `filesystem` handles ephemeral
//! workspace manipulation (SCRATCH-01), while `vault` manages permanent
//! mission-discovery and engine telemetry that must survive system
//! restarts. Acts as the **Sovereign Translation Layer** between high-level
//! engine intents and provider-specific implementations (Discord, FS).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Permission denied on restricted workspace paths,
//!   Discord webhook rate limits (429), or Vault decryption failure
//!   due to corrupted metadata.
//! - **Telemetry Link**: Search for `[Adapter]` or `[Vault]` in `tracing`
//!   logs for bridge lifecycle events.
//! - **Trace Scope**: `server-rs::adapter`

pub mod discord;
pub mod filesystem;
#[cfg(test)]
mod tests_filesystem;
pub mod vault;
