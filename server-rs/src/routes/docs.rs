//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / docs
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::NotFound`, `AppError::BadRequest`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `docs::tests::test_knowledge_title_generation`, `docs::tests::test_path_traversal_blocked`

use crate::error::AppError;
use axum::{
    extract::Path as AxumPath,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct DocEntry {
    pub category: String,
    pub name: String,
    pub title: String,
}

/// Helper to find the project root or relevant data directory
pub fn find_doc_path(relative: &str) -> Option<PathBuf> {
    let paths = [
        PathBuf::from(relative),                         // From root
        PathBuf::from("docs").join(relative),            // From root/docs
        PathBuf::from("..").join(relative),              // From server-rs
        PathBuf::from("..").join("docs").join(relative), // From server-rs/docs
    ];

    for p in &paths {
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    None
}

fn markdown_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/markdown; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=300".parse().unwrap(),
    );
    headers
}

/// GET /v1/docs/knowledge
/// Returns a list of all available knowledge base documents.
#[tracing::instrument(name = "docs::list_knowledge")]
pub async fn list_knowledge_docs() -> Result<impl IntoResponse, AppError> {
    let mut entries = Vec::new();
    let knowledge_path = match find_doc_path("src/data/knowledge") {
        Some(p) => p,
        None => {
            return Ok((StatusCode::NOT_FOUND, Json(entries)).into_response());
        }
    };

    if let Ok(mut categories) = tokio::fs::read_dir(knowledge_path).await {
        while let Ok(Some(category_entry)) = categories.next_entry().await {
            let category_name = category_entry.file_name().to_string_lossy().to_string();
            let category_path = category_entry.path();
            if let Ok(metadata) = tokio::fs::metadata(&category_path).await {
                if metadata.is_dir() {
                    if let Ok(mut files) = tokio::fs::read_dir(category_path).await {
                        while let Ok(Some(file_entry)) = files.next_entry().await {
                            let filename = file_entry.file_name().to_string_lossy().to_string();
                            if let Some(stem) = filename.strip_suffix(".md") {
                                let title = stem
                                    .split('-')
                                    .map(|s| {
                                        let mut c = s.chars();
                                        match c.next() {
                                            None => String::new(),
                                            Some(f) => {
                                                f.to_uppercase().collect::<String>() + c.as_str()
                                            }
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" ");

                                entries.push(DocEntry {
                                    category: category_name.clone(),
                                    name: filename,
                                    title,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Json(entries).into_response())
}

/// GET /v1/docs/knowledge/:category/:name
#[tracing::instrument(name = "docs::get_knowledge_entry")]
pub async fn get_knowledge_doc(
    AxumPath((category, name)): AxumPath<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let base_path = match find_doc_path("src/data/knowledge") {
        Some(p) => p,
        None => {
            return Err(AppError::NotFound(
                "Knowledge base directory not found".to_string(),
            ));
        }
    };

    // Sanitize input to prevent directory traversal using validate_path
    let relative_path = format!("{}/{}", category, name);
    let safe_path = crate::utils::security::validate_path(&base_path, &relative_path)?;
    tracing::debug!("📖 Fetching knowledge doc: {:?}", safe_path.as_path());

    match tokio::fs::read_to_string(safe_path.as_path()).await {
        Ok(content) => Ok((StatusCode::OK, markdown_headers(), content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(AppError::NotFound(format!("Document not found: {}", name)))
        }
        Err(e) => {
            tracing::error!(
                "❌ Failed to read knowledge doc {:?}: {}",
                safe_path.as_path(),
                e
            );
            Err(AppError::InternalServerError(format!(
                "Failed to read document: {}",
                e
            )))
        }
    }
}

/// GET /v1/docs/operations-manual
#[tracing::instrument(name = "docs::get_ops_manual")]
pub async fn get_operations_manual() -> Result<impl IntoResponse, AppError> {
    let path = match find_doc_path("OPERATIONS_MANUAL.md") {
        Some(p) => p,
        None => {
            tracing::error!("❌ OPERATIONS_MANUAL.md not found in root or current dir");
            return Err(AppError::NotFound(
                "Operations Manual not found".to_string(),
            ));
        }
    };

    tracing::debug!("📖 Fetching Operations Manual: {:?}", path);

    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Ok((StatusCode::OK, markdown_headers(), content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(AppError::NotFound(
            "Operations Manual not found".to_string(),
        )),
        Err(e) => {
            tracing::error!("❌ Failed to read Operations Manual {:?}: {}", path, e);
            Err(AppError::InternalServerError(format!(
                "Failed to read Operations Manual: {}",
                e
            )))
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_knowledge_title_generation() {
        let filename = "system-architecture-overview.md";
        let stem = filename.strip_suffix(".md").unwrap();
        let title = stem
            .split('-')
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        assert_eq!(title, "System Architecture Overview");
    }

    #[test]
    fn test_path_traversal_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().to_path_buf();
        let traversal = "../../../secret.key";
        assert!(crate::utils::security::validate_path(&base, traversal).is_err());
    }
}
