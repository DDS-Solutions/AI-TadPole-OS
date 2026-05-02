> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Infrastructure:Execution**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: api-patterns
description: API design principles and decision-making. REST vs GraphQL vs tRPC selection, response formats, versioning, pagination.
allowed-tools: Read, Write, Edit, Glob, Grep
---

# API Patterns

**Think before designing.**

## Decision Checklist
- [ ] **Style**: REST (Public) vs GraphQL (Complex) vs tRPC (TS Monorepo).
- [ ] **Versioning**: URL (`/v1`) vs Header.
- [ ] **Auth**: JWT, OAuth, Keys.
- [ ] **Protection**: Rate Limiting, CORS.
- [ ] **Docs**: OpenAPI/Swagger.

---

## 🧠 Aletheia Reasoning Protocol (API Design)

**Protocols are forever. Design with evolution in mind.**

### 1. Generator (Interface Styles)
*   **Style Options**: "Is this a resource graph (GraphQL), a simple resource (REST), or an action (RPC)?".
*   **Consumer Mapping**: "Mobile needs small payloads. Desktop needs high throughput.".
*   **Version Strategy**: "Header versioning vs URL versioning?".

### 2. Verifier (The Contract Test)
*   **Chatty Check**: "Does the client need 5 calls to show one screen?".
*   **Breaking Change**: "If I rename this field, who breaks?".
*   **Error Leaks**: "Does the 500 status return a stack trace?".

### 3. Reviser (Simplification)
*   **Flatten**: Reduce nested JSON depth.
*   **Standardize**: Ensure all dates are ISO 8601 UTC.

---

## 🛡️ Security & Safety Protocol (API)

1.  **Auth is P0**: Never design an "open" API unless clearly public data. Plan AuthN/AuthZ first.
2.  **Rate Limiting**: Every API MUST have rate limiting (Token Bucket/Leaky Bucket) to prevent abuse.
3.  **Input Validation**: Validate ALL inputs. Reject unknown fields (Allowlist over Blocklist).
4.  **No Enumeration**: Use UUIDs or non-sequential IDs to prevent resource scraping.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
