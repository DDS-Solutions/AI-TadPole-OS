//! @docs ARCHITECTURE:OutwardGateway
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / outward_routes::tests
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::catalog::{
    import_catalog_handler, search_catalog_handler, ImportCatalogPayload, SearchQueryParams,
};
use super::profile::{update_profile_handler, UpdateProfilePayload};
use super::rate_limit::{IpRateLimiter, MAX_TRACKED_IPS};
use super::state::{OutwardAppState, PersistedInfoCard};
use crate::error::AppError;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};

#[test]
fn test_ip_rate_limiter_enforcement() {
    let limiter = IpRateLimiter::new(3, 60); // Allow max 3 requests
    let ip = "10.0.0.1";

    assert!(limiter.check_and_record(ip)); // 1
    assert!(limiter.check_and_record(ip)); // 2
    assert!(limiter.check_and_record(ip)); // 3
    assert!(!limiter.check_and_record(ip)); // 4th blocked
}

#[test]
fn test_ip_rate_limiter_zero_limit() {
    let limiter = IpRateLimiter::new(0, 60);
    assert!(!limiter.check_and_record("127.0.0.1"));
}

#[test]
fn test_ip_rate_limiter_memory_cleanup_and_hard_cap() {
    let limiter = IpRateLimiter::new(60, 60);
    // Insert 12,000 distinct IPs into rate limiter map
    for i in 0..12_000 {
        limiter.check_and_record(&format!("192.168.{}.{}", i / 256, i % 256));
    }

    // Memory DoS Protection: map must strictly not exceed MAX_TRACKED_IPS
    let map_len = limiter.records.lock().unwrap().len();
    assert!(
        map_len <= MAX_TRACKED_IPS,
        "Limiter map len {} exceeded MAX_TRACKED_IPS {}",
        map_len,
        MAX_TRACKED_IPS
    );
}

#[tokio::test]
async fn test_import_rejects_malformed_payload_without_catalog_mutation() {
    let state = OutwardAppState::new("Import Validation Test");
    let result = import_catalog_handler(
        State(state.clone()),
        Json(ImportCatalogPayload {
            csv_content: None,
            qb_json_content: Some("{not-json}".to_string()),
            company_key: None,
        }),
    )
    .await;

    assert!(matches!(result, Err(AppError::BadRequest(_))));
    assert!(state.catalog.lock().unwrap().items.is_empty());
}

#[tokio::test]
async fn test_profile_persistence_and_reload() {
    let temp_file =
        std::env::temp_dir().join(format!("test_outward_{}.json", uuid::Uuid::new_v4()));
    let state = OutwardAppState::new_with_storage("Initial SMB", Some(temp_file.clone()));

    let payload = UpdateProfilePayload {
        company_key: None,
        business_name: Some("Updated SMB Hardware".to_string()),
        description: Some("Durable description".to_string()),
        model_profile: Some("gemma4:e4b".to_string()),
        skills: None,
        address: Some("42 Harbor Road".to_string()),
        hours: Some("Mon-Sat 8-8".to_string()),
        support_email: Some("help@smb.io".to_string()),
        support_phone: None,
        return_policy: Some("30 days refund".to_string()),
        info_cards: Some(vec![PersistedInfoCard {
            id: "card-1".to_string(),
            title: "Return Policy".to_string(),
            content: "30 days refund".to_string(),
            category: "policy".to_string(),
            tags: vec!["returns".to_string()],
            updated_at: "2026-04-12T10:00:00Z".to_string(),
        }]),
    };

    let res = update_profile_handler(State(state), Json(payload)).await;
    assert!(res.is_ok());
    assert!(temp_file.exists());

    // Reload fresh state from disk
    let reloaded = OutwardAppState::new_with_storage("Default Fallback", Some(temp_file.clone()));
    let gateway = reloaded.gateway.lock().unwrap();
    assert_eq!(gateway.get_agent_card().name, "Updated SMB Hardware");
    assert_eq!(gateway.profile().description, "Durable description");
    assert_eq!(gateway.profile().address.as_deref(), Some("42 Harbor Road"));
    assert_eq!(gateway.profile().hours.as_deref(), Some("Mon-Sat 8-8"));
    assert_eq!(
        gateway.profile().support_email.as_deref(),
        Some("help@smb.io")
    );
    let cards = reloaded.info_cards.lock().unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].title, "Return Policy");

    let _ = std::fs::remove_file(temp_file);
}

