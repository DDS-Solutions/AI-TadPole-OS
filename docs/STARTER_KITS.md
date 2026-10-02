> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / STARTER_KITS
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🚀 Sovereign Starter Kits

> **Status**: Active  
> **Last Verified**: 2026-10-01 (tip `bbcf0d4`)  
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


## Skill resolution caveat (Round 3)

Starter-kit `agents/*.json` `skills` arrays must name **registry-native** tools the dispatcher actually implements (examples: `read_file`, `write_file`, `get_file_contents`, `fetch_url`, `spawn_subagent`, `recruit_specialist`, `issue_alpha_directive`, `update_working_memory`).

Historically some kit JSON listed **aspirational** skill names (`analyze_budget`, `edit_content`, `keyword_research`, `delegate`, `list_commits`, …) that are **not** in the tool registry — installs succeed (`POST /v1/engine/templates/install` exists) but those skills fail at resolution time. As of 2026-10-01 the worst offenders in `starter_kits/**/agents/*.json` were remapped to registry-real skills; treat any remaining unknown names as aspirational templates, not guaranteed capabilities.

Industry template catalogs (223-agent / multi-industry) live in the separate [AI-TadPole-OS-Industry-Templates](https://github.com/DDS-Solutions/AI-TadPole-OS-Industry-Templates) repo — do not confuse those marketing counts with the **four** tracked kits under `starter_kits/`.

## How To Use

### Option A: Install From a Git Repository

The current template-install endpoint expects a repository URL plus a path inside that repository:

`POST /v1/engine/templates/install`

```json
{
  "repository_url": "https://github.com/DDS-Solutions/AI-Tadpole-OS.git",
  "path": "starter_kits/marketing_growth"
}
```

What the engine currently does:
- Clones the target repository into `data/.bunker_cache/<uuid>/`
- Validates agent JSON, executable workflow headings, `swarm.json`, MCP launchers/placeholders, supported skill types, and all SkillSpector results before the first write
- Copies agent JSON files into `data/swarm_config/agents/` with create-only semantics
- Copies `swarm.json` into `data/swarm_config/installed/<kit>/`
- Copies workflow markdown into the canonical `directives/` registry
- Copies reviewed skills/scripts from `skills/` into `execution/`
- Merges non-conflicting `mcps.json` servers into `.agent/mcp_config.json` through a staged replacement
- Rolls back prior file/configuration writes if a later commit step fails
- Returns a structured receipt containing the cloned revision and exact planned/installed asset counts

Installed agent files are discovered by the normal agent-registry refresh/startup lifecycle; the install endpoint does not silently claim live activation or knowledge ingestion.

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
