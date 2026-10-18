---
name: lint-and-validate
description: "Automatic quality control, linting, and static analysis procedures. Use after every code modification to ensure syntax correctness and project standards. Triggers onKeywords: lint, format, check, validate, types, static analysis."
when_to_use: "After any code change to ensure zero lint errors, type correctness, invariant enforcement, and ADG v2 compliance before committing or declaring work complete."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / lint-and-validate
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Type mismatches, uninspected lint regressions, or ADG manifest drift.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Lint and Validate Quality Gate Protocol

> **Sovereign Invariant**: Run appropriate validation gates after **EVERY** code modification. No task may be declared complete or reported as "done" while syntax errors, type regressions, lint warnings, or broken invariants remain.  
> **Workflow Bindings**: Integrated into [`/test`](file:///D:/TadpoleOS-Dev/.agent/workflows/test.md), [`/refactor`](file:///D:/TadpoleOS-Dev/.agent/workflows/refactor.md), and [`/audit`](file:///D:/TadpoleOS-Dev/.agent/workflows/audit.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/static_analysis_and_quality_gates.md`](file:///D:/TadpoleOS-Dev/.agent/skills/lint-and-validate/references/static_analysis_and_quality_gates.md) | The 4-Tier Quality Gate Hierarchy, Clippy policies, and ADG v2 standards | Auditing multi-ecosystem quality gates & configuring linters |
| [`scripts/lint_runner.py`](file:///D:/TadpoleOS-Dev/.agent/skills/lint-and-validate/scripts/lint_runner.py) | Unified multi-ecosystem runner (Node.js, Rust Cargo, Python) with `--test` mode | Running all applicable linters in a single deterministic pass |
| [`scripts/type_coverage.py`](file:///D:/TadpoleOS-Dev/.agent/skills/lint-and-validate/scripts/type_coverage.py) | Type safety and coverage auditor detecting `any`/`Any` escapes | Auditing TypeScript and Python type coverage ratios |

---

## 🚦 1. The 4-Tier Sovereign Quality Hierarchy

Execute checks from lowest to highest tier:

```
[TIER 1: SYNTAX & TYPES] ──> [TIER 2: LINTERS] ──> [TIER 3: INVARIANTS/ADG] ──> [TIER 4: PARITY]
  tsc / cargo check            eslint / clippy          adg:verify / vitest        parity_guard.py
```

### Tier 1: Syntax & Type Compilation
- **TypeScript (`src/`)**: `npx tsc --noEmit`
- **Rust (`server-rs/`)**: `cargo check --manifest-path server-rs/Cargo.toml`
- **Python**: `python -m py_compile <script.py>`

### Tier 2: Static Analysis & Linting
- **TypeScript**: `npm run lint`
- **Rust**: `cargo clippy --manifest-path server-rs/Cargo.toml --all-targets -- -D warnings`
- **Python**: `ruff check <path>`

### Tier 3: Safety Invariants & Active Documentation (ADG v2)
- **ADG v2 Verification**: `npm run adg:verify`
- **Fast Invariant Test Suite**: `npm run test:invariants:fast`

### Tier 4: System Parity & AI Context Guard
- **Documentation Parity**: `python execution/parity_guard.py .`
- **AI Context Alignment**: `python execution/verify_ai_context.py <modified_files>`
- **Internal Doc Links**: `python execution/verify_doc_links.py`

---

## 🔄 2. The Quality Loop

1. **Write/Edit Code**: Implement changes adhering to clean-code standards.
2. **Execute Tier 1 & 2**: Run `npx tsc --noEmit` and `npm run lint` (or `cargo check` / `cargo clippy` if backend).
3. **Execute Tier 3**: Run `npm run adg:verify` and `npm run test:invariants:fast`.
4. **Fix & Verify**: Never bypass a failing gate by silencing it with `@ts-ignore`, `any`, or `# noqa`.

---

## 🧠 3. Socratic Gate for Quality Validation

Before reporting task completion, verify these 4 gates:
- **Gate 1 (Zero Warnings Policy)**: Did `npx tsc --noEmit` and `cargo clippy` complete with zero errors and zero warnings?
- **Gate 2 (No Type Escapes)**: Are there any newly introduced `: any` or `@ts-ignore` escapes?
- **Gate 3 (ADG v2 Parity)**: Does `npm run adg:verify` confirm that manifest integrity is current with no v1 forbidden headers?
- **Gate 4 (Invariant Safety)**: Did all executable invariant test suites in `tests/invariants/` pass?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Type Escape via `any`** | Silencing compiler warnings with `: any` creates silent runtime crashes. | Define strict interfaces or discriminated union types. |
| **`@ts-ignore` Masking** | Suppressing TypeScript errors instead of resolving broken contracts. | Fix the root type mismatch; `@ts-ignore` is strictly prohibited. |
| **Rust Clippy Suppression** | Adding `#![allow(clippy::all)]` to silence compiler warnings. | Refactor code to satisfy Clippy; keep warnings as errors (`-D warnings`). |
| **Skipping Invariant Tests** | Changing core security/vault code without running invariant tests. | Always run `npm run test:invariants:fast` before concluding any edit. |
| **Hand-Written Fact Claims** | Writing `Witness Tests:` or `Invariants:` manually in source file headers. | Forbidden v1 pattern flagged by `adg:lint`. Rely on machine-maintained manifests. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate quality scripts and audit project health:

```powershell
# 1. Run built-in self-tests
python .agent/skills/lint-and-validate/scripts/lint_runner.py --test
python .agent/skills/lint-and-validate/scripts/type_coverage.py --test

# 2. Run multi-ecosystem lint runner
python .agent/skills/lint-and-validate/scripts/lint_runner.py .

# 3. Audit type coverage
python .agent/skills/lint-and-validate/scripts/type_coverage.py src
```