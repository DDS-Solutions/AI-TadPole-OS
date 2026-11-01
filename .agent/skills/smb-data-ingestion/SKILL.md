---
name: smb-data-ingestion
description: "Defines preview-only validation for synthetic SMB data samples pending a company-scoped runtime."
when_to_use: "Use when preparing synthetic SMB sample data for a future company-scoped ingestion design; never load customer data through the generic runner."
allowed-tools: Read, Glob, Grep
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

> **Purpose**: Specify how synthetic SMB samples could be mapped and validated for a future company-scoped ingestion workflow.
> **Execution Plane**: **Plane 2 — Company-Agnostic SMB AI Digital Twin**. The isolated storage requirements are specified in [`docs/specs/mock-smb-digital-twin.md`](../../../docs/specs/mock-smb-digital-twin.md), whose implementation is pending. This skill does not provide or verify workspace isolation. Do not write imported records through unrestricted filesystem or shell tools, or into the engine database.
> **`FR-11` runtime status**: The generic runner rejects only identities whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive). Skill metadata does not enforce this boundary; use synthetic samples and stop at preview.
> **Workflow Bindings**: Integrated into [`/create`](../../workflows/create.md) and [`/migrate`](../../workflows/migrate.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign ETL rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/smb_data_mapping_and_normalization.md`](./references/smb_data_mapping_and_normalization.md) | Date/currency coercion rules, fuzzy header mapping dictionaries, and transaction batching | Mapping raw CSV columns & normalizing types |
| [`scripts/smb_ingest_validator.py`](./scripts/smb_ingest_validator.py) | Automated currency string cleaner, ISO-8601 date normalizer, and record validator | Scrubbing and parsing raw legacy files |

---

## 🔄 1. The 4-Phase Ingestion Pipeline

```
[1. DETECT] ───> [2. MAP HEADERS] ───> [3. SANITIZE & PII] ───> [4. PREVIEW]
 Identify format  Map to a future       Normalize sample data,  Validate only;
 & file encoding  approved schema        scrub synthetic text    no live writes
```

1. **Format Identification**: Detect file format (CSV, XLSX, JSON, legacy SQL dump) and verify encoding (UTF-8 with fallback to Windows-1252 / latin-1).
2. **Schema Mapping**: For synthetic samples, map source headers to a proposed schema. Do not load real exports through the generic runner.
3. **Data Sanitization**:
   - Strip leading/trailing whitespace.
   - Convert currency strings (`"$1,429.50"`) to numeric decimals via `smb_ingest_validator.py`.
   - Normalize dates (`"10/24/2026"`) to ISO 8601 (`"2026-10-24T00:00:00Z"`).
   - Run PII scrubbing on all free-text notes fields prior to storage.
4. **Preview Only**: Validate synthetic samples and produce a preview or quarantine report. Stop here until company-scoped authorization, storage, and an approved connector are implemented.

The outward product catalog accepts explicit `price`, `price_usd`, or `unit_price` columns. It retains only allowlisted product metadata; unknown columns are dropped, including ambiguous `cost` columns.

---

## 🧠 2. Socratic Gate for Data Ingestion

Before a future company-scoped runtime imports SMB data, verify:
- **Gate 1 (Zero Data Loss)**: Does every raw row map to either a valid database record or an explicit quarantine error log?
- **Gate 2 (PII Sanitation)**: Have customer emails, phone numbers, and SSNs in unstructured fields passed through `pii-redaction`?
- **Gate 3 (Transaction Safety)**: Are inserts wrapped in a single database transaction with automatic rollback on error? The current preview workflow performs no database writes.

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

These scripts require an authorized operator or a dedicated, sandboxed runner. The Plane 2 skill does not grant general command execution; do not run them from a business agent until that execution boundary exists.

```powershell
# 1. Run built-in self-tests
python .agent/skills/smb-data-ingestion/scripts/smb_ingest_validator.py --test

# 2. Test currency string normalization
python .agent/skills/smb-data-ingestion/scripts/smb_ingest_validator.py --currency "$1,249.50"

# 3. Test date string normalization
python .agent/skills/smb-data-ingestion/scripts/smb_ingest_validator.py --date "10/24/2026"
```
