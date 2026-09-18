> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / api-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[versioning]`)

# Versioning Strategies

> Plan for API evolution from day one.

## Decision Factors

| Strategy | Implementation | Trade-offs |
|----------|---------------|------------|
| **URI** | /v1/users | Clear, easy caching |
| **Header** | Accept-Version: 1 | Cleaner URLs, harder discovery |
| **Query** | ?version=1 | Easy to add, messy |
| **None** | Evolve carefully | Best for internal, risky for public |

## Versioning Philosophy

```
Consider:
├── Public API? → Version in URI
├── Internal only? → May not need versioning
├── GraphQL? → Typically no versions (evolve schema)
├── tRPC? → Types enforce compatibility
└── Handle old APIs professionally with standard sunset headers (e.g., Sunset: <date>)
```