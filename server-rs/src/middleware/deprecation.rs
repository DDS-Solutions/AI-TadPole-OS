//! @docs ARCHITECTURE:Interface
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / deprecation
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Behavioral]` Deterministic segment-boundary route matching; longest matching registered route takes precedence.
//!   - enforced_by: `test_deprecation_longest_prefix_precedence`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_deprecation_headers_exact`, `test_deprecation_sibling_unaffected`, `test_deprecation_longest_prefix_precedence`

use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Checks if a request `path` matches a `registered_route` on segment boundaries.
///
/// Returns `true` if:
/// - `path == registered_route` (exact match)
/// - `path` starts with `registered_route` followed immediately by a path segment separator `/`
/// - Matches with or without leading `/v1` prefix (handling route nesting normalization)
///
/// Prevents false positives where `/v1/infra/providers-v2` incorrectly matches `/v1/infra/providers`.
pub fn matches_deprecated_route(path: &str, registered_route: &str) -> bool {
    let check = |p: &str, r: &str| -> bool {
        if p == r {
            return true;
        }
        if let Some(remainder) = p.strip_prefix(r) {
            return remainder.starts_with('/');
        }
        false
    };

    if check(path, registered_route) {
        return true;
    }

    if let Some(stripped_path) = path.strip_prefix("/v1") {
        if check(stripped_path, registered_route) {
            return true;
        }
    }

    if let Some(stripped_reg) = registered_route.strip_prefix("/v1") {
        if check(path, stripped_reg) {
            return true;
        }
    }

    false
}

/// Middleware that injects Deprecation and Sunset headers for legacy endpoints.
///
/// Supported Headers:
/// - `Deprecation`: Signals that the endpoint is deprecated (RFC / draft standard boolean "true").
/// - `Sunset`: Signals the timestamp when the endpoint will be removed (RFC 8594 / RFC 1123 format).
/// - `Link`: Related URI pointing to migration/documentation resources (appended to preserve existing links).
pub async fn deprecation_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();

    // Deterministically resolve the longest (most specific) matching deprecated route
    let deprecation_info = {
        let guard = state.governance.deprecated_routes.read();
        guard
            .iter()
            .filter(|(key, _)| matches_deprecated_route(&path, key))
            .max_by_key(|(key, _)| key.len())
            .map(|(_, info)| info.clone())
    };

    let mut response = next.run(req).await;

    if let Some((sunset, link)) = deprecation_info {
        tracing::debug!(
            "⚠️ [Deprecation] Client accessed deprecated endpoint: {}",
            path
        );

        let headers = response.headers_mut();

        // Deprecation: Boolean (true)
        headers.insert("Deprecation", HeaderValue::from_static("true"));

        // Sunset: Date when the endpoint is expected to be REMOVED (RFC 1123 / RFC 8594)
        // Example: Fri, 01 Jan 2027 23:59:59 GMT
        if let Ok(sunset_val) = HeaderValue::from_str(&sunset) {
            headers.insert("Sunset", sunset_val);
        }

        // Link: Link to documentation about the transition (Appended to preserve existing Link headers)
        if let Ok(link_val) = HeaderValue::from_str(&link) {
            headers.append("Link", link_val);
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    async fn dummy_handler() -> StatusCode {
        StatusCode::OK
    }

    #[test]
    fn test_matches_deprecated_route() {
        assert!(matches_deprecated_route(
            "/v1/infra/providers",
            "/v1/infra/providers"
        ));
        assert!(matches_deprecated_route(
            "/v1/infra/providers/sub",
            "/v1/infra/providers"
        ));
        assert!(!matches_deprecated_route(
            "/v1/infra/providers-v2",
            "/v1/infra/providers"
        ));
        assert!(!matches_deprecated_route(
            "/api/v1/infra/providers",
            "/v1/infra/providers"
        ));
    }

    #[tokio::test]
    async fn test_deprecation_headers_exact() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/v1/infra/providers", get(dummy_handler))
            .route("/v1/healthy", get(dummy_handler))
            .layer(axum::middleware::from_fn_with_state(
                state,
                deprecation_middleware,
            ));

        // 1. Deprecated route should have headers
        let req = Request::builder()
            .uri("/v1/infra/providers")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().contains_key("Deprecation"));
        assert!(res.headers().contains_key("Sunset"));

        // 2. Normal route should NOT have headers
        let req = Request::builder()
            .uri("/v1/healthy")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(!res.headers().contains_key("Deprecation"));
    }

    #[tokio::test]
    async fn test_deprecation_sibling_unaffected() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let app = Router::new()
            .route("/v1/infra/providers-v2", get(dummy_handler))
            .layer(axum::middleware::from_fn_with_state(
                state,
                deprecation_middleware,
            ));

        // Sibling path should NOT match /v1/infra/providers
        let req = Request::builder()
            .uri("/v1/infra/providers-v2")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(!res.headers().contains_key("Deprecation"));
    }

    #[tokio::test]
    async fn test_deprecation_longest_prefix_precedence() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        {
            let mut guard = state.governance.deprecated_routes.write();
            guard.insert(
                "/v1/test".to_string(),
                (
                    "Sat, 01 Jan 2028 00:00:00 GMT".to_string(),
                    "<https://docs.example.com/test>; rel=\"deprecation\"".to_string(),
                ),
            );
            guard.insert(
                "/v1/test/specific".to_string(),
                (
                    "Sun, 01 Jan 2030 00:00:00 GMT".to_string(),
                    "<https://docs.example.com/specific>; rel=\"deprecation\"".to_string(),
                ),
            );
        }

        let app = Router::new()
            .route("/v1/test/specific", get(dummy_handler))
            .layer(axum::middleware::from_fn_with_state(
                state,
                deprecation_middleware,
            ));

        let req = Request::builder()
            .uri("/v1/test/specific")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let sunset = res.headers().get("Sunset").unwrap().to_str().unwrap();
        assert_eq!(
            sunset, "Sun, 01 Jan 2030 00:00:00 GMT",
            "Should choose the most specific route's sunset date"
        );
    }
}
