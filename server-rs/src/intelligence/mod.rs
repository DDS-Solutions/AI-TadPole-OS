//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / intelligence
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling, bounded graph traversal, and fault-tolerant parsing.
//!
//! ### Subsystem Responsibilities
//! - **`graph`** (`CodeSymbolGraph`): In-memory runtime symbol graph, incremental AST cache, and fast BFS blast-radius analysis.
//! - **`graph_store`**: Persisted SQLite code review database with FTS5 search, criticality-ranked architectural flows, and risk scoring.
//! - **`markdown_graph`** (`MarkdownMemoryGraph`): In-memory Markdown DAG, lexical path resolution, and bounded ancestor breadcrumbs.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `graph::GraphError`, `crate::error::AppError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `intelligence::graph::tests`, `intelligence::graph_store::tests`, `intelligence::markdown_graph::tests`

pub mod graph;
pub mod graph_store;
pub mod markdown_graph;

pub use graph::EXCLUDED_DIRS;
