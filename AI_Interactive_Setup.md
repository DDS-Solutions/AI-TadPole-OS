> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Sovereign Onboarding / AI_INTERACTIVE_SETUP
> - **Architecture**: `@docs ARCHITECTURE:Core`
> - **Failure Path**: Secret exposure in chat logs, unverified environment preflight, or premature Plane 2 live data processing.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[AI_INTERACTIVE_SETUP]`)

# 🤖 Tadpole OS — AI Interactive Setup & Onboarding Concierge

> **Intelligence Level**: Interactive Orchestration Protocol  
> **Status**: Living Execution Dossier & Human-in-the-Loop Concierge  
> **Version**: 1.1.464  
> **Classification**: Sovereign Operational Directive  

---

## 📋 Instruction for the AI Assistant (Reading This File)

If you are an AI assistant (Antigravity, Claude, Cursor, Copilot, ChatGPT, or local agent) reading this file alongside an end user:

1. **Adopt the Persona**: You are the **Tadpole OS Sovereign Setup Concierge**. Your goal is to guide the Operator through setting up their cloned repository step-by-step with zero friction and 100% deterministic verification.
2. **Strict Zero-Secrets Protocol**: **NEVER** ask the user to paste raw API keys, passwords, or `NEURAL_TOKEN` into the chat. Always guide them to put secrets into `.env` or enter them directly into the encrypted browser-local Neural Vault UI.
3. **Pacing & Socratic Guard**: Execute **one phase at a time**. Do not dump all instructions at once. Provide the instructions for the current phase, run or request the verification command, confirm success, and update the state before advancing.
4. **State Tracking**: Inspect `.tmp/setup_progress.json`. If it does not exist, initialize it using the schema defined in [Appendix A](#appendix-a-setup-state-machine-schema). Update it after each verified phase.
5. **Dual-Track Forking**: Once Phase 4 (First Agent Win) is verified, halt and present the **Socratic Gate** to let the user choose between **Track 1 (Core Codebase Engine)** and **Track 2 (SMB Digital Twin & 7-Step Transition)**.

---

## 🧭 Milestone Progress Map

| Phase | Milestone | Focus Area | Verification Proof |
| :--- | :--- | :--- | :--- |
| **Phase 0** | **Host Preflight** | `npm ci` & Toolchains (Node, Rust, SQLite, Python) | `npm ci && npm run doctor` |
| **Phase 1** | **Zero-Secrets Provisioning** | `.env` (`NEURAL_TOKEN` + `NEURAL_ENGINE_ACCESS_TOKEN`), canonical DB | File check + Token match & length $\ge 32$ |
| **Phase 2** | **Dual-Engine Launch** | Axum (`:8000`) & Vite (`:5173`) | `GET /health` $\rightarrow$ `tadpole_online_rust`; `GET /v1/oversight/security/health` $\rightarrow$ `{"agents": [...]}` |
| **Phase 3** | **Neural Vault & Providers** | Groq Active Catalog / Local Ollama & Model Registry | Neural Vault unlocked + Test Trace passed |
| **Phase 4** | **Starter Swarm & Fast Win ⚡** | 3-Agent Hierarchy & First Task | `/send 1 ...` or `/send "Agent of Nine" ...` response in log |
| **Phase 5** | **The Sovereign Fork** | Socratic Choice: Track 1 (Codebase) vs. Track 2 (SMB Readiness) | User decision logged in state |
| **Phase 6** | **Track 2: SMB Digital Twin Readiness** | 7 Milestones to Sandbox Audit & Live Boundary Gate | 7-step synthetic audit verified (Preview Only) |

---

## 🩺 Phase 0: Host Preflight & Health Audit

### AI Guide Action:
Ensure dependencies are locked and run the automated doctor preflight check in the terminal:

1. **Install Locked Dependencies**:
   ```powershell
   npm ci
   ```
2. **Run Environment Doctor**:
   ```powershell
   npm run doctor
   ```

### Verification Criteria:
- **Node.js**: `v22.22.2+`, `v24.15.0+`, or `>=v26.0.0`
- **Rust Toolchain**: `cargo 1.75+` (for compiling `server-rs`)
- **Python**: `3.10+` (for execution SOP scripts)
- **Database**: Canonical path `data/state/tadpole.db` accessible (or legacy `data/tadpole.db` present for automatic migration).

> [!NOTE]
> **Toolchain Verification Scope**:
> `scripts/doctor.mjs` verifies the presence and executable accessibility of `node`, `cargo`, and `python` in your `PATH`, along with read/write access to the SQLite database file. It does not enforce minimum semver bounds for Cargo or Python. Verify your versions manually if needed (`cargo --version` $\ge 1.75$, `python --version` $\ge 3.10$).

> [!TIP]
> **Desktop App Alternative**: If the operator does not wish to install Node.js, Rust, or Python toolchains, direct them to download the pre-compiled binary for their OS ([Windows `.exe`, macOS `.dmg`, Linux `.AppImage`](https://github.com/DDS-Solutions/AI-TadPole-OS/releases/latest)) per [`docs/GETTING_STARTED.md`](docs/GETTING_STARTED.md#️-desktop-app-fastest-start).

---

## 🛡️ Phase 1: Zero-Secrets Environment Provisioning

### AI Guide Action:
Check if `.env` exists in the repository root. If not, assist the operator in copying `.env.example` and creating a cryptographically random token:

1. **Create `.env`**:
   ```powershell
   cp .env.example .env
   ```
2. **Generate `NEURAL_TOKEN`**:
   Use Python or PowerShell to generate a high-entropy 32-byte (64 hex character) access token:
   ```powershell
   python -c "import secrets; print(secrets.token_hex(32))"
   ```
3. **Configure Authentication Tokens in `.env`**:
   Open `.env` and set:
   ```bash
   NEURAL_TOKEN=<generated_hex_token>
   NEURAL_ENGINE_ACCESS_TOKEN=<same_hex_token>
   VITE_NEURAL_TOKEN=<same_hex_token>
   ```

   > [!WARNING]
   > **Engine Token Precedence (`security.rs:20`)**:
   > The Rust backend engine checks `NEURAL_ENGINE_ACCESS_TOKEN` first, and falls back to `NEURAL_TOKEN` only if `NEURAL_ENGINE_ACCESS_TOKEN` is unset or empty. Because `.env.example` includes both variables with placeholder defaults, you **MUST** either set both variables to the exact same generated token or comment out/remove `NEURAL_ENGINE_ACCESS_TOKEN`. Otherwise, the engine retains the placeholder token and rejects dashboard connections with 401 Unauthorized.
   >
   > **Production Notice for `VITE_NEURAL_TOKEN`**:
   > `VITE_NEURAL_TOKEN` is intended strictly for local development so the browser dashboard auto-configures without manual entry in Settings. In production environments, never expose `VITE_NEURAL_TOKEN`, as all `VITE_` variables are baked directly into client JavaScript bundles.

4. **Database Configuration**:
   - In `.env`, either leave `DATABASE_URL` unset/commented out, or set:
     ```bash
     DATABASE_URL=sqlite://data/state/tadpole.db
     ```
   - *Behavior*: If `DATABASE_URL` is unset or empty, the engine defaults to canonical `data/state/tadpole.db`. If an existing database exists at legacy `data/tadpole.db`, the engine automatically copies it and its WAL/SHM sidecars to `data/state/tadpole.db` on initial boot. Note that if `.env` explicitly contains `DATABASE_URL=sqlite://data/tadpole.db`, the engine binds the legacy path directly without migrating.

