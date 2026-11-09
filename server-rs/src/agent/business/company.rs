//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! Manages business companies, catalog offerings, and profile freshness lifecycle.
//! ### 🔍 Debugging & Observability
//! Traces identify company lookup failures and corrupt profile overrides.

use super::service::{decode, timestamp, BusinessService, FIXTURES};
use super::sops;
use super::types::*;
use crate::error::AppError;
use std::path::Path;
use uuid::Uuid;

fn is_valid_catalog_item(item: &Offering, prefix: &str) -> bool {
    item.display_reference.starts_with(prefix)
        && item.source_id.starts_with(prefix)
        && item.title.starts_with(prefix)
        && item.price_minor >= 0
        && timestamp(&item.source_updated_at).is_some()
        && timestamp(&item.fresh_until).is_some()
}

fn load_profile_or_fallback(
    profile_path: &Path,
    fixture: &str,
    key: &str,
) -> Result<Company, AppError> {
    if !profile_path.exists() {
        return decode(fixture);
    }
    match std::fs::read_to_string(profile_path)
        .map_err(|e| e.to_string())
        .and_then(|s| decode(&s).map_err(|e| e.to_string()))
    {
        Ok(c) => Ok(c),
        Err(err) => {
            tracing::warn!(
                target: "business",
                company_key = key,
                path = ?profile_path,
                error = %err,
                "[Business] Corrupt profile override detected; falling back to built-in fixture"
            );
            decode(fixture)
        }
    }
}

fn validate_company_identity(company: &Company, key: &str) -> bool {
    let prefix = format!("{key}-");
    let names = [
        &company.agent_display_name,
        &company.workflow_display_name,
        &company.offering.display_reference,
        &company.offering.title,
        &company.offering.source_id,
        &company.customer.display_reference,
        &company.customer.name,
    ];
    company.company_key == key
        && company.display_name == key
        && company.environment == "mock"
        && matches!(company.connector_mode.as_str(), "mock" | "read_only")
        && company.workflow_definition_id == WORKFLOW_DEFINITION_ID
        && company.workflow_version == WORKFLOW_VERSION
        && names.iter().all(|name| name.starts_with(&prefix))
}

fn validate_company_ids_and_prices(company: &Company) -> bool {
    company.offering.price_minor >= 0
        && company.offering.currency == "USD"
        && Uuid::parse_str(&company.workspace_id).is_ok()
        && Uuid::parse_str(&company.offering.id).is_ok()
        && Uuid::parse_str(&company.customer.id).is_ok()
        && timestamp(&company.offering.source_updated_at).is_some()
        && timestamp(&company.offering.fresh_until).is_some()
}

fn validate_company_fixture(company: &Company, key: &str) -> Result<(), AppError> {
    if !validate_company_identity(company, key) || !validate_company_ids_and_prices(company) {
        return Err(AppError::InternalServerError(
            "Business fixture configuration is invalid".into(),
        ));
    }
    Ok(())
}

impl BusinessService {
    pub fn companies(&self) -> Result<Vec<Company>, AppError> {
        FIXTURES.iter().map(|(key, _)| self.company(key)).collect()
    }

    pub async fn companies_with_readiness(&self) -> Result<Vec<Company>, AppError> {
        let mut result = Vec::with_capacity(FIXTURES.len());
        for (key, _) in FIXTURES {
            result.push(self.company_with_readiness(key).await?);
        }
        Ok(result)
    }

    pub async fn company_with_readiness(&self, key: &str) -> Result<Company, AppError> {
        let mut company = self.company(key)?;
        company.catalog_offerings = self.load_catalog_offerings(&company);
        company.readiness = Some(self.evaluate_readiness(key).await?);
        Ok(company)
    }

    pub fn load_catalog_offerings(&self, company: &Company) -> Vec<Offering> {
        let mut offerings = vec![company.offering.clone()];
        let catalog_path = self
            .root
            .join(format!("{}-catalog.json", company.company_key));
        let prefix = format!("{}-", company.company_key);
        let Ok(content) = std::fs::read_to_string(&catalog_path) else {
            return offerings;
        };
        let Ok(extra) = serde_json::from_str::<Vec<Offering>>(&content) else {
            return offerings;
        };
        for item in extra {
            let not_duplicate = !offerings
                .iter()
                .any(|existing| existing.display_reference == item.display_reference);
            if is_valid_catalog_item(&item, &prefix) && not_duplicate {
                offerings.push(item);
            }
        }
        offerings
    }

    pub fn renew_company_facts(&self, key: &str) -> Result<Company, AppError> {
        let mut company = self.company(key)?;
        let now = (self.clock)();
        company.offering.source_updated_at = now.to_rfc3339();
        company.offering.fresh_until = (now + chrono::Duration::days(90)).to_rfc3339();
        company.catalog_offerings.clear();
        company.readiness = None;
        std::fs::create_dir_all(&self.root)?;
        let profile_path = self.root.join(format!("{key}-profile.json"));
        let temp_path = self
            .root
            .join(format!("{key}-profile.json.tmp.{}", Uuid::new_v4()));
        let encoded = serde_json::to_string_pretty(&company).map_err(|_| {
            AppError::InternalServerError("Failed to encode company profile".into())
        })?;
        std::fs::write(&temp_path, &encoded)?;
        if let Err(err) = std::fs::rename(&temp_path, &profile_path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(AppError::InternalServerError(format!(
                "Atomic profile persist failed: {err}"
            )));
        }
        self.company(key)
    }

    pub fn company(&self, key: &str) -> Result<Company, AppError> {
        let (_, fixture) = FIXTURES
            .iter()
            .find(|(allowed, _)| *allowed == key)
            .ok_or_else(|| AppError::NotFound("Business company not found".into()))?;
        let profile_override_path = self.root.join(format!("{key}-profile.json"));
        let mut company = load_profile_or_fallback(&profile_override_path, fixture, key)?;
        company.catalog_offerings.clear();
        company.readiness = None;
        sops::attach_document(&mut company)?;
        validate_company_fixture(&company, key)?;
        Ok(company)
    }
}
