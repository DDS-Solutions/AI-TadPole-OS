> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[CHANGELOG]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.351] - 2026-07-22

### Added
- Action Ledger view mode toggle (`HITL Approvals` vs `Auto-Approved`) adhering to `design.md` & `DESIGN_SYNERGY.md` specs.
- Hybrid 7-Day Rolling Telemetry Log Retention and `.gitignore` Git Hygiene policy for `data/logs/`.
- Native Rust telemetry sink auto-prune logic in `server-rs/src/telemetry/sink.rs`.
- Layer 3 fast-feedback tools (`fast_hitl_gate.py`, `cargo_fast_check.py`, and `clean_telemetry_logs.py`).

### Fixed
- Fixed missing `selected_cluster_id` memoization and filtering bug in `Action_Ledger.tsx`.
- Optimized `Params_Cell` lazy JSON formatting in Action Ledger table cells.

## [1.1.351] - 2026-05-24

### Added
- Shared Idempotent Tool Caching (A) for local read-only files and symbols tools.
- Concurrent Conflict Locks (B) with automated 30-second lease TTL.
- Incremental AST Caching (C) in `CodeSymbolGraph` only parsing modified/new files.
- Monologue Turn Preservation & Compaction (D) keeping last 4 turns raw, truncating old code blocks, and providing deterministic fallback.

### Fixed
- Rebuild bottlenecks in AST graph query CLI and API routes.
- Monologue context overflow crashes from massive tool output payloads.

## [1.1.351] - 2026-04-16

### Added
- Centralized versioning system via `version.json` and `sync_version.py`.
- Git pre-commit hook for automated version synchronization.
- Docker `HEALTHCHECK` monitoring.
- Unit tests for `request_id.rs` middleware.

### Changed
- Refactored agent identity management to use centralized constants (`AGENT_CEO`, `AGENT_COO`, `AGENT_ALPHA`).
- Hardened backend security by removing `unwrap()` calls in hot paths.
- Optimized `SecretRedactor` to reduce redundant regex evaluations.
- Fixed no-op trace propagation tests in `lifecycle.rs`.

## [1.1.351] - 2026-04-16

### Added
- Standardized RFC 9457 Problem Details for all backend errors.
- End-to-end W3C TraceContext propagation.

### Fixed
- Swarm recruitment priority deadlocks in CI.
- Frontend store desynchronization in Sovereign Chat.

[//]: # (Metadata: [CHANGELOG])
