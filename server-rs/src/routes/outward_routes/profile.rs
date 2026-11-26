//! @docs ARCHITECTURE:OutwardGateway
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / outward_routes::profile
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::state::{
    atomic_write_json, company_card_url, company_outward_path, validate_practice_company_key,
    ApiResponse, OutwardAppState, PersistedInfoCard, PersistedOutwardState,
};
use crate::agent::outward::{A2aAgentCard, ModelProfile, OutwardGateway};
use crate::error::AppError;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;

/// Length limits on incoming profile fields to prevent memory bloat.
const MAX_BUSINESS_NAME_LEN: usize = 256;
const MAX_DESCRIPTION_LEN: usize = 4096;
const MAX_MODEL_PROFILE_LEN: usize = 64;
const MAX_CONTACT_FIELD_LEN: usize = 1024;
const MAX_INFO_CARD_COUNT: usize = 100;
const MAX_INFO_CARD_TITLE_LEN: usize = 256;
const MAX_INFO_CARD_CONTENT_LEN: usize = 16384;

#[derive(Debug, Deserialize)]
pub struct UpdateProfilePayload {
    pub business_name: Option<String>,
    pub description: Option<String>,
    pub model_profile: Option<String>,
    pub skills: Option<Vec<crate::agent::outward::A2aSkill>>,
    pub address: Option<String>,
    pub hours: Option<String>,
    pub support_email: Option<String>,
    pub support_phone: Option<String>,
    pub return_policy: Option<String>,
    #[serde(default)]
    pub info_cards: Option<Vec<PersistedInfoCard>>,
    #[serde(default)]
    pub company_key: Option<String>,
}

/// PUT /a2a/v1/profile
/// Updates the outward business profile, model profile, advertised skills, and persisted FAQ/Policy info_cards.
/// @docs API_REFERENCE:UpdateOutwardProfile
pub async fn update_profile_handler(
    State(state): State<OutwardAppState>,
    Json(mut payload): Json<UpdateProfilePayload>,
) -> Result<Response, AppError> {
    validate_profile_payload(&payload)?;
    let stamped_cards = stamp_info_cards(payload.info_cards.take());

    // Serialize profile read -> atomic disk write -> memory commit to prevent divergence races
    let _write_guard = state.profile_write_lock.lock().await;

    let (message, card) = match payload.company_key.take() {
        Some(company_key) => {
            let card = update_company_profile(&state, &company_key, payload, stamped_cards).await?;
            (
                format!("Company-scoped Outward Gateway profile updated for {company_key}"),
                card,
            )
        }
        None => {
            let card = update_global_profile(&state, payload, stamped_cards).await?;
            (
                "Outward Gateway profile updated successfully".to_string(),
                card,
            )
        }
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message,
            data: Some(card),
        }),
    )
        .into_response())
}

fn check_max_len(field_name: &str, value: &Option<String>, max_len: usize) -> Result<(), AppError> {
    if let Some(v) = value {
        if v.len() > max_len {
            return Err(AppError::BadRequest(format!(
                "{field_name} exceeds maximum length of {max_len} characters"
            )));
        }
    }
    Ok(())
}

fn validate_profile_payload(payload: &UpdateProfilePayload) -> Result<(), AppError> {
    check_max_len(
        "business_name",
        &payload.business_name,
        MAX_BUSINESS_NAME_LEN,
    )?;
    check_max_len("description", &payload.description, MAX_DESCRIPTION_LEN)?;
    check_max_len(
        "model_profile",
        &payload.model_profile,
        MAX_MODEL_PROFILE_LEN,
    )?;

    if let Some(ref mprofile) = payload.model_profile {
        ModelProfile::parse(mprofile)?;
    }

    for (field_name, val) in [
        ("address", &payload.address),
        ("hours", &payload.hours),
        ("support_email", &payload.support_email),
        ("support_phone", &payload.support_phone),
        ("return_policy", &payload.return_policy),
    ] {
        check_max_len(field_name, val, MAX_CONTACT_FIELD_LEN)?;
    }

    validate_info_cards(payload.info_cards.as_deref())
}

fn validate_info_cards(cards: Option<&[PersistedInfoCard]>) -> Result<(), AppError> {
    let Some(cards) = cards else {
        return Ok(());
    };
    if cards.len() > MAX_INFO_CARD_COUNT {
        return Err(AppError::BadRequest(format!(
            "Cannot persist more than {} outward info cards",
            MAX_INFO_CARD_COUNT
        )));
    }
    for card in cards {
        if card.title.len() > MAX_INFO_CARD_TITLE_LEN {
            return Err(AppError::BadRequest(format!(
                "info card title exceeds maximum length of {} characters",
                MAX_INFO_CARD_TITLE_LEN
            )));
        }
        if card.content.len() > MAX_INFO_CARD_CONTENT_LEN {
            return Err(AppError::BadRequest(format!(
                "info card content exceeds maximum length of {} characters",
                MAX_INFO_CARD_CONTENT_LEN
            )));
        }
    }
    Ok(())
}

fn stamp_info_cards(cards: Option<Vec<PersistedInfoCard>>) -> Option<Vec<PersistedInfoCard>> {
    let mut cards = cards?;
    let now_str = chrono::Utc::now().to_rfc3339();
    for card in &mut cards {
        if card.updated_at.is_empty() {
            card.updated_at = now_str.clone();
        }
    }
    Some(cards)
}

