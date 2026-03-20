# 🛡️ Tadpole OS Security Policy & Architecture

<!-- @docs-ref SECURITY:ScanningPolicies (scanner.rs) -->

> [!IMPORTANT]
> **Status**: In-Progress / Parity Guard Enabled
> **Version**: 1.1.3
> **Last Updated**: 2026-03-20 (Verified 100% Quality Pass)
> **Documentation Level**: Sovereign Standard (Antigravity Std)

## Overview
Tadpole OS implements a multi-layered, zero-trust security architecture designed to prevent autonomous agent rogue behavior, secret leakage, and unauthorized financial expenditure.

---

## 1. Governance & Oversight

### 1.1 The Oversight Gate
Tadpole OS utilizes a "Human-in-the-loop" governance model for high-risk operations. The system automatically intercepts and blocks tool execution for:
- **File System Modification**: Deleting or overwriting non-workspace files.
- **Budgetary Impact**: High-token-usage calls or manual budget adjustments.
- **Subprocess Spawning**: Executing shell scripts or external binaries.
- **Mission Completion**: Finalized delivery of mission-critical reports.
- **External Web Access**: All `fetch_url` calls trigger a mandatory Oversight Gate to prevent data exfiltration.

### 1.2 Capability-Based Security (CBS)
Capabilities (Skills) are defined via structured JSON manifests. 
- **Permissions-First**: Every skill must explicitly declare required permissions (e.g., `shell:execute`).
- **Standardization**: All tool calls are routed through the kernel's `McpHost`, ensuring consistent policy enforcement regardless of tool source.

---

## 2. Proactive Defenses

### 2.1 Shell Safety Scanner (`scanner.rs`)
The engine includes a proactive regex-based scanner that inspects agent-generated code (Python/Bash) before execution.
- **Redaction Protocol**: Prevents code from accessing environment variables like `$NEURAL_TOKEN`, `os.environ`, or specific API keys.
- **Enforcement Modes**:
    - **AUDIT**: Logs detections without blocking (Informational).
    - **ENFORCE**: Terminates the tool call and notifies the agent of the safety violation (Block). Defaults to `ENFORCE` in production.
- **Compliance Link**: Scanner findings are cross-referenced by the **Parity Guard** to ensure documentation reflects current safety protocols.

### 2.2 Budget Guard & Persistent Metering
Financial security is enforced at the kernel level using SQLite persistence.
- **Persistent Quotas**: Budgets survive server restarts and are enforced across multiple sessions.
- **Downtime protection**: If the `BudgetGuard` cannot verify remaining quota (e.g., DB lock), it defaults to a **Fail-Closed** state, blocking execution.
- **Self-Healing Throttle**: The `AgentHealth` module monitors for "fail-looping" agents. If an agent wastes budget on consecutive errors, it is automatically throttled or suspended.

---

## 3. Cryptographic Accountability

### 3.1 Merkle Audit Trail (`audit.rs`)
Tadpole OS records all critical actions (ToolCalls, Decisions, PolicyChanges) in a tamper-evident cryptographic ledger.
- **Hash Chaining**: Each record is SHA-256 linked to the previous entry, creating a linear chain of custody.
- **ED25519 Signatures**: Every audit entry is cryptographically signed using the ED25519 algorithm. This provides **Non-repudiation**, ensuring that an agent's actions can be definitively traced back to a specific session and verified by the human operator.
- **Granular Verification**: The system supports both full-chain verification (`verify_chain()`) and per-record validation (`verify_record()`). Individual audit logs can be verified independently to detect granular tampering.
- **Identity Context (IBM Research Alignment)**: Every audit entry now records the **User ID** and **Mission Instance ID**. These fields are included in the SHA-256 hash chaining, making the entire identity context (Human -> App -> Agent -> Data) tamper-evident.
- **Integrity Verification**: The system provides a `/v1/oversight/security/integrity` endpoint to cryptographically prove that the log has not been manually altered.
- **Production Requirement**: A valid hex-encoded Ed25519 key must be provided via `AUDIT_PRIVATE_KEY` for production deployments. If missing, the Merkle Hub will fail-closed to prevent unsigned (and therefore repudiable) actions.

---

## 4. Secret Management (Neural Vault)

### 4.1 Client-Side Encryption (Web Worker Isolated)
API keys are protected via the **NeuralVault** infrastructure.
- **AES-256-GCM**: Keys are encrypted client-side using a user-provided Master Passphrase.
- **SubtleCrypto Protocol**: Encryption relies on the browser's hardware-accelerated SubtleCrypto API.
- **Secure Context Barrier**: Cryptographic functions are automatically disabled if the application is not served over a secure channel (HTTPS) or local host alias (`localhost`/`127.0.0.1`).
- **Worker Isolation**: Decryption occurs inside a dedicated **Web Worker** thread, isolating key material from the main UI thread during sensitive operations.
- **Volatile Execution**: Keys are injected into the agent's environment only at the moment of execution and are never persisted to disk or indexedDB.
- **Auto-Locking**: Inactivity timers (default 30m) wipe the Master Key from memory, deep-freezing the engine until re-authorization.
- **Emergency Purge**: An **Emergency Vault Reset** protocol allows users to clear all encrypted configurations from local storage if a passphrase is lost, ensuring no stale encrypted data remains.

---

---

## 5. Resource Guard & Sandbox Awareness

Tadpole OS proactively monitors its execution environment to prevent platform-level attacks and resource starvation.

### 5.1 Resource Exhaustion Defense
The engine tracks critical system metrics to maintain stability and security:
- **RAM Pressure Monitoring**: Real-time tracking of memory usage. The interface alerts the operator if the engine or host system approaches memory limits, preventing OOM (Out Of Memory) conditions that could lead to denial of service.
- **CPU Load Verification**: Monitors processing intensity to detect recursive agent loops or malformed tool execution that could saturate host resources.

### 5.2 Sandbox Detection
The system automatically identifies its runtime environment to assess available security primitives:
- **Environment Awareness**: Detects if running within **Docker**, **Kubernetes**, or a **Virtual Machine**.
- **Capability Mapping**: Adjusts security assumptions based on the sandbox type (e.g., restricted filesystem access in containerized environments).

---

## 6. Network & Infrastructure Safety

### 6.1 Sandboxed Workspaces
Agents are restricted to physical directories under `workspaces/<cluster-id>/`.
- **Absolute Paths**: All file tools enforce absolute, canonicalized paths to prevent symlink-escape attacks.
- **Process Guard**: External subprocesses are wrapped in an asynchronous watchdog with a 60s timeout to prevent resource exhaustion (Fork Bombs / Deadlocks).

---

## 7. Reporting Vulnerabilities
If you discover a security vulnerability in Tadpole OS, please do not open a public issue. Instead, report it to the core security team via the designated sovereign channel.
