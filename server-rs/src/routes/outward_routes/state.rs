//! @docs ARCHITECTURE:OutwardGateway
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / outward_routes::state
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[OutwardRoutes]`

use super::rate_limit::IpRateLimiter;
use crate::agent::outward::{CustomerCatalog, OutwardGateway};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Persisted FAQ / Policy Info Card for outward knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistedInfoCard {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub updated_at: String,
}

/// Persisted state for outward company agent card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedOutwardState {
    pub business_name: String,
    pub description: String,
    pub model_profile: String,
    pub skills: Vec<crate::agent::outward::A2aSkill>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub hours: Option<String>,
    #[serde(default)]
    pub support_email: Option<String>,
    #[serde(default)]
    pub support_phone: Option<String>,
    #[serde(default)]
    pub return_policy: Option<String>,
    #[serde(default)]
    pub info_cards: Vec<PersistedInfoCard>,
}

/// Standard outward JSON response envelope.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

/// Shared Outward Gateway Service State.
#[derive(Clone)]
pub struct OutwardAppState {
    pub gateway: Arc<Mutex<OutwardGateway>>,
    pub catalog: Arc<Mutex<CustomerCatalog>>,
    pub info_cards: Arc<Mutex<Vec<PersistedInfoCard>>>,
    pub rate_limiter: IpRateLimiter,
    pub storage_path: Option<PathBuf>,
    /// Serializes profile read-persist-commit transactions so concurrent PUTs cannot diverge disk vs memory.
    pub profile_write_lock: Arc<tokio::sync::Mutex<()>>,
}

impl OutwardAppState {
    #[cfg(test)]
    pub fn new(business_name: &str) -> Self {
        Self::new_with_storage(business_name, None)
    }

    pub fn new_with_storage(business_name: &str, storage_path: Option<PathBuf>) -> Self {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
        let card_url = std::env::var("TADPOLE_AGENT_CARD_URL").unwrap_or_else(|_| {
            format!("http://localhost:{}/a2a/v1/company-agent-card.json", port)
        });

        let mut gateway = OutwardGateway::new(business_name, card_url);
        let mut catalog = CustomerCatalog::new(business_name);
        let mut info_cards = Vec::new();

        if let Some(persisted) = storage_path.as_deref().and_then(load_persisted_state_sync) {
            apply_persisted_to_gateway(&mut gateway, &persisted);
            catalog.business_name = persisted.business_name;
            catalog.default_model_profile = persisted.model_profile;
            info_cards = persisted.info_cards;
            tracing::info!(
                "[OutwardRoutes] Loaded persisted outward profile from {:?}",
                storage_path
            );
        }

        Self {
            gateway: Arc::new(Mutex::new(gateway)),
            catalog: Arc::new(Mutex::new(catalog)),
            info_cards: Arc::new(Mutex::new(info_cards)),
            rate_limiter: IpRateLimiter::new(60, 60),
            storage_path,
            profile_write_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }
}

fn load_persisted_state_sync(path: &Path) -> Option<PersistedOutwardState> {
    if !path.exists() {
        return None;
    }
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            tracing::error!(
                "[OutwardRoutes] Failed to read outward state file at {:?}: {}. Using default profile.",
                path,
                e
            );
            return None;
        }
    };
    match serde_json::from_str::<PersistedOutwardState>(&content) {
        Ok(persisted) => Some(persisted),
        Err(e) => {
            tracing::error!(
                "[OutwardRoutes] Failed to deserialize outward state file at {:?}: {}. Using default profile.",
                path,
                e
            );
            None
        }
    }
}

pub(super) fn apply_persisted_to_gateway(
    gateway: &mut OutwardGateway,
    persisted: &PersistedOutwardState,
) {
    gateway.update_business_profile(&persisted.business_name, &persisted.description);
    if let Err(e) = gateway.set_model_profile(&persisted.model_profile) {
        tracing::warn!(
            "[OutwardRoutes] Failed to apply persisted model profile: {}",
            e
        );
    }
    gateway.update_skills(persisted.skills.clone());
    if persisted.address.is_some() || persisted.hours.is_some() {
        gateway.update_hours_and_location(persisted.address.clone(), persisted.hours.clone());
    }
    if persisted.support_email.is_some() || persisted.support_phone.is_some() {
        gateway.update_support_contact(
            persisted.support_email.clone(),
            persisted.support_phone.clone(),
        );
    }
    if let Some(ref policy) = persisted.return_policy {
        gateway.update_return_policy(Some(policy.clone()));
    }
}

pub(super) fn validate_practice_company_key(company_key: &str) -> Result<(), AppError> {
    if !matches!(company_key, "Mock-1-Product" | "Mock-2-Service") {
        return Err(AppError::BadRequest(format!(
            "Unregistered company_key '{}'; only registered practice companies are permitted",
            company_key
        )));
    }
    Ok(())
}

pub(super) fn company_workspaces_dir(state: &OutwardAppState) -> Option<PathBuf> {
    state
        .storage_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|data_dir| data_dir.join("business").join("workspaces"))
}

pub(super) fn company_outward_path(state: &OutwardAppState, company_key: &str) -> Option<PathBuf> {
    company_workspaces_dir(state)
        .map(|workspaces| workspaces.join(format!("{company_key}-outward.json")))
}

pub(super) fn company_catalog_path(state: &OutwardAppState, company_key: &str) -> PathBuf {
    company_workspaces_dir(state)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("data")
                .join("business")
                .join("workspaces")
        })
        .join(format!("{company_key}-catalog.json"))
}

pub(super) fn company_card_url(company_key: &str) -> String {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    format!(
        "http://localhost:{}/a2a/v1/company-agent-card.json?company_key={}",
        port, company_key
    )
}

/// Atomically writes serialized JSON to `path` via a unique temp file + rename.
pub(super) async fn atomic_write_json<T: Serialize>(
    path: &Path,
    value: &T,
    label: &str,
) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(AppError::Io)?;
    }
    let json_str = serde_json::to_string_pretty(value).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize {}: {}", label, e))
    })?;

    let tmp_path = path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
    if let Err(e) = tokio::fs::write(&tmp_path, json_str.as_bytes()).await {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(AppError::Io(e));
    }
    if let Err(e) = tokio::fs::rename(&tmp_path, path).await {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(AppError::Io(e));
    }
    Ok(())
}
