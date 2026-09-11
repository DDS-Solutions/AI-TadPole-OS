---
title: "Getting Started"
tier: "1"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "required"
risk-tags:
  - "RISK: HIGH"
  - "RISK: MEDIUM"
---

# 🚀 Getting Started

☁️ REQUIRES NETWORK

This guide walks you through booting your Tadpole OS local node, accessing the dashboard for the first time, and running your first virtual employee project.

---

## 1. Prerequisites

Before starting, ensure you have the following installed on your office computer:

| Requirement | Minimum Version | Purpose |
|-------------|----------------|---------|
| **Node.js** | v18+ | Frontend build tooling |
| **Rust** | 1.75+ | Backend engine compilation |
| **Python** | 3.10+ | Execution scripts and parity checks |
| **Ollama** (optional) | Latest | Local LLM inference (fully offline) |

> [!TIP]
> For a fully local, zero-network setup, install **Ollama** and download a model such as `llama3:8b`. This allows all AI processing to remain on your hardware.

→ *See [[Installation|§2]] for detailed compilation and dependency steps.*

---

## 2. Starting the Engine

### 2.1 Start the Backend (Rust Engine)

Navigate to the `server-rs/` directory and run:

```powershell
cargo run --release
```

The engine binds to `127.0.0.1:8000` by default (configurable via `PORT` [RISK: LOW] and `BIND_ADDRESS` [RISK: HIGH]).

> [!NOTE]
> **Runtime Governance Anchor**: All persistence paths resolve from `AppState.base_dir`, ensuring consistent folder roots across local-dev and server-mounted office networks.

### 2.2 Start the Frontend (Vite Dashboard)

In a separate terminal, from the project root:

```powershell
npm run dev
```

The dashboard opens at `http://localhost:5173` by default.

### 2.3 Verify Connection

Once both services are running, open your browser to `http://localhost:5173`. The **Engine Status** indicator in the `PageHeader` should show **🟢 ONLINE**, confirming the frontend is connected to the backend via WebSocket.

---

## 3. First-Time Dashboard Tour

The Tadpole OS dashboard is a multi-tab, multi-context interface:

1. **Dashboard Tab**: Your central operations hub showing engine health, active agents, and system metrics.
2. **Org Chart Tab**: A visual force-graph showing your virtual employee hierarchy.
3. **Standups Tab**: Voice-driven team coordination portal.
4. **Workspaces Tab**: Manage project folders, shared documents, and file assets.
5. **Docs Tab**: Access to embedded documentation and knowledge bases.
6. **Settings Tab**: Configure providers, profiles, and system parameters.

→ *See [[Dashboard-Guide|§1.1]] for a complete breakdown of every interactive element.*

### 3.1 Tab Navigation Anchors

- **[Tab] Tab Item**
  - *Default state*: Active on first tab (Dashboard).
  - *Visible location*: Multi-Tab Bar at the top of the viewport.
  - *Observable side effect*: Focuses the selected tab area and preserves navigation states.
  - *Keyboard shortcut*: `[Shortcut: 1-6]` to route to tabs 1-6.
  - *Missing in build*: Always present.

---

## 4. Running Your First Project

### Step 1: Configure a Provider
Navigate to **Settings → Model Manager** and unlock the [[Glossary#secure-credentials-vault|Secure Credentials Vault]] by entering a master passphrase. See [[LLM-Providers|§1.1]] for the vault visual reference anchors. Add at least one LLM provider:
- **Local (Ollama)**: Set `OLLAMA_HOST` [RISK: MEDIUM] to `http://localhost:11434` (default).
- **Cloud (OpenAI, Anthropic, etc.)**: Enter the API key for your chosen provider.

> [!IMPORTANT]
> Cloud providers require a network connection. Prompt payloads and decrypted API tokens are sent directly to the provider endpoint. See [[LLM-Providers|§2.1]] for the complete list of supported providers and their data paths.

### Step 2: Create a Virtual Employee
Navigate to **Settings → Agent Configuration** and create a new agent profile. See [[Virtual-Employees|§1]] for Agent Configuration Panel visual reference anchors:
- **Name**: e.g., "Support Analyst"
- **Model**: Select your configured LLM (e.g., `llama3:8b` via Ollama).
- **Budget**: Set a spending cap (default: `$1.00` USD per agent, configurable via `DEFAULT_AGENT_BUDGET_USD` [RISK: MEDIUM]).

→ *See [[Virtual-Employees|§1]] for advanced profile tuning.*

### Step 3: Create a Project Workspace
Navigate to **Workspaces** and create a new project folder (see [[Projects-&-Missions|§1]] for project creation wizard visual reference anchors):
- **Name**: e.g., "Customer Feedback Q2"
- **Folder Path**: Select a local directory containing your business documents (e.g., `reviews.csv`).
- **Assign Agent**: Attach your "Support Analyst" to this workspace.

### Step 4: Issue a Directive
Open the [[Chat-&-Voice|SovereignChat]] panel and type:

```
Analyze reviews.csv, group them into categories, and write a summary report.txt in the same directory.
```

### Step 5: Approve Actions
The agent will request permission to read files. Approve the `read_file` action in the **Oversight Queue** on the [[Approvals-&-Quotas|Oversight]] page. See [[Approvals-&-Quotas|§1]] for Oversight Queue visual reference anchors.

### Step 6: Review Output
Once the agent completes, review the generated draft on the **Workspaces** page and click **[Button] Approve Draft** (see [[Projects-&-Missions|§5]] for draft approval UI anchors) to merge it into your project folder.

---

## 5. Quick Reference: Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `[Shortcut: Ctrl+K]` or `[Shortcut: Ctrl+/]` | Toggle Command Palette |
| `[Shortcut: 1-6]` | Quick-navigate to tabs (Dashboard, Org Chart, Standups, Workspaces, Docs, Settings) |

> [!NOTE]
> Keyboard shortcuts are active only when no input fields are focused. Cited in [useLayoutNavigation.ts:L51-83](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/src/hooks/layout/useLayoutNavigation.ts#L51-L83).

---

## 6. What's Next?

| Goal | Go To |
|------|-------|
| Explore the full dashboard layout | [[Dashboard-Guide]] |
| Learn chat and voice commands | [[Chat-&-Voice]] |
| Set up multiple projects | [[Projects-&-Missions]] |
| Follow real SMB workflows | [[Daily-Workflows]] |
| Configure environment variables | [[Configuration]] |
| Understand security controls | [[Vault-&-Credentials]] |

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Getting-Started)


[//]: # (Metadata: [wiki-ops])
