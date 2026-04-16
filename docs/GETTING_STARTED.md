# 🚀 Tadpole OS — Getting Started Guide

> **Intelligence Level**: Super AI-Awakened (Level 5)  
> **Status**: Verified Production-Ready  
> **Version**: 1.7.0  
> **Last Hardened**: 2026-04-10  
> **Classification**: Sovereign  

---

## 📚 Table of Contents

- [🛡️ Sovereign Configuration (The Zero-Secrets Handshake)](#-sovereign-configuration-the-zero-secrets-handshake)
- [🏗️ Hardware Requirements (Scaling Spec)](#-hardware-requirements-scaling-spec)
- [Step 1: Connect to the Engine](#step-1-connect-to-the-engine)
- [Step 2: Unlock the Neural Vault & Add Your Groq Provider](#step-2-unlock-the-neural-vault-add-your-groq-provider)
- [Step 3: Deployment with Sovereign Starter Kits](#step-3-deployment-with-sovereign-starter-kits)
- [Step 4: Local Intelligence (Local LLMs via Ollama)](#step-4-local-intelligence-local-llms-via-ollama)
- [Step 5: Add Models to the Registry](#step-5-add-models-to-the-registry)
- [Step 6: Configure an Agent Node](#step-6-configure-an-agent-node)
- [Step 7: Bulk Capability Assignment (Skills Hub)](#step-7-bulk-capability-assignment-skills-hub)
- [Step 8: Importing External Capabilities (.md)](#step-8-importing-external-capabilities-md)
- [Step 9: Create and Execute a Mission](#step-9-create-and-execute-a-mission)
- [Step 10: Workspace & Cluster Management](#step-10-workspace-cluster-management)
- [Step 11: External Adapters & Workspace Tools](#step-11-external-adapters-workspace-tools)
- [Step 12: SME Data Intelligence (Connectors & Workflows)](#step-12-sme-data-intelligence-connectors-workflows)
- [Step 13: Send a Task & Get Results](#step-13-send-a-task-get-results)
- [Step 14: Performance Analysis & Real-time Telemetry](#step-14-performance-analysis-real-time-telemetry)
- [Step 15: The Swarm Template Ecosystem](#step-15-the-swarm-template-ecosystem)
- [🛠️ Useful Terminal Commands](#-useful-terminal-commands)
- [🧩 Troubleshooting](#-troubleshooting)
- [🐸 Starter Swarm Configuration (Quick-Deploy)](#-starter-swarm-configuration-quick-deploy)
- [🎯 Showcase Mission: "Competitive Intelligence Swarm"](#-showcase-mission-competitive-intelligence-swarm)
- [🏗️ Architecture Quick Reference](#-architecture-quick-reference)

---

## 🛡️ Sovereign Configuration (The Zero-Secrets Handshake)

To ensure your instance of Tadpole OS remains private and sovereign, you must supply your own API keys. **Never commit your keys to version control.**

### 1. Initialize Your Environment
1. Copy the template: `cp .env.example .env`
2. Open `.env` and generate a unique `NEURAL_TOKEN` (e.g., `openssl rand -hex 32`). This token secures the connection between your browser and the engine.

### 2. Supply Your Provider Keys
Add your keys to the following variables in `.env`:
- **Google Gemini**: `GOOGLE_API_KEY` ([AI Studio](https://aistudio.google.com/))
- **Anthropic Claude**: `ANTHROPIC_API_KEY` ([Anthropic Console](https://console.anthropic.com/))
- **OpenAI GPT**: `OPENAI_API_KEY` ([OpenAI Platform](https://platform.openai.com/))
- **Groq**: `GROQ_API_KEY` ([Groq Cloud](https://console.groq.com/))

### 3. Local-First (Zero Cost) Option
If you prefer not to use external APIs, install **Ollama** and set `PRIVACY_MODE=true`. This forces the engine to use local models for all reasoning tasks.

> [!IMPORTANT]
> Your keys are only stored in `.env`. When you save provider configurations in the UI, the engine automatically sanitizes them and only stores metadata (URLs, model names) in the repo-committed JSON files.

---

## 🏗️ Hardware Requirements (Scaling Spec)

Tadpole OS is optimized for low-footprint Rust execution. Requirements scale linearly with agent count and mission complexity.

| Tier | Agents | Clusters | **Min RAM** | **vCPU** | Deployment |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Micro (Demo)** | 1-2 | 1 | **1 GB** | 1 | Hybrid |
| **Standard (Bunker)** | 2-9 | 1-2 | **2 GB** | 2 | Hybrid |
| **Cluster Max** | 10-25 | 4+ | **4 GB** | 4 | Hybrid |
| **ROBUST (PRO)** | 25+ | Full | **8 GB+** | 4-8 | Full Remote |

> [!TIP]
> **Robust Recommendation**: For high-vocal missions with real-time audio and massive `fetch_url` research, an **8GB / 4-vCPU** instance ensures zero latency in the context bus and allows for full remote rebuilds without OOMs.

---

## Step 1: Connect to the Sovereign Dashboard

1. Open Tadpole OS in your browser:
   - **Local Dev**: `http://localhost:5173` (Vite 6 + React 19)
   - **Production (Bunker)**: `http://<bunker-ip>:8000` (Axum 0.8)
2. Go to **⚙️ System Configuration** from the sidebar.
3. Under **Engine Connection**, verify the URL (**TadpoleOSUrl**) is set to your engine endpoint and the **Neural Engine Access Token** (formerly Neural Token) matches your `.env` value.
4. Click **Save Changes** — the dashboard auto-reconnects immediately using the **Lazy Singleton Socket** protocol.
5. In the **Multi-Tab Bar**, you can now open additional operational contexts (Missions, Hierarchy, etc.) without losing your current view.
6. The top-tier **PageHeader** should show **🟢 ONLINE** and display real-time engine telemetry.

> [!TIP]
> **Multi-Monitor Setup**: Tadpole OS supports **State-Preserved Detachment**. Click the **External Link** icon on any tab or high-fidelity components like the **Swarm Pulse Visualizer** to "pop out" that context into a dedicated portal window. This uses a shared JS heap for zero-latency cross-window synchronization.

> **Development Note**: For reliable database persistence on Windows, ensure `DATABASE_URL` is an **absolute path** (e.g., `sqlite:D:\TadpoleOS-Dev\tadpole.db`).

---

## Step 2: Unlock the Neural Vault & Add Your Groq Provider

The Neural Vault is an encrypted vault that stores your API keys. You must unlock it before configuring providers.

1. Go to **🧠 AI Provider Manager** from the sidebar
2. You'll see the **NEURAL VAULT** lock screen
3. Enter a master password (this encrypts your keys locally) → click **Commit Authorization**
4. You'll now see the **Provider Cards** section

### Adding Groq as a Provider

5. If Groq is already listed, click the **Edit** (pencil) icon on the Groq card
6. If not listed, click **+ ADD PROVIDER** at the bottom:
   - **Name**: `Groq`
   - **Icon**: `⚡` (or any emoji)
   - Click **Create**
7. On the Groq provider card:
   - **API Key**: Paste your Groq API key
   - **Base URL**: `https://api.groq.com/openai/v1`
   - **Protocol**: `OpenAI (OpenRT)` — Groq uses OpenAI-compatible API
8. Click **Save** on the provider card.
9. **Test Trace (Handshake)**: Click the "Test Trace" button to perform a real-time connectivity handshake. This verifies your API Key, Endpoint, and Protocol are valid before deployment.

> [!TIP]
> The vault auto-locks after inactivity. Your key is encrypted with your master password and stored in the browser — it never leaves your machine.

---

## Step 3: Deployment with Sovereign Starter Kits (Optional)

For rapid SME deployment, Tadpole OS allows you to choose a **Sovereign Starter Kit** (Marketing, Customer Success, Finance). See the full [Starter Kits Guide](./STARTER_KITS.md) for the current built-in kits and install paths.

---

## Step 4: Local Intelligence (Local LLMs via Ollama)

1. Follow the [Qwen3.5-9B Local Integration Guide](./QWEN_LOCAL_INTEGRATION.md) for detailed setup.
2. Once configured, you can add local models similarly to the steps below.

---

## Step 5: Add Models to the Registry

Still on the **🧠 AI Provider Manager** page, scroll down to the **Model Registry** section.

1. Click **+ ADD MODEL**
2. Fill in:
   - **Model Name**: `llama-3.3-70b-versatile`
   - **Provider**: Select `Groq` from the dropdown
   - **RPM** *(optional)*: e.g., `30` — prevents exceeding Groq's free-tier rate limits
   - **TPM** *(optional)*: e.g., `14000` — the engine will throttle automatically
3. Click the **✓** checkmark to save

**Recommended models to add:**

| Model Name | Provider | Best For |
|------------|----------|----------|
| `llama-3.3-70b-versatile` | Groq | General tasks, tool calling |
| `llama-3.1-8b-instant` | Groq | Fast responses, simple tasks |
| `qwen3.5:9b` | Ollama | **Local Power**, high logic fidelity |

Repeat for each model you want available.

---

## Step 6: Configure an Agent Node

1. Go to **🏛️ Agent Hierarchy Layer** from the sidebar
2. You'll see the **Neural Command Hierarchy** — your agent org chart
3. Click on any agent card (e.g., **Nexus**, **Cipher**, etc.)
4. The **Agent Config Panel** slides open on the right

### In the Config Panel:

5. **Identity Section** (top):
   - **Name**: Give it a descriptive name (e.g., `Research Bot`)
   - **Role**: Select from the dropdown (e.g., `Researcher`, `Engineer`, `Analyst`)

6. **Cognition Tab (MCP Tools & Skills)**:
   - **Skills & Workflows**: Toggle standard skills like `web_search` or `code_execute`.
   - **MCP Tools**: Select external tools from the high-density grid. These are specifically designed for Model Context Protocol integration.
   - **Model Settings**: Configure model, provider, and temperature for each slot.

7. **Voice & Governance Tabs**:
   - **Voice**: Configure TTS/STT identity.
   - **Governance**: Toggle the **Requires Oversight** flag (Junior Agent mode) and set persistent USD budget caps for this specific node.

8. Click **💾 SAVE CONFIG** at the bottom.

> [!IMPORTANT]
> The save pushes your config to the Rust backend, so it persists across devices and restarts. You'll see **Capability Badges** appear on the agent's card in the Agent Manager, showing the count of assigned tools.

---

## Step 7: Bulk Capability Assignment (Skills Hub)

Instead of configuring agents one-by-one, you can assign tools to multiple agents simultaneously.

1. Go to **🛠️ Skills & Workflows** from the sidebar.
2. Select any **Skill**, **Workflow**, or **MCP Tool**.
3. Click the **"Assign to Agents"** button in the details panel.
4. Select all agents you want to receive this capability.
5. Click **Commit Assignments**. The engine will bulk-sync the configurations live.

---

## Step 8: Importing External Capabilities (.md)

Tadpole OS allows you to rapidly build your agent's library by importing existing documentation.

1. Go to **🛠️ Skills & Workflows**.
2. Click the **"Import .md"** button in the header.
3. Select a `.md` file containing a skill or workflow definition.
4. Review the **Import Preview** to verify the parsed logic and ID.
5. Click **Confirm Import**. The capability is now available in your **User Registry** for assignment.

---

## Step 9: Create and Execute a Mission

1. Go to **🎯 Mission Management** from the sidebar
2. Click **+ NEW MISSION** in the top-right of the cluster sidebar
3. Fill in:
   - **Mission Name**: e.g., `Market Research Sprint`
   - **Department**: Select the relevant department (e.g., `Research`)
4. Click **Create**

### Assign Agents to the Mission:

5. Select your new mission in the sidebar (it'll highlight)
6. In the **Available Agents** pool on the right, click **+ Assign** next to each agent you want on this mission
226. **Hierarchical Recruitment**: High-level agents (Alphas) can recruit ephemeral sub-agents. The engine uses modular specialists from `runner/mission_tools.rs` to delegate tasks with strategic context handoffs.
227. **Parallel Swarming (PERF-06)**: **Tadpole OS** utilizes `FuturesUnordered` to parallelize tool calls. Recruitment of multiple specialists happens simultaneously, reducing swarm startup latency by up to 80%.
228. **Swarm Pulse Visualizer**: Toggle the "Neural Map" icon on the mission dashboard to see a real-time Force-Graph visualization of cluster connectivity, featuring 10Hz binary telemetry pulses.
229. **Neural Swarm Optimization**: When you type an objective, the engine proactively suggests mission-specific templates via the **Template Discovery Hub**. 
230. **Recursion Guard**: To prevent circular token-burn, the engine enforces a maximum **Swarm Depth of 5** (managed in `AppState`).
231. **Mission Analysis (Agent 99)**: Toggle the **"Analysis"** switch next to the Run button to trigger a post-mission debrief powered by LanceDB vector synthesis.

---

## Step 10: Workspace & Cluster Management

Tadpole OS allows you to organize your swarm into **Mission Clusters**.

- **Defaults**: The engine starts with **4 predefined clusters** (Strategic Command, Strategic Ops, Core Intelligence, Applied Growth).
- **Custom Scaling**: You can create new clusters or retire existing ones from the **🎯 Missions** page.
- **Persistence**: Agent roles and system configurations are stored in the backend **SQLite database** (`tadpole.db`). Logical mission clusters are managed by the frontend in **LocalStorage**.
- **Physical Sandboxes**: Each cluster maps to a dedicated directory in the backend `./workspaces/{clusterId}` folder, ensuring file isolation.

---

## Step 11: External Adapters & Workspace Tools

The engine can now connect to your local environment and external services.

### Workspace File Operations
Agents with matching skills can read and write files within their cluster sandbox:
- **`read_file`**: Read a file from the workspace (e.g., load a spec document).
- **`write_file`**: Write a file to the workspace (e.g., save generated code).
- **`list_files`**: List files in a workspace directory.
- **`delete_file`**: Delete a file *(requires Oversight Gate approval)*.

Files are stored under `workspaces/<cluster-id>/` on the server. Each cluster is fully isolated.

### Local Markdown Vault (Obsidian)
Enabled agents can now use the `archive_to_vault` tool.
1. Create a `vault/` directory in the `server-rs` root.
2. Agents will automatically append findings to files in this directory when requested.

### Discord Notifications
1. Add `DISCORD_WEBHOOK="your_webhook_url"` to your `.env` file.
2. Use the `notify_discord` tool from an agent to alert your team.

### Environment Security (.env)
Ensure your `.env` file in the root directory contains:

| Variable | Description | Requirement |
| :--- | :--- | :--- |
| `DATABASE_URL` | Path to `tadpole.db` | **Absolute path REQUIRED on Windows** (e.g., `D:\TadpoleOS-Dev\tadpole.db`) |
| `AUDIT_PRIVATE_KEY` | Ed25519 Private Key (Hex) | **REQUIRED for production**. Enables non-repudiation and tamper-evident logging. |
| `NEURAL_TOKEN` | Engine Access Token for WebSocket/API access | **Required in production** — engine panics if not set. |
| `MERKLE_AUDIT_ENABLED` | Toggle tamper-evident cryptographic logging | Default: `true` |
| `RESOURCE_GUARD_ENABLED` | Toggle real-time RAM/CPU monitoring | Default: `true` |
| `SANDBOX_AWARENESS` | Enable Docker/K8s detection status | Default: `true` |
| `LIFECYCLE_HOOKS_ENABLED` | Toggle pre/post execution hooks | Default: `true` |
| `ANTHROPIC_API_KEY` | Claude Provider Key | No |
| `OPENAI_API_KEY` | OpenAI Provider Key | No |
| `OLLAMA_HOST` | Local LLM Endpoint | Default: `http://localhost:11434` |
| `DISCORD_WEBHOOK` | Discord notification URL | Required only for `notify_discord` tool |
| `TADPOLE_NULL_PROVIDERS`| Forces graceful provider degradation | Dev/Test only. |
| `SME_SYNC_INTERVAL_MINS`| Ingestion Worker sync frequency (minutes) | Default: `30` |

### Standardized Observability (HATEOAS)
All resource endpoints in Tadpole OS implement the **HATEOAS** pattern. Responses include a `_links` object, enabling self-discovery of related actions. Error responses strictly follow **RFC 9457 (Problem Details)** for consistent machine-readable debugging.

---

## Step 12: SME Data Intelligence (Connectors & Workflows)

Tadpole OS includes a 4-phase data intelligence layer for SME onboarding.

### Phase 1: Hybrid RAG
The Neural Memory engine (`memory.rs`) automatically combines vector similarity with keyword proximity scoring for higher-fidelity context retrieval. This is transparent — no configuration required.

### Phase 2: Data Connectors (Background Sync)
1. Go to **🏛️ Agent Hierarchy Layer** → select an agent → open the **Memory** tab.
2. In the **Connector Config** section, click **+ Add Source**.
3. Set **Type** to `fs` (file system) and **URI** to the directory to watch (e.g., `/data/business-docs/`).
4. Click **Save**. The **Ingestion Worker** will begin crawling this directory at the interval set by `SME_SYNC_INTERVAL_MINS`.
5. Monitor sync status (idle/syncing/error) in the **Memory Section** UI.

> [!TIP]
> The Ingestion Worker uses a **SyncManifest** to track file modification times. Only new or changed files are re-embedded, minimizing compute costs.

### Phase 3: Deterministic SOP Workflows
1. Create a markdown file in `data/workflows/` on the server (e.g., `data/workflows/onboarding.md`).
2. Format it with numbered steps — each step becomes a discrete agent turn.
3. Assign the workflow to an agent via the **Cognition** tab in the Agent Config Panel.
4. When the agent receives a mission, the **SOP Engine** will execute each step in guaranteed order.

### Phase 4: Document Parsing
The Data Connectors automatically use the **Layout-Aware Parser** (`parser.rs`) for all ingested files. Supported formats: `.txt`, `.md`, `.csv`, `.pdf` (text-layer). Documents are chunked with 25% overlap for optimal embedding quality.

---

## Step 13: Send a Task & Get Results

### Option A: From the Terminal Bar

The terminal bar is at the bottom of every page.

1. Click the terminal input field
2. Type a command:
   ```
   /send Research Bot Analyze the top 3 competitors in the AI agent space and summarize their pricing models
   ```
   Format: `/send <agent-name> <your task message>`
3. Press **Enter**
4. Watch the **System Log** on the dashboard — you'll see:
   - `📡 Task dispatched to Research Bot`
   - Live agent status updates
   - The final response from the LLM

### Option B: Command Palette (Global Nav)

1. Press **`Cmd+K`** (Mac) or **`Ctrl+K`** (Windows) anywhere.
2. Search for an Agent, Cluster, or Directive.
3. Select an agent to instantly focus them in the chat interface.

### Option C: From the OPS Dashboard

1. Go to the **Dashboard** (home page)
2. The **Live Agent Status** cards show real-time activity
3. The **System Log** captures all responses and events.
4. **Discover Nodes**: Click the **"Discover Nodes"** button in the Infra section to scan your local network for secondary Bunker nodes. Discovered nodes will automatically appear in your dashboard for unified oversight.

### Option D: Neural Sync (Voice-to-Swarm)

1. Go to **🎙️ Voice Interface** from the sidebar
2. **Select Target**: Choose an Agent (e.g., Agent of Nine) or a Mission Cluster.
3. Click **Start Sync** → Speak your high-level objective clearly.
4. **Hands-Free Response**: The speaker icon activates automatically. The agent (typically Agent of Nine) will transcribe your intent via **Groq Whisper** and then synthesize a strategic confirmation back to you via **OpenAI TTS**.
5. Click **End Sync** once the verbal handshake is complete.

---

## Useful Terminal Commands

| Command | What it does |
|---------|-------------|
| `/send <agent> <message>` | Send a task to a specific agent |
| `/pause <agent>` | Pause a running agent |
| `/resume <agent>` | Resume a paused agent |
| `/status` | Show all agent statuses |
| `/swarm status` | Inventory mission clusters |
| `/clear` | Clear the system log |

### 🛠️ Maintenance & Integrity (Python Environment)

For advanced users and AI agents, the `execution/` directory contains standardized tools for maintaining the "Intelligence Grade" of the codebase:

| Script | Purpose | Protocol |
| :--- | :--- | :--- |
| `python execution/verify_all.py` | **Full System Audit** | Performs pre-flight checks on engine & services |
| `python execution/parity_guard.py` | **Integrity Gate** | Ensures documentation matches backend routing |
| `python execution/scout.py` | **System Search** | High-fidelity recursive search with relative pathing |

> [!NOTE]
> All scripts in `execution/` follow a strict **`[OK]` / `[FAIL]`** machine-readable reporting protocol for autonomous agents.

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Dashboard shows **OFFLINE** | Check that the engine is running (`npm run engine` or Docker container is up) |
| Agent returns no response | Verify the model exists in the Model Registry and the API key is valid |
| Neural Vault won't unlock | The vault creates a new encryption key on first use — use any password. If locked out, use **Emergency Vault Reset** at the bottom of the unlock screen. |
| Model dropdown is empty | Go to **🧠 AI Provider Manager** → unlock vault → add models to the registry |
| Agent config doesn't save | Check browser console — engine must be online for persistence to work |
| Tool-Calling fails (Groq) | The engine includes **Self-Healing Retries** for Groq. Malformed tool syntax is automatically corrected in a second pass. |
| Agent is slow / rate limited | The engine enforces `rpm`/`tpm` limits set on the model. The agent will wait for the quota window to reset rather than drop requests. |
| `NEURAL_TOKEN` panic on start | A `NEURAL_TOKEN` env var is required for the engine to start. Set it in your `.env` file and make the dashboard token match it. |
| Workspace file access denied | Agent tried to access a path outside its sandbox. Check `cluster_id` mapping and ensure no path traversal in the filename. |

---

## 🐸 Starter Swarm Configuration (Quick-Deploy)

This section provides a **ready-to-use 3-agent swarm** with concrete settings optimized for Groq's free tier. Follow this to have a working hierarchical swarm in under 5 minutes.

### Agent Roster

| Agent | ID | Role | Model | Provider | Temperature | Budget | Skills |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Agent of Nine** | `1` | CEO | `llama-3.3-70b-versatile` | Groq | `0.8` | `$2.00` | `issue_alpha_directive`, `web_search` |
| **Tadpole** | `2` | COO (Alpha) | `llama-3.3-70b-versatile` | Groq | `0.6` | `$3.00` | `web_search`, `write_file`, `read_file`, `spawn_subagent` |
| **Elon** | `3` | CTO (Specialist) | `llama-3.3-70b-versatile` | Groq | `0.3` | `$1.00` | `code_execute`, `write_file`, `read_file`, `list_files` |

> [!TIP]
> **Why these settings?**
> - **Agent of Nine** uses high temperature (`0.8`) for creative strategic thinking and delegation.
> - **Tadpole** (Alpha) gets medium temperature (`0.6`) for balanced coordination and the `spawn_subagent` skill for recruitment.
> - **Elon** (Specialist) runs a fast, cheap model at low temperature (`0.3`) for precise code execution.
> - Budget caps prevent runaway token spend on Groq's free tier.

### Rate Limits (Groq Free Tier Safe)

Set these in the **🧠 Providers → Model Registry**:

| Model | RPM | TPM |
| :--- | :--- | :--- |
| `llama-3.3-70b-versatile` | `30` | `14000` |
| `llama-3.1-8b-instant` | `30` | `14000` |

### Applying via the UI

1. Go to **🏛️ Agent Hierarchy Layer** → click each agent card
2. Set the **Model**, **Temperature**, and **Budget** as shown above
3. Expand **Skills & Workflows** → toggle the listed skills for each agent
4. Click **💾 SAVE CONFIG** for each agent

### Applying via API (curl)

If you prefer programmatic setup, here are the exact payloads:

**Configure Agent of Nine (CEO):**
```bash
curl -X PUT http://localhost:8000/v1/agents/1 \
  -H "Authorization: Bearer YOUR_NEURAL_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model_id": "llama-3.3-70b-versatile",
    "provider": "groq",
    "temperature": 0.8,
    "budget_usd": 2.0,
    "skills": ["issue_alpha_directive", "web_search"]
  }'
```

**Configure Tadpole (Alpha/COO):**
```bash
curl -X PUT http://localhost:8000/v1/agents/2 \
  -H "Authorization: Bearer YOUR_NEURAL_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model_id": "llama-3.3-70b-versatile",
    "provider": "groq",
    "temperature": 0.6,
    "budget_usd": 3.0,
    "skills": ["web_search", "write_file", "read_file", "spawn_subagent"]
  }'
```

**Configure Elon (Specialist/CTO):**
```bash
curl -X PUT http://localhost:8000/v1/agents/3 \
  -H "Authorization: Bearer YOUR_NEURAL_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model_id": "llama-3.1-8b-instant",
    "provider": "groq",
    "temperature": 0.3,
    "budget_usd": 1.0,
    "skills": ["code_execute", "write_file", "read_file", "list_files"]
  }'
```

> [!NOTE]
> Replace `YOUR_NEURAL_TOKEN` with the value from your `.env` file. There is no built-in development token anymore.

---

## 🎯 Showcase Mission: "Competitive Intelligence Swarm"

This mission demonstrates the full power of hierarchical swarming — strategic delegation, parallel research, file collaboration, and synthesis. Run this after applying the Starter Swarm configuration above.

### Mission Overview

```
┌──────────────────────────────────────────────────────────┐
│  YOU (Overlord)                                          │
│  "Analyze the top 3 AI agent frameworks and write        │
│   a competitive brief with code comparison."             │
└─────────────────────┬────────────────────────────────────┘
                      │ Neural Handoff
┌─────────────────────▼────────────────────────────────────┐
│  Agent of Nine (CEO) — Depth 0                           │
│  Refines intent → issues alpha directive to Tadpole      │
└─────────────────────┬────────────────────────────────────┘
                      │ issue_alpha_directive
┌─────────────────────▼────────────────────────────────────┐
│  Tadpole (Alpha/COO) — Depth 1                           │
│  Decomposes into parallel research tasks                 │
│  ├─ spawn_subagent("researcher_a") → CrewAI analysis     │
│  ├─ spawn_subagent("researcher_b") → AutoGen analysis    │
│  └─ Assigns Elon to code comparison                      │
└──────┬──────────────┬───────────────┬────────────────────┘
       │              │               │ (Parallel)
   ┌───▼───┐    ┌─────▼─────┐   ┌────▼─────────────────┐
   │Rsrchr A│    │ Rsrchr B  │   │  Elon (CTO) — D2     │
   │CrewAI  │    │ AutoGen   │   │  code_execute +       │
   │research│    │ research  │   │  write_file           │
   └───┬────┘    └─────┬─────┘   └────┬─────────────────┘
       │               │              │
       └───────────────┼──────────────┘
                       │ Results flow up
┌──────────────────────▼───────────────────────────────────┐
│  Tadpole (Alpha) — Synthesis                             │
│  Merges all findings → write_file("competitive_brief.md")│
└──────────────────────────────────────────────────────────┘
```

### Step 1: Dispatch the Mission

**From the Terminal Bar:**
```
/send Agent of Nine Analyze the top 3 AI agent frameworks (CrewAI, AutoGen, LangGraph). For each, research their architecture, pricing, and developer experience. Then have our CTO write a Python code comparison showing how each framework defines a simple 2-agent team. Synthesize everything into a competitive_brief.md in our workspace.
```

**Or via curl:**
```bash
curl -X POST http://localhost:8000/v1/agents/1/tasks \
  -H "Authorization: Bearer YOUR_NEURAL_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Analyze the top 3 AI agent frameworks (CrewAI, AutoGen, LangGraph). For each, research their architecture, pricing, and developer experience. Then have our CTO write a Python code comparison showing how each framework defines a simple 2-agent team. Synthesize everything into a competitive_brief.md in our workspace.",
    "provider": "groq",
    "model_id": "llama-3.3-70b-versatile",
    "budget_usd": 2.0
  }'
```

### Step 2: Watch the Swarm Execute

Open these dashboard views to observe the swarm in real-time:

| View | What You'll See |
| :--- | :--- |
| **🏛️ Hierarchy** | Agent status lights change: `idle` → `thinking` → `active` as each node activates |
| **🎯 Missions** | The cluster sidebar shows task assignments and handoff chains |
| **📊 OPS Dashboard** | Live token burn, cost tracking, and the System Log streaming agent outputs |
| **🔒 Oversight** | If `write_file` or `delete_file` triggers, you'll see approval requests here |

### What Happens Under the Hood

1. **Agent of Nine** receives the prompt, applies strategic reasoning, and fires `issue_alpha_directive` to Tadpole with a refined, tactical breakdown.
2. **Tadpole** decomposes the directive into 3 parallel tasks:
   - Spawns **Researcher A** (ephemeral) → searches for CrewAI architecture and pricing
   - Spawns **Researcher B** (ephemeral) → searches for AutoGen architecture and pricing
   - Sends a direct task to **Elon** → write Python code comparing all 3 frameworks
3. **Parallel Swarming (PERF-06)** kicks in — all 3 sub-tasks execute concurrently via `FuturesUnordered`.
4. As results flow back, Tadpole's **synthesis turn** merges them and calls `write_file("competitive_brief.md")` to save the final deliverable.
5. Cost and token metrics are tracked per-agent in real-time on the OPS Dashboard.

### Expected Output

After ~30-60 seconds (depending on Groq load), you'll find:
- `workspaces/<cluster-id>/competitive_brief.md` — The final synthesized report
- **System Log** entries showing the full delegation chain with swarm lineage breadcrumbs
- **Per-agent cost breakdown** on each hierarchy node card

### Scaling This Pattern

| Adjustment | How |
| :--- | :--- |
| **Add more researchers** | Give Tadpole more budget and increase `swarmDepth` |
| **Use multiple providers** | Assign Claude to Agent of Nine, Groq to specialists |
| **Enable voice dispatch** | Use **🎙️ Standups → Neural Sync** instead of typing |
| **Auto-approve safe tools** | Set `autoApproveSafeSkills: true` in Oversight Settings |
| **Save as a template** | Use **"Promote to Role"** on your configured agents |

---

## Step 14: Performance Analysis & Real-time Telemetry

Tadpole OS provides "Top Tier" observability into swarm health and technical performance.

### 1. Real-time Telemetry (Swarm Visualizer)
The **Engine Dashboard** features a high-performance **Swarm Visualizer** (God View):
- **Binary Swarm Pulse**: Driven by a 10Hz MessagePack stream (`0x02` header) for sub-millisecond state parity.
- **Topology Map**: Visualizes the swarm as a 2D force-graph, showing agent status and recruitment relationships.
- **Detach & Recall**: Pop the visualizer into a dedicated window for persistent oversight during deep-context missions.
- **Fiscal Burn**: Real-time USD/token tracking via the **TPM** indicator.
- **Swarm Density**: Monitor agent instantiation relative to system capacity.

### 2. Performance Analysis (Benchmarks)
1. Go to **📊 Performance Analysis** from the sidebar.
2. **Timeline View**: Review historical benchmark results (latency, throughput, status).
3. **Comparison Tool**: Select any two runs to calculate performance deltas. 
   - *Example*: Compare "Current" vs "Baseline" to identify code regressions or provider latency spikes.
4. **Target Enforcement**: Metrics are color-coded against the technical specifications in `Benchmark_Spec.md`.

---

## Step 15: The Swarm Template Ecosystem

Instead of manually configuring agents, Tadpole OS allows you to instantly download full, industry-specific agent swarms.

1. Go to **⚙️ System Configuration** from the sidebar.
2. Scroll down to the **Template Ecosystem** panel and click **Open Template Store**.
3. **Discover**: Browse the store or use the fuzzy search to find templates tailored to your industry (e.g., "Legal Contract Review", "Healthcare Patient Intake").
4. **Install**: Click **Install Swarm**. The engine uses native **Git Cloning** to securely fetch the template from the central Tadpole repository and unpacks it into your `data/swarm_config` directory.
5. **Sapphire Shield Approval**: If the downloaded template requests powerful execution skills (like Shell Access or API payments), the engine will freeze initialization and require you (the Overlord) to manually approve the skills, ensuring Zero-Trust security.

Once installed, the Rust engine "hot-loads" the template from your local `/data/swarm_config/` directory, and your new specialized agents will immediately appear in the **🏛️ Hierarchy**.

---

## Architecture Quick Reference

```
┌─────────────────────────────────────┐
│         Your Browser (React)        │
│  Dashboard │ Hierarchy │ Missions   │
│  Providers │ Oversight │ Settings   │
└───────────────┬─────────────────────┘
                │ HTTP + WebSocket
┌───────────────▼─────────────────────┐
│       Svc->>Rust: POST /v1/agents   │
│  Agent Registry │ Task Router       │
│  Oversight Gate │ Persistence       │
└───────────────┬─────────────────────┘
                │ API Calls
┌───────────────▼─────────────────────┐
│        LLM Provider (Groq)          │
│  llama-3.3-70b │ mixtral-8x7b      │
└─────────────────────────────────────┘
```