#[tokio::test]
async fn test_profile_partial_update_preserves_merged_fields_in_memory_and_disk() {
    let temp_file =
        std::env::temp_dir().join(format!("test_outward_merge_{}.json", uuid::Uuid::new_v4()));
    let state = OutwardAppState::new_with_storage("Merge SMB", Some(temp_file.clone()));

    let init_payload = UpdateProfilePayload {
        company_key: None,
        business_name: Some("Merge Test Corp".to_string()),
        description: Some("Base description".to_string()),
        model_profile: Some("gemma4:e4b".to_string()),
        skills: None,
        address: Some("123 Main Street".to_string()),
        hours: Some("9am - 5pm".to_string()),
        support_email: Some("contact@mergetest.com".to_string()),
        support_phone: Some("+1-555-0199".to_string()),
        return_policy: Some("14 days".to_string()),
        info_cards: None,
    };
    let res1 = update_profile_handler(State(state.clone()), Json(init_payload)).await;
    assert!(res1.is_ok());

    let partial_payload = UpdateProfilePayload {
        company_key: None,
        business_name: None,
        description: None,
        model_profile: None,
        skills: None,
        address: Some("456 New Road".to_string()),
        hours: None,
        support_email: Some("newemail@mergetest.com".to_string()),
        support_phone: None,
        return_policy: None,
        info_cards: None,
    };
    let res2 = update_profile_handler(State(state.clone()), Json(partial_payload)).await;
    assert!(res2.is_ok());

    {
        let gw = state.gateway.lock().unwrap();
        let prof = gw.profile();
        assert_eq!(prof.address.as_deref(), Some("456 New Road"));
        assert_eq!(
            prof.hours.as_deref(),
            Some("9am - 5pm"),
            "In-memory hours must not be wiped out by partial address update"
        );
        assert_eq!(
            prof.support_email.as_deref(),
            Some("newemail@mergetest.com")
        );
        assert_eq!(
            prof.support_phone.as_deref(),
            Some("+1-555-0199"),
            "In-memory phone must not be wiped out by partial email update"
        );
    }

    let reloaded = OutwardAppState::new_with_storage("Fallback", Some(temp_file.clone()));
    {
        let gw = reloaded.gateway.lock().unwrap();
        let prof = gw.profile();
        assert_eq!(prof.address.as_deref(), Some("456 New Road"));
        assert_eq!(prof.hours.as_deref(), Some("9am - 5pm"));
        assert_eq!(
            prof.support_email.as_deref(),
            Some("newemail@mergetest.com")
        );
        assert_eq!(prof.support_phone.as_deref(), Some("+1-555-0199"));
    }

    let _ = std::fs::remove_file(temp_file);
}

#[tokio::test]
async fn test_update_profile_rejects_oversized_fields() {
    let state = OutwardAppState::new("Oversized Validation SMB");

    let bad_name_payload = UpdateProfilePayload {
        company_key: None,
        business_name: Some("A".repeat(300)),
        description: None,
        model_profile: None,
        skills: None,
        address: None,
        hours: None,
        support_email: None,
        support_phone: None,
        return_policy: None,
        info_cards: None,
    };
    let res = update_profile_handler(State(state.clone()), Json(bad_name_payload)).await;
    assert!(matches!(res, Err(AppError::BadRequest(_))));

    let bad_card_payload = UpdateProfilePayload {
        company_key: None,
        business_name: None,
        description: None,
        model_profile: None,
        skills: None,
        address: None,
        hours: None,
        support_email: None,
        support_phone: None,
        return_policy: None,
        info_cards: Some(vec![PersistedInfoCard {
            id: "card-huge".to_string(),
            title: "Valid Title".to_string(),
            content: "X".repeat(20_000),
            category: "policy".to_string(),
            tags: vec![],
            updated_at: "".to_string(),
        }]),
    };
    let res2 = update_profile_handler(State(state), Json(bad_card_payload)).await;
    assert!(matches!(res2, Err(AppError::BadRequest(_))));
}

