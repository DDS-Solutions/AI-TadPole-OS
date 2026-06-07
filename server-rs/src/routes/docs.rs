//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **API Documentation (OpenAPI/Knowledge Gateway)**: Orchestrates the
//! discovery and retrieval of system documentation and knowledge base
//! entries for the Tadpole OS engine. Features **Dynamic Knowledge
//! Indexing**: crawls the `src/data/knowledge` directory to provide
//! a categorized list of markdown documents. Implements **Secure
//! Document Fetching**: retrieves specific knowledge entries and the
//! global `OPERATIONS_MANUAL.md` with built-in path sanitization
//! (directory traversal protection). AI agents should use these
//! endpoints to synchronize their internal knowledge graphs with the
//! ground-truth documentation stored on-disk (DOC-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 404 Not Found due to incorrect path resolution
//!   logic in `find_doc_path`, 403 Forbidden for attempted directory
//!   traversal, or file-lock errors during high-frequency reads.
//! - **Telemetry Link**: Search for `📖 Fetching knowledge doc` in
//!   `tracing` logs for document access history.
//! - **Trace Scope**: `server-rs::routes::docs`

use crate::error::AppError;
use axum::{extract::Path as AxumPath, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct DocEntry {
    pub category: String,
    pub name: String,
    pub title: String,
}

/// Helper to find the project root or relevant data directory
fn find_doc_path(relative: &str) -> Option<PathBuf> {
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
                            if filename.ends_with(".md") {
                                let title = filename
                                    .replace(".md", "")
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
        Ok(content) => Ok((StatusCode::OK, content)),
        Err(e) => {
            tracing::error!(
                "❌ Failed to read knowledge doc {:?}: {}",
                safe_path.as_path(),
                e
            );
            Err(AppError::NotFound(format!("Document not found: {}", name)))
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
        Ok(content) => Ok((StatusCode::OK, content)),
        Err(e) => {
            tracing::error!("❌ Failed to read Operations Manual {:?}: {}", path, e);
            Err(AppError::NotFound(
                "Operations Manual not found".to_string(),
            ))
        }
    }
}

// Metadata: [docs]

// Metadata: [docs]
