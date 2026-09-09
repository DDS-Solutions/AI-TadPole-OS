> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / clean-code
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: God-objects, deep nesting, or unformatted git commits.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[CLEAN_CODE]`)

# Clean Code & Version Control Standards (L3)

---

## 1. The 70/20/10 Testing Pyramid

| Level | Share | Target | Tools |
|---|---|---|---|
| **Unit Tests** | 70% | Pure functions, domain calculations, state reducers | `cargo test`, `vitest`, `pytest` |
| **Integration Tests** | 20% | Route handlers, database queries, WebSocket events | Axum TestClient, Supertest |
| **E2E Tests** | 10% | Critical user journeys & multi-agent swarms | Playwright |

---

## 2. Git Commit Standards (Conventional & Imperative)

- Use imperative present tense: `feat(sec): enforce Ed25519 token verification`
- Atomic commits: Exactly one logical architectural change per commit.