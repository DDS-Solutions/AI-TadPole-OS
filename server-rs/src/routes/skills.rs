use crate::agent::skill_manifest::SkillManifest;
use crate::state::AppState;
use crate::routes::error::AppError;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

/// GET /v1/skills
/// 
/// Lists all valid skill manifests currently loaded into the registry.
/// Skill manifests contain the JSON schemas used for LLM tool-calling.
///
/// @docs API_REFERENCE:ListSkills
pub async fn list_skills(State(state): State<Arc<AppState>>) -> Result<Json<Vec<SkillManifest>>, AppError> {
    let mut skills: Vec<SkillManifest> = state.registry.skill_registry.manifests.values().cloned().collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(skills))
}

/// GET /v1/skills/:name
/// Returns a specific skill manifest by name.
pub async fn get_skill(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<SkillManifest>, AppError> {
    if let Some(manifest) = state.registry.skill_registry.get(&name) {
        Ok(Json(manifest.clone()))
    } else {
        Err(AppError::NotFound(format!("Skill '{}' not found", name)))
    }
}