#[tokio::test]
async fn test_import_response_honesty_on_ingest_errors() {
    let state = OutwardAppState::new("Honesty Test SMB");
    let csv_with_bad_price = "title,category,price,description\nGood Item,General,19.99,Desc\nBad Item,General,not-a-number,Desc\n";

    let payload = ImportCatalogPayload {
        csv_content: Some(csv_with_bad_price.to_string()),
        qb_json_content: None,
        company_key: None,
    };
    let res = import_catalog_handler(State(state), Json(payload)).await;
    assert!(res.is_ok());
    let response = res.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_company_scoped_catalog_import_prevents_cross_company_contamination() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_path = temp_dir.path().join("company_agent_card.json");
    let state = OutwardAppState::new_with_storage("Isolation SMB", Some(storage_path));

    let csv_product =
        "title,category,price,description\nWidget Alpha,Hardware,49.99,Product only\n";
    let res1 = import_catalog_handler(
        State(state.clone()),
        Json(ImportCatalogPayload {
            csv_content: Some(csv_product.to_string()),
            qb_json_content: None,
            company_key: Some("Mock-1-Product".to_string()),
        }),
    )
    .await;
    assert!(res1.is_ok());

    let csv_service =
        "title,category,price,description\nAudit Retainer,Consulting,199.00,Service only\n";
    let res2 = import_catalog_handler(
        State(state.clone()),
        Json(ImportCatalogPayload {
            csv_content: Some(csv_service.to_string()),
            qb_json_content: None,
            company_key: Some("Mock-2-Service".to_string()),
        }),
    )
    .await;
    assert!(res2.is_ok());

    assert!(state.catalog.lock().unwrap().items.is_empty());

    let workspaces_dir = temp_dir.path().join("business").join("workspaces");
    let svc_json =
        std::fs::read_to_string(workspaces_dir.join("Mock-2-Service-catalog.json")).unwrap();
    assert!(svc_json.contains("Audit Retainer"));
    assert!(
        !svc_json.contains("Widget Alpha"),
        "Mock-2-Service catalog must never contain Mock-1-Product items"
    );
}

#[tokio::test]
async fn test_company_scoped_catalog_search_returns_imported_items() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_path = temp_dir
        .path()
        .join("outward")
        .join("company_agent_card.json");
    std::fs::create_dir_all(storage_path.parent().unwrap()).unwrap();
    let root_card = temp_dir.path().join("company_agent_card.json");
    let state = OutwardAppState::new_with_storage("Search Isolation SMB", Some(root_card));

    let csv_product =
        "title,category,price,description\nTitanium Wrench,Tools,34.50,Precision shop wrench\n";
    import_catalog_handler(
        State(state.clone()),
        Json(ImportCatalogPayload {
            csv_content: Some(csv_product.to_string()),
            qb_json_content: None,
            company_key: Some("Mock-1-Product".to_string()),
        }),
    )
    .await
    .expect("import should succeed");

    let req = axum::http::Request::builder()
        .uri("/a2a/v1/catalog/search?company_key=Mock-1-Product&q=Titanium")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = search_catalog_handler(
        State(state.clone()),
        Query(SearchQueryParams {
            q: Some("Titanium".to_string()),
            limit: Some(10),
            company_key: Some("Mock-1-Product".to_string()),
        }),
        req,
    )
    .await
    .expect("search should succeed");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let items = body["data"].as_array().expect("data array");
    assert_eq!(
        items.len(),
        1,
        "expected imported company catalog item in search results, got {:?}",
        body
    );
    assert!(items[0]["title"]
        .as_str()
        .unwrap_or_default()
        .contains("Titanium Wrench"));
}
