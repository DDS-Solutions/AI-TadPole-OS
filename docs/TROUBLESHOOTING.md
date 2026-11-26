> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / TROUBLESHOOTING
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🧩 Troubleshooting & Diagnostics

> **Intelligence Level**: Operational (Level 4)
> **Status**: Operational troubleshooting guidance; this is not a production-readiness certification
> **Version**: 1.1.464
> **Last Hardened**: 2026-11-20
> **Classification**: Sovereign

---

> [!IMPORTANT]
> ## 🔍 Phase 0: AI-Indexable Asset Check *(IDENTITY.md Directive #6 — Always First)*
> Before inspecting any log, UI panel, or source file, check the AI-indexable assets in order:
> 1. **[`docs/ERROR_REGISTRY.json`](./ERROR_REGISTRY.json)** — search by symptom keyword or error code. It maps error codes to source files and remediation paths.
> 2. **[`docs/SECURITY_REGISTRY.json`](./SECURITY_REGISTRY.json)** — search by security policy (`SEC-01` to `SEC-08`) or threat vector (`ASI01`–`ASI10`, `AML.TXXXX`) to identify enforcing files and validation commands.
> 3. **[`docs/TELEMETRY_MAP.json`](./TELEMETRY_MAP.json)** — trace by tag (e.g., `[VaultStore]`, `[AgentService]`) to find the log emitter location.
> 4. **For budget exhaustion**, look up the runtime code `BUDGET_EXHAUSTED` in `ERROR_REGISTRY.json`. `STASIS`, `LOGIC_BLOCKER`, and `COMPLIANCE_DRIFT` are governance labels, not registered runtime errors; the corresponding procedures are not automatic engine behavior.
>
> Only proceed to the sections below if these assets yield no result.

---

## 🏗️ System Connectivity

### 🔴 Dashboard shows "OFFLINE"
**Symptoms**: The PageHeader status indicator is red, and "Connection Lost" toasts appear.
1.  **Check Engine Status**: Ensure the Rust backend is running (`npm run engine` or `cargo run --release`).
2.  **Verify Endpoint**: Go to **System Config** and ensure `TadpoleOSUrl` matches the server address (typically `http://localhost:8000`).
3.  **CORS & Origin Mismatch**: If accessing from a remote IP (e.g., Tailscale), ensure the server binds `0.0.0.0` AND lists the exact browser origin in `ALLOWED_ORIGINS` (in production, empty `ALLOWED_ORIGINS` fails closed and rejects all browser CORS/WebSocket origins).

### 🔑 WebSocket Connection Refused or `403 Forbidden` on Admin Actions
**Symptoms**: "WebSocket Error: 401 Unauthorized" or `403 Forbidden` on backup, kill-switch, agent CRUD, or oversight settings.
1.  **Token Handshake**: The `NEURAL_TOKEN` in your `.env` MUST match the **Neural Engine Access Token** in your dashboard settings.
2.  **Step-Up Admin Elevation (`403 Forbidden`)**: Privileged routes enforce `RequireAdmin` (`NEURAL_ADMIN_TOKEN`). Enter your `NEURAL_ADMIN_TOKEN` in **Settings → Step-Up Administrative Token** (held strictly in memory, never saved to `localStorage`).
3.  **Privacy Shield & Live Voice (`403`)**: `/v1/engine/live-voice` returns `403 Forbidden` while Privacy Mode is active and requires `Sec-WebSocket-Protocol: bearer.<token>` pre-auth.
4.  **Interpreter Blocked (`command_guard`)**: Agent shell calls invoking `python`, `node`, `cargo`, `npm`, `pnpm`, or `rustc` are denied unless `TADPOLE_TRUST_INTERPRETERS=1` is explicitly enabled.
5.  **Redaction Intercept**: If the token contains special characters that trigger the `secret_redactor.rs` regex, it might be stripped. Use a standard hex string (32+ chars).

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
1.  **Canonicalization Error**: Tadpole OS uses strict path verification (`server-rs/src/adapter/filesystem.rs`). Paths must be relative to the cluster root — do not use `../` to attempt to exit the workspace.
2.  **Windows Paths**: Leave `DATABASE_URL` unset to use `data/state/tadpole.db`, or set a valid path such as `sqlite:data/state/tadpole.db` or an absolute SQLite URL. When unset, an existing legacy `data/tadpole.db` is copied to the canonical path at startup if the canonical database is absent.

---

## 🔒 Security & Vault

### 🛡️ Forget Master Password
**Symptoms**: Unable to unlock the Neural Vault.
1.  **Emergency Reset**: Click the **Emergency Vault Reset** button at the bottom of the unlock screen. This will purge all encrypted provider keys from the registry, allowing you to set a new password and re-enter your keys.

### ⛓️ Audit Trail Broken
**Symptoms**: UI shows "Verification Failed" on audit logs.
1.  **Tamper Detection**: The Merkle Audit Trail (`server-rs/src/security/audit/`) detects if the SQLite database has been manually edited. Check the **Security Dashboard** integrity status, or query the integrity endpoint: `curl -H "Authorization: Bearer <NEURAL_TOKEN>" http://localhost:8000/v1/oversight/security/integrity`. You can also run unit tests via `cargo test --manifest-path server-rs/Cargo.toml security::audit` to verify the logic.

---

## 🛠️ Performance & Telemetry

### 📉 UI Jank during Swarming
**Symptoms**: Browser becomes unresponsive when 5+ agents are active.
1.  **RAF Throttling**: Ensure **Telemetry Batching** is enabled in Settings. This flushes engine pulses to the UI on `requestAnimationFrame` to maintain 60fps.
2.  **Visualizer Detachment**: If the Swarm Visualizer is too heavy, click the **Detach** icon to move it to a separate window, freeing up the main thread.

### 🔴 OpenTelemetry Export Error (TonicTracesClient)
**Symptoms**: Log shows `ERROR opentelemetry_sdk: BatchSpanProcessor.ExportError ... connection refused`.
1.  **Collector Offline**: This occurs if the OpenTelemetry collector (e.g. Jaeger or Datadog) is not running on your host machine.
2.  **To Disable**: Set `OTEL_STDOUT_EXPORTER=false` and unset or comment out `OTEL_EXPORTER_OTLP_ENDPOINT` in your `.env` to prevent the engine from attempting to send trace spans.
