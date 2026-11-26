//! @docs ARCHITECTURE:OutwardGateway
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / outward_routes::agent_card
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::state::{
    company_card_url, company_outward_path, validate_practice_company_key, OutwardAppState,
    PersistedInfoCard, PersistedOutwardState,
};
use crate::agent::outward::{A2aAgentCard, OutwardGateway};
use crate::error::AppError;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct AgentCardQueryParams {
    #[serde(default)]
    pub company_key: Option<String>,
}

/// GET /a2a/v1/company-agent-card.json
/// Returns the public A2A agent card. No bearer token is required; requests are IP rate limited.
/// @docs API_REFERENCE:GetOutwardAgentCard
pub async fn get_agent_card_handler(
    State(state): State<OutwardAppState>,
    Query(params): Query<AgentCardQueryParams>,
    req: axum::extract::Request,
) -> Result<Response, AppError> {
    let client_ip = crate::middleware::extract_client_ip(&req);
    if !state.rate_limiter.check_and_record(&client_ip) {
        return Err(AppError::RateLimit(
            "Capped at 60 requests per minute".to_string(),
        ));
    }

    let (card, model_profile, info_cards, company_key) = match params.company_key {
        Some(ref key) => {
            validate_practice_company_key(key)?;
            let (card, profile, cards) = load_company_agent_card(&state, key).await;
            (card, profile, cards, Some(key.as_str()))
        }
        None => {
            let (card, profile, cards) = load_global_agent_card(&state)?;
            (card, profile, cards, None)
        }
    };

    let value = serialize_agent_card(&card, model_profile, info_cards, company_key)?;
    Ok((StatusCode::OK, Json(value)).into_response())
}

async fn load_company_agent_card(
    state: &OutwardAppState,
    company_key: &str,
) -> (A2aAgentCard, String, Vec<PersistedInfoCard>) {
    let mut gw = OutwardGateway::new(company_key, company_card_url(company_key));
    let mut info_cards = Vec::new();

    if let Some(persisted) = read_company_persisted_state(state, company_key).await {
        gw.update_business_profile(&persisted.business_name, &persisted.description);
        let _ = gw.set_model_profile(&persisted.model_profile);
        gw.update_skills(persisted.skills);
        gw.update_hours_and_location(persisted.address, persisted.hours);
        gw.update_support_contact(persisted.support_email, persisted.support_phone);
        gw.update_return_policy(persisted.return_policy);
        info_cards = persisted.info_cards;
    }

    (
        gw.get_agent_card().clone(),
        gw.get_model_profile().to_string(),
        info_cards,
    )
}

async fn read_company_persisted_state(
    state: &OutwardAppState,
    company_key: &str,
) -> Option<PersistedOutwardState> {
    let path = company_outward_path(state, company_key)?;
    let content = tokio::fs::read_to_string(&path).await.ok()?;
    serde_json::from_str::<PersistedOutwardState>(&content).ok()
}

fn load_global_agent_card(
    state: &OutwardAppState,
) -> Result<(A2aAgentCard, String, Vec<PersistedInfoCard>), AppError> {
    let (card, model_profile) = {
        let gateway = state.gateway.lock().map_err(|_| {
            AppError::InternalServerError("Outward gateway state lock is poisoned".to_string())
        })?;
        (
            gateway.get_agent_card().clone(),
            gateway.get_model_profile().to_string(),
        )
    };
    let info_cards = state
        .info_cards
        .lock()
        .map_err(|_| {
            AppError::InternalServerError("Outward info cards state lock is poisoned".to_string())
        })?
        .clone();
    Ok((card, model_profile, info_cards))
}

fn serialize_agent_card(
    card: &A2aAgentCard,
    model_profile: String,
    info_cards: Vec<PersistedInfoCard>,
    company_key: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut value = serde_json::to_value(card).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize agent card: {}", e))
    })?;

    if let Some(obj) = value.as_object_mut() {
        if let Some(key) = company_key {
            obj.insert(
                "company_key".to_string(),
                serde_json::Value::String(key.to_string()),
            );
        }
        obj.insert(
            "model_profile".to_string(),
            serde_json::Value::String(model_profile),
        );
        obj.insert(
            "info_cards".to_string(),
            serde_json::to_value(info_cards).unwrap_or(serde_json::Value::Array(Vec::new())),
        );
    }
    Ok(value)
}
