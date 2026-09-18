//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database & Migrations / seed
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[seed]`, `[Database]`, `[System]`
//! - **Witness Tests**: none declared

use anyhow::Result;
use sqlx::SqlitePool;

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
        .await?;

    if agent_count > 1 {
        return Ok(());
    }

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
        let agents: Vec<serde_json::Value> = serde_json::from_str(&content)?;
        let mut tx = pool.begin().await?;

        for agent_val in agents {
            let id = agent_val["id"].as_str().unwrap_or_default();
            if id.is_empty() {
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

            let model_config2_val = agent_val
                .get("model_config2")
                .or_else(|| agent_val.get("modelConfig2"))
                .or_else(|| agent_val.get("planningSlot"));
            let model_config2 =
                model_config2_val.map(|v| serde_json::to_string(v).unwrap_or_default());

            let model_config3_val = agent_val
                .get("model_config3")
                .or_else(|| agent_val.get("modelConfig3"))
                .or_else(|| agent_val.get("executionSlot"));
            let model_config3 =
                model_config3_val.map(|v| serde_json::to_string(v).unwrap_or_default());

            sqlx::query(
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
            .bind(serde_json::to_string(&agent_val["skills"]).unwrap_or_else(|_| "[]".to_string()))
            .bind(serde_json::to_string(&agent_val["workflows"]).unwrap_or_else(|_| "[]".to_string()))
            .bind("[]")
            .bind(1)
            .bind("user")
            .bind(model_2)
            .bind(model_3)
            .bind(model_config2)
            .bind(model_config3)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
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
    .await?;
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
                tokio::fs::copy(&src_path, &dest_path).await?;
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
                            tokio::fs::copy(&path, &dest_path).await?;
                        }
                    }
                }
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
            tokio::fs::copy(&src_path, &dest_path).await?;
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
