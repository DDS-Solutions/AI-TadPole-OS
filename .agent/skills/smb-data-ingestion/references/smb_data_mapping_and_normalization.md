# SMB Data Mapping, Normalization & Ingestion Schemas

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / smb-data-ingestion / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. Normalization Rules for Legacy SMB Data

| Raw Source Type | Example Raw Input | Normalized Output Format |
|---|---|---|
| **Currency** | `"$1,429.50"`, `"(45.00)"`, `"1.250,00 €"` | Numeric decimal (`1429.50`, `-45.00`) |
| **Dates** | `"10/24/2026"`, `"24-Oct-26"`, `"2026.10.24"` | ISO 8601 (`"2026-10-24T00:00:00Z"`) |
| **Phone Numbers** | `"(555) 123-4567"`, `"555.123.4567"` | E.164 (`"+15551234567"`) |
| **Booleans** | `"Y"`, `"yes"`, `"1"`, `"true"`, `"T"` | Boolean (`true` / `false`) |
| **Free-text Notes** | Freeform text with customer info | Scrubbed via `pii-redaction` prior to persistence |

---

## 2. Ingestion Pipeline & Quality Gates

```
[RAW EXPORT: CSV / XLSX / JSON]
               │
               ▼
   [ENCODING DETECTION (UTF-8 / Windows-1252)]
               │
               ▼
   [HEADER FUZZY MAPPING (Domain Glossary)]
               │
               ▼
   [TYPE COERCION & DATE/CURRENCY NORMALIZATION]
               │
               ▼
   [PII REDACTION ON UNSTRUCTURED FIELDS]
               │
               ▼
   [PARAMETERIZED INSERT INTO SQLITE / LANCEDB]
```

---

## 3. SQLite Ingestion Invariants
- Always wrap batch inserts in a single transaction (`BEGIN TRANSACTION` ... `COMMIT`).
- Use parameterized bindings (`?` or `$1`) to eliminate SQL injection hazards.
- Retain an audit trail of row count imported vs rows rejected due to corruption.
