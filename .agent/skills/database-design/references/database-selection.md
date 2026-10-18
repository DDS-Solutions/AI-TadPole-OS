> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / database-design
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Suboptimal database engine selection, resource starvation, edge latency penalties.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[database_selection]`)

# Database Selection Guide

> Choose database based on operational topology and latency requirements, never by unthinking habit.

---

## 🧭 Database Selection Decision Tree

```
What are your deployment and latency requirements?
│
├── Sovereign Local-First / Desktop / Embedded Node
│   ├── Primary Structured State → SQLite (via sqlx in Rust or better-sqlite3 in Node)
│   └── Dense Vector Memory / Semantic Embeddings → LanceDB (columnar, zero-server)
│
├── Edge Deployment / Ultra-Low Global Latency
│   └── Turso (libSQL distributed edge SQLite)
│
├── Cloud Relational / Multi-Tenant SaaS
│   ├── Serverless & Ephemeral Branching → Neon / Supabase (PostgreSQL)
│   └── High-Throughput Dedicated Cluster → Self-hosted PostgreSQL
│
└── Massive Horizontal Scale / Global Multi-Master
    └── CockroachDB / PlanetScale (Distributed MySQL/PG)
```

---

## 📊 Comprehensive Comparison Matrix

| Database Engine | Architecture | Best For | Trade-offs & Limitations |
|---|---|---|---|
| **SQLite (`sqlx`)** | Embedded, serverless, single-file | Local-first apps, CLI tools, sovereign nodes, desktop apps | Single-writer lock concurrency; requires WAL mode for concurrent readers. |
| **LanceDB** | Embedded columnar vector store | Dense AI embeddings, vector search, zero-daemon RAG | Optimized for append & scan; not designed for transactional row updates. |
| **PostgreSQL** | Client-server relational | Complex relational queries, JSONB, row-level security (RLS) | High operational overhead; requires persistent connection pooling (e.g. PgBouncer). |
| **Neon** | Serverless PostgreSQL with storage-compute separation | Modern cloud web apps, staging branching, autoscaling | Cold-start latency on inactive branches; cloud dependency. |
| **Turso** | Distributed libSQL at the edge | Edge runtimes (Cloudflare Workers, Vercel Edge), low read latency | Write queries route to primary location; limited complex extensions. |
| **Supabase** | Managed PostgreSQL + PostgREST + Auth | Rapid full-stack prototypes requiring instant auth and realtime | Vendor lock-in risk around Supabase proprietary tooling. |

---

## 🏛️ Tadpole OS Standards

1. **Embedded Sovereign Core**: The core daemon executes against local SQLite via `sqlx` ([server-rs/Cargo.toml](file:///D:/TadpoleOS-Dev/server-rs/Cargo.toml)). No external database daemon is required for desktop node operations.
2. **Dense Memory Subsystem**: Vector storage is backed by LanceDB (`vector-memory` feature flag) with Arrow columnar indexing for high-speed similarity search without external vector DB infrastructure.
3. **External Integrations**: For external cloud services or dashboards, prefer Neon or Supabase when relational integrity and row-level security are paramount.

---

## ❓ Architectural Discovery Checklist

Before finalizing engine selection, answer these 5 questions:
1. **Deployment Boundary**: Will this run on an offline sovereign edge device or a centralized cloud cluster?
2. **Concurrency Profile**: What is the ratio of concurrent writes to concurrent reads? (SQLite excels when reads dominate; heavy concurrent writes need PostgreSQL).
3. **Vector / RAG Needs**: Does the application require nearest-neighbor vector search? (Use LanceDB locally or `pgvector` in PostgreSQL).
4. **Latency Budget**: Is cold start or network roundtrip acceptable? (Embedded engines provide sub-millisecond local reads).
5. **Operational Complexity**: Can the customer or environment maintain an external database cluster?