//! Utility Foundation & Shared Core Logic - The Tooling Engine
//!
//! Provides the primary primitives for semantic parsing, graph-based RAG, 
//! security hardening, and safe data serialization.
//!
//! @docs ARCHITECTURE:UtilityFoundation
//!
//! ### AI Assist Note
//! **Tooling Synergy**: The `parser` and `graph` modules work in tandem to 
//! provide codebase context, while `security` and `serialization` ensure 
//! that any tool output is safe for external consumption.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: AST parsing failure (unsupported syntax), RAG embedding dimension mismatch, or serialization buffer overflow.
//! - **Telemetry Link**: Search for `[Utils]` or `[RAG]` in `tracing` logs.
//! - **Trace Scope**: `server-rs::utils`

pub mod graph;
pub mod parser;
pub mod security;
pub mod serialization;
