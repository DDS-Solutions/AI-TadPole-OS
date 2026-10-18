---
name: smb-data-ingestion
description: "Parses and validates messy SMB data exports (CSV, Excel, Quickbooks PDFs, legacy SQL) into clean JSON-Schema & SQLite state for AI Digital Twins."
when_to_use: "Use when importing SMB customer records, inventory lists, financial transactions, or legacy database dumps into Sovereign memory or SQLite."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / smb-data-ingestion
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Silent data corruption during type coercion, date parsing failures, or unredacted PII storage.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# SMB Data Ingestion & ETL Protocol

> **Purpose**: Convert un-normalized, messy SMB business data exports (CSV, Excel, accounting dumps) into clean, validated JSON schemas and SQLite state with zero loss of precision.  
> **Workflow Bindings**: Integrated into [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md) and [`/migrate`](file:///D:/TadpoleOS-Dev/.agent/workflows/migrate.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign ETL rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/smb_data_mapping_and_normalization.md`](file:///D:/TadpoleOS-Dev/.agent/skills/smb-data-ingestion/references/smb_data_mapping_and_normalization.md) | Date/currency coercion rules, fuzzy header mapping dictionaries, and transaction batching | Mapping raw CSV columns & normalizing types |
| [`scripts/smb_ingest_validator.py`](file:///D:/TadpoleOS-Dev/.agent/skills/smb-data-ingestion/scripts/smb_ingest_validator.py) | Automated currency string cleaner, ISO-8601 date normalizer, and record validator | Scrubbing and parsing raw legacy files |

---

## 🔄 1. The 4-Phase Ingestion Pipeline

```
[1. DETECT] ───> [2. MAP HEADERS] ───> [3. SANITIZE & PII] ───> [4. LOAD]
 Identify format  Fuzzy match to       Normalize dates/cur,    Parameterized
 & file encoding  domain glossary      scrub PII from text     SQLite batch
```

1. **Format Identification**: Detect file format (CSV, XLSX, JSON, legacy SQL dump) and verify encoding (UTF-8 with fallback to Windows-1252 / latin-1).
2. **Schema Ingestion & Mapping**: Map source column headers to `CONTEXT.md` / `docs/GLOSSARY.md` domain entities using fuzzy matching.
3. **Data Sanitization**:
   - Strip leading/trailing whitespace.
   - Convert currency strings (`"$1,429.50"`) to numeric decimals via `smb_ingest_validator.py`.
   - Normalize dates (`"10/24/2026"`) to ISO 8601 (`"2026-10-24T00:00:00Z"`).
   - Run PII scrubbing on all free-text notes fields prior to storage.
4. **Target Loading**: Load records into SQLite or LanceDB using parameterized batch queries inside a single explicit transaction.

---

## 🧠 2. Socratic Gate for Data Ingestion

Before importing SMB data records into the database, verify:
- **Gate 1 (Zero Data Loss)**: Does every raw row map to either a valid database record or an explicit quarantine error log?
- **Gate 2 (PII Sanitation)**: Have customer emails, phone numbers, and SSNs in unstructured fields passed through `pii-redaction`?
- **Gate 3 (Transaction Safety)**: Are inserts wrapped in a single database transaction with automatic rollback on error?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **String Concatenated SQL** | Building `INSERT INTO ... VALUES ('" + val + "')` causes SQL injection. | Always use parameterized bindings (`?` or `$1`). |
| **Row-by-Row Inserts** | Inserting 50,000 rows without a transaction takes 10 minutes. | Wrap all inserts in a single transaction block (`BEGIN` ... `COMMIT`). |
| **Silent Type Coercion** | Silently converting `"N/A"` to `0.00` corrupts financial metrics. | Distinguish `NULL` / missing values from actual zero balances. |
| **Encoding Crashes** | Assuming pure ASCII on Windows exports crashes on `cp1252` accents. | Open files with `errors='replace'` or auto-detect encoding. |

---

## 🛠️ 4. Deterministic Verification Gate

Validate data cleaning and date normalization:

```powershell
# 1. Run built-in self-tests
python .agent/skills/smb-data-ingestion/scripts/smb_ingest_validator.py --test

# 2. Test currency string normalization
python .agent/skills/smb-data-ingestion/scripts/smb_ingest_validator.py --currency "$1,249.50"

# 3. Test date string normalization
python .agent/skills/smb-data-ingestion/scripts/smb_ingest_validator.py --date "10/24/2026"
```