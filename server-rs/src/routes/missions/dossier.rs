//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / missions / dossier
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - [Structural] Deterministic internal state integrity and strict interface contract compliance.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::NotFound, AppError::Sqlx
//! - **Telemetry Targets**: none declared

use super::contracts::*;
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::{QueryBuilder, Row, Sqlite};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub async fn get_mission_dossier(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.resources.pool;
    let mission_id = id.trim();
    if mission_id.is_empty() {
        return Err(AppError::BadRequest(
            "Mission ID cannot be empty".to_string(),
        ));
    }

    // 1. Fetch Mission Header
    let mission_row = sqlx::query(
        r#"SELECT 
            m.id, 
            m.agent_id, 
            m.title, 
            m.status, 
            m.budget_usd, 
            m.cost_usd, 
            COALESCE(m.is_pinned, 0) as is_pinned,
            strftime('%Y-%m-%dT%H:%M:%SZ', m.created_at) as created_at_iso,
            strftime('%Y-%m-%dT%H:%M:%SZ', m.updated_at) as updated_at_iso,
            (SELECT COUNT(*) FROM mission_logs l WHERE l.mission_id = m.id) as log_count,
            (SELECT COUNT(*) FROM file_revisions r WHERE r.mission_id = m.id) as diff_count
        FROM mission_history m
        WHERE m.id = ?"#,
    )
    .bind(mission_id)
    .fetch_optional(pool)
    .await?;

    let mission_row = match mission_row {
        Some(r) => r,
        None => {
            return Err(AppError::NotFound(format!(
                "Mission not found: {}",
                mission_id
            )))
        }
    };

    let is_pinned_raw: i64 = mission_row.try_get("is_pinned").unwrap_or(0);
    let mission = MissionSummaryItem {
        id: mission_row.try_get("id").unwrap_or_default(),
        agent_id: mission_row.try_get("agent_id").unwrap_or_default(),
        title: mission_row.try_get("title").unwrap_or_default(),
        status: mission_row.try_get("status").unwrap_or_default(),
        budget_usd: mission_row.try_get("budget_usd").unwrap_or(0.0),
        cost_usd: mission_row.try_get("cost_usd").unwrap_or(0.0),
        is_pinned: is_pinned_raw == 1,
        created_at: mission_row
            .try_get("created_at_iso")
            .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()),
        updated_at: mission_row
            .try_get("updated_at_iso")
            .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()),
        log_count: mission_row.try_get("log_count").unwrap_or(0),
        diff_count: mission_row.try_get("diff_count").unwrap_or(0),
    };

    // 2. Fetch Mission Logs and Compute Merkle Verification (S-02 & Defect #2: Bounded by LIMIT 2000, with truncation check)
    let total_logs: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM mission_logs WHERE mission_id = ?")
            .bind(mission_id)
            .fetch_one(pool)
            .await
            .unwrap_or(0);
    let chain_truncated = total_logs > 2000;

    let log_rows = sqlx::query(
        r#"SELECT 
            id, mission_id, agent_id, source, text, severity, 
            timestamp, metadata, hash, prev_hash
        FROM mission_logs
        WHERE mission_id = ?
        ORDER BY timestamp ASC, id ASC
        LIMIT 2000"#,
    )
    .bind(mission_id)
    .fetch_all(pool)
    .await?;

    let mut logs = Vec::with_capacity(log_rows.len());
    let mut expected_prev_hash: Option<String> = None;
    let mut has_tamper = false;
    let mut has_unverified = false;

    for r in log_rows {
        let log_id: String = r.try_get("id").unwrap_or_default();
        let agent_id: String = r.try_get("agent_id").unwrap_or_default();
        let source: String = r.try_get("source").unwrap_or_default();
        let raw_text: String = r.try_get("text").unwrap_or_default();
        let severity: String = r.try_get("severity").unwrap_or_default();
        // Defect #1: Read the exact string stored in SQLite to avoid timestamp reformatting drift
        let timestamp_str: String = r
            .try_get("timestamp")
            .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());
        let meta_raw: Option<String> = r.try_get("metadata").ok();
        let hash: Option<String> = r.try_get("hash").unwrap_or_default();
        let prev_hash: Option<String> = r.try_get("prev_hash").unwrap_or_default();

        // Strict Merkle chain validation against authoritative compute_log_hash with RAW unredacted text
        let is_verified = match (&hash, &prev_hash) {
            (Some(curr), ph) => {
                let exp_clean = expected_prev_hash.as_deref().filter(|s| !s.is_empty());
                let got_clean = ph.as_deref().filter(|s| !s.is_empty());
                let prev_matches = match (exp_clean, got_clean) {
                    (None, None) => true,
                    (Some(exp), Some(got)) => exp == got,
                    _ => false,
                };

                // Defect #20: Normalize empty string prev_hash to None
                let prev_for_hash = ph.as_deref().filter(|s| !s.is_empty());
                let computed = crate::agent::mission::compute_log_hash(
                    prev_for_hash,
                    mission_id,
                    &agent_id,
                    &source,
                    &raw_text,
                    &timestamp_str,
                );

                let hash_matches = curr == &computed;
                let valid = prev_matches && hash_matches;
                if !valid {
                    has_tamper = true;
                    tracing::warn!(
                        mission_id = %mission_id,
                        log_id = %log_id,
                        prev_matches = %prev_matches,
                        hash_matches = %hash_matches,
                        "Merkle hash verification mismatch detected in mission flight log step"
                    );
                }
                expected_prev_hash = Some(curr.clone());
                valid
            }
            (None, _) => {
                // Legacy unhashed entry
                has_unverified = true;
                false
            }
        };

        // S-01: Apply secret redactor to text and metadata before constructing response
        let text = state.security.secret_redactor.redact(&raw_text);
        let metadata: Option<serde_json::Value> = meta_raw
            .map(|s| state.security.secret_redactor.redact(&s))
            .filter(|s| !s.trim().is_empty())
            .and_then(|s| {
                serde_json::from_str(&s).map_err(|e| {
                    tracing::warn!(mission_id = %mission_id, log_id = %log_id, error = %e, "Mission log metadata is not valid JSON after redaction");
                    e
                }).ok()
            });

        logs.push(MerkleStepRecord {
            id: log_id,
            mission_id: mission_id.to_string(),
            agent_id,
            source,
            text,
            severity,
            timestamp: timestamp_str,
            metadata,
            hash,
            prev_hash,
            is_verified,
        });
    }

    // 3. Fetch CAS File Revisions & Generate Content Diffs (Defect #3 & #14: Latest version per file, capped at LIMIT 500 & 1 MB payload check)
    let rev_rows = sqlx::query(
        r#"SELECT 
            r.id, r.file_path, r.hash, r.size_bytes, r.version_num, 
            r.created_at,
            CASE WHEN length(b.content) > 1048576 THEN NULL ELSE b.content END as pre_blob
        FROM file_revisions r
        LEFT JOIN cas_blobs b ON r.hash = b.hash
        WHERE r.mission_id = ?
          AND r.version_num = (
              SELECT MAX(r2.version_num) 
              FROM file_revisions r2 
              WHERE r2.mission_id = r.mission_id AND r2.file_path = r.file_path
          )
        ORDER BY r.file_path ASC
        LIMIT 500"#,
    )
    .bind(mission_id)
    .fetch_all(pool)
    .await?;

    let mut diffs = Vec::with_capacity(rev_rows.len());
    for r in rev_rows {
        let rev_id: i64 = r.try_get("id").unwrap_or(0);
        let file_path: String = r.try_get("file_path").unwrap_or_default();
        let hash: String = r.try_get("hash").unwrap_or_default();
        let size_bytes: i64 = r.try_get("size_bytes").unwrap_or(0);
        let version_num: i64 = r.try_get("version_num").unwrap_or(0);
        let created_at: String = r.try_get("created_at").unwrap_or_default();
        let pre_blob: Option<Vec<u8>> = r.try_get("pre_blob").ok().flatten();

        let pre_mutation_content = if size_bytes > 1_048_576 || pre_blob.is_none() {
            "[CAS snapshot exceeds 1 MB preview ceiling]".to_string()
        } else {
            pre_blob
                .and_then(|b| String::from_utf8(b).ok())
                .unwrap_or_default()
        };

        // Safely validate and boundedly read current file on disk within base_dir (SEC-008 & Defect #14)
        let safe_res = crate::security::path_guard::validate_path(&state.base_dir, &file_path);
        let current_content = match safe_res {
            Ok(safe_p) => {
                let p = safe_p.as_path();
                if p.is_file() {
                    match tokio::fs::File::open(p).await {
                        Ok(file) => {
                            use tokio::io::AsyncReadExt;
                            let mut limited = file.take(1_048_577);
                            let mut buf = Vec::new();
                            match limited.read_to_end(&mut buf).await {
                                Ok(n) if n > 1_048_576 => {
                                    "[File exceeds 1 MB preview ceiling]".to_string()
                                }
                                Ok(_) => String::from_utf8(buf).unwrap_or_else(|_| {
                                    "[Binary or Unreadable File Content]".to_string()
                                }),
                                Err(e) => format!("[Read Error: {}]", e),
                            }
                        }
                        Err(e) => format!("[Open Error: {}]", e),
                    }
                } else {
                    "[File not present on active disk]".to_string()
                }
            }
            Err(e) => format!("[Access Denied: {}]", e),
        };

        let diff_summary = compute_line_diff_summary(&pre_mutation_content, &current_content);

        diffs.push(CasDiffRecord {
            id: rev_id,
            file_path,
            version_num,
            hash,
            size_bytes,
            created_at,
            pre_mutation_content,
            current_content,
            diff_summary,
        });
    }

    // 4. Fetch Swarm Context (Memory Palace) with LIMIT 500 and telemetry error guard (S-09)
    let context_rows = match sqlx::query(
        r#"SELECT 
            id, mission_id, agent_id, topic, finding, 
            strftime('%Y-%m-%dT%H:%M:%SZ', timestamp) as timestamp_iso
        FROM swarm_context
        WHERE mission_id = ?
        ORDER BY timestamp ASC
        LIMIT 500"#,
    )
    .bind(mission_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, mission_id = %mission_id, "Failed to fetch swarm_context records for mission");
            Vec::new()
        }
    };

    let mut memory_palace = Vec::with_capacity(context_rows.len());
    for r in context_rows {
        memory_palace.push(SwarmFindingRecord {
            id: r.try_get("id").unwrap_or_default(),
            mission_id: r.try_get("mission_id").unwrap_or_default(),
            agent_id: r.try_get("agent_id").unwrap_or_default(),
            topic: r.try_get("topic").unwrap_or_default(),
            finding: r.try_get("finding").unwrap_or_default(),
            timestamp: r
                .try_get("timestamp_iso")
                .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()),
        });
    }

    // 5. Fetch TrustGraph (Nodes & Relations) with LIMIT 500 and telemetry error guard (S-09)
    let entity_rows = match sqlx::query(
        r#"SELECT id, name, type, description, mission_id, created_at
        FROM graph_entities
        WHERE mission_id = ?
        ORDER BY created_at ASC
        LIMIT 500"#,
    )
    .bind(mission_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, mission_id = %mission_id, "Failed to fetch graph_entities for mission");
            Vec::new()
        }
    };

    let mut entities = Vec::with_capacity(entity_rows.len());
    for r in entity_rows {
        entities.push(TrustGraphNode {
            id: r.try_get("id").unwrap_or_default(),
            name: r.try_get("name").unwrap_or_default(),
            node_type: r.try_get("type").unwrap_or_default(),
            description: r.try_get("description").ok(),
            mission_id: r.try_get("mission_id").ok(),
            created_at: r.try_get("created_at").unwrap_or(0),
        });
    }

    let relation_rows = match sqlx::query(
        r#"SELECT id, source_entity_id, target_entity_id, relation_type, weight, mission_id, created_at
        FROM graph_relations
        WHERE mission_id = ?
        ORDER BY created_at ASC
        LIMIT 500"#,
    )
    .bind(mission_id)
    .fetch_all(pool)
    .await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, mission_id = %mission_id, "Failed to fetch graph_relations for mission");
            Vec::new()
        }
    };

    let mut relations = Vec::with_capacity(relation_rows.len());
    for r in relation_rows {
        relations.push(TrustGraphEdge {
            id: r.try_get("id").unwrap_or_default(),
            source_entity_id: r.try_get("source_entity_id").unwrap_or_default(),
            target_entity_id: r.try_get("target_entity_id").unwrap_or_default(),
            relation_type: r.try_get("relation_type").unwrap_or_default(),
            weight: r.try_get("weight").unwrap_or(1.0),
            mission_id: r.try_get("mission_id").ok(),
            created_at: r.try_get("created_at").unwrap_or(0),
        });
    }

    // 6. Fetch Economics & Ledger (R-07 & Defect #7: Capped participant set with IN clauses to prevent variable limit overflow)
    let mut participant_addresses: HashSet<String> = HashSet::new();
    participant_addresses.insert(mission_id.to_string());
    participant_addresses.insert(format!("local:{}", mission_id));
    if !mission.agent_id.is_empty() {
        participant_addresses.insert(mission.agent_id.clone());
        participant_addresses.insert(format!("local:{}", mission.agent_id));
    }
    for l in &logs {
        if !l.agent_id.is_empty() {
            participant_addresses.insert(l.agent_id.clone());
            participant_addresses.insert(format!("local:{}", l.agent_id));
        }
    }
    for m in &memory_palace {
        if !m.agent_id.is_empty() {
            participant_addresses.insert(m.agent_id.clone());
            participant_addresses.insert(format!("local:{}", m.agent_id));
        }
    }

    let mut addrs: Vec<String> = participant_addresses.into_iter().take(50).collect();
    if addrs.is_empty() {
        addrs.push(mission_id.to_string());
    }

    let mut tx_qb = QueryBuilder::<Sqlite>::new(
        "SELECT id, transaction_type, buyer_address, seller_address, asset_id, amount, status, audit_signature, created_at FROM transaction_ledger WHERE (buyer_address IN ("
    );
    let mut sep1 = tx_qb.separated(", ");
    for addr in &addrs {
        sep1.push_bind(addr);
    }
    tx_qb.push(") OR seller_address IN (");
    let mut sep2 = tx_qb.separated(", ");
    for addr in &addrs {
        sep2.push_bind(addr);
    }
    tx_qb.push(")) ORDER BY created_at ASC LIMIT 500");

    let tx_rows = match tx_qb.build().fetch_all(pool).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, mission_id = %mission_id, "Failed to fetch transaction_ledger records for mission");
            Vec::new()
        }
    };

    let mut transactions = Vec::with_capacity(tx_rows.len());
    for r in tx_rows {
        transactions.push(TransactionLedgerRecord {
            id: r.try_get("id").unwrap_or_default(),
            transaction_type: r.try_get("transaction_type").unwrap_or_default(),
            buyer_address: r.try_get("buyer_address").ok(),
            seller_address: r.try_get("seller_address").ok(),
            asset_id: r.try_get("asset_id").ok(),
            amount: r.try_get("amount").unwrap_or(0),
            status: r.try_get("status").unwrap_or_default(),
            audit_signature: r.try_get("audit_signature").ok(),
            created_at: r.try_get("created_at").unwrap_or(0),
        });
    }

    // Compute token rollup across turns and agent state (FR-5)
    let mut turn_tokens: u64 = 0;
    for l in &logs {
        if let Some(m) = &l.metadata {
            if let Some(tu) = m
                .get("token_usage")
                .and_then(|u| u.get("total_tokens"))
                .and_then(|t| t.as_u64())
            {
                turn_tokens += tu;
            } else if let Some(tu) = m.get("tokens_used").and_then(|t| t.as_u64()) {
                turn_tokens += tu;
            } else if let Some(tu) = m.get("total_tokens").and_then(|t| t.as_u64()) {
                turn_tokens += tu;
            }
        }
    }
    // Defect #5: Query real tokens_used column on agents table
    let total_tokens = if turn_tokens > 0 {
        turn_tokens
    } else {
        sqlx::query_scalar::<_, i64>("SELECT tokens_used FROM agents WHERE id = ?")
            .bind(&mission.agent_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .unwrap_or(0)
            .max(0) as u64
    };

    // 7. Synthesize Debrief & Recommendations (SEC-05 & Defect #6: Strictly scoped to mission_id to prevent cross-mission note leaks)
    let agent99_log: Option<String> = match sqlx::query_scalar(
        r#"SELECT text FROM mission_logs 
        WHERE mission_id = ?1 AND (agent_id = '99' OR source = 'Agent 99' OR LOWER(agent_id) LIKE '%auditor%')
        ORDER BY timestamp DESC, id DESC LIMIT 1"#,
    )
    .bind(mission_id)
    .fetch_optional(pool)
    .await {
        Ok(opt) => opt,
        Err(e) => {
            tracing::warn!(error = %e, mission_id = %mission_id, "Failed to fetch Agent 99 auditor log");
            None
        }
    };
    let agent99_notes = agent99_log.map(|t| state.security.secret_redactor.redact(&t));

    // R-01: Derive drift score only from actual alert or log metadata; no synthetic fallbacks
    let drift_score = if let Some(alert) = state.drift_alerts.get(mission_id) {
        alert.get("drift_score").and_then(|v| v.as_f64())
    } else {
        logs.iter().rev().find_map(|l| {
            l.metadata
                .as_ref()
                .and_then(|m| m.get("drift_score"))
                .and_then(|v| v.as_f64())
        })
    };

    let mut recommendations = Vec::new();
    // Defect #17: Case-insensitive severity matching
    let error_count = logs
        .iter()
        .filter(|l| l.severity.eq_ignore_ascii_case("error"))
        .count();
    let warn_count = logs
        .iter()
        .filter(|l| {
            l.severity.eq_ignore_ascii_case("warning") || l.severity.eq_ignore_ascii_case("warn")
        })
        .count();
    let is_empty = logs.is_empty();

    if has_tamper {
        recommendations.push("CRITICAL: Merkle hash-chain anomaly detected. Validate disk logs against remote backup.".to_string());
    } else if chain_truncated {
        recommendations.push(format!("NOTICE: Mission flight logs exceed 2000 entries ceiling ({} total); tail truncation prevented complete Merkle verification.", total_logs));
    } else if has_unverified {
        recommendations.push(
            "NOTICE: One or more log entries lack Merkle hashes (legacy pre-hash records)."
                .to_string(),
        );
    } else if is_empty {
        recommendations
            .push("NOTICE: No execution step logs recorded for this mission.".to_string());
    }

    if error_count > 0 {
        recommendations.push(format!(
            "Mission logged {} error events. Inspect Merkle log entries with error severity.",
            error_count
        ));
    }
    if warn_count > 3 {
        recommendations.push(format!(
            "High warning density ({} warnings). Review rate limits or tool parameter tolerances.",
            warn_count
        ));
    }
    if diffs.is_empty() {
        recommendations.push(
            "No CAS file mutations recorded for this mission (read-only execution).".to_string(),
        );
    } else {
        recommendations.push(format!(
            "Verified {} file mutations preserved in deduplicated CAS BLOB storage.",
            diffs.len()
        ));
    }

    // R-02 & Defect #2: Tri-state verification verdict with chain truncation guard
    let verdict = match mission.status.to_lowercase().as_str() {
        "completed" => {
            if has_tamper {
                Some("COMPLETED_WITH_TAMPER_FLAG".to_string())
            } else if is_empty || has_unverified || chain_truncated {
                Some("COMPLETED_UNVERIFIED".to_string())
            } else {
                Some("VERIFIED_SUCCESS".to_string())
            }
        }
        "failed" => {
            if has_tamper {
                Some("FAILED_WITH_TAMPER_FLAG".to_string())
            } else if is_empty || has_unverified || chain_truncated {
                Some("FAILED_UNVERIFIED".to_string())
            } else {
                Some("FAILED_VERIFIED".to_string())
            }
        }
        "active" => Some("IN_PROGRESS".to_string()),
        _ => Some("STATUS_NOMINAL".to_string()),
    };

    let dossier = MissionDossier {
        // Defect #4: Use mission struct's own values rather than re-querying mission_row
        economics: EconomicsPayload {
            total_cost_usd: mission.cost_usd,
            budget_usd: mission.budget_usd,
            total_tokens,
            transactions,
        },
        mission,
        logs,
        diffs,
        memory_palace,
        trust_graph: TrustGraphPayload {
            entities,
            relations,
        },
        debrief: DebriefPayload {
            agent99_notes,
            drift_score,
            verdict,
            recommendations,
        },
    };

    Ok((StatusCode::OK, Json(dossier)))
}

//  DIFF COMPUTATION HELPER
// ─────────────────────────────────────────────────────────

/// S-11: Multiset frequency counter for accurate line addition/deletion counting
pub(crate) fn compute_line_diff_summary(pre: &str, current: &str) -> CasDiffSummary {
    let mut pre_counts: HashMap<&str, usize> = HashMap::new();
    for line in pre.lines() {
        *pre_counts.entry(line).or_insert(0) += 1;
    }

    let mut cur_counts: HashMap<&str, usize> = HashMap::new();
    for line in current.lines() {
        *cur_counts.entry(line).or_insert(0) += 1;
    }

    let mut additions = 0;
    for (line, &cur_count) in &cur_counts {
        let pre_count = pre_counts.get(line).copied().unwrap_or(0);
        if cur_count > pre_count {
            additions += cur_count - pre_count;
        }
    }

    let mut deletions = 0;
    for (line, &pre_count) in &pre_counts {
        let cur_count = cur_counts.get(line).copied().unwrap_or(0);
        if pre_count > cur_count {
            deletions += pre_count - cur_count;
        }
    }

    let changes = additions.min(deletions);

    CasDiffSummary {
        additions,
        deletions,
        changes,
    }
}
