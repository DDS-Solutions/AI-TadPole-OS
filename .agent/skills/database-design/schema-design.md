> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Schema Design Principles
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Schema Design Principles

> Normalization, primary keys, timestamps, relationships.

## Normalization Decision

```
When to normalize (separate tables):
├── Data is repeated across rows
├── Updates would need multiple changes
├── Relationships are clear
└── Query patterns benefit

When to denormalize (embed/duplicate):
├── Read performance critical
├── Data rarely changes
├── Always fetched together
└── Simpler queries needed
```

## Primary Key Selection

| Type | Use When |
|------|----------|
| **UUID** | Distributed systems, security |
| **ULID** | UUID + sortable by time |
| **Auto-increment** | Simple apps, single database |
| **Natural key** | Rarely (business meaning) |

## Timestamp Strategy

```
For every table:
├── created_at → When created
├── updated_at → Last modified
└── deleted_at → Soft delete (if needed)

Use TIMESTAMPTZ (with timezone) not TIMESTAMP
```

## Relationship Types

| Type | When | Implementation |
|------|------|----------------|
| **One-to-One** | Extension data | Separate table with FK |
| **One-to-Many** | Parent-children | FK on child table |
| **Many-to-Many** | Both sides have many | Junction table |

## Foreign Key ON DELETE

```
├── CASCADE → Delete children with parent
├── SET NULL → Children become orphans
├── RESTRICT → Prevent delete if children exist
└── SET DEFAULT → Children get default value
```

[//]: # (Metadata: [schema_design])

[//]: # (Metadata: [schema_design])