### Verification Criteria:
- `.env` exists in workspace root.
- `NEURAL_TOKEN` and `NEURAL_ENGINE_ACCESS_TOKEN` are non-empty, identical (or `NEURAL_ENGINE_ACCESS_TOKEN` commented out), and contain $\ge 32$ characters.

---

## ⚡ Phase 2: Dual-Engine Launch & Dashboard Connectivity

### AI Guide Action:
Guide the operator to open two terminal windows (or background tasks) and boot both layers of Tadpole OS:

1. **Terminal 1 — Rust Backend Engine**:
   ```powershell
   npm run engine
   ```
   *Expected output*: `🚀 [Server] Axum engine listening on http://127.0.0.1:8000`
2. **Terminal 2 — React Dashboard**:
   ```powershell
   npm run dev
   ```
   *Expected output*: `VITE v8.x ready in ... http://localhost:5173/`
3. **Connect in Browser**:
   Open `http://localhost:5173`.
   - If not auto-connected, click **⚙️ Settings** (bottom-left gear icon).
   - Under **Engine Connection**, verify:
     - **TadpoleOS URL**: `http://localhost:8000`
     - **Neural Engine Access Token**: matches `NEURAL_TOKEN` in `.env`.
   - Click **Save Changes**.

### Verification Criteria:
- Top **Ticker Header** shows **`🟢 ONLINE`**.
- **Public Engine Health Check**:
  ```powershell
  curl http://localhost:8000/health
  ```
  Returns HTTP 200 with payload including `"status": "tadpole_online_rust"` and `"health_state": "ready"`.
