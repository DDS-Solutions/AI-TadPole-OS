> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / GETTING_STARTED
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🚀 Tadpole OS — Getting Started Guide

> **Intelligence Level**: Configuration: Standard Business Setup
> **Status**: Setup guide; feature availability depends on the selected build and configuration
> **Version**: 1.1.464
> **Last Hardened**: 2026-08-19
> **Classification**: Sovereign

---

## 📚 Table of Contents

- [🖥️ Desktop App (Fastest Start)](#️-desktop-app-fastest-start)
- [🛡️ Secure Configuration (The Zero-Secrets Handshake)](#-secure-configuration-the-zero-secrets-handshake)
- [🏗️ Hardware Requirements (Scaling Spec)](#-hardware-requirements-scaling-spec)
- [Step 1: Connect to the Operations Dashboard](#step-1-connect-to-the-operations-dashboard)
- [Step 2: Unlock the Neural Vault & Add Your Groq Provider](#step-2-unlock-the-neural-vault--add-your-groq-provider)
- [Step 3: Deployment with Business Starter Kits](#step-3-deployment-with-business-starter-kits-optional)
- [Step 4: Local Intelligence (Local LLMs via Ollama)](#step-4-local-intelligence-local-llms-via-ollama)
- [Step 5: Add Models to the Registry](#step-5-add-models-to-the-registry)
- [Step 6: Configure an Agent Node](#step-6-configure-an-agent-node)
- [Step 7: Send Your First Task & Get Results (Fast Win! ⚡)](#step-7-send-your-first-task--get-results-fast-win-)
- [Step 8: Bulk Capability Assignment (Skills Hub)](#step-8-bulk-capability-assignment-skills-hub)
- [Step 9: Importing External Capabilities (.md)](#step-9-importing-external-capabilities-md)
- [Step 10: Create and Execute a Mission Cluster](#step-10-create-and-execute-a-mission-cluster)
- [Step 11: Workspace Sandboxing & Cluster Management](#step-11-workspace-sandboxing--cluster-management)
- [Step 12: External Adapters & Workspace Tools](#step-12-external-adapters--workspace-tools)
- [Step 13: SME Data Intelligence (Connectors & SOP Workflows)](#step-13-sme-data-intelligence-connectors--sop-workflows)
- [Step 14: Performance Analysis & Real-time Telemetry](#step-14-performance-analysis--real-time-telemetry)
- [Step 15: The Swarm Template Ecosystem](#step-15-the-swarm-template-ecosystem)
- [🛠️ Useful Terminal Commands](#-useful-terminal-commands)
- [🧩 Troubleshooting](#-troubleshooting)
- [🐸 Starter Swarm Configuration (Quick-Deploy)](#-starter-swarm-configuration-quick-deploy)
- [🎯 Showcase Mission: "Competitive Intelligence Swarm"](#-showcase-mission-competitive-intelligence-swarm)
- [🏘️ Swarm Topology Guide](#-swarm-topology-guide-how-many-bunkers-do-you-need)
- [🏗️ Architecture Quick Reference](#-architecture-quick-reference)

---

> [!IMPORTANT]
> **Product Release Classification: Sovereign Developer Engine (Plane 1 GA)**
> Tadpole OS `v1.1` is released as a **Sovereign Local Multi-Agent Swarm Engine for Developers & Engineers**.
> - **Plane 1 (Active & Production Ready)**: Full sovereign codebase lifecycle engine, local Axum + Tokio backend, LanceDB vector memory, AST Code Graphs, and local multi-agent swarms.
> - **Plane 2 (Developer Sandbox & Scenario Preview)**: Company-agnostic SMB virtual employees (Shopify, QuickBooks, Gmail) are provided as a **Developer Sandbox & Scenario Preview**. In accordance with `FR-11` security policy and [`docs/specs/mock-smb-digital-twin.md`](./specs/mock-smb-digital-twin.md), live customer data and live mutating connectors are fail-closed until multi-tenant backend isolation is deployed.

## 🖥️ Desktop App (Fastest Start)

No Node.js, Rust, or build toolchain required. Download the pre-compiled native desktop bundle for your platform:

| Platform | Download | Notes |
| :--- | :--- | :--- |
| **Windows** | [⬇️ Windows Installer (.exe)](https://github.com/DDS-Solutions/AI-TadPole-OS/releases/latest) | NSIS installer + standalone portable `.exe` |
| **macOS Universal** | [🍏 macOS Universal (.dmg)](https://github.com/DDS-Solutions/AI-TadPole-OS/releases/latest) | Native slices for **Apple Silicon (M1–M4)** and **Intel x86_64** in one fat binary |
| **Linux** | [🐧 Linux (.deb / .AppImage)](https://github.com/DDS-Solutions/AI-TadPole-OS/releases/latest) | Debian package or universal AppImage |

> [!NOTE]
> **macOS Gatekeeper**: The macOS build is ad-hoc signed without notarization. System Settings → Privacy & Security may require approval. For a trusted local build:
> ```bash
> xattr -cr /Applications/AI-Tadpole-OS.app
> ```
> **Persistent data** (database, neural token, logs) defaults to `~/Library/Application Support/com.tadpole.os/`. Set an absolute `TADPOLE_DATA_DIR` before launch to override it. Unavailable or invalid directories stop startup.

> [!TIP]
> The universal target includes Apple Silicon and Intel slices. Check the [native verification status](./BLUEPRINT-universal-macos-build.md); local configuration tests do not establish that an installed package has passed on both architectures.

---

## 🛡️ Secure Configuration (The Zero-Secrets Handshake)

To ensure your instance of Tadpole OS remains private and sovereign, you must supply your own API keys. **Never commit your keys to version control.**

### 1. Initialize Your Environment
1. Copy the template: `cp .env.example .env`
2. Open `.env` and generate a unique `NEURAL_TOKEN` (e.g., `openssl rand -hex 32`). This token secures the connection between your browser and the engine.

### 2. Complete Environment Security Variable Reference

| Variable | Description | Requirement / Default |
| :--- | :--- | :--- |
| `DATABASE_URL` | Path to the SQLite database | Optional. If unset, the engine uses `data/state/tadpole.db`; if only the legacy `data/tadpole.db` exists, startup copies it and its WAL/SHM sidecars to the canonical path. To set it explicitly, use `sqlite:data/state/tadpole.db` or an absolute path. |
| `NEURAL_TOKEN` | Engine Access Token for WebSocket/API access | **Required** — engine panics if neither `NEURAL_TOKEN` nor `NEURAL_ENGINE_ACCESS_TOKEN` is set. |
| `NEURAL_ENGINE_ACCESS_TOKEN` | Alias for `NEURAL_TOKEN` | Can be used instead of `NEURAL_TOKEN`. Engine checks this first. |
| `VITE_NEURAL_TOKEN` | Frontend token (Vite env var) | Set to match `NEURAL_TOKEN` so the frontend auto-configures. |
| `AUDIT_PRIVATE_KEY` | Ed25519 Private Key (Hex) | **Recommended for production**. Enables non-repudiation and tamper-evident Merkle logging. |
| `ALLOWED_ORIGINS` | Comma-separated CORS origins | Default: `http://localhost:5173` |
| `MAX_SWARM_DEPTH` | Maximum agent recursion/spawn depth | Default: `5` |
| `MAX_AGENTS` | Maximum agents in the registry | Default: `50` |
| `MAX_CLUSTERS` | Maximum active clusters | Default: `10` |
| `GOOGLE_API_KEY` | Google Gemini Provider Key | Optional ([AI Studio](https://aistudio.google.com/)) |
| `ANTHROPIC_API_KEY` | Anthropic Claude Provider Key | Optional ([Anthropic Console](https://console.anthropic.com/)) |
| `OPENAI_API_KEY` | OpenAI GPT Provider Key | Optional ([OpenAI Platform](https://platform.openai.com/)) |
| `GROQ_API_KEY` | Groq Provider Key | Optional ([Groq Cloud](https://console.groq.com/)) |
| `DEEPSEEK_API_KEY` | DeepSeek Provider Key | Optional ([DeepSeek Platform](https://platform.deepseek.com/)) |
| `OLLAMA_HOST` | Local LLM Endpoint | Default: `http://localhost:11434` |
| `DISCORD_WEBHOOK` | Discord notification URL | Required only for `notify_discord` tool |
| `PRIVACY_MODE` | Block all cloud providers, enforce local-only | Default: `false` |
| `AUTO_APPROVE_SAFE_SKILLS` | Auto-approve tools tagged as "safe" | Default: `false` |
| `SME_SYNC_INTERVAL_MINS` | Ingestion Worker sync frequency (minutes) | Default: `30` |

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
| **Standard (Office PC)** | 2-9 | 1-2 | **2 GB** | 2 | Hybrid |
| **Cluster Max** | 10-25 | 4+ | **4 GB** | 4 | Hybrid |
| **ROBUST (PRO)** | 25+ | Full | **8 GB+** | 4-8 | Full Remote |

> [!TIP]
> **Robust Recommendation**: For high-vocal missions with real-time audio and massive `fetch_url` research, an **8GB / 4-vCPU** instance ensures zero latency in the context bus and allows for full remote rebuilds without OOMs.

---

## Step 1: Connect to the Operations Dashboard

1. Open Tadpole OS in your browser:
   - **Local Dev**: `http://localhost:5173` (Vite 8 / React 19)
   - **Production (Local Server)**: `http://<local-ip>:8000` (Axum)
2. Go to **⚙️ Settings** from the sidebar (bottom-left gear icon).
3. Under **Engine Connection**, verify the **TadpoleOS URL** is set to your engine endpoint and the **Neural Engine Access Token** matches your `.env` value.
4. Click **Save Changes** — the dashboard auto-reconnects immediately using the **Lazy Singleton Socket** protocol.
5. In the **Tab Bar**, you can now open additional operational contexts (Missions, Hierarchy, etc.) without losing your current view.
6. The top **Ticker Header** should show **🟢 ONLINE** and display real-time engine telemetry.

> [!TIP]
> **Multi-Monitor Setup**: Tadpole OS supports **State-Preserved Detachment**. Click the **External Link** icon on any tab to "pop out" that context into a dedicated portal window. This uses a shared JS heap for zero-latency cross-window synchronization.

> **Development Note**: If `DATABASE_URL` is unset, the engine uses `data/state/tadpole.db` relative to the engine base directory. An existing legacy `data/tadpole.db` is copied there at startup when the canonical file is absent. To choose a path explicitly, use `sqlite:data/state/tadpole.db` or an absolute SQLite URL.

---

## Step 2: Unlock the Neural Vault & Add Your Groq Provider

The Neural Vault is an encrypted vault that stores your API keys. You must unlock it before configuring providers.

> [!NOTE]
> **UI Architecture Logic Flow**:
> `Provider (The Door)` ➔ `Model (The Person Inside)` ➔ `Agent (The Job Role)`

1. Go to **Providers** from the sidebar (under the Intelligence section)
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
> The vault stores your key encrypted in browser storage. When you dispatch a task, the browser sends the decrypted key to the configured Tadpole OS engine, which uses it for the selected provider. Use a trusted engine endpoint and HTTPS when connecting remotely.

---

## Step 3: Deployment with Business Starter Kits (Optional)

For rapid SME deployment, Tadpole OS allows you to choose a **Business Starter Kit** (Marketing, Customer Success, Finance). See the full [Starter Kits Guide](./STARTER_KITS.md) for the current built-in kits and install paths.

---

## Step 4: Local Intelligence (Local LLMs via Ollama)

1. Follow the [Qwen3.5-9B Local Integration Guide](./QWEN_LOCAL_INTEGRATION.md) for detailed setup.
2. Once configured, you can add local models similarly to the steps below.

---

## Step 5: Add Models to the Registry

Still on the **Providers** page, scroll down to the **Model Registry** section.

1. Click **+ ADD MODEL**
2. Fill in:
   - **Model Name**: `llama-3.3-70b-versatile`
   - **Provider**: Select `Groq` from the dropdown
   - **RPM** *(optional)*: e.g., `30` — prevents exceeding Groq's free-tier rate limits
   - **TPM** *(optional)*: e.g., `14000` — the engine will throttle automatically
   - **# / Context Field** *(optional)*: e.g., `128000` — sets token context capacity
   - **$/M input tokens** *(optional)*: e.g., `0.59` — custom cost per million prompt tokens
   - **$ /M output token** *(optional)*: e.g., `0.79` — custom cost per million completion tokens
3. Click the **✓** checkmark to save
   *(Tip: Use the **SHOW RATE LIMITS & PRICING** button on any inventory card to inspect or edit these values at any time).*

> [!NOTE]
> **Groq Model Catalog Advisory**:
> Groq officially decommissioned free/developer-tier endpoints for `llama-3.3-70b-versatile` and `llama-3.1-8b-instant`. To check active models currently supported on your Groq API key, run:
> ```bash
> curl -X GET "https://api.groq.com/openai/v1/models" -H "Authorization: Bearer $GROQ_API_KEY"
> ```
> Recommended active cloud models include `openai/gpt-oss-120b`, `qwen/qwen3.6-27b`, or local zero-cloud endpoints (`qwen3.5:9b`, `gemma4:e4b`).

**Recommended models to add:**

| Model Name | Provider | Best For |
|------------|----------|----------|
| `openai/gpt-oss-120b` | Groq | High-performance general tasks, tool calling |
| `qwen/qwen3.6-27b` | Groq | Fast responses, code reasoning |
| `qwen3.5:9b` | Ollama | **Local Power**, high logic fidelity, zero cloud egress |
| `gemma4:e4b` | Ollama | **Lightweight Local**, fast summarization |

Repeat for each model you want available.

---

## Step 6: Configure an Agent Node

1. Go to **Hierarchy** from the sidebar
2. You'll see the **Neural Command Hierarchy** — your agent org chart
3. Click on any agent card (e.g., **Agent of Nine**, **Tadpole**, etc.)
4. The **Agent Config Panel** slides open on the right

### In the Config Panel:

5. **Identity Section** (top):
   - **Name**: Give it a descriptive name (e.g., `Research Bot`)
   - **Role**: Select from the dropdown (e.g., `Researcher`, `Engineer`, `Analyst`)

6. **Cognition Tab (MCP Tools & Skills)**:
   - **Skills & Workflows**: Toggle standard skills like `web_search` or `execute_shell`.
   - **MCP Tools**: Select external tools from the high-density grid. These are specifically designed for Model Context Protocol integration.
   - **Model Settings**: Configure model, provider, and temperature for each slot (agents support up to 3 model slots).

7. **Voice & Governance Tabs**:
   - **Voice**: Configure TTS/STT identity and voice engine (supports `gemini-live` and standard OpenAI TTS).
   - **Governance**: Toggle the **Requires Oversight** flag (Junior Agent mode) and set persistent USD budget caps for this specific node.

8. Click **💾 SAVE CONFIG** at the bottom.

> [!IMPORTANT]
> The save pushes your config to the Rust backend, so it persists across devices and restarts. You'll see **Capability Badges** appear on the agent's card in the Hierarchy view, showing the count of assigned tools.

---

## Step 7: Send Your First Task & Get Results (Fast Win! ⚡)

Now that your agent node is configured, send a quick test task to verify live operation immediately before exploring advanced swarm features.

### Option A: From the Terminal Bar (Bottom of Page)

1. Click the terminal input field at the bottom of any screen.
2. Type a command:
   ```text
   /send Research Bot Analyze the top 3 competitors in the AI agent space and summarize their pricing models
   ```
   Format: `/send <agent-name> <your task message>`
3. Press **Enter**.
4. Watch the **System Log** on the dashboard for real-time status updates and the final LLM response:
   - `📡 Task dispatched to Research Bot`

### Option B: Command Palette (`Cmd+K` / `Ctrl+K`)

1. Press **`Cmd+K`** (Mac) or **`Ctrl+K`** (Windows) anywhere.
2. Search for your Agent or Mission Cluster.
3. Select the agent to open their active chat window.

### Option C: Neural Sync (Voice-to-Swarm)

1. Go to **Voice Interface** from the sidebar (under "Comms & Assets").
2. Select your Target Agent or Mission Cluster → Click **Start Sync**.
3. Speak your objective clearly. The engine transcribes intent via **Whisper** and responds via TTS.

---

## Step 8: Bulk Capability Assignment (Skills Hub)

Instead of configuring agents one-by-one, you can assign tools to multiple agents simultaneously.

1. Go to **Skills** from the sidebar (under the Intelligence section, indented below Agents).
2. Select any **Skill**, **Workflow**, or **MCP Tool**.
3. Click the **"Assign to Agents"** button in the details panel.
4. Select all agents you want to receive this capability.
5. Click **Commit Assignments**. The engine will bulk-sync the configurations live.

---

## Step 9: Importing External Capabilities (.md)

Tadpole OS allows you to rapidly build your agent's library by importing existing documentation.

1. Go to **Skills**.
2. Click the **"Import .md"** button in the header.
3. Select a `.md` file containing a skill or workflow definition.
4. Review the **Import Preview** to verify the parsed logic and ID.
5. Click **Confirm Import**. The capability is now available in your **User Registry** for assignment.

---

## Step 10: Create and Execute a Mission Cluster

1. Go to **Missions** from the sidebar
2. Click **+ NEW MISSION** in the top-right of the cluster sidebar
3. Fill in:
   - **Mission Name**: e.g., `Market Research Sprint`
   - **Department**: Select the relevant department (e.g., `Research`)
   - **Local-Only Air-Gap Mode** *(Optional)*: Toggle ON to route this cluster's model calls through local Ollama. This affects provider routing only; it does not create company-scoped storage or authorize live Plane 2 data. Keep business workflows at synthetic preview until the company runtime is available.
4. Click **Create**

### Assign Agents to the Mission:

5. Select your new mission in the sidebar (it'll highlight)
6. In the **Available Agents** pool on the right, click **+ Assign** next to each agent you want on this mission
7. **Hierarchical Recruitment**: High-level agents (Alphas) can recruit ephemeral sub-agents. The engine uses modular specialists from `runner/mission_tools/` to delegate tasks with strategic context handoffs.
8. **Parallel Swarming (PERF-06)**: **Tadpole OS** utilizes `FuturesUnordered` to parallelize tool calls. Recruitment of multiple specialists happens simultaneously, reducing swarm startup latency by up to 80%.
9. **Swarm Pulse Visualizer**: Toggle the "Neural Map" icon on the mission dashboard to see a real-time Force-Graph visualization of cluster connectivity, featuring 10Hz binary telemetry pulses.
10. **Neural Swarm Optimization**: When you type an objective, the engine proactively suggests mission-specific templates via the **Template Discovery Hub**.
11. **Recursion Guard**: To prevent circular token-burn, the engine enforces a configurable **Max Swarm Depth** (default: **5**, set via `MAX_SWARM_DEPTH` in `.env`).
12. **Mission Analysis (Agent 99)**: Toggle the **"Analysis"** switch next to the Run button to trigger a post-mission debrief powered by LanceDB vector synthesis.

---

## Step 11: Workspace Sandboxing & Cluster Management

Tadpole OS allows you to organize your swarm into **Mission Clusters**.

- **Defaults**: The engine starts with **4 predefined clusters** (Strategic Command, Strategic Ops, Core Intelligence, Applied Growth).
- **Custom Scaling**: You can create new clusters or retire existing ones from the **Missions** page.
- **Per-Cluster Privacy Routing**: Enable the **Privacy Shield** (🛡️) for a cluster to make runner model selection prefer local endpoints and exclude configured cloud model slots for that run. This does not block general network egress; the global privacy monitor sends periodic reachability probes. Use host/network controls for an actual air gap.
- **Persistence**: Agent roles and system configurations are stored in the backend **SQLite database** (`data/state/tadpole.db` by default; an existing legacy `data/tadpole.db` is copied on startup when needed). Logical mission clusters and per-cluster governance policies are saved in `data/governance_settings.json` and frontend state.
- **Cluster Workspaces**: The runner creates cluster workspaces under `data/workspaces/{clusterId}`. This is workspace organization and path scoping, not company or tenant isolation.

---

## Step 12: External Adapters & Workspace Tools

The engine can now connect to your local environment and external services.

### Workspace File Operations
Agents with matching skills can read and write files within their cluster sandbox:
- **`read_file`**: Read a file from the workspace (e.g., load a spec document).
- **`write_file`**: Write a file to the workspace (e.g., save generated code).
- **`list_files`**: List files in a workspace directory.
- **`delete_file`**: Delete a file *(requires Oversight Gate approval)*.

Runner workspace files are stored under `data/workspaces/<cluster-id>/` on the server. File tools apply path checks against the configured workspace root, but this directory layout does not isolate company data or restrict every other tool and network path.

### Local Markdown Vault (Obsidian)
Enabled agents can use the `archive_to_vault` tool.
1. Create a `vault/` directory in the `server-rs` root.
2. Agents will automatically append findings to files in this directory when requested.

### Discord Notifications
1. Add `DISCORD_WEBHOOK="your_webhook_url"` to your `.env` file.
2. Use the `notify_discord` tool from an agent to alert your team.

### Standardized Observability (HATEOAS)
All resource endpoints in Tadpole OS implement the **HATEOAS** pattern. Responses include a `_links` object, enabling self-discovery of related actions. Error responses strictly follow **RFC 9457 (Problem Details)** for consistent machine-readable debugging.

---

## Step 13: SME Data Intelligence (Connectors & SOP Workflows)

Tadpole OS includes a 5-phase data intelligence layer for SME onboarding.

> [!WARNING]
> **Plane 2 is preview-only.** Company-scoped authorization, memory, and storage are pending. The generic runner rejects only an agent whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not trigger the guard. Until the company-scoped runtime exists, use synthetic inputs and stop business workflows at preview—do not connect real customer data, import it, persist it, publish it, or dispatch live workflows through the generic runner.

### Phase 1: Hybrid RAG
The Neural Memory engine (`memory.rs`) automatically combines vector similarity with keyword proximity scoring for higher-fidelity context retrieval. This is transparent — no configuration required.

### Phase 2: Data Connectors (Background Sync)
1. Go to **Hierarchy** → select an agent → open the **Memory** tab.
2. In the **Connector Config** section, click **+ Add Source**.
3. For evaluation, set **Type** to `fs` (file system) and **URI** to a synthetic sample directory only (e.g., `/data/sample-docs/`).
4. Click **Save**. The **Ingestion Worker** will begin crawling this directory at the interval set by `SME_SYNC_INTERVAL_MINS`.
5. Monitor sync status (idle/syncing/error) in the **Memory Section** UI.

### Phase 3: Outward Customer Catalog & A2A Gateway (`/customer-catalog`)
The current catalog uses shared outward state and is not company-scoped. Use synthetic content and stop at the preview; do not publish a live company card or use the public search endpoint for customer data until the scoped runtime is available.
1. Open the left sidebar and click **Customer Catalog** (`/customer-catalog`).
2. Set your **SMB Business Name** and select the **Outward Local Model Profile** (default: `gemma4:e4b`).
3. Fill out required business information, FAQs, policies, and operating hours using the interactive **Business FAQ & Operating Info Card Manager**.
4. Click **CSV Catalog**, **QuickBooks**, or **PDF Pricelist** to ingest catalog items.
5. Review the `company-agent-card.json` preview only. Do not publish or distribute the public endpoint for a live company workflow.

#### 🌐 Publishing Online via GitHub Pages (Guide for Cloners & Forkers)
If you clone or fork this repository and want to host your A2A Agent Card publicly online without exposing your local machine:
1. **Fork the Repository**: Push your cloned repo to GitHub (e.g., `https://github.com/<your-username>/<your-repo>`).
2. **Enable GitHub Pages**: Go to `Settings -> Pages` in GitHub, set Source to **GitHub Actions** (the `docs.yml` workflow handles VitePress deployment automatically).
3. **Place Your Agent Card**: Your static A2A assets live in `docs/public/` — VitePress automatically includes them in the deployed site:
   - `docs/public/a2a/v1/company-agent-card.json` — your A2A agent card
   - `docs/public/.well-known/agent.json` — IETF standard discovery endpoint
4. **Live Public Endpoint**: After the `Deploy VitePress Site` workflow runs, your agent card is live at:
   `https://<your-username>.github.io/<your-repo>/a2a/v1/company-agent-card.json`
5. **Automatic AI Discovery**: External web agents, AI crawlers, and search engines automatically locate your card via:
   - Global IETF discovery: `/.well-known/agent.json`
   - HTML `<head>` meta tags: `<link rel="agent-card" href="/a2a/v1/company-agent-card.json">`
   - AI search crawler directives in `robots.txt` and `sitemap.xml`

> [!TIP]
> Real-time client-side PII scanning (with Luhn Mod-10 credit card verification) warns you if sensitive keys or card numbers are detected in your knowledge cards before publishing.

### Phase 4: Deterministic SOP Workflows
The generic SOP engine does not bind runs to a company key. Use it only for engine workflows or synthetic previews; do not assign customer-facing workflows until company-scoped execution is implemented.
1. Create a markdown file in `data/workflows/` on the server (e.g., `data/workflows/onboarding.md`).
2. Format it with numbered steps — each step becomes a discrete agent turn.
3. Assign the workflow to an agent via the **Cognition** tab in the Agent Config Panel.
4. When the agent receives a mission, the **SOP Engine** will execute each step in guaranteed order.

### Phase 5: Layout-Aware Document Parsing
The Data Connectors automatically use the **Layout-Aware Parser** (`parser.rs`) for all ingested files. Supported formats: `.txt`, `.md`, `.csv`, `.pdf` (text-layer). Documents are chunked with 25% overlap for optimal embedding quality.

---

## Step 14: Performance Analysis & Real-time Telemetry

Tadpole OS provides "Top Tier" observability into swarm health and technical performance.

### 1. Real-time Telemetry (Swarm Visualizer)
The **Engine** page (sidebar: "Engine" under Intelligence) features a high-performance **Swarm Visualizer**:
- **Binary Swarm Pulse**: Driven by a 10Hz MessagePack stream (`0x02` header) for sub-millisecond state parity.
- **Topology Map**: Visualizes the swarm as a 2D force-graph, showing agent status and recruitment relationships.
- **Detach & Recall**: Pop the visualizer into a dedicated window for persistent oversight during deep-context missions.
- **Fiscal Burn**: Real-time USD/token tracking via the **TPM** indicator.
- **Swarm Density**: Monitor agent instantiation relative to system capacity.

### 2. Performance Analysis (Benchmarks)
1. Go to **Benchmarks** from the sidebar (under Intelligence).
2. **Timeline View**: Review historical benchmark results (latency, throughput, status).
3. **Comparison Tool**: Select any two runs to calculate performance deltas.
   - *Example*: Compare "Current" vs "Baseline" to identify code regressions or provider latency spikes.
4. **Target Enforcement**: Metrics are color-coded against the technical specifications in `Benchmark_Spec.md`.

### 3. Neural Map (Graph Visualization)
Navigate to **Neural Map** in the sidebar for a live code intelligence graph — visual blast-radius analysis and symbol-level relationship mapping of the codebase.

---

## Step 15: The Swarm Template Ecosystem

Instead of manually configuring agents, Tadpole OS allows you to instantly download full, industry-specific agent swarms.

1. Go to **Store** from the sidebar (under Intelligence) — or access it via the **Template Store** banner at the bottom of the **Settings** page.
2. **Discover**: Browse the store or use the fuzzy search to find templates tailored to your industry and company size (e.g., "Legal Contract Review", "Healthcare Patient Intake"). Filter by industry and headcount.
3. **Preview**: Click **Preview** on any template card to inspect the full `swarm.json` configuration before committing.
4. **Install**: Click **Install Swarm**. The engine uses native **Git Cloning** to securely fetch the template from `github.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates` and unpacks it into your `data/swarm_config` directory.
5. **Local Privacy Shield / Oversight Approval**: If the downloaded template requests powerful execution skills (like Shell Access or API payments), the engine will freeze initialization and require you (the Operator) to manually approve the skills, ensuring Zero-Trust security.

Once installed, the Rust engine "hot-loads" the template from your local `/data/swarm_config/` directory, and your new specialized agents will immediately appear in the **Hierarchy**.

---

## 🛠️ Useful Terminal Commands

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

> [!NOTE]
> All scripts in `execution/` follow a strict **`[OK]` / `[FAIL]`** machine-readable reporting protocol for autonomous agents.

---

## 🧩 Troubleshooting

| Issue | Fix |
|-------|-----|
| Dashboard shows **OFFLINE** | Check that the engine is running (`npm run engine` or Docker container is up) |
| Agent returns no response | Verify the model exists in the Model Registry and the API key is valid |
| Neural Vault won't unlock | The vault creates a new encryption key on first use — use any password. If locked out, use **Emergency Vault Reset** at the bottom of the unlock screen. |
| Model dropdown is empty | Go to **Providers** → unlock vault → add models to the registry |
| Agent config doesn't save | Check browser console — engine must be online for persistence to work |
| Tool-Calling fails (Groq) | The engine includes **Self-Healing Retries** for Groq. Malformed tool syntax is automatically corrected in a second pass. |
| Agent is slow / rate limited | The engine enforces `rpm`/`tpm` limits set on the model. The agent will wait for the quota window to reset rather than drop requests. |
| `NEURAL_TOKEN` panic on start | Either `NEURAL_TOKEN` or `NEURAL_ENGINE_ACCESS_TOKEN` env var is required for the engine to start. Set it in your `.env` file. |
| Workspace file access denied | Agent tried to access a path outside its sandbox. Check `cluster_id` mapping and ensure no path traversal in the filename. |
| Database not found on Windows | Leave `DATABASE_URL` unset to use `data/state/tadpole.db`, or set it to `sqlite:data/state/tadpole.db` / an absolute SQLite path |

---

## 🐸 Starter Swarm Configuration (Quick-Deploy)

This section provides a **ready-to-use 3-agent swarm** with concrete settings optimized for Groq's free tier. Follow this to have a working hierarchical swarm in under 5 minutes.

### Agent Roster

| Agent | ID | Role | Model | Provider | Temperature | Budget | Skills |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Agent of Nine** | `1` | CEO | `openai/gpt-oss-120b` | Groq | `0.8` | `$2.00` | `issue_alpha_directive`, `web_search` |
| **Tadpole** | `2` | COO (Alpha) | `openai/gpt-oss-120b` | Groq | `0.6` | `$3.00` | `web_search`, `write_file`, `read_file`, `spawn_subagent` |
| **Elon** | `3` | CTO (Specialist) | `qwen/qwen3.6-27b` | Groq | `0.3` | `$1.00` | `execute_shell`, `write_file`, `read_file`, `list_files` |

> [!TIP]
> **Why these settings?**
> - **Agent of Nine** uses high temperature (`0.8`) for creative strategic thinking and delegation.
> - **Tadpole** (Alpha) gets medium temperature (`0.6`) for balanced coordination and the `spawn_subagent` skill for recruitment.
> - **Elon** (Specialist) runs a fast, cheap model at low temperature (`0.3`) for precise code execution.
> - Budget caps prevent runaway token spend on Groq's free tier.

### Rate Limits & Pricing (Groq Free Tier Safe)

Set these in the **Providers → Model Registry** (click **SHOW RATE LIMITS & PRICING** or click the edit icon on the model row):

| Model | RPM | TPM | Context Window | Input Cost ($/M) | Output Cost ($/M) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `llama-3.3-70b-versatile` | `30` | `14000` | `128000` | `$0.59` | `$0.79` |
| `llama-3.1-8b-instant` | `30` | `14000` | `128000` | `$0.05` | `$0.08` |

### Applying via the UI

1. Go to **Hierarchy** → click each agent card
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
    "skills": ["execute_shell", "write_file", "read_file", "list_files"]
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
│  YOU (Operator)                                          │
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
   │CrewAI  │    │ AutoGen   │   │  execute_shell +      │
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
| **Hierarchy** | Agent status lights change: `idle` → `thinking` → `active` as each node activates |
| **Missions** | The cluster sidebar shows task assignments and handoff chains |
| **Operations** | Live token burn, cost tracking, and the System Log streaming agent outputs |
| **Oversight** | If `write_file` or `delete_file` triggers, you'll see approval requests here |

### What Happens Under the Hood

1. **Agent of Nine** receives the prompt, applies strategic reasoning, and fires `issue_alpha_directive` to Tadpole with a refined, tactical breakdown.
2. **Tadpole** decomposes the directive into 3 parallel tasks:
   - Spawns **Researcher A** (ephemeral) → searches for CrewAI architecture and pricing
   - Spawns **Researcher B** (ephemeral) → searches for AutoGen architecture and pricing
   - Sends a direct task to **Elon** → write Python code comparing all 3 frameworks
3. **Parallel Swarming (PERF-06)** kicks in — all 3 sub-tasks execute concurrently via `FuturesUnordered`.
4. As results flow back, Tadpole's **synthesis turn** merges them and calls `write_file("competitive_brief.md")` to save the final deliverable.
5. Cost and token metrics are tracked per-agent in real-time on the Operations Dashboard.

### Expected Output

After ~30-60 seconds (depending on Groq load), you'll find:
- `data/workspaces/<cluster-id>/competitive_brief.md` — The final synthesized report
- **System Log** entries showing the full delegation chain with swarm lineage breadcrumbs
- **Per-agent cost breakdown** on each hierarchy node card

### Scaling This Pattern

| Adjustment | How |
| :--- | :--- |
| **Add more researchers** | Give Tadpole more budget and increase `MAX_SWARM_DEPTH` in `.env` |
| **Use multiple providers** | Assign Claude to Agent of Nine, Groq to specialists |
| **Enable voice dispatch** | Use **Voice Interface → Neural Sync** instead of typing |
| **Auto-approve safe tools** | Set `AUTO_APPROVE_SAFE_SKILLS=true` in `.env` or toggle in Oversight Settings |
| **Save as a template** | Use **"Promote to Role"** on your configured agents |

---

## 🏘️ Swarm Topology Guide (How Many Bunkers Do You Need?)

> [!NOTE]
> **Bunker vs. Cluster Definition**:
> - **Bunker**: Refers to the physical or virtual node (the underlying hardware instance/host running an engine node).
> - **Cluster**: Refers to a logical grouping or runner workspace of agents running within that node (`data/workspaces/<cluster-id>`); it does not establish company or tenant isolation.

Not every deployment needs multiple nodes. Use this table to choose the right topology for your organization before configuring agents.

| Company Size | Recommendation |
| :--- | :--- |
| **1–15 people** | **Single swarm, always.** The operational overhead of managing multiple Bunker nodes eliminates the value for small teams. One box, all agents. |
| **15–50 people** | **Single swarm with logical cluster organization.** Use `cluster_id` to organize agents and knowledge entries by department — Finance, Sales, Ops — without the infrastructure complexity of multi-node. This is not an authorization or tenant-isolation boundary; the deployment is still one box. |
| **50–150 people** | **Multi-swarm makes sense *if* you have someone technical.** One Bunker per major department (3–4 nodes max). Each node runs its own engine instance; the Bunker Mesh (mDNS peer discovery) handles coordination automatically. |
| **150+ people** | **Multi-swarm with full mesh.** You have IT staff, real compliance requirements, and the budget for dedicated hardware per department. The P2P Knowledge Sync and Gossip Relay layer handles cross-node intelligence sharing. |

> [!TIP]
> **Start with evaluation.** Cluster scoping is not company isolation. Keep business workflows at synthetic preview until the company-scoped Plane 2 runtime is available; adding clusters or nodes does not establish the required boundary.

> [!NOTE]
> **Multi-swarm compliance benefit**: For regulated industries (finance, healthcare, legal), running sensitive data on a dedicated Bunker that never gossips to other nodes is a structural compliance advantage. Each node's Merkle audit trail is an independent, tamper-evident record.

---

## 🏗️ Architecture Quick Reference

AI-Tadpole-OS documents a **Dual-Plane Sovereign Hierarchy** ([`directives/IDENTITY.md`](../directives/IDENTITY.md)); the Plane 2 runtime is planned, not available for live business workflows:
- **Plane 1: Sovereign Codebase Lifecycle Engine (`Engine-Maintenance`)**: Knows, operates, diagnoses, fixes, maintains, and improves the `AI-Tadpole-OS` codebase over time (`server-rs/`, `src/`, `execution/`, `ADG v2`, `parity_guard.py`, `Agent 99`).
- **Plane 2: Company-Agnostic SMB AI Digital Twin (`SMB-Digital-Twin`, planned)**: Intended to provide business workflows and isolated practice workspaces; implementation and `FR-11` company-scoped enforcement remain pending. Use synthetic inputs and stop at preview.

```
┌─────────────────────────────────────┐
│         Your Browser (React)        │
│  Operations │ Hierarchy │ Missions  │
│  Providers  │ Oversight │ Settings  │
│  Skills     │ Engine    │ Store     │
└───────────────┬─────────────────────┘
                │ HTTP + WebSocket
┌───────────────▼─────────────────────┐
│       Axum Rust Engine :8000        │
│  Agent Registry │ Task Router       │
│  Oversight Gate │ Persistence       │
│  Skill Runner   │ MCP Host          │
└───────────────┬─────────────────────┘
                │ API Calls
┌───────────────▼─────────────────────┐
│        LLM Provider (Groq)          │
│  llama-3.3-70b │ llama-3.1-8b      │
└─────────────────────────────────────┘
                │
┌───────────────▼─────────────────────┐
│  SQLite (data/state/tadpole.db)     │
│  Agents │ Models │ Audit Trail      │
└─────────────────────────────────────┘
```
