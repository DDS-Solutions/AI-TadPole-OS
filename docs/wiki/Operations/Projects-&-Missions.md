---
title: "Projects & Missions"
tier: "1"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "none"
risk-tags:
  - "RISK: MEDIUM"
---

# 📁 Projects & Missions

This page documents how to create, manage, and monitor projects (missions) within your Tadpole OS local node. Each project represents a scoped business goal with assigned virtual employees, a dedicated workspace folder, and a lifecycle state machine.

---

## 1. Defining Project Goals (`Missions`)

Clear project scoping ensures virtual teams execute tasks quickly and stay within your budget.

**Interactive Elements:**

- **[Button] New Project**
  - *Default state*: Enabled.
  - *Visible location*: Top header of the Projects dashboard.
  - *Observable side effect*: Opens the setup modal drawer to define goal scopes.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Input] Project Name**
  - *Default state*: Empty string (`""`).
  - *Visible location*: Setup modal drawer form.
  - *Observable side effect*: Captures custom string identifier for the new directory.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Textarea] Goal**
  - *Default state*: Empty string (`""`).
  - *Visible location*: Setup modal drawer form.
  - *Observable side effect*: Stores the primary task prompt for the Department Lead.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Selector] Agent Assignment**
  - *Default state*: Empty list.
  - *Visible location*: Team segment of the setup drawer.
  - *Observable side effect*: Binds virtual employee profiles to the active workspace.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Create Project**
  - *Default state*: Disabled until Name and Goal are filled.
  - *Visible location*: Bottom-right corner of the setup drawer.
  - *Observable side effect*: Dispatches workspace initialization and sets state to SpecReview.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Card] Collaboration Graph**
  - *Default state*: Active/Rendering.
  - *Visible location*: Right side of the active project panel.
  - *Observable side effect*: Draws the SVG graph displaying agent communication channels.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

## 2. Project Lifecycle Status States

Swarm missions transition through a strict state machine representing different phases of execution (defined in [[Glossary#missionstatus|MissionStatus]] and backend [`types.rs:L129-136`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/agent/types.rs#L129-L136)):

| State | Description |
|-------|-------------|
| **Pending** | The project has been created by the operator but has not yet been initiated. |
| **SpecReview** | The [[Glossary#department-lead|Department Lead]] agent reviews the project goals, scopes the recipes, and validates the required inputs and skills before beginning work. |
| **Active** | Specialists are actively executing tools, processing documents, and reasoning in the workspace sandbox. |
| **Completed** | The primary goal is achieved, and a final summary report is compiled by the Department Lead. |
| **Failed** | The swarm encountered a terminal error, exceeded the budget, or reached maximum delegation depth. |
| **Paused** | Processing is temporarily suspended by the operator or awaiting manual oversight resolution. |

---

## 3. Workspace Cluster Limits

To prevent system overload on local nodes, the maximum number of concurrent projects is capped by the `MAX_CLUSTERS` [RISK: MEDIUM] engine setting (default: `10` in code [`state/mod.rs:L552-556`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/state/mod.rs#L552-L556) and [`.env.example`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/.env.example)).

If the active workspace count matches this limit:
- Any attempts by the operator to initialize a new project workspace are **blocked**.
- A warning log is emitted: `Cluster limit reached` (logged as level `'warning'` by [`workspace_service.ts:L65-68`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/src/services/workspace_service.ts#L65-L68)).

> [!TIP]
> For standard office hardware, keeping active projects under 5 is recommended to ensure smooth operation. The absolute maximum is 10.

---

## 4. Shared Directories & File Assets (`Workspaces`)

Centralized coordination of local storage paths, department folders, and local files.

**Interactive Elements:**

- **[Button] Scan Workspace**
  - *Default state*: Enabled.
  - *Visible location*: Top toolbar of the Workspaces page.
  - *Observable side effect*: Initiates background check for files matching workspace path rules.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Gauge] Scan Progress**
  - *Default state*: Hidden or inactive.
  - *Visible location*: Overlay or status row in workspaces panel.
  - *Observable side effect*: Renders percentage progress of file tree scanning.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Table] Discovered Files**
  - *Default state*: Populated with scanned file listing.
  - *Visible location*: Central grid of the Workspace manager page.
  - *Observable side effect*: Displays file name, size, type, and options to assign individual files or read drafts.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 4a. Dynamic Environment Detection & OKF Governance

To ensure safety during execution, Tadpole OS runs a dynamic telemetry engine to detect host runtime environments and validate them against security rules specified in Open Knowledge Format (OKF) playbooks:

- **Dynamic Environment Detection**:
  - Automatically probes the host for runtime sandboxes (`docker`, `wasm_sandbox`), execution interfaces (`tauri_shell`, `vs_code`, `jupyter_lab`), and orchestration environments (`k8s_node`, `headless`).
  - Displays detected systems on the Workspaces dashboard using monochromatic Zinc-scaled badges.

- **OKF Playbook Safety Gates**:
  - Active workspaces are automatically scanned against required safety profiles specified in OKF playbooks (e.g. `requires: docker`, `requires k8s_node`).
  - If a required environment is missing (e.g. Docker daemon is offline but a playbook requires containerization), the workspace validation transitions to `warning` or `critical`, rendering a high-contrast safety warning banner on the dashboard.

- **Knowledge Playbook Auto-Mounting**:
  - The engine resolves active OKF playbooks assigned to a cluster's ID and automatically mounts them to the cluster's directory on disk as a read-only local JSON file (`playbooks.json`), allowing local agents to access playbooks without making direct database calls.
  - Active playbooks are rendered on the dashboard as clickable badges that deep-link to the **Semantic OKF Graph** page.

→ *See [[RAG-&-Memory|§1]] for local data synchronization details.*

---

## 5. Task Draft Branching Workflow

To protect core business documents from accidental modifications, virtual employees do not edit primary files directly:

1. Agents write proposed edits to temporary draft files (e.g., `report_draft.md`).
2. A draft queue appears on the Workspaces dashboard waiting for the operator's approval.

**Interactive Elements:**

- **[Button] Approve Draft**
  - *Default state*: Pending approval.
  - *Visible location*: Proposal card inside the Workspace Manager dashboard.
  - *Observable side effect*: Overwrites the canonical file with the draft contents and clears the proposal from the active list.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Reject Draft**
  - *Default state*: Pending approval.
  - *Visible location*: Proposal card inside the Workspace Manager dashboard.
  - *Observable side effect*: Deletes the draft file, cancels the operation, and feeds rejection parameters to the agent.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

## 6. Operations Center & Command Chain (`Ops_Dashboard`)

The central nervous system for monitoring running processes and hardware loads:

- **Operations Center**: Real-time agent status dashboard and engine health checks.
- **Org Chart**: Visual graph of the command chain from [[Glossary#department-lead|Department Lead]] to sub-agents.
- **Scheduled Jobs**: Manage recurring tasks (e.g., scraping weekly invoices, building weekly status updates).

→ *See [[Approvals-&-Quotas|§1]] for oversight queue details.*
→ *See [[Cost-Monitoring|§2]] for security dashboard and cost quota details.*
→ *See [[Dashboard-Guide|§3]] for the swarm visualizer that powers the Org Chart.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Projects-&-Missions)


[//]: # (Metadata: [wiki-ops])
