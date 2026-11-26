# Clean Code, Refactoring & Git Standards (L3 Reference)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / clean-code / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: God-objects, deep nesting, swallowed errors, unformatted git commits.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This guide provides concrete refactoring patterns, the 70/20/10 testing pyramid architecture, and Conventional Commits standards for the Tadpole OS repository.

---

## 1. The 70/20/10 Testing Pyramid

Quality is built in layers. Never invert the pyramid into an ice-cream cone of slow, flaky E2E tests:

```
                         ▲
                        / \
                       /E2E\      10% (Critical journeys & multi-agent swarms)
                      /-----\
                     / INTEG \    20% (IPC, API routes, database, WebSockets)
                    /---------\
                   /   UNIT    \  70% (Pure functions, domain logic, state reducers)
                  /_____________\
```

### Layer Breakdown & Verification Tooling

| Level | Target Coverage | Scope & Boundaries | Verification Command |
|---|---|---|---|
| **Unit Tests** | ~70% of tests | Isolated logic, pure domain algorithms, math, validation rules, reducers. No network or disk I/O. | `npx vitest run tests/invariants/` / `cargo test --lib` |
| **Integration Tests** | ~20% of tests | Axum routes, Tauri IPC commands, SQLite/LanceDB transactions, WebSocket event roundtrips. | `cargo test --test integration` / `npx vitest run tests/integration/` |
| **E2E Tests** | ~10% of tests | End-to-end user workflows, full desktop UI startup, multi-agent orchestration. | `npx playwright test` |

---

## 2. Refactoring Catalog & Code Smell Remedies

### Smell 1: Long Function (Decompose)
- **Threshold**: Function exceeds 50 lines or does more than one logical operation.
- **Remedy**: **Extract Method**. Group cohesive statements into a private helper function whose name reveals the intent.

### Smell 2: Arrow Anti-Pattern / Deep Nesting (Flatten)
- **Threshold**: Code indents deeper than 3 levels (e.g. nested `if` / `else` / `for` / `match`).
- **Remedy**: **Guard Clauses / Early Returns**. Invert conditional statements to exit early on error or boundary conditions.

```typescript
// ❌ BAD: Deep nesting
function handlePayment(user: User, amount: number) {
  if (user) {
    if (user.isActive) {
      if (amount > 0) {
        processTransaction(user, amount);
      } else {
        throw new Error("Invalid amount");
      }
    } else {
      throw new Error("Inactive user");
    }
  } else {
    throw new Error("User missing");
  }
}

// ✅ GOOD: Guard clauses
function handlePayment(user: User, amount: number) {
  if (!user) throw new Error("User missing");
  if (!user.isActive) throw new Error("Inactive user");
  if (amount <= 0) throw new Error("Invalid amount");

  processTransaction(user, amount);
}
```

### Smell 3: Swallowed Errors (Contextual Propagation)
- **Threshold**: Empty `catch (e) {}`, `except: pass`, or uninformative error dropping.
- **Remedy**: Catch specifically, log telemetry if appropriate, and wrap with domain context or return typed Result.

```rust
// ❌ BAD: Swallowed error / panic in production
let config = std::fs::read_to_string("config.json").unwrap();

// ✅ GOOD: Typed domain error propagation
let config = std::fs::read_to_string("config.json")
    .map_err(|e| ConfigError::ReadFailed { path: "config.json".into(), source: e })?;
```

### Smell 4: Magic Numbers & Magic Strings (Symbolic Constants)
- **Threshold**: Literal numbers (`86400`, `30`, `5000`) or raw string identifiers embedded directly in algorithmic logic.
- **Remedy**: Extract to uppercase named constants at file or module headers (`SESSION_TTL_SECS`, `MAX_RETRIES`).

---

## 3. Conventional Commits 1.0 Standards

All commits in this repository must strictly adhere to the Conventional Commits specification:

```
<type>(<scope>): <imperative present tense description>

[optional body explaining non-obvious WHY, context, or trade-offs]

[optional footer(s) e.g., BREAKING CHANGE: <explanation>]
```

### Allowed Types
- **`feat`**: A new user-facing or agent capability.
- **`fix`**: A bug fix or defect correction.
- **`refactor`**: Code changes that neither fix a bug nor add a feature.
- **`perf`**: A code change that improves performance or decreases memory/token footprints.
- **`test`**: Adding missing tests or correcting existing tests.
- **`docs`**: Documentation-only changes.
- **`chore`**: Build scripts, dependency updates, configuration tweaks.

### Standard Scopes for Tadpole OS
- `(core)`: Rust kernel, server-rs, or Tokio runtime.
- `(tauri)`: Tauri IPC bridge, desktop windowing, native connectors.
- `(neural)`: Model routing, Ollama/Groq providers, embeddings.
- `(sec)`: Vault encryption, capability keys, Zero-Trust permissions.
- `(adg)`: Active Documentation Guard, lint headers, manifests.
- `(skills)`: Agent skills registry and deterministic tooling.
- `(ui)`: React pages, components, Zustand stores.

### Examples:
- `feat(sec): enforce Ed25519 token signature verification`
- `fix(neural): resolve timeout race condition during Ollama model switch`
- `refactor(skills): decouple Socratic Gate validation from chat presentation`

---

## 4. Pre-PR Quality Checklist

Before submitting code for review or declaring a task complete:
- [ ] No files exceed 500 lines.
- [ ] No functions exceed 50 lines.
- [ ] No nesting exceeds 3 indentation levels.
- [ ] Zero `as any` in TypeScript; zero unhandled `.unwrap()` in production Rust.
- [ ] All new functions have unit tests adhering to the 70% pyramid base.
- [ ] `npm run adg:verify` passes with 0 header violations.
- [ ] `python execution/parity_guard.py` returns 0 drift errors.