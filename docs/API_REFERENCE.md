> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is the high-fidelity hub for the Tadpole OS API ecosystem.
> - **@docs ARCHITECTURE:ApiHub**
> - **Telemetry Link**: Verified via `execution/parity_guard.py`.

# 🛰️ API Reference: Tadpole OS Engine

**Intelligence Level**: High-Fidelity (ECC Optimized)  
**Version**: 1.1.13  
**Last Hardened**: 2026-04-17 (Alignment Patch)  
**Classification**: Sovereign  

---

## 🎯 Executive Summary

The Tadpole OS Engine provides a comprehensive network interface for sovereign agent orchestration. To maintain a clean **API Contract**, the documentation is decomposed into specialized modules based on the transport layer and operational domain.

---

## 🔌 API Documentation Suite

| Module | Description | Transport |
|:--- |:--- |:--- |
| **[📄 API Contract](./API_CONTRACT.md)** | REST endpoints, request/response bodies, and standards. | HTTP (REST) |
| **[📡 WebSocket Events](./WEBSOCKET_EVENTS.md)** | Real-time events, binary pulses, and telemetry. | WebSockets |
| **[🛠️ CLI Tools](./CLI_TOOLS.md)** | Mission debriefing, parity audits, and terminal scripts. | Python (CLI) |

---

## 🏗️ API Hardening & Standards

The Tadpole engine is engineered for enterprise-grade reliability:
- **RFC 9457**: 100% compliance with Problem Details for machine-readable error handling.
- **HATEOAS**: Level 3 REST maturity via navigable `_links` in response envelopes.
- **Binary Telemetry**: Sub-millisecond "Swarm Pulse" for high-frequency visualization.
- **Neural_Vault**: Client-side AES-256 encryption using the SubtleCrypto API.

---

## 📐 Versioning Strategy

Tadpole OS utilizes a structured versioning path to ensure compatibility:
- **`/v1/`**: The current stable business logic layer (e.g., `/v1/agents`).
- **Root Paths**: Paths like `/agents` are legacy aliases and are scheduled for deprecation in v1.2.0.
- **Engine Operations**: Internal control routes are consolidated under `/v1/engine/`.

---

## 🧪 Integration Notes

- **Authentication**: All protected endpoints require a `Bearer <NEURAL_TOKEN>`.
- **Latency**: Use the binary WebSocket pulse for visualization; use REST for state mutations.
- **Tracing**: Always propagate the `X-Request-Id` header for end-to-end observability.
