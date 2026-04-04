//! Infrastructure Adapters — User workspace and bridge framework
//!
//! Provides standardized interfaces for the engine to interact with external 
//! resources (Filesystem, Discord, Vault). Acts as the translation layer 
//! between generic engine intents and provider-specific implementations.
//!
//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Two-Tier Storage**: Note the distinction between `filesystem` (ephemeral 
//! user workspace manipulation) and `vault` (permanent mission-discovery and 
//! engine telemetry storage). Use `vault` for data that must survive 
//! workspace transitions.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Permission denied on workspace paths, Discord webhook rate limits, or Vault decryption errors.
//! - **Telemetry Link**: Search for `[Adapter]` or `[Vault]` in `tracing` logs.
//! - **Trace Scope**: `server-rs::adapter`

pub mod discord;
pub mod filesystem;
#[cfg(test)]
mod tests_filesystem;
pub mod vault;
