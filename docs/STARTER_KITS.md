> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / STARTER_KITS
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🚀 Sovereign Starter Kits & Industry Templates

> **Status**: Active  
> **Last Verified**: 2026-10-22  
> **Classification**: Sovereign  

---

Tadpole OS ships templates for common SMB workflows and integrates with the cloud-hosted **[AI-Tadpole-OS Swarm Architect & Industry Templates Portal](https://dds-solutions.github.io/AI-Tadpole-OS-Industry-Templates/)**. These templates do not provide a company-isolated Plane 2 runtime. Each kit includes a `swarm.json` definition, agent blueprints, and at least one workflow markdown file.

> [!TIP]
> **Swarm Architect Web Builder**:  
> Explore and export starter swarm templates from the external catalog. Templates are starting points, not production certification or an isolated Plane 2 runtime; review and test them before use: **[AI-Tadpole-OS Industry Templates](https://dds-solutions.github.io/AI-Tadpole-OS-Industry-Templates/)**.

---

## 🌐 Online Swarm Architect & Template Catalog

For turnkey SME deployments and customized multi-agent architectures, explore the official web builder and remote catalog:

- **Portal URL**: [https://dds-solutions.github.io/AI-Tadpole-OS-Industry-Templates/](https://dds-solutions.github.io/AI-Tadpole-OS-Industry-Templates/)
- **GitHub Repository**: [DDS-Solutions/AI-Tadpole-OS-Industry-Templates](https://github.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates)
- **Key Features**:
  - **Dual-Experience Builder**:
    - **Guided Setup (Business Owners & Operators)**: A 4-step plain-language workflow to establish operational objectives, safety boundaries (*What it reads, What it prepares, What requires approval*), verify tools, and export ready-to-run packages.
    - **Advanced Setup (Developers & Administrators)**: Granular control over system prompts (≤ 800 chars), LLM provider and model selections (Gemma4, Ollama, Groq, etc.), capability IDs (`read_file`, `write_file`, `execute_shell`, etc.), and MCP connectors.
  - **25+ Industry Blueprints**: Field Services, Accounting, Healthcare, Legal, Manufacturing, Marketing, Wholesale & Distribution, and more.
  - **220+ Specialist Agent Blueprints**: Curated roles with validated guardrails and oversight settings.
  - **Direct Import Support**: Export a blueprint directly from the website and import it via the Tadpole OS UI (**Template Store -> Downloaded Swarms**) or the `/v1/engine/templates/import` endpoint.

---

## 📦 Built-in Local Starter Kits

All built-in kits live in `starter_kits/`.

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

---

## 🛠️ How To Use

### Option A: Install From a Remote Git Repository

The template-install endpoint expects a repository URL plus a path inside that repository. You can install templates directly from the official [Industry Templates Repository](https://github.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates) or any compliant Git repo:

`POST /v1/engine/templates/install`

```json
{
  "repository_url": "https://github.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates.git",
  "path": "starter_kits/marketing_growth"
}
```

What the engine does:
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

### Option B: Import Custom Blueprint from Swarm Architect Web App

1. Visit **[https://dds-solutions.github.io/AI-Tadpole-OS-Industry-Templates/](https://dds-solutions.github.io/AI-Tadpole-OS-Industry-Templates/)**.
2. Customize your swarm using either **Guided Setup** or **Advanced Setup**.
3. Download/export your completed `swarm.json` blueprint file.
4. In Tadpole OS UI, navigate to **Intelligence -> Store** (or Settings -> Template Store).
5. Click **Downloaded Swarms** and select the exported `.json` file to open the interactive preview modal.
6. Click **Deploy Swarm** (which issues `POST /v1/engine/templates/import`).

### Option C: Manual Local Install

1. Copy a starter-kit directory from `starter_kits/<kit>/`.
2. Place agent JSON files where your runtime expects them, typically `data/swarm_config/agents/`.
3. Copy any workflow markdown into `directives/` if you want it available immediately to the workflow/skill system.
4. Restart the engine or refresh the relevant registries if needed.

---

## 🛡️ Governance Defaults

Starter kits are templates, not trust bypasses. Review these before production use:

- Agent budgets and oversight behavior still depend on your runtime settings and policies.
- **`FR-11` status** ([`directives/IDENTITY.md`](../directives/IDENTITY.md)): Business starter kits (`marketing_growth`, `customer_success`, `finance_compliance`) are templates, not isolated Plane 2 runtimes. Company-scoped authorization and storage remain pending; use synthetic inputs and stop at preview.
- The generic runner rejects only profiles whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not trigger that guard. Do not dispatch live company workflows through the generic runner.
- Sensitive tool calls should remain approval-gated for SMB deployments.
- Use local model endpoints when required, and pair them with host/network egress controls. `PRIVACY_MODE=true` constrains runner model routing but is not an air gap. Keep business data synthetic until company-scoped Plane 2 storage and authorization exist.
