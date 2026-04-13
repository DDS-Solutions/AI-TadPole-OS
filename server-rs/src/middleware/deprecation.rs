//! @docs ARCHITECTURE:Interface
//! 
//! ### AI Assist Note
//! **Deprecation Handler**: Orchestrates the communication of API lifecycle 
//! transitions. Injects **Sunset** and **Deprecation** headers into 
//! legacy responses to facilitate graceful migration. Enforces **RFC 1123 
//! Compliance** for date-based headers, ensuring that automated LLM-based 
//! clients can parse remaining support windows (DEP-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Missing lifecycle headers in legacy responses or 
//!   malformed date strings preventing client-side expiration logic.
//! - **Trace Scope**: `server-rs::middleware::deprecation`

use axum::{
    body::Body,
    http::{Request, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Middleware that injects Deprecation and Sunset headers for legacy endpoints.
/// 
/// Supported Headers:
/// - `Deprecation`: Signals that the endpoint is deprecated.
/// - `Sunset`: Signals the timestamp when the endpoint will be removed (RFC 1123).
pub async fn deprecation_middleware(req: Request<Body>, next: Next) -> Response {
    // Check for deprecated routes
    // Currently targeting /v1/infra/providers as a test case for swarm-wide migration.
    let is_deprecated = req.uri().path().contains("/infra/providers");
    let path = if is_deprecated { Some(req.uri().path().to_string()) } else { None };

    let mut response = next.run(req).await;

    if let Some(path) = path {
        tracing::warn!("⚠️ [Deprecation] Client accessed deprecated endpoint: {}", path);
        
        let headers = response.headers_mut();
        
        // Deprecation: Boolean (true) or Date
        headers.insert("Deprecation", HeaderValue::from_static("true"));
        
        // Sunset: Date when the endpoint is expected to be REMOVED (RFC 1123)
        // Example: Fri, 01 Jan 2027 23:59:59 GMT
        headers.insert("Sunset", HeaderValue::from_static("Fri, 01 Jan 2027 23:59:59 GMT"));
        
        // Link: Link to documentation about the transition (Optional but recommended)
        headers.insert("Link", HeaderValue::from_static("<https://docs.tadpole.so/api/v2/providers>; rel=\"alternate\""));
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;
    use axum::http::StatusCode;

    async fn dummy_handler() -> StatusCode {
        StatusCode::OK
    }

    #[tokio::test]
    async fn test_deprecation_headers() {
        let app = Router::new()
            .route("/v1/infra/providers", get(dummy_handler))
            .route("/v1/healthy", get(dummy_handler))
            .layer(axum::middleware::from_fn(deprecation_middleware));

        // 1. Deprecated route should have headers
        let req = Request::builder().uri("/v1/infra/providers").body(Body::empty()).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().contains_key("Deprecation"));
        assert!(res.headers().contains_key("Sunset"));

        // 2. Normal route should NOT have headers
        let req = Request::builder().uri("/v1/healthy").body(Body::empty()).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(!res.headers().contains_key("Deprecation"));
    }
}
