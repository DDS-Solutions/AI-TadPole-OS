//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **TrustGraph Engine**: Core semantic graph database layer built as an
//! overlay on the local SQLite schema. Integrates **Breadth-First Search (BFS)**
//! recursive traversals to establish semantic relationships between entities
//! (Agents, Missions, Concepts, Outcomes, Files). This serves as the foundation
//! for the hybrid GraphRAG contextual indexing.
//! Features **O(N) Traversal Complexity** and **Relational Subgraph Extraction**
//! returning both Node metadata and directed Edges for LLM reasoning.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: SQLite foreign key violations on un-linked entity deletions,
//!   BFS loop infinite execution (mitigated by explicit cycle checking and depth caps),
//!   or pool contention.
//! - **Trace Scope**: `server-rs::agent::trustgraph`

use crate::error::AppError;
use sqlx::{Row, SqlitePool};

/// A semantic vertex representation in the Swarm Knowledge Graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrustGraphNode {
    pub id: String,
    pub name: String,
    pub r#type: String, // 'AGENT', 'MISSION', 'CONCEPT', 'FILE', 'OUTCOME'
    pub description: Option<String>,
    pub mission_id: Option<String>,
}

/// A directed, weighted relation (edge) linking two semantic entities.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrustGraphRelation {
    pub id: String,
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub relation_type: String, // 'DEPENDS_ON', 'CREATED', 'MUTATED', 'RESOLVED', 'COMMUNICATED'
    pub weight: f32,
    pub mission_id: Option<String>,
}

/// Dynamic response containing both adjacent entity vertices and their relations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Subgraph {
    pub nodes: Vec<TrustGraphNode>,
    pub relations: Vec<TrustGraphRelation>,
}

/// Engine interface to manage, persist, and traverse graph entities and relations.
pub struct TrustGraphEngine {
    pool: SqlitePool,
}