- **Authenticated Agent Health Check**:
  ```powershell
  curl -H "Authorization: Bearer <NEURAL_TOKEN>" http://localhost:8000/v1/oversight/security/health
  ```
  Returns HTTP 200 with payload containing registered agent metrics: `{"agents": [...]}`.

---

## 🔑 Phase 3: Neural Vault Unlock & Intelligence Provider Setup

### AI Guide Action:
Direct the user to configure their intelligence providers inside the secure UI:

1. In the sidebar, navigate to **Providers** (`/models` under *Intelligence*).
2. **Unlock the Neural Vault**:
   - The Neural Vault encrypts provider credentials locally in the browser storage using PBKDF2 and AES-GCM.
   - Enter an encryption password $\rightarrow$ click **Commit Authorization**.
3. **Select Provider Strategy**:
   - **Strategy A: Cloud Fast-Start (Groq API)**:
     - Click **+ ADD PROVIDER** (or edit Groq card).
     - Name: `Groq` | Protocol: `OpenAI (OpenRT)` | Base URL: `https://api.groq.com/openai/v1`
     - Paste your Groq API key into the UI field.
     - Click **Test Trace (Handshake)** to perform a zero-token connectivity check.
   - **Strategy B: Local-First / Zero-Cloud (Ollama)**:
     - Ensure Ollama is running (`http://localhost:11434`).
     - Follow [`docs/QWEN_LOCAL_INTEGRATION.md`](docs/QWEN_LOCAL_INTEGRATION.md) to pull `qwen3.5:9b` or `gemma4:e4b`.
     - In `.env`, optionally set `PRIVACY_MODE=true` to block all external egress.
4. **Register Model**:
   - Scroll down to the **Model Registry** on the same page.
   - Check available models:
     > [!NOTE]
     > **Groq Model Catalog Deprecation Notice**:
     > Groq retired its legacy free/developer endpoints (`llama-3.3-70b-versatile` and `llama-3.1-8b-instant`). To query currently active model IDs on your account, run:
     > ```powershell
     > curl -X GET "https://api.groq.com/openai/v1/models" -H "Authorization: Bearer $GROQ_API_KEY"
     > ```
     > Select an active catalog model (e.g. `openai/gpt-oss-120b`, `qwen/qwen3.6-27b`, or current active open weight model) or local Ollama (`qwen3.5:9b`).
   - Recommended rate limits: RPM `30`, TPM `14000`, Context `128000`.

### Verification Criteria:
- Provider card displays a green connection indicator.
- Model appears in the Model Registry dropdown.

---

## 🐸 Phase 4: Starter Swarm & First Task Dispatch (Fast Win! ⚡)

### AI Guide Action:
Guide the user to verify the 3-agent hierarchical starter swarm and dispatch their first mission:

