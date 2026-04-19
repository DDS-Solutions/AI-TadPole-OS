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

> **@docs ARCHITECTURE:SecurityModel**

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
- **Merkle Audit Trail**: A SHA-256 linked chain of custody for every tool call.
- **Oversight Gate**: A `oneshot` channel-based suspension system for human approval.
- **Neural Shield**: A regex-driven redaction engine that blocks API keys and secrets in outbound logs.

### 3. Real-time Enforcement (The Gatekeepers)
The active runtime modules that block unauthorized actions.
- **Sandbox Isolation**: The `filesystem_adapter` enforces strict path-rooting.
- **Token Bucket Limiter**: Prevents engine-level denial-of-service via rate-limit enforcement.

---

## 🔒 Mandatory Safety Guidelines

### ☣️ Filesystem Canonicalization (SEC-03)
The absolute primary security control for file-system safety.
- **Principle**: The engine NEVER trusts a relative path or a user-provided string directly.
- **Mechanism**: Every path is passed through `std::fs::canonicalize` before any operation occurs.
- **Enforcement**: This is enforced in the `filesystem.rs` adapter. Attempts to "break out" of a workspace via symlinks or `../../` are mathematically impossible because the final canonicalized path MUST start with the `workspace_root`.

### ⛓️ Merkle Audit Trail (`audit.rs`)
Ensures that the engine's history cannot be retrospectively tampered with.
- **Hash Chaining**: Each tool execution is linked to the hash of the preceding event.
- **Ed25519 Signing**: All critical decisions are cryptographically signed by the engine instance's private key.
- **Integrity Score**: Verified in real-time by the dashboard via `verify_chain()`.

### 🚦 Oversight Gate (`oneshot` suspension)
Bridges autonomous reasoning with human responsibility.
- **Mode 1 (Whitelisted)**: Safe tools (e.g., `ls`, `read_file`) proceed without delay.
- **Mode 2 (Gated)**: Dangerous tools (e.g., `bash`, `write_file`) trigger a synchronous pause.
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
