> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / Security_Model
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🛡️ Security Model: Sovereign Protection

> **Intelligence Level**: High (Sovereign Context)  
> **Status**: Engine security controls are implemented in selected areas; Plane 2 execution and company workspace isolation are pending
> **Version**: 1.2.2  
> **Last Reviewed**: 2026-10-30
> **Fault Ledger**: [`directives/FAULT_REGISTRY.md`](../directives/FAULT_REGISTRY.md). Open items are tracked in `docs/SECURITY_REMEDIATION_PLAN.md`.  
> **Audit Context**: `reports/intelligence/audit_context.json`  

Tadpole OS implements selected Zero-Trust controls for local and distributed agent clusters. The dual-plane and company-isolation controls below include target requirements; the current SMB digital-twin specification marks their implementation pending. Do not treat this document as evidence that `FR-11` or `NFR-1` has been enforced end to end.

---


## Residual risk (2026-10-04)

The command guard is a string filter, not an operating-system sandbox. Interpreter commands (`python`, `node`, `cargo`) and shell hook launchers are denied unless the operator sets `TADPOLE_TRUST_INTERPRETERS=1` or `TADPOLE_TRUST_HOOK_SHELLS=1`. Those flags re-enable host execution as the engine user. A separate OS user, network namespace, and binary allowlist remain follow-up work.

WebSocket upgrades require a `bearer.<token>` subprotocol. A pulse-only handshake is rejected.

## 🏗️ Technical Hierarchy: Policy → Mechanism → Enforcement

### Permission module boundaries (2026-10-22)

`security/permissions.rs` remains the policy facade and cache owner. Its `types`, `evaluation`, `repository`, and `prompt` modules separate decision types, lookup order, SQL facts, and asynchronous human decisions. Exact capability resources are checked before domain fallbacks, with agent, role, then global precedence at each level. Missing facts continue evaluation; malformed rows and database errors deny access.

Cache refresh does not warm decisions below a mandatory capability floor. Such decisions are evaluated live so signed override expiration is checked again. Cancellation of a pending approval removes both its queue entry and resolver.

`runner/tools/resolved_policy.rs` shares policy metadata and token target resolution with MCP. The runner uses granular capability evaluation; MCP's final guard retains its legacy policy lookup. This refactor does not change MCP approval semantics. WebSocket origin checks and redaction live in `routes/ws/shared.rs`, with separate authenticated telemetry and voice handlers in `telemetry.rs` and `live.rs`.

The security layer is organized into three distinct stages to ensure that every agent action is safe, verifiable, and attributable.

