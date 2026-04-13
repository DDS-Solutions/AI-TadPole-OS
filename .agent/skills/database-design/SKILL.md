---
name: database-design
description: Database design principles and decision-making. Schema design, indexing strategy, ORM selection.
allowed-tools: Read, Write, Edit, Glob, Grep
---

# Database Design

**Structure data carefully. It outlives code.**

## Core Principle
- ASK for preferences.
- Context determines DB/ORM choice.
- Don't default to Postgres.

## Decision Checklist
- [ ] **DB Choice**: Relational vs Document?
- [ ] **Deployment**: Serverless (Neon/Turso) or Managed?
- [ ] **Indexes**: Planned for read patterns?
- [ ] **Relationships**: Defined correctly?

---

## 🧠 Aletheia Reasoning Protocol (Data Modeling)

### 1. Generator (Schema Options)
*   **SQL vs NoSQL**: "Relational (User -> Orders) or Document (Log Store)?".
*   **Normalization**: "3NF vs Denormalized read speed?".
*   **Keys**: "UUID vs Integer vs CUID?".

### 2. Verifier (Scale Audit)
*   **The "Million Row" Test**: "Will this query survive a table scan?".
*   **Locking**: "Will this migration lock the table?".
*   **Access Pattern**: "Write once, read often?".

### 3. Reviser (Optimization)
*   **Index Tuning**: "Add composite index on (status, created_at)".
*   **Constraint Hardening**: "Add NOT NULL/FKs".

---

## 🛡️ Security & Safety Protocol (Database)

1.  **Injection**: Parameterized Queries / ORM ONLY.
2.  **Least Privilege**: App user cannot `DROP TABLE`.
3.  **Encryption**: Encrypt PII/Tokens at rest.
4.  **Backups**: Verify recovery enabled.
