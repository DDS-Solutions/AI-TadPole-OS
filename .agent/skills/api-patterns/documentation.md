> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / api-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[documentation]`)

# API Documentation Principles

> Good docs = happy developers = API adoption.

## OpenAPI/Swagger Essentials

```
Include:
├── All endpoints with examples
├── Request/response schemas
├── Authentication requirements
├── Error response formats
└── Rate limiting info
```

## Good Documentation Has

```
Essentials:
├── Quick start / Getting started
├── Authentication guide
├── Complete API reference
├── Error handling guide
├── Code examples (multiple languages)
├── Interactive testing interface (e.g., Swagger UI, Stoplight)
└── Changelog
```