fn merge_profile_state(
    existing: PersistedOutwardState,
    payload: UpdateProfilePayload,
    stamped_cards: Option<Vec<PersistedInfoCard>>,
) -> PersistedOutwardState {
    PersistedOutwardState {
        business_name: payload.business_name.unwrap_or(existing.business_name),
        description: payload.description.unwrap_or(existing.description),
        model_profile: payload.model_profile.unwrap_or(existing.model_profile),
        skills: payload.skills.unwrap_or(existing.skills),
        address: payload.address.or(existing.address),
        hours: payload.hours.or(existing.hours),
        support_email: payload.support_email.or(existing.support_email),
        support_phone: payload.support_phone.or(existing.support_phone),
        return_policy: payload.return_policy.or(existing.return_policy),
        info_cards: stamped_cards.unwrap_or(existing.info_cards),
    }
}

fn apply_merged_to_gateway(
    gateway: &mut OutwardGateway,
    merged: &PersistedOutwardState,
) -> Result<A2aAgentCard, AppError> {
    gateway.update_business_profile(&merged.business_name, &merged.description);
    gateway.update_hours_and_location(merged.address.clone(), merged.hours.clone());
    gateway.update_support_contact(merged.support_email.clone(), merged.support_phone.clone());
    gateway.update_return_policy(merged.return_policy.clone());
    gateway.set_model_profile(&merged.model_profile)?;
    gateway.update_skills(merged.skills.clone());
    Ok(gateway.get_agent_card().clone())
}

async fn update_company_profile(
    state: &OutwardAppState,
    company_key: &str,
    payload: UpdateProfilePayload,
    stamped_cards: Option<Vec<PersistedInfoCard>>,
) -> Result<A2aAgentCard, AppError> {
    validate_practice_company_key(company_key)?;

    let path_opt = company_outward_path(state, company_key);
    let existing = load_or_default_company_state(path_opt.as_deref(), company_key).await;
    let merged = merge_profile_state(existing, payload, stamped_cards);

    if let Some(ref path) = path_opt {
        atomic_write_json(path, &merged, "company outward profile").await?;
    }

    let mut gw = OutwardGateway::new(&merged.business_name, company_card_url(company_key));
    apply_merged_to_gateway(&mut gw, &merged)
}

async fn load_or_default_company_state(
    path: Option<&std::path::Path>,
    company_key: &str,
) -> PersistedOutwardState {
    if let Some(path) = path {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            if let Ok(state) = serde_json::from_str::<PersistedOutwardState>(&content) {
                return state;
            }
        }
    }
    PersistedOutwardState {
        business_name: company_key.to_string(),
        description: String::new(),
        model_profile: "gemma4:e4b".to_string(),
        skills: Vec::new(),
        address: None,
        hours: None,
        support_email: None,
        support_phone: None,
        return_policy: None,
        info_cards: Vec::new(),
    }
}

async fn update_global_profile(
    state: &OutwardAppState,
    payload: UpdateProfilePayload,
    stamped_cards: Option<Vec<PersistedInfoCard>>,
) -> Result<A2aAgentCard, AppError> {
    let current_state = snapshot_global_state(state)?;
    let merged = merge_profile_state(current_state, payload, stamped_cards);

    if let Some(ref storage_path) = state.storage_path {
        atomic_write_json(storage_path, &merged, "outward profile").await?;
    }

    commit_global_state(state, &merged)
}

fn snapshot_global_state(state: &OutwardAppState) -> Result<PersistedOutwardState, AppError> {
    let gw = state.gateway.lock().map_err(|_| {
        AppError::InternalServerError("Outward gateway state lock is poisoned".to_string())
    })?;
    let prof = gw.profile();
    let card = gw.get_agent_card();
    let cards = state.info_cards.lock().map_err(|_| {
        AppError::InternalServerError("Outward info cards state lock is poisoned".to_string())
    })?;

    Ok(PersistedOutwardState {
        business_name: prof.name.clone(),
        description: prof.description.clone(),
        model_profile: gw.get_model_profile().to_string(),
        skills: card.skills.clone(),
        address: prof.address.clone(),
        hours: prof.hours.clone(),
        support_email: prof.support_email.clone(),
        support_phone: prof.support_phone.clone(),
        return_policy: prof.return_policy.clone(),
        info_cards: cards.clone(),
    })
}

fn commit_global_state(
    state: &OutwardAppState,
    merged: &PersistedOutwardState,
) -> Result<A2aAgentCard, AppError> {
    {
        let mut stored_cards = state.info_cards.lock().map_err(|_| {
            AppError::InternalServerError("Outward info cards state lock is poisoned".to_string())
        })?;
        *stored_cards = merged.info_cards.clone();
    }

    let card = {
        let mut gateway = state.gateway.lock().map_err(|_| {
            AppError::InternalServerError("Outward gateway state lock is poisoned".to_string())
        })?;
        apply_merged_to_gateway(&mut gateway, merged)?
    };

    {
        let mut catalog = state.catalog.lock().map_err(|_| {
            AppError::InternalServerError("Customer catalog state lock is poisoned".to_string())
        })?;
        catalog.business_name = merged.business_name.clone();
        catalog.default_model_profile = merged.model_profile.clone();
    }

    Ok(card)
}