impl TrustGraphEngine {
    /// Creates a new instance of the TrustGraphEngine with the active SQLite pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Inserts or replaces a node in the graph database.
    pub async fn add_node(&self, node: &TrustGraphNode) -> Result<(), AppError> {
        sqlx::query::<sqlx::Sqlite>(
            "INSERT OR REPLACE INTO graph_entities (id, name, type, description, mission_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(&node.id)
        .bind(&node.name)
        .bind(&node.r#type)
        .bind(&node.description)
        .bind(&node.mission_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Links two existing entities with a directed relationship.
    pub async fn add_relation(&self, rel: &TrustGraphRelation) -> Result<(), AppError> {
        sqlx::query::<sqlx::Sqlite>(
            "INSERT OR REPLACE INTO graph_relations (id, source_entity_id, target_entity_id, relation_type, weight, mission_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(&rel.id)
        .bind(&rel.source_entity_id)
        .bind(&rel.target_entity_id)
        .bind(&rel.relation_type)
        .bind(rel.weight as f64)
        .bind(&rel.mission_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Performs a high-performance recursive BFS traversal to extract adjacent relations up to $N$ hops.
    /// Cycle-resilient using visited tracking. Complexity is $O(N)$ with $O(1)$ lookups.
    pub async fn traverse_subgraph(
        &self,
        seed_entity_ids: &[String],
        max_depth: u32,
    ) -> Result<Subgraph, AppError> {
        if seed_entity_ids.is_empty() || max_depth == 0 {
            return Ok(Subgraph {
                nodes: Vec::new(),
                relations: Vec::new(),
            });
        }

        // Use a Map to ensure uniqueness and O(1) lookup
        let mut relations_map = std::collections::HashMap::new();
        let mut visited_ids = std::collections::HashSet::new();
        let mut current_frontier = seed_entity_ids.to_vec();

        for seed in seed_entity_ids {
            visited_ids.insert(seed.clone());
        }

        for _ in 0..max_depth {
            if current_frontier.is_empty() {
                break;
            }

            let mut next_frontier = Vec::new();

            // Chunk current frontier to avoid excessive binder limits in SQLite (max 999 parameters)
            for chunk in current_frontier.chunks(200) {
                let placeholders = vec!["?"; chunk.len()].join(",");
                let query_str = format!(
                    "SELECT id, source_entity_id, target_entity_id, relation_type, weight, mission_id \
                     FROM graph_relations \
                     WHERE source_entity_id IN ({}) OR target_entity_id IN ({})",
                    placeholders, placeholders
                );

                let mut query = sqlx::query::<sqlx::Sqlite>(&query_str);
                // Bind for both source_entity_id and target_entity_id
                for id in chunk.iter().chain(chunk.iter()) {
                    query = query.bind(id);
                }

                let rows = query.fetch_all(&self.pool).await?;
                for row in rows {
                    let id: String = row.get("id");

                    // O(1) uniqueness check
                    if relations_map.contains_key(&id) {
                        continue;
                    }

                    let source: String = row.get("source_entity_id");
                    let target: String = row.get("target_entity_id");
                    let relation_type: String = row.get("relation_type");
                    let weight_f64: f64 = row.get("weight");
                    let mission_id: Option<String> = row.get("mission_id");

                    relations_map.insert(
                        id.clone(),
                        TrustGraphRelation {
                            id,
                            source_entity_id: source.clone(),
                            target_entity_id: target.clone(),
                            relation_type,
                            weight: weight_f64 as f32,
                            mission_id,
                        },
                    );

                    // Queue for next BFS hop if not visited
                    if visited_ids.insert(source.clone()) {
                        next_frontier.push(source);
                    }
                    if visited_ids.insert(target.clone()) {
                        next_frontier.push(target);
                    }
                }
            }
            current_frontier = next_frontier;
        }

        // B. Fetch node metadata for all visited node IDs in chunks to respect binders limit
        let visited_list: Vec<String> = visited_ids.into_iter().collect();
        let mut nodes = Vec::new();

        for chunk in visited_list.chunks(200) {
            let placeholders = vec!["?"; chunk.len()].join(",");
            let query_str = format!(
                "SELECT id, name, type, description, mission_id \
                 FROM graph_entities \
                 WHERE id IN ({})",
                placeholders
            );

            let mut query = sqlx::query::<sqlx::Sqlite>(&query_str);
            for id in chunk {
                query = query.bind(id);
            }

            let rows = query.fetch_all(&self.pool).await?;
            for row in rows {
                nodes.push(TrustGraphNode {
                    id: row.get("id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    description: row.get("description"),
                    mission_id: row.get("mission_id"),
                });
            }
        }

        let relations = relations_map.into_values().collect();

        Ok(Subgraph { nodes, relations })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_in_memory_db() -> Result<SqlitePool, AppError> {
        let pool = SqlitePool::connect("sqlite::memory:").await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to connect in-memory DB: {}", e))
        })?;

        sqlx::query(
            "CREATE TABLE graph_entities (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                description TEXT,
                mission_id TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );",
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to create graph_entities: {}", e))
        })?;

        sqlx::query(
            "CREATE TABLE graph_relations (
                id TEXT PRIMARY KEY,
                source_entity_id TEXT NOT NULL,
                target_entity_id TEXT NOT NULL,
                relation_type TEXT NOT NULL,
                weight REAL NOT NULL DEFAULT 1.0,
                mission_id TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );",
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to create graph_relations: {}", e))
        })?;

        Ok(pool)
    }

    #[tokio::test]
    async fn test_trust_graph_lifecycle() -> Result<(), AppError> {
        let pool = setup_in_memory_db().await?;
        let engine = TrustGraphEngine::new(pool);

        let node1 = TrustGraphNode {
            id: "node-1".to_string(),
            name: "Agent Alpha".to_string(),
            r#type: "AGENT".to_string(),
            description: Some("Lead Orchestrator".to_string()),
            mission_id: Some("mission-123".to_string()),
        };
        let node2 = TrustGraphNode {
            id: "node-2".to_string(),
            name: "Dockerfile".to_string(),
            r#type: "FILE".to_string(),
            description: Some("Workspace configuration file".to_string()),
            mission_id: Some("mission-123".to_string()),
        };

        engine.add_node(&node1).await?;
        engine.add_node(&node2).await?;

        let relation = TrustGraphRelation {
            id: "rel-1".to_string(),
            source_entity_id: "node-1".to_string(),
            target_entity_id: "node-2".to_string(),
            relation_type: "MUTATED".to_string(),
            weight: 1.0,
            mission_id: Some("mission-123".to_string()),
        };

        engine.add_relation(&relation).await?;

        // Traverse Graph BFS (Depth 1)
        let subgraph = engine.traverse_subgraph(&["node-1".to_string()], 1).await?;

        assert_eq!(subgraph.relations.len(), 1);
        assert_eq!(subgraph.relations[0].source_entity_id, "node-1");
        assert_eq!(subgraph.relations[0].target_entity_id, "node-2");
        assert_eq!(subgraph.relations[0].relation_type, "MUTATED");

        // Verify Node Metadata rehydrated successfully
        assert_eq!(subgraph.nodes.len(), 2);
        assert!(subgraph.nodes.iter().any(|n| n.id == "node-1"));
        assert!(subgraph.nodes.iter().any(|n| n.id == "node-2"));

        Ok(())
    }
}

// Metadata: [trustgraph]

// Metadata: [trustgraph]
