# Static Analysis & Quality Gate Protocols (L3)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / lint-and-validate / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Type mismatches, uninspected lint regressions, or ADG manifest drift.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[LINT_AND_VALIDATE]`)

---

## 1. The 4-Tier Sovereign Quality Gate Hierarchy

In Tadpole OS, code verification follows a deterministic, 4-tier hierarchy. No code modification is considered complete until all applicable tiers pass with zero errors:

```
┌────────────────────────────────────────────────────────┐
│ TIER 1: Syntax & Type Compilation                      │
│ (npx tsc --noEmit, cargo check)                        │
├────────────────────────────────────────────────────────┤
│ TIER 2: Static Analysis & Linting                      │
│ (npm run lint, cargo clippy, ruff check)               │
├────────────────────────────────────────────────────────┤
│ TIER 3: Safety Invariants & Active Documentation (ADG) │
│ (npm run adg:verify, npm run test:invariants:fast)     │
├────────────────────────────────────────────────────────┤
│ TIER 4: System Parity & AI Context Guard               │
│ (python execution/parity_guard.py, verify_doc_links.py)│
└────────────────────────────────────────────────────────┘
```

---

## 2. Tooling Commands by Ecosystem

### 1. TypeScript & Frontend (`src/`)
```powershell
# 1. Type Safety Check (Zero emitted files, full compile check)
npx tsc --noEmit

# 2. ESLint & Code Formatting
npm run lint

# 3. Production Bundle Validation
npm run build
```

### 2. Rust Backend (`server-rs/`)
```powershell
# 1. Fast Compilation & Syntax Check
cargo check --manifest-path server-rs/Cargo.toml

# 2. Strict Clippy Linter (Zero warnings policy)
cargo clippy --manifest-path server-rs/Cargo.toml --all-targets -- -D warnings

# 3. Code Formatting Check
cargo fmt --manifest-path server-rs/Cargo.toml -- --check
```

### 3. Python Automation & Scripts (`execution/`, `.agent/`)
```powershell
# 1. Fast Modern Linter
ruff check .

# 2. Strict Type Hint Checking
mypy execution/

# 3. AI Context Alignment & Header Check
python execution/verify_ai_context.py .
```

### 4. Sovereign Invariants & ADG v2 Gates
```powershell
# 1. ADG v2 Verification (No hand-written fact claims, manifest integrity)
npm run adg:verify

# 2. Fast Invariant Test Suite
npm run test:invariants:fast

# 3. System Parity Guard
python execution/parity_guard.py .
```

---

## 3. Strict Rules & Anti-Patterns

1. **No `@ts-ignore` or `any` Escapes**: Never silence type errors using `@ts-ignore` or casting to `any`. Fix the interface or use a discriminated union.
2. **No `# noqa` or `# type: ignore`**: Do not suppress linting or typing warnings in Python scripts without explicit justification.
3. **No Clippy Allow Directives**: Never insert `#![allow(clippy::all)]` into Rust hub routes. Resolve compiler warnings directly.
4. **No Broken Invariant Commit**: Any commit that breaks `npm run test:invariants:fast` violates core Sovereign Zero Trust invariants.

---

## 4. Verification Tools

Run the unified lint and type coverage checkers:
- [`scripts/lint_runner.py`](../scripts/lint_runner.py)
- [`scripts/type_coverage.py`](../scripts/type_coverage.py)
