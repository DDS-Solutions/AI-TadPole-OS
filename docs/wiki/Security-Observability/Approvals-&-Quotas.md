---
title: "Approvals & Quotas"
tier: "3"
status: "verified"
version: "1.3.0"
last-verified: "2026-08-08"
commit: "b1c347b1"
network-badge: "none"
risk-tags:
  - "RISK: HIGH"
  - "RISK: MEDIUM"
---

# ✅ Approvals & Quotas

All action oversight, budget caps, and system verification checks operate locally under your node boundaries.

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

### 1.2 Auto-Approve Safe Skills

- `AUTO_APPROVE_SAFE_SKILLS` [RISK: HIGH] — default `true`.
- When active, automatically permits standard read/search actions, queueing only write or shell tool calls.

- **[Toggle] Auto-Approve Safe Skills**
  - *Default state*: `true` (Enabled).
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

- **[Selector] Filter by Agent**
  - *Default state*: `""` (Empty filter).
  - *Visible location*: Top toolbar of Action Ledger panel on the Oversight page.
  - *Observable side effect*: Filters historical decision ledger items to display only those involving the specified Agent ID.
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
| Budget enforcement engine | `SecurityHub::budget_guard` ([sec.rs:L31](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/state/hubs/sec.rs#L31)) | Persistent budget governance and metering |

When an agent exceeds its budget, a `BudgetExhausted` error ([[Glossary#runnererror|RunnerError]]) is raised and the mission transitions to `Failed` status.

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

## 4. Safe Execution Mode

The **Safe Execution** toggle in [[Chat-&-Voice|SovereignChat]] provides an additional layer of control:

- When enabled (`is_safe_mode = true`), **all** tool executions require explicit approval, regardless of the `AUTO_APPROVE_SAFE_SKILLS` setting.
- Useful for high-stakes operations like financial document processing or contract editing.

---

## 5. Local-First Pledge

> [!IMPORTANT]
> **100% Local-First / Zero Data Leaks.**
> - All approval decisions, quota settings, and system health metrics are stored and processed entirely on your local machine.
> - No local network configuration or administrative telemetry is ever sent to external endpoints.

---

→ *See [[Kill-Switches|§1]] for emergency controls when oversight is insufficient.*
→ *See [[Cost-Monitoring|§1]] for detailed token cost tracking.*
→ *See [[Configuration|§6]] for engine limit settings.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Approvals-&-Quotas)


[//]: # (Metadata: [wiki-security])
