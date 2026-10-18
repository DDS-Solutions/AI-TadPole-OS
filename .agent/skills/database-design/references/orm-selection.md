> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / database-design
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Mismatched abstraction layer, cold-start bundle bloat, runtime type mismatch, query compilation overhead.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[orm_selection]`)

# ORM & Query Layer Selection Guide

> Select your data access layer based on language runtime, cold-start latency limits, and deployment environment.

---

## 🧭 Query Layer Decision Tree

```
What is your runtime and operational environment?
│
├── Rust (High Performance / Core Daemon / Systems)
│   ├── Compile-Time Checked SQL → sqlx (Zero-cost async, supports SQLite, Postgres, MySQL)
│   └── Synchronous / Heavy Macro ORM → Diesel
│
├── TypeScript / JavaScript (Node.js & Edge Functions)
│   ├── Edge Runtimes / Minimal Bundle / SQL Familiarity → Drizzle ORM
│   ├── Rapid Prototyping / Visual DB GUI / Unified Schema → Prisma
│   └── Type-Safe Query Builder Without Heavy Runtime → Kysely
│
└── Python (Data Engineering / AI Swarms)
    ├── Async Relational Powerhouse → SQLAlchemy 2.0 (with asyncpg / aiosqlite)
    └── Fast Pydantic Integration → SQLModel
```

---

## 📊 Comprehensive ORM Comparison Matrix

| Query Layer | Runtime Target | Primary Strengths | Trade-offs & Watchouts |
|---|---|---|---|
| **`sqlx`** | Rust (Tokio) | Compile-time query verification against live DB; zero runtime reflection overhead; native async. | Requires database connection or offline schema cache (`sqlx-data.json`) during compilation. |
| **Drizzle ORM** | TypeScript / Edge | Tiny bundle footprint (~10kB); SQL-like syntax; zero cold-start delay; runs everywhere (Workers, Node, Browser). | Newer ecosystem; relational query API is still evolving compared to Prisma. |
| **Prisma** | TypeScript / Node | Exceptional developer experience (DX); single `schema.prisma` file; auto-generated type definitions; Prisma Studio. | Heavy binary/WASM query engine (~15MB); cold-start latency penalties in serverless edge runtimes. |
| **Kysely** | TypeScript | Pure type-safe SQL query builder; zero magic; zero engine binaries; full control over SQL generation. | No built-in automated migration generator (requires manual migration scripts). |
| **SQLAlchemy 2.0** | Python | Industry standard; enterprise unit-of-work pattern; mature Alembic migration engine. | Steep learning curve; async setup requires careful session management. |

---

## 🏛️ Codebase Context (Tadpole OS)

1. **Rust Core Daemon**: Uses **`sqlx`** with embedded SQLite in [server-rs/Cargo.toml](file:///D:/TadpoleOS-Dev/server-rs/Cargo.toml). All SQL queries are validated and executed through async Tokio connection pools with zero ORM overhead.
2. **Frontend & Extension Apps**: When scaffolding client tools, web apps, or edge plugins, use **Drizzle** if targeting edge runtimes or **Prisma** if building internal admin portals with complex relations.

---

## 🚫 ORM Anti-Patterns

- ❌ **Using Heavy ORMs on Serverless Edge**: Bundling Prisma into Cloudflare Workers or AWS Lambda where cold-start latency causes perceptible user delays. Use Drizzle or raw queries instead.
- ❌ **Over-Reliance on Lazy Loading**: Allowing the ORM to trigger hidden downstream network roundtrips when accessing entity relationships. Always prefer explicit JOINs or eager batching.
- ❌ **Fighting the ORM on Complex Analytics**: Forcing a complex CTE, window function, or recursive query into ORM syntax. Drop down to raw, parameterized SQL for analytical queries.