### 1. Security Policy (The Rules)
Policies define the boundaries of what is "Safe" vs. "Dangerous".
- **Default-Deny**: Any tool requiring destructive action (e.g., `write_file`, `shell_execute`) is denied by default unless specifically Whitelisted or Gated.
- **Dual-Plane Permission Firewall (`FR-11`, `IDENTITY.md` Directive #1)**: The generic runner rejects an agent before workflow, context, or tool execution only when its stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive). Other department labels do not activate this guard, so company-facing identities must use this exact marker and remain disabled until the company-scoped runtime exists. This is a fail-closed gate, not a Plane 2 runtime; admitted engine-agent permissions remain role-based. Plane 1 agents must keep customer PII and company records out of source control and engine-level memory.
- **Workspace Database Isolation (`NFR-1`)**: Separate practice/tenant stores from the engine database as required by [`mock-smb-digital-twin.md`](specs/mock-smb-digital-twin.md). Company-keyed run binding and isolated business storage are pending; existing workspace files do not establish end-to-end isolation.
- **Hierarchical Role-Based Access Control (RBAC)**: Enforces agent-specific and role-based policies resolved hierarchically: `Agent-Specific Policy` -> `Role-Based Policy` -> `Global Tool Policy` -> `Prompt (Sovereign Safety)`. Defined in SQLite tables `agent_permission_policies` and `role_permission_policies`.
- **Privacy Shield (Global & Per-Cluster Model Routing)**: Runner calls use local model endpoints while the effective global or cluster privacy policy is enabled, and configured cloud model slots are excluded. This does not block arbitrary network egress or establish an air gap. The global privacy monitor makes periodic reachability probes; enforce a disconnected boundary at the host/network layer.
- **Budget Compliance**: Provider calls are denied when their estimated cost cannot be reserved against the applicable agent and mission quotas. This does not terminate every mission or pause the entire engine.

### 💰 Financial Governance: The Budget Guard
Tadpole OS implements a **Debounced Persistence** pattern for budget tracking:
- **In-Memory Buffering**: Token usage is captured in thread-safe `DashMap` storage.
- **Aggregated Enforcement**: The kernel monitors both on-disk totals and in-memory buffs for instant enforcement.
- **Background Sync**: A dedicated task in `startup.rs` flushes buffered usage to SQLite every 10 seconds to minimize database contention.
- **Budget exhaustion behavior**: `BudgetGuard` reserves estimated provider-call cost against agent and mission quotas. When a reservation is denied, the runner reports `BUDGET_EXHAUSTED` (HTTP 402) and does not dispatch that call. A system-wide `STASIS` state, swarm suspension, budget snapshot, `BUDGET_BREACH` alert, and `RESUME`/`TERMINATE` lockout are policy requirements only; the runtime does not implement them.

> Consult `docs/ERROR_REGISTRY.json` → `BUDGET_EXHAUSTED` for the implemented quota error. Follow the operator procedures in `directives/IDENTITY.md` separately; they are not enforced as a global runtime state.

### 2. Guarding Mechanisms (The Tools)
The specific cryptographic and logical controls used to protect the system.
- **Capability-Based Security (CBS)**: Non-forgeable `CapabilityToken` system that replaces ambient authority with explicit permission grants.
- **Write-Ahead Logging (WAL)**: A mandatory persistence layer in `server-rs/src/agent/runner/tools/mod.rs` that records tool intent before execution begins.
- **Cryptographic Oversight Gate**: A `oneshot` channel-based suspension system for human approval of sensitive operations, requiring an Ed25519 digital signature generated by the client and validated by the backend.
- **Distributed Tracing Trace propagation**: Request-lifespan trace IDs (`X-Request-Id`) are propagated across asynchronous boundary steps for system-wide auditing and request correlation.
- **Neural Shield**: A regex-driven redaction engine that blocks API keys and secrets in outbound logs.

### 3. Real-time Enforcement (The Gatekeepers)
The active runtime modules that block unauthorized actions.
- **Safe Command Lexer**: A whitelist-based shell validator that prevents command substitution and injection.
- **Tool Context**: The generic `ToolContext` carries the shared `AppState` and is currently for the engine runner. It is not isolated or least-privilege. Plane 2 identities are rejected before receiving it; this context must not be presented as a Plane 2 boundary.
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

### 🚦 Cryptographic Oversight Gate (`oneshot` suspension)
Bridges autonomous reasoning with human responsibility through cryptographic signed consent.
- **Safe Command Lexer**: All shell commands are scanned against a whitelist of authorized binaries (e.g., `cargo`, `npm`, `git`).
- **Substitution Blocking**: Proactively blocks `$()`, ` `` `, and `${}` injection patterns.
- **Operator Signature Verification**: Operator approval decisions must be digitally signed client-side with an Ed25519 keypair. The REST and WebSocket endpoints verify the signature (`signature`, `verifying_key`) of `entry_id` + `decision` + `timestamp` prior to resolving the tokio `oneshot` channel.
- **Protocol**: The runner task is "parked" on a `tokio::sync::oneshot` channel until the operator issues a verified, cryptographically signed approval.

## 🛡️ Agent-to-Agent (A2A) Financial Containment
To enable a decentralized, localized economy where agents can pay other agents for specialized services, Tadpole OS enforces strict transaction safety:
- **Two-Phase Commit (2PC) Rollback Guarantees**: A2A transactions follow a 2PC protocol (`Prepare` $\rightarrow$ `Commit` / `Rollback`). Funds are first reserved/locked (`Prepare`). If the sub-mission fails, terminates, or if the system experiences a teardown (via a SIGINT or engine shutdown), the engine executes an explicit `.rollback()` method to refund all pending locked balances.
- **Isolated Ledger Database**: Financial records are stored in a dedicated `transaction_ledger` table with transaction-level SQL isolation, separate from standard agent configuration and memory tables, ensuring non-repudiability and prevention of double-spending or unauthorized balance drift.
- **Cross-Swarm Cryptographic Handshakes**: Incoming remote A2A network envelopes are validated against their payload hash using HMAC-SHA256 signatures generated from the active `KEYRING`. This ensures remote swarms cannot issue malicious instructions to local agents without a pre-shared cryptographic key.

## 🛡️ Information Isolation: Context Sandboxing
To protect sensitive system parameters and operational details from leaking across agent boundaries:
- **Context Stripping**: The parent agent's internal reasoning traces, monologue, raw prompt construction, and system-level tool call failures are stripped from child agent context.
- **Observation Injection**: Only explicit user/parent directives and sanitized findings are injected as observations, providing a clean operational sandbox.
- **Prompt Injection Defense**: By sanitizing and isolating child contexts, malicious code or prompt injection payloads in parent workspaces are prevented from automatically propagating downstream.

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
