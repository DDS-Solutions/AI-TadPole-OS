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
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.

# 🧩 Troubleshooting & Diagnostics

> **Intelligence Level**: Operational (Level 4)  
> **Status**: Verified Production-Ready  
> **Version**: 1.1.17  
> **Last Hardened**: 2026-04-12  
> **Classification**: Sovereign  

---

## 🏗️ System Connectivity

### 🔴 Dashboard shows "OFFLINE"
**Symptoms**: The PageHeader status indicator is red, and "Connection Lost" toasts appear.
1.  **Check Engine Status**: Ensure the Rust backend is running (`npm run engine` or `cargo run --release`).
2.  **Verify Endpoint**: Go to **System Config** and ensure `TadpoleOSUrl` matches the server address (typically `http://localhost:8000`).
3.  **CORS Mismatch**: If accessing from a remote IP (e.g., Tailscale), ensure the server is started with the correct host binding (`0.0.0.0`).

### 🔑 WebSocket Connection Refused
**Symptoms**: "WebSocket Error: 401 Unauthorized" in browser console.
1.  **Token Handshake**: The `NEURAL_TOKEN` in your `.env` MUST match the **Neural Engine Access Token** in your dashboard settings.
2.  **Redaction Intercept**: If the token contains special characters that trigger the `secret_redactor.rs` regex, it might be stripped. Use a standard hex string (32+ chars).

---

## 🧠 Intelligence & LLMs

### 🌑 Agent Returns No Response (Silent Failure)
**Symptoms**: Mission log shows "Thinking..." indefinitely or transitions directly to "Idle" without output.
1.  **Registry Audit**: Go to **AI Provider Manager** → scroll to **Model Registry**. Ensure the model assigned to the agent actually exists in the registry.
2.  **Vault Lock**: If the Neural Vault is locked, the engine cannot access decrypted API keys. Unlock the vault to resume operations.
3.  **Quota Exhaustion**: Check the **RPM/TPM** metrics in the System Log. If you've hit model limits, the agent will "park" until the window resets.

### 🧪 Malformed Tool Calls (Groq/Llama)
**Symptoms**: "Error: Malformed JSON in tool call."
1.  **Self-Healing Pass**: The engine automatically attempts a re-synthesis pass for common Llama-3 malformations. If it still fails, decrease the agent's **Temperature** to `0.1` or `0.3` for higher precision.

---

## 💾 Filesystem & Workspaces

### 🚫 "Access Denied" or "Sandbox Escape"
**Symptoms**: Tool fails with a security error when reading/writing files.
1.  **Canonicalization Error**: Tadpole OS uses strict path verification (`adapter/filesystem.rs`). Paths must be relative to the cluster root — do not use `../` to attempt to exit the workspace.
2.  **Windows Absolute Paths**: Ensure your `DATABASE_URL` in `.env` is an absolute path (e.g., `sqlite:D:\TadpoleOS-Dev\tadpole.db`).

---

## 🔒 Security & Vault

### 🛡️ Forget Master Password
**Symptoms**: Unable to unlock the Neural Vault.
1.  **Emergency Reset**: Click the **Emergency Vault Reset** button at the bottom of the unlock screen. This will purge all encrypted provider keys from the registry, allowing you to set a new password and re-enter your keys.

### ⛓️ Audit Trail Broken
**Symptoms**: UI shows "Verification Failed" on audit logs.
1.  **Tamper Detection**: The Merkle Audit Trail (`audit.rs`) detects if the SQLite database has been manually edited. Run `python execution/verify_all.py` to diagnose the specific point of failure.

---

## 🛠️ Performance & Telemetry

### 📉 UI Jank during Swarming
**Symptoms**: Browser becomes unresponsive when 5+ agents are active.
1.  **RAF Throttling**: Ensure **Telemetry Batching** is enabled in Settings. This flushes engine pulses to the UI on `requestAnimationFrame` to maintain 60fps.
2.  **Visualizer Detachment**: If the Swarm Visualizer is too heavy, click the **Detach** icon to move it to a separate window, freeing up the main thread.
