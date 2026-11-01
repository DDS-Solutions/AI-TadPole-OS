//! @docs ARCHITECTURE:Networking
//! ### AI Assist Note
//! Test the production router and authentication boundary with isolated company databases.
//! ### 🔍 Debugging & Observability
//! Assertions inspect HTTP results and cross-company rejection without printing credentials.

use crate::{router::create_router, state::AppState, types::SubsystemStatus};
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

async fn call(
    app: &Router,
    token: Option<&str>,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");
    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    let mut request = request
        .body(
            body.map(|value| Body::from(value.to_string()))
                .unwrap_or_else(Body::empty),
        )
        .unwrap();
    request
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(std::net::SocketAddr::from((
            [127, 97, 88, 63],
            41001,
        ))));
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn authenticated_business_routes_preserve_company_and_sop_boundary() {
    let state = Arc::new(AppState::new_minimal_mock().await);
    for subsystem in [
        "Database",
        "Agents",
        "MCP",
        "Heartbeat",
        "SecurityEviction",
        "PrivacyGuard",
        "BudgetFlush",
        "Network",
        "SwarmPulse",
        "CodeGraph",
        "CodeGraphDbRefresh",
    ] {
        state
            .resources
            .set_subsystem_status(subsystem, SubsystemStatus::Ready);
    }
    let app = create_router(state.clone());
    let token = Some(state.security.deploy_token.as_str());
    let base = "/v1/business/companies";
    assert_eq!(
        call(&app, None, "GET", base, None).await.0,
        StatusCode::UNAUTHORIZED
    );
    let (status, listed) = call(&app, token, "GET", base, None).await;
    assert_eq!(status, StatusCode::OK);
    let companies = listed["companies"].as_array().unwrap();
    assert_eq!(companies.len(), 2);
    let product = companies
        .iter()
        .find(|company| company["company_key"] == "Mock-1-Product")
        .unwrap();
    let service = companies
        .iter()
        .find(|company| company["company_key"] == "Mock-2-Service")
        .unwrap();
    assert_ne!(product["workspace_id"], service["workspace_id"]);
    assert_eq!(
        product["workflow_definition_id"],
        service["workflow_definition_id"]
    );
    assert_ne!(product["sop"]["id"], service["sop"]["id"]);
    assert!(product["sop"]["document"]
        .as_str()
        .unwrap()
        .contains("Mock-1-Product-Owner"));
    assert_eq!(
        call(&app, token, "GET", &format!("{base}/mock-1-product"), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let product_path = format!("{base}/Mock-1-Product/runs");
    let service_path = format!("{base}/Mock-2-Service/runs");
    let body = json!({"idempotency_key":"router-case-1","customer_reference":product["customer"]["display_reference"],"offering_reference":product["offering"]["display_reference"],"enquiry":"What are the listed price and policy?"});
    assert_eq!(
        call(&app, None, "POST", &product_path, Some(body.clone()))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let mut forged = body.clone();
    forged["company_key"] = json!("Mock-2-Service");
    assert_eq!(
        call(&app, token, "POST", &product_path, Some(forged))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(&app, token, "POST", &service_path, Some(body.clone()))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    let (first, retry) = tokio::join!(
        call(&app, token, "POST", &product_path, Some(body.clone())),
        call(&app, token, "POST", &product_path, Some(body.clone()))
    );
    assert!(matches!(
        (first.0, retry.0),
        (StatusCode::CREATED, StatusCode::OK) | (StatusCode::OK, StatusCode::CREATED)
    ));
    let run = first.1;
    assert_eq!(run["run_id"], retry.1["run_id"]);
    assert_eq!(run["sop_id"], product["sop"]["id"]);
    assert_eq!(run["sop_digest"].as_str().unwrap().len(), 64);
    assert_eq!(run["status"], "pending_approval");
    let id = run["run_id"].as_str().unwrap();
    let path = format!("{product_path}/{id}");
    let other_path = format!("{service_path}/{id}");
    assert_eq!(
        call(&app, token, "GET", &other_path, None).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, token, "GET", &format!("{other_path}/report"), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let approve = json!({"expected_version":1,"decision":"approve"});
    assert_eq!(
        call(
            &app,
            token,
            "POST",
            &format!("{other_path}/decision"),
            Some(approve.clone())
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let mut changed = body;
    changed["enquiry"] = json!("Changed request");
    assert_eq!(
        call(&app, token, "POST", &product_path, Some(changed))
            .await
            .0,
        StatusCode::CONFLICT
    );
    let report = call(&app, token, "GET", &format!("{path}/report"), None)
        .await
        .1;
    assert_eq!(report["verification"]["verified"], false);
    assert_eq!(report["sop_evaluation"]["status"], "pending");
    let stale = json!({"expected_version":99,"decision":"approve"});
    assert_eq!(
        call(
            &app,
            token,
            "POST",
            &format!("{path}/decision"),
            Some(stale)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(
            &app,
            token,
            "POST",
            &format!("{path}/decision"),
            Some(approve.clone())
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            token,
            "POST",
            &format!("{path}/decision"),
            Some(approve)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let report = call(&app, token, "GET", &format!("{path}/report"), None)
        .await
        .1;
    assert_eq!(report["verification"]["verified"], true);
    assert_eq!(report["sop_evaluation"]["status"], "passed");
    assert_eq!(report["sop_evaluation"]["agent_execution_evaluated"], false);
    let service_body = json!({"idempotency_key":"router-case-1","customer_reference":service["customer"]["display_reference"],"offering_reference":service["offering"]["display_reference"],"enquiry":"What is the service policy?"});
    let (status, run) = call(&app, token, "POST", &service_path, Some(service_body)).await;
    assert_eq!(status, StatusCode::CREATED);
    let service_run = format!("{service_path}/{}", run["run_id"].as_str().unwrap());
    let reject = json!({"expected_version":1,"decision":"reject"});
    assert_eq!(
        call(
            &app,
            token,
            "POST",
            &format!("{service_run}/decision"),
            Some(reject)
        )
        .await
        .0,
        StatusCode::OK
    );
    let report = call(&app, token, "GET", &format!("{service_run}/report"), None)
        .await
        .1;
    assert_eq!(report["run"]["status"], "rejected");
    assert!(report["run"]["ticket"].is_null());
    assert_eq!(report["sop_evaluation"]["status"], "passed");
    assert_eq!(report["verification"]["verified"], false);
    let history = call(&app, token, "GET", &product_path, None).await.1;
    assert_eq!(history["runs"].as_array().unwrap().len(), 1);
}
