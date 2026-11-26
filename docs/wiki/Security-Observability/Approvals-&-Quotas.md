---
title: "Approvals & Quotas"
tier: "3"
status: "reviewed"
version: "1.1.464"
last-verified: "2026-11-20"
network-badge: "none"
risk-tags:
  - "RISK: HIGH"
  - "RISK: MEDIUM"
commit: "3eef5c2d"
---

# ✅ Approvals & Quotas

The engine stores oversight and quota state locally, while configured model providers and external tools may still receive requests over the network. Local governance state is not a network-egress boundary.

---

## 1. Oversight Gate & Approval Queue (`Oversight`)

### 1.1 The Approval Queue

The Oversight Queue displays actions that require explicit confirmation before execution:

| Action Type | Example | Approval Required? |
|-------------|---------|-------------------|
| **Read file** | `read_file reviews.csv` | Depends on `AUTO_APPROVE_SAFE_SKILLS` |
| **Write file** | `write_file report.md` | ✅ Always |
| **Execute shell** | `run_command npm test` | ✅ Always |
| **Browse web** | `web_search "market trends"` | ✅ Always |

**Interactive Elements:**

- **[Button] Approve**
  - *Default state*: Waiting.
  - *Visible location*: Individual tool request cards in the Oversight timeline.
  - *Observable side effect*: Resolves the backend blocking promise, allowing the tool payload execution (`approved`).
  - *Keyboard shortcut*: `A` (when request card is focused/selected).
  - *Missing in build*: Always present.

- **[Button] Reject**
  - *Default state*: Waiting.
  - *Visible location*: Individual tool request cards in the Oversight timeline.
  - *Observable side effect*: Returns a cancellation trace to the agent loop (`rejected`).
  - *Keyboard shortcut*: `R` (when request card is focused/selected).
  - *Missing in build*: Always present.

### 1.1.1 Administrative Role & Cryptographic Decision Signing
- **Admin RBAC (`RequireAdmin`)**: `POST /v1/oversight/{id}/decide`, `PUT /v1/oversight/settings`, `PUT /v1/oversight/security/quotas/{entity_id}`, `PUT /v1/oversight/security/missions/{id}/quota`, and `PUT /v1/oversight/security/policies` all require `NEURAL_ADMIN_TOKEN` (browser Step-Up Administrative Token); operational `NEURAL_TOKEN` returns `403 Forbidden`. Submitting `oversight:decision` over WebSocket (`/v1/engine/ws`) likewise requires `AuthenticatedRole::Admin`.
- **Production Ed25519 Signature Pinning (`ovs:v2`)**: In production (`TADPOLE_ENV=production`), decisions must include a valid Ed25519 signature over canonical `ovs:v2` bytes with ±5 min timestamp freshness and a single-use `nonce`, pinned to `OVERSIGHT_PUBLIC_KEY`.
- **Android Companion Hardware Biometric Signing**: Paired Android devices submitting `/v1/remote/oversight/decide` unlock their Ed25519 signing key strictly through an `AndroidKeyStore` **RSA-2048 OAEP** `BiometricPrompt.CryptoObject` (`BIOMETRIC_STRONG` only — `setUserAuthenticationRequired(true)`, `setInvalidatedByBiometricEnrollment(true)`).

### 1.2 Auto-Approve Safe Skills

- `AUTO_APPROVE_SAFE_SKILLS` [RISK: HIGH] — default **`false`** (fail-closed by default in `server-rs/src/state/mod.rs` and `.env.example`).
- When explicitly enabled, automatically permits standard read/search actions while queueing write or shell tool calls.

- **[Toggle] Auto-Approve Safe Skills**
  - *Default state*: `false` (Disabled).
  - *Visible location*: Security settings panel on the `Oversight` page.
  - *Observable side effect*: When active, read-only operations bypass the queue.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 1.3 Oversight Timeline Controls

- **[Card] Tool Request Card**
  - *Default state*: Unselected/Pending.
  - *Visible location*: Oversight timeline scroll area on the Oversight page.
  - *Observable side effect*: Displays tool invocation details, arguments, and status; enables Approve/Reject buttons.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Metric] Pending Count**
  - *Default state*: `0`.
  - *Visible location*: Tab or section title count badge in the Oversight dashboard.
  - *Observable side effect*: Dynamically increments when an agent requests a tool execution, and decrements upon approval/rejection.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Approve All**
  - *Default state*: None.
  - *Visible location*: Oversight timeline toolbar.
  - *Observable side effect*: Approves all queued operations concurrently.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Yes (planned control, missing in current build).