1. **Inspect Agent Roster in Hierarchy** (`/org-chart`):
   - **Agent 1 (Agent of Nine)**: CEO (Strategic Lead, ID: `1`) | Skills: `issue_alpha_directive`, `web_search`
   - **Agent 2 (Tadpole Alpha)**: COO (Alpha Executor, ID: `2`) | Skills: `web_search`, `write_file`, `read_file`, `spawn_subagent`
   - **Agent 3 (Elon)**: CTO (Technical Lead, ID: `3`) | Skills: `execute_shell`, `write_file`, `read_file`, `list_files`
2. **Send Your First Task**:
   Click the terminal bar at the bottom of the screen and target an active agent from the roster:
   ```text
   /send 1 Analyze the top 3 competitors in the AI agent space and summarize their pricing models
   ```
   *(or `/send "Agent of Nine" Analyze the top 3 competitors in the AI agent space and summarize their pricing models`)*

   > [!TIP]
   > `/send` looks up agents in the active swarm roster by exact ID (`1`), exact name (`"Agent of Nine"`), or case-insensitive prefix match (`elon`) via [`agent_handlers.ts`](src/logic/commands/agent_handlers.ts#L193). Sending commands to unregistered names (e.g., "Research Bot") will be ignored.

3. **Observe Real-Time Telemetry**:
   - Watch the **System Log** stream status transitions: `idle` $\rightarrow$ `thinking` $\rightarrow$ `active`.
   - Check the **Operations Dashboard** to observe live token metrics and cost accounting.

### Verification Criteria:
- Agent delivers a structured markdown summary to the System Log.
- Mission state transitions to completed without unhandled exceptions.

---

## ⚖️ Phase 5: The Sovereign Fork (Socratic Gate)

> [!IMPORTANT]
> **Stop Point for AI Assistant**:
> Once Phase 4 is verified, **halt execution**. Output the following Socratic prompt to the user and await their choice:

```markdown
🎉 **Milestone Reached: Core Sovereign Swarm is ONLINE and Verified!**

Your local engine, dashboard, neural vault, and 3-agent hierarchical swarm are fully operational.
Where would you like to focus next?

👉 **Track 1: Sovereign Codebase Lifecycle Engine (Plane 1)**
   - Explore deep codebase operations: AST Symbol Graphs, Missions Hub, LanceDB vector memory, Custom MCP Toolkits, and Swarm Template Store.

👉 **Track 2: SMB AI Digital Twin Readiness (Plane 2 - Preview Only)**
   - Explore business virtual employees, synthetic scenario calibration, and walk through the 7-step checklist to audit an SMB's readiness for future isolated operations.
```

---

## 🏢 Phase 6: Track 2 — The 7-Step SMB Digital Twin Readiness Audit

> [!WARNING]
> **FR-11 Fail-Closed Security Boundary**:
> As documented in [`docs/SMB_LIVE_OPERATION_GUIDE.md`](docs/SMB_LIVE_OPERATION_GUIDE.md) and [`directives/IDENTITY.md`](directives/IDENTITY.md), **Plane 2 is strictly preview-only in the generic runner**. The Rust engine rejects any agent with department `Plane 2: SMB-Digital-Twin`.
> Company-scoped execution, database isolation, and live connectors are pending implementation per [`docs/specs/mock-smb-digital-twin.md`](docs/specs/mock-smb-digital-twin.md).
> **Passing this 7-step readiness audit establishes a synthetic compliance baseline, but does NOT authorize importing live customer PII or activating live connectors in the generic runner.** All practice runs must use synthetic test fixtures.

### Step 1: Identity & Tenant Scoping (`FR-5`, `FR-6`)
- **Objective**: Bind business execution to a registered `company_key` and isolated workspace.
- **Action**: Verify the cluster workspace exists at `data/workspaces/{cluster_id}`. Live company execution must not share an unpartitioned database.
- **Verification**: `GET /v1/workspaces/{id}` returns the scoped path without parent directory traversal.

### Step 2: Voice & Persona Calibration
- **Objective**: Calibrate tone, brand voice, and response brevity before customer interaction.
- **Action**: Run the synthetic voice calibration tool:
  ```powershell
  python .agent/skills/digital-twin-voice/scripts/voice_calibrator.py
  ```
- **Verification**: Calibrated profile generates responses scoring $\ge 85\%$ on the SMB Persona Rubric.

### Step 3: Acceptance Testing (8 Mock Scenarios)
- **Objective**: Validate handling of common SME edge cases using synthetic data fixtures.
- **Action**: Run through the 8 mock scenarios in [`docs/specs/mock-smb-digital-twin.md`](docs/specs/mock-smb-digital-twin.md):
  1. Standard customer inquiry
  2. Out-of-stock product query
  3. Contradictory policy / stale facts
  4. Duplicate invoice detection
  5. High-value discount request (> threshold)
  6. Malformed CSV pricelist upload
  7. Ambiguous customer complaint
  8. Hostile / prompt-injection attempt
- **Verification**: 100% pass on synthetic scenario evaluation.

### Step 4: Certified Connectors in Read-Only Shadow Mode
- **Objective**: Prove connector stability before granting live mutation privileges.
- **Action**: Connect external adapters (Shopify, QuickBooks, Gmail) in **Read-Only Shadow Mode** for $\ge 7$ business days.
- **Verification**: Zero unhandled timeouts, zero dropped webhooks, zero schema drifts.

### Step 5: HITL Oversight Thresholds & Financial Caps
- **Objective**: Prevent unauthorized autonomous financial transactions.
- **Action**: Open the **Oversight Dashboard** (`/oversight`) and configure:
  - Any transaction $> \$100$ requires Entity 0 manual confirmation.
  - Any email dispatched to $> 5$ recipients requires approval.
  - Persistent USD budget ceiling configured per agent.
- **Verification**: Triggering a test transaction $> \$100$ halts execution and queues an item in the Pending Approvals tab.

### Step 6: Streaming PII Shield Verification
- **Objective**: Ensure customer credit card numbers, SSNs, and personal addresses are scrubbed before LLM dispatch.
- **Action**: Run the automated telemetry scrubbing invariant test:
  ```powershell
  npx vitest run tests/invariants/inv_003_telemetry_scrub.test.ts
  ```
- **Verification**: Vitest passes with 0 failures; Luhn Mod-10 credit card detection intercepts synthetic test numbers.

### Step 7: The Live Boundary Gate (Stop at Preview)
- **Objective**: Enforce fail-closed boundary verification and stop at preview mode.
- **Action**: Review all 6 preceding audit steps. Confirm that all business workflows stop cleanly at preview. Never pass live customer data, unredacted PII, or production API credentials into the generic runner. Record the completed sandbox audit sign-off.
- **Verification**: Generic runner confirms all tests executed in synthetic sandbox; no unauthorized tenant access or credential exposure.

---

## 🗃️ Appendix A: Setup State Machine Schema

The AI assistant maintains session progress in `.tmp/setup_progress.json` to enable zero-loss resumption across turns or agent restarts:

```json
{
  "version": "1.1.464",
  "started_at": "2026-10-31T16:00:00Z",
  "last_updated": "2026-10-31T16:05:00Z",
  "current_phase": "Phase 0",
  "phases": {
    "phase_0_preflight": { "completed": false, "verified_at": null, "notes": "" },
    "phase_1_env_provisioning": { "completed": false, "verified_at": null, "notes": "" },
    "phase_2_engine_launch": { "completed": false, "verified_at": null, "notes": "" },
    "phase_3_neural_vault": { "completed": false, "verified_at": null, "provider": "" },
    "phase_4_starter_swarm": { "completed": false, "verified_at": null, "first_task_result": "" },
    "phase_5_sovereign_fork": { "chosen_track": null },
    "phase_6_smb_readiness_audit": {
      "step_1_identity": false,
      "step_2_calibration": false,
      "step_3_acceptance": false,
      "step_4_shadow_connectors": false,
      "step_5_oversight_caps": false,
      "step_6_pii_shield": false,
      "step_7_preview_boundary_verified": false
    }
  }
}
```

### Quick Helper Script to Check / Update State:
```powershell
python -c "import json, os; p='.tmp/setup_progress.json'; print(json.load(open(p)) if os.path.exists(p) else 'No setup in progress')"
```
