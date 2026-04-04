//! Request Filtering & Security Middleware - The Security Pipeline
//!
//! Orchestrates the sequential processing of incoming API requests, 
//! enforcing authentication, CORS policies, rate-limiting, and 
//! transaction tracing across the entire engine.
//!
//! @docs ARCHITECTURE:MiddlewarePipeline
//! @docs OPERATIONS_MANUAL:Security
//!
//! ### AI Assist Note
//! **The Security Pipeline**: All `/v1` traffic is intercepted by these 
//! modules. `auth` handles the Bearer token verification, while 
//! `auth_rate_limit` prevents brute-force attempts on the engine's 
//! recruitment endpoints. Adding a new header requirement? Implement it 
//! in `request_id` to ensure it is propagated to all telemetry spans.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: CORS pre-flight rejection, 401 Unauthorized (Auth header mismatch), or 429 Too Many Requests (Rate limit hit).
//! - **Telemetry Link**: Search for `[Middleware]` or `[Security]` in `tracing` logs.
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
