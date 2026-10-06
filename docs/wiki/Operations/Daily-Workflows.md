---
title: "Daily Workflows"
tier: "1"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "optional"
risk-tags:
  - "RISK: MEDIUM"
---

# 📅 Daily Workflows

This page provides step-by-step, real-world workflow examples tailored for Small & Medium Business (SMB) operations. Each workflow demonstrates how to use Tadpole OS to complete a common office task.

---

## 1. Customer Feedback Analysis (Fully Local)

**Scenario**: You receive a CSV file of customer reviews and want to categorize them and generate a summary report — without any data leaving your office network.

### Steps

1. **Configure Your Team**: Create a "Support Analyst" agent (see [[Virtual-Employees|§1]] for setup anchors) using a local model (Ollama + Llama-3-8B) to keep customer data local and private.
2. **Setup the Project Workspace**: Place your customer review spreadsheet (`reviews.csv`) in your designated project folder.
3. **Issue Directive**: Type in [[Chat-&-Voice|SovereignChat]]:
   > *"Analyze reviews.csv, group them into categories, and write a summary report.txt in the same directory."*
4. **Approve Safe Actions**: Approve the `read_file` request for `reviews.csv` in the [[Approvals-&-Quotas|Oversight Queue]].
5. **Review Draft**: Once completed, review the generated report draft on your Workspaces page and click **[Button] Approve Draft** to merge it.

> [!TIP]
> This entire workflow runs without any network egress. All AI processing happens on your local hardware via Ollama.

---

## 2. Weekly Invoice Processing

☁️ REQUIRES NETWORK *(if using cloud-based OCR or LLM models)*

**Scenario**: Every Friday, you want your virtual bookkeeper to scan the `invoices/` folder, validate line items, and generate a reconciliation summary.

### Steps

1. **Configure Agent**: Create a "Bookkeeper" agent (see [[Virtual-Employees|§1]]) with a system prompt focused on financial document analysis.
2. **Map the Workspace**: Point the project workspace to your `invoices/` directory.
3. **Issue Directive**:
   > *"Scan all PDF invoices in this folder, extract vendor names and totals, and compile a reconciliation spreadsheet."*
4. **Approve File Operations**: Approve the `read_file` and `write_file` tool requests in the Oversight Queue.
5. **Review & Merge**: Inspect the generated spreadsheet draft and approve or reject.

---

## 3. SOP Enforcement for Document Review

**Scenario**: Your company has a Standard Operating Procedure (SOP) for reviewing contract documents. You want to ensure every agent follows the exact checklist.

### Steps

1. **Create Your SOP File**: Write a markdown SOP (e.g., `contract_review_sop.md`) and place it in `data/workflows/`.
2. **Assign to Agent**: In [[Virtual-Employees|§1]] (Agent Configuration), register the workflow file to your "Legal Reviewer" agent.
3. **Issue Directive**:
   > *"Review the attached contract draft against our SOP checklist."*
4. The agent will follow the exact steps defined in the SOP file, checking each item in order, and flag any compliance deviations.

> [!NOTE]
> SOP files placed in `data/workflows/` are automatically discovered by the engine and can be registered to individual agents via their profile configuration.

→ *See [[RAG-&-Memory|§3]] for details on how SOPs are indexed and enforced.*

---

## 4. Marketing Copy Review

**Scenario**: Your marketing team has drafted social media posts and you want an AI reviewer to check tone, grammar, and brand consistency.

### Steps

1. **Configure Agent**: Create a "Brand Reviewer" agent (see [[Virtual-Employees|§1]]) with a system prompt specifying your brand voice guidelines.
2. **Place Documents**: Put the draft posts (`social_posts.md`) in the project workspace.
3. **Issue Directive**:
   > *"Review social_posts.md for brand voice consistency, grammar, and suggest improvements. Write the revised version as social_posts_reviewed.md."*
4. **Approve & Merge**: Review the suggestions and approve the revised file.

---

## 5. Weekly Status Report Generation

**Scenario**: Every Monday, compile a summary of what your virtual team accomplished the prior week.

### Steps

1. **Issue Global Directive**: In [[Chat-&-Voice|SovereignChat]] with **Global Scope**:
   > *"Each department: summarize your completed tasks from last week. Compile into a single status_report.md."*
2. The [[Glossary#department-lead|Department Lead]] agent coordinates with all sub-agents, collects summaries, and compiles the final report.
3. **Review**: The compiled `status_report.md` appears in the draft queue for your approval.

---

## 6. Budgeting & Cost Awareness

All workflows incur token costs when using LLM providers. Use these guidelines to manage spend:

| Setting | Default | Purpose |
|---------|---------|---------|
| `DEFAULT_AGENT_BUDGET_USD` [RISK: MEDIUM] | `1.0` | Per-agent spending cap in USD |
| `MAX_SWARM_DEPTH` [RISK: MEDIUM] | `5` | Limits recursive delegation (Recommended: 3 for standard hardware) |
| `MAX_TASK_LENGTH` [RISK: MEDIUM] | `32768` | Caps token consumption per prompt |

> [!TIP]
> For routine tasks like email categorization or simple document review, a budget of $0.50 per agent run is typically sufficient with local models (which incur $0.00 cloud cost).

→ *See [[Cost-Monitoring|§1]] for real-time token cost monitoring.*
→ *See [[Configuration|§1]] for the complete environment variable reference.*

---

## 7. Visual Reference Anchors

- **[Button] New Workspace**
  - *Default state*: Enabled.
  - *Visible location*: Top header area of the Workspaces page.
  - *Observable side effect*: Spawns the workspace setup drawer to configure a new mapped directory.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Approve**
  - *Default state*: Waiting.
  - *Visible location*: Oversight Queue action cards.
  - *Observable side effect*: Permits the blocked agent command (e.g. `read_file` or `run_command`) to run.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Reject**
  - *Default state*: Waiting.
  - *Visible location*: Oversight Queue action cards.
  - *Observable side effect*: Instantly terminates the blocked agent command and returns a rejection signal to the executor loop.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Approve Draft**
  - *Default state*: Pending approval.
  - *Visible location*: Proposal card inside the Workspace Manager dashboard.
  - *Observable side effect*: Merges/overwrites the canonical project file with the draft contents.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Reject Draft**
  - *Default state*: Pending approval.
  - *Visible location*: Proposal card inside the Workspace Manager dashboard.
  - *Observable side effect*: Discards the proposed draft file.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Daily-Workflows)


[//]: # (Metadata: [wiki-ops])
