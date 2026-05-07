> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[CONTRIBUTING]` in audit logs.
>
> ### AI Assist Note
> 🐸 Tadpole OS Contribution Standards
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🐸 Tadpole OS Contribution Standards

Welcome, Overlord. To maintain the sovereignty and performance of AI-Tadpole-OS, please follow these guidelines when contributing.

## The 3-Layer Logic
All code contributions must respect our architecture:
1. **Directive (YAML/MD)**: Standardized instructions.
2. **Orchestration (Rust/TS)**: Intelligent routing and state management.
3. **Execution (Sub-processes/Tools)**: Deterministic execution.

---

## Getting Started

1. **Fork the Repo**: Create your own tactical instance.
2. **Clone & Install**: `npm install` in the root, then ensure `cargo check` passes in `server-rs/`.
3. **Environment**: Copy `.env.example` to `.env` and fill in required keys.
4. **Run Locally**: `npm run engine` (backend on `:8000`) + `npm run dev` (frontend on `:5173`).

---

## Pull Request Process

1. **Feature Branches**: Use descriptive names following the pattern:
   - `feat/<area>/<description>` — New features (e.g., `feat/runner/parallel-budgets`)
   - `fix/<area>/<description>` — Bug fixes (e.g., `fix/chat/scroll-position`)
   - `refactor/<area>/<description>` — Code improvements (e.g., `refactor/runner/split-tools`)
   - `docs/<description>` — Documentation only (e.g., `docs/api-versioning`)

2. **Commit Messages**: Use [Conventional Commits](https://www.conventionalcommits.org/):
   ```
   feat(runner): add parallel budget enforcement
   fix(chat): prevent scroll reset on new message
   docs: update API_REFERENCE with pagination
   refactor(state): extract rate-limit middleware
   ```

3. **Pre-Submit Checklist**:
   - [ ] `cargo check` passes in `server-rs/`
   - [ ] `cargo test` passes in `server-rs/`
   - [ ] `npm run lint` passes
   - [ ] `npm run build` passes
   - [ ] `npm run test` passes
   - [ ] **Verification Parity**: Run `python scripts/verify_all.py` and ensure 100% parity pass.
   - [ ] **Parity Guard**: Run `python scripts/parity_guard.py` to ensure security protocols are correctly reflected.
   - [ ] **Performance Audit**: Run relevant benchmarks on the [Performance Analysis](Dashboard) page and ensure no regressions (verified via Delta Analysis)
   - [ ] **Success Audit**: Run at least one mission with **Mission Analysis** enabled to verify that your changes produce optimal, high-quality agent outputs.
   - [ ] **Documentation Alignment**: Updated relevant docs (`GLOSSARY.md`, `ARCHITECTURE.md`, `API_REFERENCE.md`, `OPERATIONS_MANUAL.md`).
   - [ ] **Code Tagging**: Ensure all critical logic is tagged with `@docs-ref` for automated documentation linking.
   - [ ] **Capability Registration**: If adding new tools, ensure they are compatible with the **Import Engine** and are correctly categorized (User/AI).

4. **Code Review**: All PRs require at least one review. Reviewers should check:
   - Correctness and edge cases
   - Security implications (especially for filesystem/tool changes)
   - Performance impact (especially for hot-path code in `runner.rs`)
   - Documentation completeness

---

## Coding Standards

### Rust (`server-rs/`)
- Run `cargo clippy` and address all warnings
- Follow idiomatic async patterns with `tokio`
- Use `anyhow::Result` for fallible functions, `thiserror` for domain errors
- All tool handlers must return `anyhow::Result<String>`
- **REST Compliance**: All New API endpoints must follow **RFC 9457 (Problem Details)** for error responses and include HATEOAS navigability.
- Never create a `reqwest::Client` directly — use `state.http_client`
- Never hardcode workspace paths — use `ctx.workspace_root`

### TypeScript / React (`src/`)
- Use functional components with TypeScript strict mode
- State management via Zustand stores (never prop-drilling beyond 2 levels)
- Tailwind CSS variables for theme synergy — no inline color values
- All interactive elements must have unique IDs for E2E testing

### Security
- **Never** hardcode API keys or tokens — use `.env`
- All new tool calls that modify state must go through the Oversight Gate
- Filesystem operations must use `FilesystemAdapter` — never raw `fs` calls

---

## Issue Templates

When filing issues, include:
- **Type**: Bug / Feature / Enhancement / Docs
- **Severity**: Critical / High / Medium / Low
- **Steps to Reproduce** (for bugs): Include terminal output or screenshots
- **Expected vs Actual Behavior**
- **Environment**: OS, Rust version, Node.js version, browser

---

## Release Process

1. Update version in `package.json` and `Cargo.toml`
2. Run full test suite (`cargo test` + `npm run test`)
3. Build production bundle (`npm run build`)
4. Tag the release: `git tag -a v0.x.x -m "Release description"`
5. Deploy via `./deploy.ps1` to the Swarm Bunker

---

Thank you for strengthening the swarm.

[//]: # (Metadata: [CONTRIBUTING])
