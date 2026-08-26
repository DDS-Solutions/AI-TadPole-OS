//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / templates / catalog
//! - **Primary Entrypoints**: `fetch_catalog`, `fetch_catalog_from_url`, `get_offline_catalog`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Bounded timeout with guaranteed offline fallback on network or parse failure.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none (gracefully falls back to offline catalog)
//! - **Telemetry Targets**: `[Templates]`
//! - **Witness Tests**: `routes::templates::catalog::tests::*`

use super::dto::TemplateCatalogEntry;
use std::time::Duration;

pub const REMOTE_CATALOG_URL: &str =
    "https://raw.githubusercontent.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates/main/index.json";
pub const CATALOG_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn fetch_catalog_from_url(
    http_client: &reqwest::Client,
    catalog_url: &str,
) -> Vec<TemplateCatalogEntry> {
    tracing::info!("📥 [Templates] Fetching template catalog from {}", catalog_url);

    let catalog_fut = async {
        let res = http_client
            .get(catalog_url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Remote catalog returned HTTP status {}", res.status()));
        }

        res.json::<Vec<TemplateCatalogEntry>>()
            .await
            .map_err(|e| format!("Failed to parse remote catalog JSON: {}", e))
    };

    match tokio::time::timeout(CATALOG_FETCH_TIMEOUT, catalog_fut).await {
        Ok(Ok(entries)) => entries,
        Ok(Err(err_msg)) => {
            tracing::warn!("⚠️ [Templates] Remote catalog error: {}, serving offline fallback", err_msg);
            get_offline_catalog()
        }
        Err(_) => {
            tracing::warn!(
                "⚠️ [Templates] Remote catalog fetch timed out after {:?}, serving offline fallback",
                CATALOG_FETCH_TIMEOUT
            );
            get_offline_catalog()
        }
    }
}

pub async fn fetch_catalog(http_client: &reqwest::Client) -> Vec<TemplateCatalogEntry> {
    fetch_catalog_from_url(http_client, REMOTE_CATALOG_URL).await
}

pub fn get_offline_catalog() -> Vec<TemplateCatalogEntry> {
    vec![TemplateCatalogEntry {
        id: "marketing-swarm".to_string(),
        name: "Marketing Swarm (Offline Fallback)".to_string(),
        description: "Offline fallback for the marketing automation swarm. Run locally."
            .to_string(),
        repository_url:
            "https://github.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates.git"
                .to_string(),
        path: "templates/marketing".to_string(),
        required_models: vec!["stealth/ox-alpha".to_string()],
        required_skills: vec!["web_search".to_string(), "write_file".to_string()],
    }]
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_get_offline_catalog_has_fallback_entry() {
        let catalog = get_offline_catalog();
        assert!(!catalog.is_empty());
        assert_eq!(catalog[0].id, "marketing-swarm");
        assert!(!catalog[0].required_models.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_catalog_falls_back_on_invalid_url() {
        let client = reqwest::Client::new();
        let catalog = fetch_catalog_from_url(&client, "http://127.0.0.1:9/non_existent_url.json").await;
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog[0].id, "marketing-swarm");
    }
}
