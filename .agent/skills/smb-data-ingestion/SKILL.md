> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SKILL]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for ingesting, validating, and converting messy SMB data files into clean Digital Twin database state.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

---
name: smb-data-ingestion
description: Parses and validates messy SMB data exports (CSV, Excel, Quickbooks PDFs, legacy SQL) into clean JSON-Schema & SQLite state for AI Digital Twins.
when_to_use: "Use when importing SMB customer records, inventory lists, financial transactions, or legacy database dumps."
allowed-tools: Read, Glob, Grep, Bash, Write, Edit
---

# SMB Data Ingestion & ETL Protocol

> Purpose: Convert un-normalized SMB data exports into structured Digital Twin state with zero data loss.

---

## Operating Steps

1. **Format Identification**:
   Identify file format (CSV, XLSX, JSON, SQL dump, unstructured text).
2. **Schema Ingestion & Mapping**:
   Map source column headers to `CONTEXT.md` domain entities using fuzzy matching and type validation.
3. **Data Sanitization**:
   - Strip trailing spaces, normalize date formats (ISO 8601), convert currency strings to numeric decimals.
   - Run `@[skills/pii-redaction]` on notes/free-text fields before saving.
4. **Target Loading**:
   Load clean records into target SQLite/LanceDB table via parameterized queries or `sqlx` migrations.
