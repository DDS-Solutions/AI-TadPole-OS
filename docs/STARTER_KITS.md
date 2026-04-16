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

# 🚀 Sovereign Starter Kits

> **Status**: Active  
> **Last Verified**: 2026-04-12  
> **Classification**: Sovereign  

---

Tadpole OS ships with local starter kits for common SMB workflows. Each kit includes a `swarm.json` definition, agent blueprints, and at least one workflow markdown file.

## Location

All built-in kits live in `starter_kits/`.

## Available Kits

### 1. Marketing & Growth
- **Directory**: `starter_kits/marketing_growth`
- **Swarm ID**: `marketing-growth-starter`
- **Initial Agent**: `marketing-alpha`
- **Required Agents**: `marketing-alpha`, `content-creator`, `seo-analyst`
- **Workflow**: `workflows/campaign_gen.md`
- **Goal**: Content strategy, campaign generation, and SEO support for SMB growth teams.

### 2. Customer Success & Triage
- **Directory**: `starter_kits/customer_success`
- **Swarm ID**: `customer-success-starter`
- **Initial Agent**: `success-lead`
- **Required Agents**: `success-lead`, `triage-bot`, `feedback-synthesizer`
- **Workflow**: `workflows/support_triage.md`
- **Goal**: Ticket routing, feedback analysis, and support-response preparation.

### 3. Finance & Compliance
- **Directory**: `starter_kits/finance_compliance`
- **Swarm ID**: `finance-compliance-starter`
- **Initial Agent**: `finance-lead`
- **Required Agents**: `finance-lead`, `expense-auditor`, `compliance-agent`
- **Workflow**: `workflows/fiscal_audit.md`
- **Goal**: Expense review, compliance checks, and financial audit support.

### 4. Explorer Scout
- **Directory**: `starter_kits/explorer_scout`
- **Swarm ID**: `explorer-scout-starter`
- **Initial Agent**: `scout-alpha`
- **Required Agents**: `scout-alpha`, `github-analyst`
- **Workflow**: `workflows/github_scout.md`
- **Goal**: Repository discovery and competitive codebase scouting.

## How To Use

### Option A: Install From a Git Repository

The current template-install endpoint expects a repository URL plus a path inside that repository:

`POST /v1/engine/templates/install`

```json
{
  "repository_url": "https://github.com/DDS-Solutions/Tadpole-OS.git",
  "path": "starter_kits/marketing_growth"
}
```

What the engine currently does:
- Clones the target repository into `data/.bunker_cache/<uuid>/`
- Copies agent JSON files into `data/swarm_config/agents/`
- Copies `swarm.json` into `data/swarm_config/installed/<kit>/`
- Copies workflow markdown into `directives/`
- Merges `mcps.json` into `.agent/mcp_config.json` when present
- Persists installed agents into SQLite and the live in-memory registry

### Option B: Manual Local Install

1. Copy a starter-kit directory from `starter_kits/<kit>/`.
2. Place agent JSON files where your runtime expects them, typically `data/swarm_config/agents/`.
3. Copy any workflow markdown into `directives/` if you want it available immediately to the workflow/skill system.
4. Restart the engine or refresh the relevant registries if needed.

## Governance Defaults

Starter kits are templates, not trust bypasses. Review these before production use:

- Agent budgets and oversight behavior still depend on your runtime settings and policies.
- Sensitive tool calls should remain approval-gated for SMB deployments.
- Local-first deployments should pair starter kits with `PRIVACY_MODE=true`, scoped memory, and a real `NEURAL_TOKEN`.
