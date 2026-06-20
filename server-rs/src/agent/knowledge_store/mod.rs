//! @docs ARCHITECTURE:IKS
//!
//! ### AI Assist Note
//! **Institutional Knowledge Store (IKS)**: Cross-cluster, cross-restart
//! persistent semantic memory. Unlike `VectorMemory` (mission-scoped),
//! IKS holds durable, curated facts — agent patterns, SOPs, client knowledge,
//! human decision history — that persist indefinitely across any cluster.
//!
//! ### Architecture
//! Dual-store: SQLite metadata index (topic, cluster, TTL, dedup, **text**)
//! + LanceDB vector store for k-NN similarity search. Text content is stored
//! in SQLite alongside metadata so point-lookups (`get_by_id`) need no
//! LanceDB round-trip. Content is deduplicated by SHA-256 hash at write time.
//!
//! ### Embedding Provider
//! Always uses `text-embedding-004` via `GOOGLE_API_KEY` — never inherits
//! from the calling agent's provider config (dimensional consistency).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: `GOOGLE_API_KEY` missing, LanceDB schema mismatch,
//!   SQLite UNIQUE constraint on `content_hash` (expected — means dedup hit).
//! - **Trace Scope**: `server-rs::agent::knowledge_store` (Search `[IKS]`)

pub mod types;
pub mod store;
pub mod search;

#[cfg(test)]
pub mod tests;

pub use types::*;
pub use store::KnowledgeStore;

// Metadata: [IKS]
