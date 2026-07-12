---
title: "Cost Monitoring"
tier: "3"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "none"
risk-tags:
  - "RISK: MEDIUM"
---

# 💰 Cost Monitoring

This page documents the cost-accounting dashboard for monitoring API expenses, token consumption, and query latency across all LLM providers.

---

## 1. Neural Footprint & Token Cost Monitoring (`Command_Table`)

The `Command_Table` component is embedded in the **Oversight** page and provides a detailed breakdown of every LLM interaction.

### 1.1 Interactive Elements

- **[Table Row] Interaction Logs**
  - *Default state*: Populated with recent prompt-response pairs.
  - *Visible location*: Center region of the `Command_Table` panel.
  - *Observable side effect*: Clicking a row expands the panel to show raw request and response body text.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Column Header] Latency Tracker**
  - *Default state*: `0ms` (or latency of the most recent query).
  - *Visible location*: Upper-right column header of `Command_Table`.
  - *Observable side effect*: Displays execution durations in milliseconds to pinpoint slower local or cloud models.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Cell] Token Efficiency**
  - *Default state*: `0 tokens` (accumulated).
  - *Visible location*: Inside each query row of `Command_Table`.
  - *Observable side effect*: Details prompt and response token lengths to help optimize prompt templates.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

## 2. Budget Gauges & Cost Dashboard

### 2.1 Interactive Elements

- **[Gauge] Budget Quotas**
  - *Default state*: `0.0` USD used.
  - *Visible location*: Upper metrics grid of the Oversight dashboard.
  - *Observable side effect*: Dynamically reflects real-time cumulative LLM API expenses against configured caps.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Gauge] Total Spend**
  - *Default state*: `0.00` USD.
  - *Visible location*: Oversight dashboard or Cost analytics panel.
  - *Observable side effect*: Summarizes global API expenses across all agents and models.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Metric] Daily Burn Rate**
  - *Default state*: `0.00` USD/day.
  - *Visible location*: Upper row of the Cost dashboard metrics.
  - *Observable side effect*: Projects expected daily spending based on recent model requests.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Table] Provider Cost Breakdown**
  - *Default state*: Lists configured providers and their aggregated costs.
  - *Visible location*: Lower panel of the Cost dashboard.
  - *Observable side effect*: Allows operators to identify which LLM services are consuming the most tokens/credits.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Export Cost Report**
  - *Default state*: Enabled.
  - *Visible location*: Top right of the Cost dashboard panel.
  - *Observable side effect*: Triggers a download of cumulative cost telemetry as a CSV spreadsheet.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Yes (planned control, missing in current build).

- **[Selector] Date Range**
  - *Default state*: `Last 30 Days`.
  - *Visible location*: Top toolbar of the Cost dashboard.
  - *Observable side effect*: Updates all gauges, graphs, and tables to reflect cost telemetry in the chosen time window.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 2.2 Per-Agent Budget Tracking

Each agent tracks its own cost accumulation via `RunContext::current_cost_usd`. When the cumulative cost exceeds `RunContext::budget_usd`, the runner raises `BudgetExhausted` and the mission fails gracefully.

---

## 3. Cost Optimization Tips for SMBs

| Strategy | Impact | How To |
|----------|--------|--------|
| **Use local models (Ollama)** | $0.00 per query | Install Ollama + `llama3:8b`, set `OLLAMA_HOST` |
| **Lower reasoning depth** | Fewer tokens per turn | Set Mythos Reasoning Depth to 1-4 for simple tasks |
| **Set tight budgets** | Prevents runaway spend | `DEFAULT_AGENT_BUDGET_USD=0.50` |
| **Use auto-approve for reads** | Fewer manual approvals | Keep `AUTO_APPROVE_SAFE_SKILLS=true` |
| **Review token efficiency** | Find wasteful prompts | Inspect `Command_Table` token counts per query |

> [!TIP]
> For a typical SMB running 10 tasks/day with a mix of local and cloud models, expect monthly cloud API costs of **$5–$50 USD** depending on model selection and task complexity.

---

## 4. Telemetry & Accumulation

The engine tracks tokens globally via `GovernanceHub::tpm_accumulator` ([gov.rs:L38](../../../server-rs/src/state/hubs/gov.rs#L38)) (Tokens Per Minute), which feeds into the dashboard metrics:

| Metric | Source | Description |
|--------|--------|-------------|
| TPM (Tokens Per Minute) | `GovernanceHub::tpm_accumulator` ([gov.rs:L38](../../../server-rs/src/state/hubs/gov.rs#L38)) | Global token throughput counter |
| Per-run cost | `RunContext::current_cost_usd` | Cumulative cost for the active agent run |
| Budget cap | `RunContext::budget_usd` | Maximum spend allowed for the run |

---

## 5. Local-First Pledge

> [!IMPORTANT]
> **100% Local-First / Zero Data Leaks.**
> - All token counting, spending accumulation, and budget monitoring processes occur 100% locally.
> - No usage statistics or billing telemetry is ever transmitted to external trackers.

---

→ *See [[Approvals-&-Quotas|§2]] for budget enforcement and failure modes.*
→ *See [[Configuration|§6]] for engine limit settings (`DEFAULT_AGENT_BUDGET_USD`, `MAX_TASK_LENGTH`).*
→ *See [[LLM-Providers|§6]] for provider failover thresholds.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](../GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Cost-Monitoring)


[//]: # (Metadata: [wiki-security])
