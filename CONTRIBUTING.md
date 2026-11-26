> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / CONTRIBUTING
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[CONTRIBUTING]`)

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
   - [ ] `cargo check` and `cargo clippy --all-targets -- -D warnings` pass in `server-rs/`
   - [ ] `cargo test` passes in `server-rs/`
   - [ ] `npm run lint`, `npm run build`, `npm run test`, `npm run version:check`, and `npm run changelog:check` pass
   - [ ] **Plane 1 Fast-Gate**: Run `npm run check:plane1:fast` (TypeScript, ADG v2 manifest, invariants, header parity guard) and `npm run lint:clean:diff`
   - [ ] **Verification Parity**: Run `python execution/verify_all.py` and `python execution/parity_guard.py` to ensure full alignment
   - [ ] **Performance Audit**: Run relevant benchmarks on the **Performance Analysis** dashboard page (`/benchmarks`) and ensure no regressions (verified via Delta Analysis)
   - [ ] **Success Audit**: Run at least one mission with **Mission Analysis** enabled to verify that your changes produce optimal, high-quality agent outputs.
   - [ ] **Documentation Alignment**: Updated relevant docs (`GLOSSARY.md`, `ARCHITECTURE.md`, `API_REFERENCE.md`, `OPERATIONS_MANUAL.md`).
   - [ ] **Code Tagging**: Ensure all critical logic is tagged with `@docs` (e.g., `//! @docs ARCHITECTURE:...`) for automated ADG v2 documentation linking.
   - [ ] **Capability Registration**: If adding new tools, ensure they are compatible with the **Import Engine** and are correctly categorized (User/AI).

4. **Code Review**: All PRs require at least one review. Reviewers should check:
   - Correctness and edge cases
   - Security implications (especially for filesystem/tool changes)
   - Performance impact (especially for hot-path code in `server-rs/src/agent/runner/`)
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

1. Bump version atomically across `version.json` and all 23 synced targets via `npm run version:bump` and verify with `npm run version:check` + `npm run changelog:check` (see [`docs/RELEASE_PROCESS.md`](docs/RELEASE_PROCESS.md)).
2. Run full verification (`npm run check:plane1`, `cargo test`, `npm run test`, `npm run build`).
3. Tag the release: `git tag -a v1.1.x -m "Release description"`.
4. Deploy via cross-platform bunker scripts (`scripts/deploy-bunker-{1,2}.ps1` on Windows or `scripts/deploy-bunker-{1,2}.sh` on Linux/macOS) or `scripts/deploy-linuxlite.ps1`.

---

Thank you for strengthening the swarm.