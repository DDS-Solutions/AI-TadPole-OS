> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / CHANGELOG
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[CHANGELOG]`)

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.464] - 2026-10-27

### Changed
- Consolidated runner identity, mission, and environment state; separated context resolution, preparation, and memory enrichment.
- Centralized durable agent outcomes, provider/model commands, agent ingestion, and shared tool policy metadata.
- Decomposed chat transcript/decisions/replication, telemetry and voice sessions, artifact loading/persistence, and permission evaluation/approval lifecycle.
- Shared macOS local/CI packaging and added required architecture, signature and disk-image checks; explicitly configured ad-hoc signing.

### Fixed
- Rechecked mandatory permission floors and signed override expiration after cache refresh.
- Applied explicit null agent updates, preserved snapshot ordering metadata, and kept parallel workflow siblings within their shared execution epoch.
- Preserved saved provider credentials on blank input and recovered failed provider/model/question writes without replacing unrelated state.
- Isolated route and budget fixtures from operator settings and host memory pressure; added explicit high-pressure budget coverage.
- Staged both target-specific macOS sidecars for Tauri compilation as well as the universal bundle sidecar; rejected invalid desktop data paths and directory failures before engine launch.

### Security
- Restricted Gitleaks allowlists to test fixtures and stopped suppressing private-key and `sk-` patterns.
- Denied `python`, `node`, and `cargo` agent commands unless `TADPOLE_TRUST_INTERPRETERS=1` is set.
- Denied shell hook interpreters unless `TADPOLE_TRUST_HOOK_SHELLS=1` is set, and removed `ExecutionPolicy Bypass`.
- Pinned bunker deploy scripts under `scripts/` and required the admin token when configured.
- Required the WebSocket `bearer.<token>` subprotocol before the upgrade completes.
- Removed committed engine and `protoc` binaries from the tree. History rewrite is a follow-up.

### Performance
- Accelerated Vitest testing loops via `--no-isolate` test scripts and conditional worker reuse on Windows.
- Integrated `sccache` compiler caching for `server-rs` Rust builds and fast cargo checks.


## [1.1.463] - 2026-09-14

### Security
- Remediated high & moderate severity Vite vulnerabilities (GHSA-fx2h-pf6j-xcff, GHSA-4w7w-66w2-5vf9, GHSA-v6wh-96g9-6wx3) by scoping VitePress to Vite 6.4.3 (`npm audit` 0 vulnerabilities).

### Changed
- Synchronized React ecosystem to React 19.3.0 (`react`, `react-dom`, `@types/react`, `@types/react-dom`).
- Updated dependencies: `framer-motion` (13.2.0), `lucide-react` (1.45.0), `@types/node` (26.5.1), `tsx` (4.23.13), `syn` (3.0.5), `uuid` (1.26.1).
- Grouped Dependabot ecosystems in `.github/dependabot.yml` to ensure unified pull requests.

### Changed
- Consolidated product release metadata under `version.json`, with strict drift checks and transactional updates.
- Separated OpenAPI document versioning from the product release version and added a monotonic Android build counter.
- Hardened CI, release, and public-mirroring rules around immutable SemVer tags and release assets.
- Scoped frontend style generation and linting to application sources instead of backend build artifacts.

### Fixed
- Persisted agent deletion consistently across UI entry points, preserving local assignments when deletion fails.
- Restored registry invalidation refreshes and isolated cancellation and failure handling for shared requests.
- Bounded reconnect attempts across failed WebSocket authentication handshakes.
- Made document chunking Unicode-safe and bounded for zero limits, and corrected Markdown reference line ranges while removing repeated prefix scans.
- Made duplicate task request claims atomic and corrected release ordering for hyphenated and large numeric prereleases.
- Aligned Node.js prerequisites with the locked toolchain, repaired JavaScript test-witness discovery, and included attributed and nested Rust handlers in generated API references.
- Removed the global Vite override so VitePress and its Vue plugin use their compatible Vite dependency while the application retains Vite 8.

## [1.1.281] - 2026-07-22

### Added
- Action Ledger view mode toggle (`HITL Approvals` vs `Auto-Approved`) adhering to `design.md` & `DESIGN_SYNERGY.md` specs.
- Hybrid 7-Day Rolling Telemetry Log Retention and `.gitignore` Git Hygiene policy for `data/logs/`.
- Native Rust telemetry sink auto-prune logic in `server-rs/src/telemetry/sink.rs`.
- Layer 3 fast-feedback tools (`fast_hitl_gate.py`, `cargo_fast_check.py`, and `clean_telemetry_logs.py`).

### Fixed
- Fixed missing `selected_cluster_id` memoization and filtering bug in `Action_Ledger.tsx`.
- Optimized `Params_Cell` lazy JSON formatting in Action Ledger table cells.

## [1.1.96] - 2026-05-24

### Added
- Shared Idempotent Tool Caching (A) for local read-only files and symbols tools.
- Concurrent Conflict Locks (B) with automated 30-second lease TTL.
- Incremental AST Caching (C) in `CodeSymbolGraph` only parsing modified/new files.
- Monologue Turn Preservation & Compaction (D) keeping last 4 turns raw, truncating old code blocks, and providing deterministic fallback.

### Fixed
- Rebuild bottlenecks in AST graph query CLI and API routes.
- Monologue context overflow crashes from massive tool output payloads.

## [1.1.6] - 2026-04-16

### Added
- Centralized versioning system via `version.json` and `sync_version.py`.
- Git pre-commit hook for automated version synchronization.
- Docker `HEALTHCHECK` monitoring.
- Unit tests for `request_id.rs` middleware.
- Standardized RFC 9457 Problem Details for all backend errors.
- End-to-end W3C TraceContext propagation.

### Changed
- Refactored agent identity management to use centralized constants (`AGENT_CEO`, `AGENT_COO`, `AGENT_ALPHA`).
- Hardened backend security by removing `unwrap()` calls in hot paths.
- Optimized `SecretRedactor` to reduce redundant regex evaluations.
- Fixed no-op trace propagation tests in `lifecycle.rs`.

### Fixed
- Swarm recruitment priority deadlocks in CI.
- Frontend store desynchronization in Sovereign Chat.
