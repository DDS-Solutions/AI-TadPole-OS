> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[api_documentation]` in audit logs.
>
> ### AI Assist Note
> 📖 Directive: API Documentation (SOP-DEV-04)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 📖 Directive: API Documentation (SOP-DEV-04)

## 🎯 Primary Objective
Maintain a high-fidelity, "Level 3 Sovereign" API reference. This directive ensures zero drift between reaching the Rust backend implementation and the `API_REFERENCE.md` documentation.

---

## 🏗️ Documentation Standards

### 1. RFC 9457 Compliance (Problem Details)
- **Check**: Every error response must include `type`, `title`, `status`, and `detail`.
- **Validation**: Test against the `ProblemDetails` utility in `server-rs/src/routes/error.rs`.

### 2. HATEOAS Envelopes
- **Check**: Resource endpoints (Agents, Missions) must include a `_links` object for discoverability.
- **Validation**: Ensure `v1/agents` returns links to detail and task routes.

### 3. Swagger/OpenAPI Sync
- **Tool**: `docs/openapi.yaml`.
- **Action**: Every new route in `router.rs` must be mirrored in the OpenAPI spec.

---

## 🛠️ Audit SOP

### 1. Drift Check
- **Execution**: Run `python execution/parity_guard.py .`.
- **Action**: Fix any "ADG-DRIFT" warnings related to the API surface.

### 2. Implementation Trace
- **Check**: Verify that `docs/API_REFERENCE.md` correctly documents the authentication handshake for WebSockets (Bearer token in `Sec-WebSocket-Protocol`).

---

## 🚦 Release Condition
Major API changes require a "Documentation Freeze" phase where the `API_REFERENCE.md` is updated and reviewed BEFORE the PR is merged to the Sovereign branch.
[//]: # (Metadata: [api_documentation])