- **[Selector] Segmented Decision Filter**
  - *Default state*: `All`.
  - *Visible location*: Top toolbar of Action Ledger panel on the Oversight page.
  - *Observable side effect*: Filters ledger entries between `All`, `HITL` (Human-In-The-Loop manual decisions), and `Auto` (system auto-approved actions based on policy).
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Selector] Filter by Cluster & Agent**
  - *Default state*: Empty filters (`""`).
  - *Visible location*: Top toolbar of Action Ledger panel on the Oversight page.
  - *Observable side effect*: Filters historical decision ledger items to display only those involving the specified Cluster ID or Agent ID.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Cell] Portaled Parameter Popover (`Params_Cell`)**
  - *Default state*: Truncated preview text.
  - *Visible location*: Parameters column of the Action Ledger table.
  - *Observable side effect*: Hovering reveals a structured payload tooltip; clicking pins the JSON popover open with syntax formatting and copy-to-clipboard functionality, rendered via a React portal to prevent table clipping.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 1.4 Granular Capability Governance (SEC-05 & SEC-06)

Tadpole OS enforces strict non-overridable security floors and cryptographic capability verification:

- **Mandatory Security Floors (SEC-06):** High-risk capability classes enforce a minimum non-overridable security floor:
  - `Execute` (Shell / Binaries) $\rightarrow$ Minimum `Prompt`
  - `Install` (Skills / Workflows) $\rightarrow$ Minimum `Prompt`
  - `Modify` (System / Directives) $\rightarrow$ Minimum `Prompt`
  - `Delete` (Files / State) $\rightarrow$ Minimum `Prompt`
  - `Approve` (Vault / Credentials) $\rightarrow$ Minimum `Deny`
  Unsigned policy writes attempting to weaken controls below these floors are automatically rejected at write time.
- **Cryptographically Signed Capability Overrides (SEC-05):** `PermissionMode::Allow` for high-risk capabilities can only be granted by an Ed25519-signed manifest payload (`SignedCapabilityManifest`) verified against content hashes and approval attribution.

---

## 2. Budget Quotas

The dashboard tracks hardware safety and cloud costs in real-time:

- **[Gauge] Budget Quotas**
  - *Default state*: `0.0` USD used.
  - *Visible location*: Upper metrics grid of the Oversight dashboard.
  - *Observable side effect*: Dynamically reflects real-time cumulative LLM API expenses against configured caps.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 2.1 Budget Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `DEFAULT_AGENT_BUDGET_USD` [RISK: MEDIUM] | `1.0` | Per-agent spending cap in USD |
| Budget enforcement engine | `SecurityHub::budget_guard` ([sec.rs:L27](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/server-rs/src/state/hubs/sec.rs#L27)) | Persistent budget governance and metering |

When an agent exceeds its budget, a `BudgetExhausted` error ([[RunnerError|Glossary#runnererror]]) is raised and the mission transitions to `Failed` status.

---

## 3. Swarm Health Monitoring

- **[Metric] Swarm Health**
  - *Default state*: Perfect health indicator.
  - *Visible location*: Center-right of the metrics grid.
  - *Observable side effect*: Evaluates execution failures and warns of service namespace breakers tripping.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Alert] RAM/VRAM Pressure**
  - *Default state*: Inactive.
  - *Visible location*: Bottom panel of Oversight dashboard.
  - *Observable side effect*: Emits amber/red warning banners when system RAM or GPU memory utilization approaches VRAM limits.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

## 4. Execution Modes & Safe Execution

The **3-Mode Execution Selector (`Chat` · `Plan` · `Swarm`)** in [[SovereignChat|Chat-&-Voice]] and Voice Standups controls both tool access and Mission tracking before work ever reaches the Oversight Queue:

- **`Chat` (`execution_mode: "chat"`)**: Zero tools exposed and no `mission_history` row created — ideal for pure conversational brainstorming before mobilizing agents.
- **`Plan` (`execution_mode: "plan"`)**: Read-only codebase/knowledge research only — file writes, shell commands, and swarm delegation are blocked, and no `mission_history` row is created.
- **`Swarm` (`execution_mode: "swarm"`)**: Full tracked Mission lifecycle where hazardous tool executions pass through the Oversight Gate (and **`Launch Swarm`** promotes a `Chat`/`Plan` thread into a tracked Mission via `recent_findings`).
- **Safe Execution Toggle (`is_safe_mode`)**: Quick-toggles between `Chat` and `Swarm` in the UI (`is_safe_mode = execution_mode !== 'swarm'`). Legacy API payloads setting `safe_mode: true` without `execution_mode` automatically resolve to read-only `plan` mode on the backend.

---

## 5. Local-First Pledge

> [!IMPORTANT]
> Approval and quota state is maintained by the local engine. This describes where the engine stores its governance state; it does not guarantee that provider requests, external tools, or other configured integrations stay local.

---

→ *See [[Kill-Switches]] for emergency controls when oversight is insufficient.*
→ *See [[Cost-Monitoring]] for detailed token cost tracking.*
→ *See [[Configuration]] for engine limit settings.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Approvals-&-Quotas)


[//]: # (Metadata: [wiki-security])
