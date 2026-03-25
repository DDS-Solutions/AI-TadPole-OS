use axum::{extract::Path as AxumPath, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
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
pub async fn list_knowledge_docs() -> impl IntoResponse {
    let mut entries = Vec::new();
    let knowledge_path = match find_doc_path("src/data/knowledge") {
        Some(p) => p,
        None => {
            tracing::error!("❌ Knowledge base path not found (tried src/data/knowledge and ../src/data/knowledge)");
            return (StatusCode::NOT_FOUND, Json(entries)).into_response();
        }
    };

    if let Ok(categories) = fs::read_dir(knowledge_path) {
        for category_entry in categories.flatten() {
            let category_name = category_entry.file_name().to_string_lossy().to_string();
            if category_entry.path().is_dir() {
                if let Ok(files) = fs::read_dir(category_entry.path()) {
                    for file_entry in files.flatten() {
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

    (StatusCode::OK, Json(entries)).into_response()
}

/// GET /v1/docs/knowledge/:category/:name
pub async fn get_knowledge_doc(
    AxumPath((category, name)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    // Sanitize input to prevent directory traversal
    if category.contains("..") || name.contains("..") {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    let base_path = match find_doc_path("src/data/knowledge") {
        Some(p) => p,
        None => {
            return (StatusCode::NOT_FOUND, "Knowledge base directory not found").into_response()
        }
    };

    let path = base_path.join(category).join(name);
    tracing::debug!("📖 Fetching knowledge doc: {:?}", path);

    match fs::read_to_string(&path) {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(e) => {
            tracing::error!("❌ Failed to read knowledge doc {:?}: {}", path, e);
            (StatusCode::NOT_FOUND, "Document not found").into_response()
        }
    }
}

/// GET /v1/docs/operations-manual
pub async fn get_operations_manual() -> impl IntoResponse {
    let path = match find_doc_path("OPERATIONS_MANUAL.md") {
        Some(p) => p,
        None => {
            tracing::error!("❌ OPERATIONS_MANUAL.md not found in root or current dir");
            return (StatusCode::NOT_FOUND, "Operations Manual not found").into_response();
        }
    };

    tracing::debug!("📖 Fetching Operations Manual: {:?}", path);

    match fs::read_to_string(&path) {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(e) => {
            tracing::error!("❌ Failed to read Operations Manual {:?}: {}", path, e);
            (StatusCode::NOT_FOUND, "Operations Manual not found").into_response()
        }
    }
}
