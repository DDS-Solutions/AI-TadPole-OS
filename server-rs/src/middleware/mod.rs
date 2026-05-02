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
pub mod security_headers;

use axum::{body::Body, http::Request};
use std::net::SocketAddr;

/// Utility to extract the client IP address, respecting proxy headers.
///
/// ### 🛰️ Proxy Awareness
/// Resolves the client IP by checking the following in order:
/// 1. `CF-Connecting-IP` (Cloudflare)
/// 2. `X-Forwarded-For` (Standard Proxy - takes the first IP)
/// 3. `ConnectInfo<SocketAddr>` (Direct connection)
///
/// Falls back to "unknown" if no IP can be resolved.
pub fn extract_client_ip(req: &Request<Body>) -> String {
    // 1. Check Cloudflare specific header
    if let Some(ip) = req
        .headers()
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
    {
        return ip.trim().to_string();
    }

    // 2. Check X-Forwarded-For (Standard Proxy)
    if let Some(forwarded) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        // X-Forwarded-For can be a comma-separated list; the first one is the client
        if let Some(ip) = forwarded.split(',').next() {
            return ip.trim().to_string();
        }
    }

    // 3. Fallback to direct connection info
    req.extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|addr| addr.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

// Metadata: [mod]

// Metadata: [mod]
