> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / database-design
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Latency spikes, connection pool starvation, sequential scan bottlenecks, unindexed query plans.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[optimization]`)

# Query Optimization & Performance Tuning

> Always inspect actual query execution plans with EXPLAIN before speculating on database performance bottlenecks.

---

## 🛑 The N+1 Query Problem & Solutions

The N+1 problem occurs when an application executes 1 initial query to fetch $N$ parent records, followed by $N$ secondary queries to fetch related child records for each parent.

```
Initial Query: SELECT * FROM agents LIMIT 100;                -- 1 query
Follow-up:     SELECT * FROM missions WHERE agent_id = 1;     -- 100 separate queries!
               SELECT * FROM missions WHERE agent_id = 2;
               ...
Total Roundtrips: 101 queries (Catastrophic latency overhead)
```

### Remediation Strategies:
1. **Relational JOIN (Single Roundtrip)**:
   ```sql
   SELECT a.id, a.name, m.id AS mission_id, m.title
   FROM agents a
   LEFT JOIN missions m ON m.agent_id = a.id
   WHERE a.status = 'ACTIVE';
   ```
2. **Batched Subquery (`IN (...)`)**:
   Fetch all child records in a second single query:
   ```sql
   SELECT * FROM missions WHERE agent_id IN (1, 2, 3, ..., 100);
   ```
3. **ORM Eager Loading**:
   In Prisma, use `include: { missions: true }`. In Drizzle, use relational query syntax `db.query.agents.findMany({ with: { missions: true } })`.
4. **DataLoader (Batch & Cache)**:
   In GraphQL or microservices, coalesce concurrent requests within a single tick into a batched ID array.

---

## 🔬 Query Plan Analysis (EXPLAIN)

### PostgreSQL: `EXPLAIN (ANALYZE, BUFFERS)`
```sql
EXPLAIN (ANALYZE, BUFFERS) 
SELECT * FROM audit_logs WHERE agent_id = 'agent-007' ORDER BY created_at DESC LIMIT 50;
```
- **Sequential Scan (`Seq Scan`)**: The database examined every single page in the table. Red flag if table is large.
- **Index Scan**: Traversed B-tree index, then read heap table rows.
- **Index Only Scan**: Data resolved entirely from the index without reading the heap table (optimal).
- **Sort Method (External / Memory)**: If sort spills to disk (`external merge Disk`), increase `work_mem` or add an index covering the `ORDER BY` columns.

### SQLite: `EXPLAIN QUERY PLAN`
```sql
EXPLAIN QUERY PLAN 
SELECT * FROM audit_trail WHERE agent_id = '01' ORDER BY seq_num DESC LIMIT 20;
```
- Look for: `SCAN TABLE audit_trail` $\rightarrow$ full table scan (needs index).
- Desired output: `SEARCH TABLE audit_trail USING INDEX idx_audit_agent_seq`.

---

## 📄 High-Performance Pagination: Offset vs Keyset

| Pagination Method | SQL Syntax | Performance at Depth | Safe For Concurrent Inserts? |
|---|---|---|---|
| **Offset Pagination** | `LIMIT 20 OFFSET 50000` | Degrades quadratically ($O(N)$); DB must scan & discard 50,000 rows. | ❌ Flaky (rows shift or duplicate when new records arrive). |
| **Keyset (Cursor) Pagination** | `WHERE id > :cursor ORDER BY id ASC LIMIT 20` | Constant time ($O(1)$) utilizing the primary key or indexed timestamp. | ✅ Stable and immune to shift anomalies. |

---

## ⚡ High-Throughput Batch Operations

1. **Multi-Row Insert**:
   Never loop over individual `INSERT` statements. Coalesce into batch inserts:
   ```sql
   -- Optimal: 1 transaction, 1 network roundtrip
   INSERT INTO telemetry_events (event_id, timestamp, payload) 
   VALUES 
     ('e1', 1700000000, '{"val": 1}'),
     ('e2', 1700000001, '{"val": 2}'),
     ('e3', 1700000002, '{"val": 3}');
   ```
2. **Explicit Transactions**:
   In SQLite, every standalone `INSERT` triggers an implicit transaction and disk fsync. Wrapping 1,000 inserts in `BEGIN TRANSACTION; ... COMMIT;` accelerates performance by over 100x.