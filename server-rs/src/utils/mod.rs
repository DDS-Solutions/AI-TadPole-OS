//! @docs ARCHITECTURE:UtilityFoundation
//!
//! ### AI Assist Note
//! **Utility Foundation**: Orchestrates the primary primitives for
//! semantic parsing, graph-based RAG, security hardening, and safe
//! data serialization. Features the **Tooling Engine** synergy: `parser`
//! and `graph` modules work in tandem to provide codebase context and
//! topological maps, while `security` and `serialization` ensure that
//! all tool outputs are redacted and safe for external telemetry
//! consumption (UTIL-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: AST parsing failures (unsupported syntax),
//!   RAG embedding dimension mismatches, or serialization buffer
//!   overflows during high-volume state synchronization.
//! - **Telemetry Link**: Search for `[Utils]` or `[RAG]` in `tracing`
//!   logs for primitive execution benchmarks.
//! - **Trace Scope**: `server-rs::utils`

pub mod graph;
pub mod parser;
pub mod security;
pub mod serialization;
pub mod data_weighting;

// Metadata: [mod]

// Metadata: [mod]
