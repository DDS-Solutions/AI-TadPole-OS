//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database & Migrations / seed
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[seed]`, `[Database]`, `[System]`

use anyhow::{Context, Result};
use sqlx::SqlitePool;

/// Extracts a JSON field as a valid JSON string or falls back to a valid JSON default.
///
/// Ensures missing keys or explicit `null` values NEVER result in binding the literal text `"null"`.
pub fn json_field_or(v: &serde_json::Value, key: &str, default: &'static str) -> String {
    match v.get(key) {
        None | Some(serde_json::Value::Null) => default.to_string(),
        Some(
            val @ (serde_json::Value::Array(_)
            | serde_json::Value::Object(_)
            | serde_json::Value::String(_)),
        ) => serde_json::to_string(val).unwrap_or_else(|_| default.to_string()),
        Some(other) => other.to_string(),
    }
}

/// Resolves an optional JSON object/array from candidate keys, returning `None` if absent or null.
///
/// Binds SQL NULL when absent, avoiding storing `"null"` string representations in optional columns.
pub fn optional_json_object(v: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(val) = v.get(*key) {
            if !val.is_null() && (val.is_object() || val.is_array() || val.is_string()) {
                if let Ok(serialized) = serde_json::to_string(val) {
                    if serialized != "null" {
                        return Some(serialized);
                    }
                }
            }
        }
    }
    None
}

/// Atomically copies a file to destination via a sibling temporary file + fsync + rename.
///
/// Prevents concurrent readers from observing half-written or torn files.
pub async fn copy_file_atomic(src: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        if tokio::fs::metadata(parent).await.is_err() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    let tmp_dest = dest.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
    tokio::fs::copy(src, &tmp_dest).await?;

    if let Ok(file) = tokio::fs::File::open(&tmp_dest).await {
        let _ = file.sync_all().await;
    }

    tokio::fs::rename(&tmp_dest, dest).await?;
    Ok(())
}

/// Seeds default data (agents, providers, workflows, MCP config).
pub async fn seed_default_data(pool: &SqlitePool) -> Result<()> {
    seed_baseline_agents(pool).await?;
    seed_baseline_providers().await?;
    seed_baseline_workflows().await?;
    seed_baseline_mcp_config().await?;
    tracing::info!("✅ [seed] Database baseline seeding complete.");
    Ok(())
}

async fn seed_baseline_agents(pool: &SqlitePool) -> Result<()> {
    let agent_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM agents")
        .fetch_one(pool)
        .await
        .with_context(|| "querying agent count before baseline seed")?;

    tracing::info!("🌱 [Database] Seeding baseline agents from bundle...");
    let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
    let agents_json_path = find_bundled_file(&resource_root, "data/agents.json").await;

    if let Some(path) = agents_json_path {
        tracing::info!("📂 [Database] Found baseline agents at {:?}", path);
        let metadata = tokio::fs::metadata(&path).await?;
        if metadata.len() > 8 * 1024 * 1024 {
            return Err(anyhow::anyhow!("agents.json exceeds the 8MB size limit"));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let agents: Vec<serde_json::Value> = match serde_json::from_str(&content) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::error!(
                    "❌ [Database] Failed to parse 'agents.json' at {:?}: {}. Falling back to minimal Alpha agent.",
                    path,
                    e
                );
                if agent_count == 0 {
                    seed_minimal_alpha(pool).await?;
                }
                return Ok(());
            }
        };

        let mut tx = pool.begin().await?;
        let mut inserted_count = 0;

        for (idx, agent_val) in agents.into_iter().enumerate() {
            let id = agent_val
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if id.is_empty() {
                tracing::warn!(
                    "⚠️ [Database] Skipping baseline agent at index {} with missing or non-string id: {:?}",
                    idx,
                    agent_val.get("id")
                );
                continue;
            }

            let name = agent_val["name"].as_str().unwrap_or("Unknown");
            let role = agent_val["role"].as_str().unwrap_or("Specialist");
            let dept = agent_val["department"].as_str().unwrap_or("Swarm Core");
            let desc = agent_val["description"].as_str().unwrap_or("");
            let model_id = agent_val["model"]
                .as_str()
                .or_else(|| agent_val["model_id"].as_str());
            let provider = agent_val["model_config"]["provider"]
                .as_str()
                .unwrap_or("google");
            let theme = agent_val["theme_color"].as_str().unwrap_or("#4fd1c5");

            let model_2 = agent_val["model_2"]
                .as_str()
                .or_else(|| agent_val["model2"].as_str())
                .or_else(|| agent_val["planningSlot"]["modelId"].as_str());
            let model_3 = agent_val["model_3"]
                .as_str()
                .or_else(|| agent_val["model3"].as_str())
                .or_else(|| agent_val["executionSlot"]["modelId"].as_str());

            let model_config2 = optional_json_object(
                &agent_val,
                &["model_config2", "modelConfig2", "planningSlot"],
            );
            let model_config3 = optional_json_object(
                &agent_val,
                &["model_config3", "modelConfig3", "executionSlot"],
            );

            let skills = json_field_or(&agent_val, "skills", "[]");
            let workflows = json_field_or(&agent_val, "workflows", "[]");

            let res = sqlx::query(
                "INSERT OR IGNORE INTO agents (id, name, role, department, description, status, provider, model_id, theme_color, metadata, skills, workflows, mcp_tools, active_model_slot, category, model_2, model_3, model_config2, model_config3)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(id)
            .bind(name)
            .bind(role)
            .bind(dept)
            .bind(desc)
            .bind("idle")
            .bind(provider)
            .bind(model_id)
            .bind(theme)
            .bind("{}")
            .bind(skills)
            .bind(workflows)
            .bind("[]")
            .bind(1)
            .bind("user")
            .bind(model_2)
            .bind(model_3)
            .bind(model_config2)
            .bind(model_config3)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("seeding agent '{}' ({})", id, name))?;

            inserted_count += res.rows_affected();
        }

        tx.commit().await?;
        if inserted_count > 0 {
            tracing::info!("🌱 [Database] Seeded {} baseline agent(s)", inserted_count);
        }
    } else {
        tracing::warn!(
            "⚠️ [Database] Seed file 'agents.json' not found in bundle; falling back..."
        );
        if agent_count == 0 {
            seed_minimal_alpha(pool).await?;
        }
    }

    Ok(())
}

async fn seed_minimal_alpha(pool: &SqlitePool) -> Result<()> {
    tracing::info!("🌱 Seeding minimal Alpha agent...");
    sqlx::query(
        "INSERT OR IGNORE INTO agents (id, name, role, department, description, status, provider, model_id, theme_color, metadata, skills, workflows, mcp_tools, active_model_slot, category)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("1")
    .bind("Alpha")
    .bind("Agent of Nine")
    .bind("Swarm Core")
    .bind("The primary intelligence node of the Tadpole OS network.")
    .bind("idle")
    .bind("google")
    .bind("gemini-1.5-flash")
    .bind("#4fd1c5")
    .bind("{}")
    .bind("[]")
    .bind("[]")
    .bind("[]")
    .bind(1)
    .bind("user")
    .execute(pool)
    .await
    .with_context(|| "seeding fallback Alpha agent")?;
    Ok(())
}

async fn seed_baseline_providers() -> Result<()> {
    let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
    let base_dir = std::env::current_dir().unwrap_or_default();
    let data_dir = base_dir.join("data");

    if tokio::fs::metadata(&data_dir).await.is_err() {
        tokio::fs::create_dir_all(&data_dir).await?;
    }

    let files_to_seed = ["infra_providers.json", "infra_models.json", "routines.json"];
    for filename in files_to_seed {
        let dest_path = data_dir.join(filename);
        if tokio::fs::metadata(&dest_path).await.is_err() {
            if let Some(src_path) =
                find_bundled_file(&resource_root, &format!("data/{}", filename)).await
            {
                tracing::info!("🌱 [System] Seeding {} from {:?}...", filename, src_path);
                copy_file_atomic(&src_path, &dest_path).await?;
            }
        }
    }
    Ok(())
}

async fn seed_baseline_workflows() -> Result<()> {
    let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
    let base_dir = std::env::current_dir().unwrap_or_default();
    let directives_dir = base_dir.join("directives");

    if tokio::fs::metadata(&directives_dir).await.is_err() {
        tokio::fs::create_dir_all(&directives_dir).await?;
    }

    let bundled_workflows_dir = find_bundled_file(&resource_root, "data/workflows").await;
    if let Some(src_dir) = bundled_workflows_dir {
        if let Ok(mut entries) = tokio::fs::read_dir(&src_dir).await {
            let mut seeded_count = 0;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(filename) = path.file_name() {
                        let dest_path = directives_dir.join(filename);
                        if tokio::fs::metadata(&dest_path).await.is_err() {
                            tracing::info!(
                                "🌱 [System] Seeding workflow {:?} from {:?}...",
                                filename,
                                path
                            );
                            if let Err(e) = copy_file_atomic(&path, &dest_path).await {
                                tracing::warn!(
                                    "⚠️ [System] Failed to seed workflow {:?}: {}",
                                    filename,
                                    e
                                );
                            } else {
                                seeded_count += 1;
                            }
                        }
                    }
                }
            }
            if seeded_count > 0 {
                tracing::info!(
                    "🌱 [System] Seeded {} workflow(s) into directives/",
                    seeded_count
                );
            }
        }
    }
    Ok(())
}

async fn seed_baseline_mcp_config() -> Result<()> {
    let resource_root = std::env::var("RESOURCE_ROOT").unwrap_or_else(|_| ".".to_string());
    let base_dir = std::env::current_dir().unwrap_or_default();
    let agent_dir = base_dir.join(".agent");

    if tokio::fs::metadata(&agent_dir).await.is_err() {
        tokio::fs::create_dir_all(&agent_dir).await?;
    }

    let mcp_filename = "mcp_config.json";
    let dest_path = agent_dir.join(mcp_filename);

    if tokio::fs::metadata(&dest_path).await.is_err() {
        if let Some(src_path) =
            find_bundled_file(&resource_root, &format!(".agent/{}", mcp_filename)).await
        {
            tracing::info!(
                "🌱 [System] Seeding MCP configuration from {:?}...",
                src_path
            );
            copy_file_atomic(&src_path, &dest_path).await?;
        }
    }
    Ok(())
}

async fn find_bundled_file(resource_root: &str, relative_path: &str) -> Option<std::path::PathBuf> {
    let root = std::path::Path::new(resource_root);

    let direct = root.join(relative_path);
    if tokio::fs::metadata(&direct).await.is_ok() {
        return Some(direct);
    }

    let up_path = root.join("_up_").join(relative_path);
    if tokio::fs::metadata(&up_path).await.is_ok() {
        return Some(up_path);
    }

    let dev_path = std::path::Path::new(".").join(relative_path);
    if tokio::fs::metadata(&dev_path).await.is_ok() {
        return Some(dev_path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_field_or_guards_null_and_missing() {
        let empty_obj = serde_json::json!({});
        assert_eq!(json_field_or(&empty_obj, "skills", "[]"), "[]");

        let null_val = serde_json::json!({ "skills": null });
        assert_eq!(json_field_or(&null_val, "skills", "[]"), "[]");

        let valid_arr = serde_json::json!({ "skills": ["skill_a", "skill_b"] });
        assert_eq!(
            json_field_or(&valid_arr, "skills", "[]"),
            "[\"skill_a\",\"skill_b\"]"
        );
    }

    #[test]
    fn test_optional_json_object_returns_none_for_null_or_absent() {
        let null_val = serde_json::json!({ "model_config2": null });
        assert_eq!(
            optional_json_object(&null_val, &["model_config2", "planningSlot"]),
            None
        );

        let absent_val = serde_json::json!({ "other": 123 });
        assert_eq!(optional_json_object(&absent_val, &["model_config2"]), None);

        let present_val =
            serde_json::json!({ "planningSlot": { "provider": "google", "model": "gemini" } });
        assert!(optional_json_object(&present_val, &["model_config2", "planningSlot"]).is_some());
    }
}
