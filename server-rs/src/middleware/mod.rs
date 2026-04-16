//! @docs ARCHITECTURE:MiddlewarePipeline
//! @docs OPERATIONS_MANUAL:Security
//!
//! ### AI Assist Note
//! **Middleware Hub**: Orchestrates the sequential processing of
//! incoming API requests for the Tadpole OS engine. Enforces the
//! **Security Pipeline**, implementing **Sovereign Authentication**
//! (Bearer token), **Brute-Force Prevention** (Recruitment Rate-Limiting),
//! and **CORS Policy Enforcement**. Coordinates with `request_id` to
//! ensure that all incoming transactions are assigned a unique
//! identifier for end-to-end trace propagation.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: CORS pre-flight rejection (origin mismatch),
//!   401 Unauthorized (invalid `NEURAL_TOKEN`), or 429 Too Many
//!   Requests (Rate-limit exceeded).
//! - **Telemetry Link**: Search for `[Middleware]` or `[Security]` in
//!   `tracing` logs for block/deny events.
//! - **Trace Scope**: `server-rs::middleware`
//!
//! Middleware Hub — Request processing pipeline
//!
//! Orchestrates the layered security and observability middleware
//! for the Axum server ecosystem.
//!
//! @docs ARCHITECTURE:Networking

pub mod auth;

pub mod auth_rate_limit;
pub mod cors;
pub mod deprecation;
pub mod rate_limit;
pub mod request_id;
