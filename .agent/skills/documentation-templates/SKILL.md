> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: documentation-templates
description: Documentation templates and structure guidelines. README, API docs, code comments.
allowed-tools: Read, Glob, Grep
---

# Documentation Templates

## README Structure
1.  **Title + One-liner**
2.  **Quick Start** (<5 min)
3.  **Features**
4.  **Configuration**
5.  **API/Docs Links**
6.  **License**

## API Doc Template
```markdown
## GET /resource/:id
**Params**: `id` (Required)
**Response**: 200 OK (Object), 404 (Not Found)
```

## Code Comments
- **Comment**: Why (Logic), Complex Algos, API Contracts.
- **Don't Comment**: What (Obvious), Implementation Details.

---

## 🧠 Aletheia Reasoning Protocol (Technical Writing)

**Documentation is a product.**

### 1. Generator (Audience Mapping)
*   **Persona**: "Junior (Steps) vs Senior (Trade-offs)?".
*   **Format**: "Tutorial vs Reference?".
*   **Outcome**: "User can X after reading.".

### 2. Verifier (The "Copy-Paste" Test)
*   **Execute**: "Does the code block actually work?".
*   **Completeness**: "Did I skip prerequisites?".
*   **Freshness**: "Is screenshots outdated?".

### 3. Reviser (Clarity)
*   **Conciseness**: "Delete 'In order to'".
*   **Active Voice**: "System sends..." (not "Is sent by").

---

## 🛡️ Security & Safety Protocol (Docs)

1.  **Secrets**: NEVER document real keys. Use placeholders.
2.  **Internal**: No internal URLs/vulnerabilities.
3.  **Safety**: Warning banners for destructive commands.
4.  **Links**: Verify execution safety.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
