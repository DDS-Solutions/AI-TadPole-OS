//! @docs ARCHITECTURE:Agent
//! ### AI Assist Note
//! Atomic disk persistence, size-bounded deserialization, and structural invariant validation.
//! ### 🔍 Debugging & Observability
//! Enforces atomic file swaps, 50MB memory DoS guards, and load-time sanitization.

use std::fs;
use std::io::Write;
use std::path::Path;
use tracing::info;

use super::super::CatalogLimits;
use super::types::{is_allowed_stored_metadata_key, sanitize_catalog_value, CatalogItem};
use super::CustomerCatalog;
use crate::error::AppError;

const MAX_CATALOG_FILE_BYTES: u64 = 50 * 1024 * 1024; // 50 MB

pub fn validate_catalog_item(
    idx: usize,
    item: &CatalogItem,
    limits: &CatalogLimits,
) -> Result<(), AppError> {
    if item.title.trim().is_empty() {
        return Err(AppError::BadRequest(format!(
            "Catalog item at index {idx} has an empty title"
        )));
    }
    if item.title.chars().count() > limits.max_title_len {
        return Err(AppError::BadRequest(format!(
            "Catalog item at index {idx} title exceeds max length of {}",
            limits.max_title_len
        )));
    }
    if item.category.chars().count() > limits.max_category_len {
        return Err(AppError::BadRequest(format!(
            "Catalog item at index {idx} category exceeds max length of {}",
            limits.max_category_len
        )));
    }
    if item.description.chars().count() > limits.max_description_len {
        return Err(AppError::BadRequest(format!(
            "Catalog item at index {idx} description exceeds max length of {}",
            limits.max_description_len
        )));
    }
    if let Some(price) = item.price_usd {
        if !price.is_finite() || price < 0.0 || (price == 0.0 && price.is_sign_negative()) {
            return Err(AppError::BadRequest(format!(
                "Catalog item at index {idx} contains an invalid price: {price}"
            )));
        }
    }
    Ok(())
}

pub fn validate_catalog(catalog: &CustomerCatalog, limits: &CatalogLimits) -> Result<(), AppError> {
    if catalog.items.len() > limits.max_items {
        return Err(AppError::BadRequest(format!(
            "Catalog exceeds maximum item capacity ({} > {})",
            catalog.items.len(),
            limits.max_items
        )));
    }

    for (idx, item) in catalog.items.iter().enumerate() {
        validate_catalog_item(idx, item, limits)?;
    }

    Ok(())
}

pub fn save_catalog_to_file(catalog: &CustomerCatalog, path: &Path) -> std::io::Result<()> {
    info!(
        "[CustomerCatalog] Saving catalog atomically to file: {:?}",
        path
    );
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let content = serde_json::to_string_pretty(catalog)?;

    let mut temp_file = tempfile::Builder::new()
        .prefix("catalog_")
        .suffix(".tmp")
        .tempfile_in(parent)?;

    temp_file.write_all(content.as_bytes())?;
    temp_file.as_file().sync_all()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        let _ = std::fs::set_permissions(temp_file.path(), perms);
    }

    temp_file.persist(path).map_err(|e| e.error)?;
    Ok(())
}

fn sanitize_loaded_catalog(catalog: &mut CustomerCatalog, limits: &CatalogLimits) {
    catalog.business_name =
        sanitize_catalog_value(&catalog.business_name, limits.max_business_name_len);
    for item in &mut catalog.items {
        item.id = sanitize_catalog_value(&item.id, 128);
        item.title = sanitize_catalog_value(&item.title, limits.max_title_len);
        item.category = sanitize_catalog_value(&item.category, limits.max_category_len);
        item.description = sanitize_catalog_value(&item.description, limits.max_description_len);
        item.metadata
            .retain(|key, _| is_allowed_stored_metadata_key(key));
        item.metadata.retain(|_, value| {
            *value = sanitize_catalog_value(value, limits.max_metadata_value_len);
            !value.is_empty()
        });
    }
}

pub fn load_catalog_from_file(path: &Path) -> Result<CustomerCatalog, AppError> {
    info!("[CustomerCatalog] Loading catalog from file: {:?}", path);
    let metadata = fs::metadata(path)
        .map_err(|e| AppError::BadRequest(format!("Failed to read catalog file metadata: {e}")))?;

    if metadata.len() > MAX_CATALOG_FILE_BYTES {
        return Err(AppError::BadRequest(
            "Catalog file exceeds maximum permitted size of 50 MB".to_string(),
        ));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| AppError::BadRequest(format!("Failed to read catalog file content: {e}")))?;

    let mut catalog: CustomerCatalog = serde_json::from_str(&content)
        .map_err(|e| AppError::BadRequest(format!("Invalid catalog JSON schema: {e}")))?;

    let original_catalog = serde_json::to_value(&catalog).map_err(|e| {
        AppError::InternalServerError(format!(
            "Failed to snapshot catalog before sanitization: {e}"
        ))
    })?;

    let limits = CatalogLimits::default();
    validate_catalog(&catalog, &limits)?;
    sanitize_loaded_catalog(&mut catalog, &limits);
    validate_catalog(&catalog, &limits)?;
    catalog.rebuild_index();

    let sanitized_catalog = serde_json::to_value(&catalog).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize sanitized catalog: {e}"))
    })?;

    if original_catalog != sanitized_catalog {
        save_catalog_to_file(&catalog, path).map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to persist sanitized catalog; refusing to load unsanitized data: {e}"
            ))
        })?;
    }

    Ok(catalog)
}
