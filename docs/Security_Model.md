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

# 🛡️ Security Model: Sovereign Protection

> **Intelligence Level**: High (Sovereign Context)  
> **Status**: Verified Production-Ready  
> **Version**: 1.2.0  
> **Last Hardened**: 2026-05-01 (Zero-Trust Tooling & CBS)  

Tadpole OS implements a Zero-Trust security model designed to provide "Sovereign Protection" for both local and distributed agent clusters. Architecture follows the principle of **Hardened Containment**.

---

## 🏗️ Technical Hierarchy: Policy → Mechanism → Enforcement

The security layer is organized into three distinct stages to ensure that every agent action is safe, verifiable, and attributable.

### 1. Security Policy (The Rules)
Policies define the boundaries of what is "Safe" vs. "Dangerous".
- **Default-Deny**: Any tool requiring destructive action (e.g., `write_file`, `shell_execute`) is denied by default unless specifically Whitelisted or Gated.
- **Privacy Shield**: Prevents un-authorized outbound data leakage to cloud providers when active.
- **Budget Compliance**: Missions are terminated immediately if they exceed the pre-authorized financial allocation.

### 💰 Financial Governance: The Budget Guard
Tadpole OS implements a **Debounced Persistence** pattern for budget tracking:
- **In-Memory Buffering**: Token usage is captured in thread-safe `DashMap` storage.
- **Aggregated Enforcement**: The kernel monitors both on-disk totals and in-memory buffs for instant enforcement.
- **Background Sync**: A dedicated task in `startup.rs` flushes buffered usage to SQLite every 10 seconds to minimize database contention.

### 2. Guarding Mechanisms (The Tools)
The specific cryptographic and logical controls used to protect the system.
- **Capability-Based Security (CBS)**: Non-forgeable `CapabilityToken` system that replaces ambient authority with explicit permission grants.
- **Write-Ahead Logging (WAL)**: A mandatory persistence layer in `tools/mod.rs` that records tool intent before execution begins.
- **Oversight Gate**: A `oneshot` channel-based suspension system for human approval of sensitive operations.
- **Neural Shield**: A regex-driven redaction engine that blocks API keys and secrets in outbound logs.

### 3. Real-time Enforcement (The Gatekeepers)
The active runtime modules that block unauthorized actions.
- **Safe Command Lexer**: A whitelist-based shell validator that prevents command substitution and injection.
- **Isolated Tool Context**: A constrained `ToolContext` struct that prevents tools from accessing global application state.
- **Sandbox Isolation**: The `filesystem_adapter` enforces strict path-rooting via `SafePath` wrappers.
- **Token Bucket Limiter**: Prevents engine-level denial-of-service via rate-limit enforcement.

---

## 🔒 Mandatory Safety Guidelines

### 🛡️ Capability-Based Security (SEC-04)
The engine utilizes the **CBS** model to ensure that no tool has implicit access to the system.
- **Minting**: Unique `CapabilityTokens` are generated for each mission, containing specific `Permission` grants (e.g., `FileRead`, `ShellExecute`).
- **Verification**: The `ZeroTrustGuard` verifies tokens at the point of execution. Expired or unauthorized tokens result in immediate termination.

### 📝 Write-Ahead Logging (WAL)
Every agent action is recorded *before* it occurs to ensure a non-repudiable audit trail.
- **Intent Logging**: The `[INTENT]` record is committed to the database prior to tool invocation.
- **Outcome Logging**: The `[SUCCESS]` or `[FAILURE]` result is appended upon completion.
- **Tamper Resistance**: Links directly to the Merkle Audit Trail for cryptographic verification.

### 🚦 Oversight Gate (`oneshot` suspension)
Bridges autonomous reasoning with human responsibility.
- **Safe Command Lexer**: All shell commands are scanned against a whitelist of authorized binaries (e.g., `cargo`, `npm`, `git`).
- **Substitution Blocking**: Proactively blocks `$()`, ` `` `, and `${}` injection patterns.
- **Protocol**: The runner task is "parked" on a `tokio::sync::oneshot` channel until the operator clicks **Approve** or **Reject** on the dashboard.

---

## 🛡️ Network & Encryption

### Neural Vault
Client-side encryption for API keys built on the **W3C SubtleCrypto API**. 
- **Encryption**: AES-256-GCM.
- **Decryption**: Isolated in a dedicated **Web Worker** thread to prevent main-thread credential exposure.
- **Persistance**: Purely volatile memory or encrypted `sessionStorage`.

### TLS Strategy
- **LAN Traffic**: Encrypted via WireGuard (Tailscale).
- **Public Traffic**: Enforces HTTPS/TLS-termination via reverse proxy (Caddy/Nginx).

[//]: # (Metadata: [Security_Model])
