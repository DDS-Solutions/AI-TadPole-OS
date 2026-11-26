---
title: "Getting Started"
tier: "1"
status: "verified"
version: "1.1.464"
last-verified: "2026-11-20"
commit: "3eef5c2d"
network-badge: "required"
risk-tags:
  - "RISK: HIGH"
  - "RISK: MEDIUM"
---

# 🚀 Getting Started

☁️ REQUIRES NETWORK

This guide walks you through booting your AI-Tadpole-OS local node, accessing the dashboard for the first time, and running your first virtual employee project.

> [!TIP]
> **Skip the build step**: Download the pre-built native desktop bundle from [GitHub Releases](https://github.com/DDS-Solutions/AI-TadPole-OS/releases/latest) for instant launch on Windows, macOS, or Linux — no Rust or Node.js install required.

| Platform | Download |
|----------|----------|
| **Windows** | [⬇️ Windows Installer (.exe)](https://github.com/DDS-Solutions/AI-TadPole-OS/releases/latest) |
| **macOS Universal** | [🍏 macOS Universal (.dmg)](https://github.com/DDS-Solutions/AI-TadPole-OS/releases/latest) — Apple Silicon & Intel |
| **Linux** | [🐧 Linux (.deb / .AppImage)](https://github.com/DDS-Solutions/AI-TadPole-OS/releases/latest) |

> [!NOTE]
> **macOS Gatekeeper**: The `.dmg` is ad-hoc signed (no Apple notarization). On first launch right-click → **Open**, or run: `xattr -cr /Applications/AI-Tadpole-OS.app`.
> Runtime data is written to `~/Library/Application Support/com.tadpole.os/` — not inside the `.app` bundle.

---

## 1. Prerequisites

Before starting, ensure you have the following installed on your office computer:

| Requirement | Minimum Version | Purpose |
|-------------|----------------|---------|
| **Node.js** | 22.22.2+ (22.x), 24.15.0+ (24.x), or 26+ | Frontend build and test tooling |
| **Rust** | 1.91+ (`1.95.0` pinned in Docker/CI) | Backend engine compilation |
| **Python** | 3.10+ | Execution scripts and parity checks |
| **Ollama** (optional) | Latest | Local LLM inference (fully offline) |

> [!TIP]
> To run inference through local hardware, install **Ollama** and download a model such as `llama3:8b`. This routes model inference to that local endpoint; other engine features, integrations, and the global privacy monitor may still use the network. Apply host/network egress controls for an offline deployment.

→ *See [[Installation]] for detailed compilation and dependency steps.*

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

Once both services are running, open your browser to `http://localhost:5173`, configure your `NEURAL_TOKEN` (and optional in-memory **Step-Up Administrative Token** if using a distinct `NEURAL_ADMIN_TOKEN`) under **Settings → Engine Connection**, and verify the **Engine Status** indicator in the `PageHeader` shows **🟢 ONLINE** (connected via `/v1/engine/ws` using post-connect `{"type":"auth"}` handshake).

---

## 3. First-Time Dashboard Tour

The Tadpole OS dashboard is a multi-tab, multi-context interface:

1. **Dashboard Tab**: Your central operations hub showing engine health, active agents, and system metrics.
2. **Org Chart Tab**: A visual force-graph showing your virtual employee hierarchy.
3. **Standups Tab**: Voice-driven team coordination portal.
4. **Workspaces Tab**: Manage project folders, shared documents, and file assets.
5. **Docs Tab**: Access to embedded documentation and knowledge bases.
6. **Settings Tab**: Configure providers, profiles, and system parameters.

→ *See [[Dashboard-Guide]] for a complete breakdown of every interactive element.*

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
Navigate to **Settings → Model Manager** and unlock the [[Secure Credentials Vault|Glossary#secure-credentials-vault]] by entering a master passphrase. See [[LLM-Providers]] for the vault visual reference anchors. Add at least one LLM provider:
- **Local (Ollama)**: Set `OLLAMA_HOST` [RISK: MEDIUM] to `http://localhost:11434` (default).
- **Cloud (OpenAI, Anthropic, etc.)**: Enter the API key for your chosen provider.

> [!IMPORTANT]
> Cloud providers require a network connection. Prompt payloads and decrypted API tokens are sent directly to the provider endpoint. See [[LLM-Providers]] for the complete list of supported providers and their data paths.

### Step 2: Create a Virtual Employee
Navigate to **Settings → Agent Configuration** and create a new agent profile. See [[Virtual-Employees]] for Agent Configuration Panel visual reference anchors:
- **Name**: e.g., "Support Analyst"
- **Model**: Select your configured LLM (e.g., `llama3:8b` via Ollama).
- **Budget**: Set a spending cap (default: `$1.00` USD per agent, configurable via `DEFAULT_AGENT_BUDGET_USD` [RISK: MEDIUM]).

→ *See [[Virtual-Employees]] for advanced profile tuning.*

### Step 3: Create a Project Workspace
Navigate to **Workspaces** and create a new project folder (see [[Projects-&-Missions]] for project creation wizard visual reference anchors):
- **Name**: e.g., "Customer Feedback Q2"
- **Folder Path**: Select a local directory containing your business documents (e.g., `reviews.csv`).
- **Assign Agent**: Attach your "Support Analyst" to this workspace.

### Step 4: Issue a Directive
Open the [[SovereignChat|Chat-&-Voice]] panel and type:

```
Analyze reviews.csv, group them into categories, and write a summary report.txt in the same directory.
```

### Step 5: Approve Actions
The agent will request permission to read files. Approve the `read_file` action in the **Oversight Queue** on the [[Oversight|Approvals-&-Quotas]] page. See [[Approvals-&-Quotas]] for Oversight Queue visual reference anchors.

### Step 6: Review Output
Once the agent completes, review the generated draft on the **Workspaces** page and click **[Button] Approve Draft** (see [[Projects-&-Missions]] for draft approval UI anchors) to merge it into your project folder.

---

## 5. Quick Reference: Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `[Shortcut: Ctrl+K]` or `[Shortcut: Ctrl+/]` | Toggle Command Palette |
| `[Shortcut: 1-6]` | Quick-navigate to tabs (Dashboard, Org Chart, Standups, Workspaces, Docs, Settings) |

> [!NOTE]
> Keyboard shortcuts are active only when no input fields are focused. Cited in [useLayoutNavigation.ts:L51-83](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/src/hooks/layout/useLayoutNavigation.ts#L51-L83).

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
| Build or install macOS Universal `.dmg` | [[Installation#7-macos-universal-desktop-installation]] |

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Getting-Started)


[//]: # (Metadata: [wiki-ops])
