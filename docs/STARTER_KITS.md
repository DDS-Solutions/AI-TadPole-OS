# Sovereign Starter Kits — Tadpole OS

> [!NOTE]
> **Status**: Production Ready
> **Target Market**: Small-to-Medium Enterprises (SMEs)
> **Governance**: Pre-configured with BudgetGuard & Oversight Gates

Tadpole OS provides "one-click" autonomous swarm templates called **Sovereign Starter Kits**. These kits are designed to solve common business problems using specialized agent swarms with zero "AI Tax" (no per-seat fees).

## 📂 Location
All kits are located in the [starter_kits/](file:///c:/Users/Home%20Office_PC/.gemini/antigravity/playground/tadpole-os/starter_kits) directory.

## 🚀 Available Kits

### 1. Marketing & Growth ("Artemis")
**Goal**: Automate content strategy, SEO, and social distribution.
- **Lead Agent**: Artemis (Marketing Director)
- **Specialists**: SEO Strategist, Content Creator, Social Media Manager.
- **Workflow**: `market_research.md` -> `content_strategy.md` -> `distribute.md`.

### 2. Customer Success & Triage ("Beacon")
**Goal**: Automate customer feedback analysis and support triage.
- **Lead Agent**: Beacon (Customer Success Lead)
- **Specialists**: Triage Agent, Sentiment Analyst (Oracle), Knowledge Specialist.
- **Workflow**: `triage_tickets.md` -> `sentiment_audit.md` -> `draft_responses.md`.

### 3. Finance & Compliance ("Ledger")
**Goal**: Autonomous auditing and regulatory compliance monitoring.
- **Lead Agent**: Ledger (Finance Director)
- **Specialists**: Expense Auditor (AuditBot), Compliance Officer (Gavel).
- **Workflow**: `audit_expenses.md` -> `verify_compliance.md` -> `fiscal_report.md`.

## 🛠️ How to Use

### Option A: Via Template API
You can install a kit dynamically via the engine's template endpoint:
`POST /v1/engine/templates/install`
```json
{
  "template_id": "marketing_growth",
  "installation_path": "data/workspaces/my-company"
}
```

### Option B: Manual Installation
1. Copy the contents of the desired kit (e.g., `starter_kits/finance_compliance/*`) to your [data/workspaces/](file:///c:/Users/Home%20Office_PC/.gemini/antigravity/playground/tadpole-os/server-rs/data/workspaces) directory.
2. The engine will automatically detect the `swarm.json` and register the agents upon next startup or registry refresh.

## 🛡️ Sovereign Governance
Each kit is pre-configured with:
- **Budget Limits**: Hard-capped USD limits per agent.
- **Oversight Gates**: Sensitive tools (e.g., `notify_discord`, `git_push`) require human approval by default.
- **Memory Dedup**: Automatic semantic pruning via LanceDB to prevent context bloat.
