> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Infrastructure:Execution**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> REST Principles
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# REST Principles

> Resource-based API design - nouns not verbs.

## Resource Naming Rules

```
Principles:
├── Use NOUNS, not verbs (resources, not actions)
├── Use PLURAL forms (/users not /user)
├── Use lowercase with hyphens (/user-profiles)
├── Nest for relationships (/users/123/posts)
└── Keep shallow (max 3 levels deep)
```

## HTTP Method Selection

| Method | Purpose | Idempotent? | Body? |
|--------|---------|-------------|-------|
| **GET** | Read resource(s) | Yes | No |
| **POST** | Create new resource | No | Yes |
| **PUT** | Replace entire resource | Yes | Yes |
| **PATCH** | Partial update | No | Yes |
| **DELETE** | Remove resource | Yes | No |

## Status Code Selection

| Situation | Code | Why |
|-----------|------|-----|
| Success (read) | 200 | Standard success |
| Created | 201 | New resource created |
| No content | 204 | Success, nothing to return |
| Bad request | 400 | Malformed request |
| Unauthorized | 401 | Missing/invalid auth |
| Forbidden | 403 | Valid auth, no permission |
| Not found | 404 | Resource doesn't exist |
| Conflict | 409 | State conflict (duplicate) |
| Validation error | 422 | Valid syntax, invalid data |
| Rate limited | 429 | Too many requests |
| Server error | 500 | Our fault |

[//]: # (Metadata: [rest])

[//]: # (Metadata: [rest])
