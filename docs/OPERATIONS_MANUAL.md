> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[OPERATIONS_MANUAL]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS local business infrastructure, adapted for Small & Medium Business (SMB) use.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🧠 Tadpole OS: Business Operations Manual (SMB Edition)

> **Deployment Class**: Small & Medium Business (SMB) Local Node  
> **Status**: Verified Production-Ready  
> **Version**: 1.1.158
> **Last Hardened**: 2026-05-01 (Local-First Provider Optimization & Cost-Safeguard Pass)  
> **Data Privacy**: 100% Local-First / Zero Data Leaks  

---

## 📖 Terminology & Business Concepts

To navigate the local swarm engine of Tadpole OS, business operators must understand the core concepts that drive the virtual teams.

- **Department Lead** (formerly Alpha Node) [DEFINE: Alpha Node]: The orchestrating agent in a workspace responsible for breaking down goals, recruiting specialist sub-agents, and compiling reports.
- **Project Workspace** (formerly Mission Cluster) [DEFINE: Mission Cluster]: A collaborative local directory grouping multiple virtual agents and shared assets toward a single business goal.
- **Swarm Depth** [DEFINE: Swarm Depth]: The levels of delegation from the business operator down to terminal specialist nodes (Recommended Max: 3 for standard office hardware; Absolute Max: 5, configurable via `MAX_SWARM_DEPTH`).
- **Secure Vault** (formerly Neural Vault) [DEFINE: Neural Vault]: The encrypted, client-side repository for all LLM API keys and provider credentials. Auto-locks after **30 minutes** of inactivity and syncs unlock state across browser tabs via `BroadcastChannel`.
- **Private Swarm Environment** (formerly Sovereign Reality) [DEFINE: Sovereign Reality]: The secure, local-first runtime environment where virtual agents execute business processes under human control.

> [!TIP]
> **Complete Lexicon**: For a full technical breakdown of system terms, refer to the [GLOSSARY.md](./GLOSSARY.md). Every term marked with a `[DEFINE:]` tag corresponds to a high-fidelity entry in the glossary.

---

## 🗺️ Operational Tiers (Table of Contents)

### Tier 1: The Business Operator (Dashboard & Daily Ops)
*For team managers directing virtual agents.*
- [1. Multi-Tab Business Interface](#1-multi-tab-business-interface)
  - [1.1 The Multi-Tab Bar](#11-the-multi-tab-bar)
  - [1.2 Unified Dashboard Header](#12-unified-dashboard-header)
  - [1.3 Directing Agents via Chat & Voice](#13-directing-agents-via-chat--voice)
- [2. Project & Folder Management](#2-project--folder-management)
  - [2.1 Defining Project Goals](#21-defining-project-goals)
  - [2.2 Shared Directories & File Assets](#22-shared-directories--file-assets)
  - [2.3 Operations Center & Command Chain](#23-operations-center--command-chain)

### Tier 2: The Systems Administrator (Config & Cost Control)
*For managing engine parameters, API keys, and local hardware.*
- [3. Shared Folder & Asset Management](#3-shared-folder--asset-management)
  - [3.1 Workspace Manager](#31-workspace-manager)
  - [3.2 Task Draft Branching Workflow](#32-task-draft-branching-workflow)
- [4. Core Intelligence Management Suite](#4-core-intelligence-management-suite)
  - [4.1 LLM Provider Configuration](#41-llm-provider-configuration)
  - [4.2 Custom Virtual Employee Profiles](#42-custom-virtual-employee-profiles)
  - [4.3 Local Data Synchronization (RAG)](#43-local-data-synchronization-rag)

### Tier 3: Security & Performance Optimization
*For budget-guarding, hardware tuning, and diagnostics.*
- [5. Security, Approvals & Budget Safeguards](#5-security-approvals--budget-safeguards)
  - [5.1 Oversight Gate & Approval Queue](#51-oversight-gate--approval-queue)
  - [5.2 Security Dashboard & Cost Quotas](#52-security-dashboard--cost-quotas)
  - [5.3 Secure Credentials Vault](#53-secure-credentials-vault)
- [6. The Workspace Heartbeat (Observability)](#6-the-workspace-heartbeat-observability)
  - [6.1 Visualizing the Team and Codebase](#61-visualizing-the-team-and-codebase)
  - [6.2 Neural Footprint & Token Cost Monitoring](#62-neural-footprint--token-cost-monitoring)

- [Appendix A: Daily Operational Workflows](#appendix-a-daily-operational-workflows)
- [Appendix B: Maintenance & Local Backups](#appendix-b-maintenance--local-backups)

---

## 1. Multi-Tab Business Interface

The primary navigation system of Tadpole OS, allowing for multi-context orchestration within a single, high-performance web dashboard. **Multi-Monitor Friendly**: All tabs can be detached into separate windows for distributed oversight across dual office displays.

> [!NOTE]
> **Runtime Governance Anchor**: All persistence paths (workspaces, local databases, and temporary caches) resolve from the backend `AppState.base_dir`, ensuring local-dev and server-mounted office networks use the same folder root.

### 1.1 The Multi-Tab Bar
Located permanently at the top of the viewport, the Multi-Tab Bar manages your active workspaces:
- **Dynamic Contexts**: Tabs represent individual operational sectors (Dashboard, Projects, Team Graph, Settings).
- **Context Preservation**: Switching tabs preserves the state of each page, enabling seamless multitasking.
- **Detachable Portals**: Click the **External Link** icon revealed on hover to pop out any tab into a native browser window for dual-monitor layouts.
- **Header Sync**: The global `PageHeader` automatically updates its action controls to match the active tab.

### 1.2 Unified Dashboard Header (`PageHeader`)
A persistent command surface at the top of the page:
- **Engine Status**: Real-time local connection status (**🟢 ONLINE**).
- **Page-Specific Actions**: Direct action shortcuts (e.g., "Start New Project" when on the Projects tab).
- **Core Metrics**: High-level telemetry tailored to the active operation, synchronized via the `useEngineStatus` hook.

### 1.3 Directing Agents via Chat & Voice

#### 1.3.1 Business Chat Portal (`SovereignChat`)
The primary natural language interface for issuing directives to your virtual departments. It supports granular audience targeting:
- **Agent Scope** [DEFINE: RAG Scope]: 1:1 communication with a selected virtual employee. Context is limited to that agent's specific role memory.
- **Department Scope** [DEFINE: Mission Cluster]: Department-wide instructions (e.g., `#marketing`).
- **Global Scope** [DEFINE: Swarm Pulse]: Broadcasts messages to all active agents.

**Command Syntax:**
- **Standard Input**: Type a message and hit `Enter` to send in the active scope.
- **Targeted Agent**: Use `@AgentName <Message>` to send a task to a specific agent regardless of active scope.
- **Targeted Department**: Use `#DepartmentName <Message>` to broadcast to all agents in a specific work folder.

**Interactive Elements:**
- **[Toggle] Safe Execution**: Enable to prevent agents from running destructive tools (like file deletion or shell execution) without user confirmation.
- **[Button] Detach Interface**: Pop out the chat into a dedicated window.
- **[Button] Minimize Dock**: Minimize the chat window to a floating icon in the corner.
- **[Selector] Scope Tab**: Instantly switch between Agent, Department, and Global layers.
- **[Breadcrumb] Lineage path**: Displays the organizational path of the currently selected agent.

#### 1.3.2 Voice Interface (`VoiceClient` / `Standups`)
A voice-driven extension providing hands-free inputs and a dedicated "Standup Meeting" portal for team coordination.

**Core Interface Elements:**
- **Status Header**: Displays connection state (e.g., `SECURE LIVE CHANNEL` vs `Ready for Voice`).
- **Target Selection Matrix**: A control box for defining voice handshake destinations.
    - **Toggle (Agent Node)**: Restricts the voice input to a single virtual employee.
    - **Toggle (Workspace Cluster)**: Broadcasts the voice capture to all agents assigned to a specific folder.
    - **Selection Dropdown**: A precision list of active agents or department IDs.
- **Volume Visualizer**: A multi-spectrum bar graph reflecting local audio input levels.
- **Connection Command Bar**:
    - **[Button] Start/End Channel**: Large green/red control for opening or terminating the voice line.
    - **[Button] Local Mic Mute**: Mutes your local microphone without closing the audio line.
- **Live Transcript Log**:
    - **Identity Attribution**: Color-coded identifiers showing speakers (`U` for User, `A` for Agent).
    - **Active Telemetry**: Displays `REC` (Recording) vs `IDLE` status and real-time "Agent X is speaking..." feedback.

#### 1.3.3 Swarm Visualizer Interaction
The **`SovereignChat`** is integrated with the visual **Team Graph**. Clicking any virtual employee node on the graph automatically focuses that agent's project context in the chat, allowing for instant coordination.

#### 1.3.4 Activity Lineage Stream (`Lineage_Stream`)
A real-time telemetry sidebar that visualizes the propagation of data and instructions through your virtual organization.
- **Real-time Feed**: Scrollable timeline of every agent event, tool execution, and system message.
- **Chain of Command Tracker**: Visualizes the path an instruction took from the Department Lead down to the final terminal agent.
- **Payload Panel**: Displays the raw text content of individual agent thoughts and actions.

**Interactive Controls:**
- **[Sidebar] Resizable Boundary**: Drag the edge of the stream to expand the details view.
- **[Card] Event Node**: Click any card in the stream to open the **Cinematic Depth View** modal.
- **[Overlay] Cinematic Depth View**: A high-density modal showing full workflow paths, agent configurations, and timestamped metadata.

---

## 2. Project & Folder Management

### 2.1 Defining Project Goals (`Missions`)
Clear project scoping ensures virtual teams execute tasks quickly and stay within your budget:
- **New Project**: Initialize standard business objectives.
- **Scope Parameters**: Define the target goal, constraints, and outputs.
- **Assign Team**: Select which virtual employee profiles to assign to the project folder.
- **Collaboration Graph**: SVG-rendered diagram showing how agents will interact and delegate.

### 2.2 Shared Directories & File Assets (`Workspaces`)
Centralized coordination of local storage paths, department folders, and local files:
- **Department Folders**: Shared folders grouping virtual employees by function (e.g., Marketing, Bookkeeping).
- **Lead Assignee**: The primary manager agent responsible for final reports compiling.
- **Local Scan**: Auto-scans local workspaces for active developer files, spreadsheets, or text logs.

### 2.3 Operations Center & Command Chain (`Ops_Dashboard`)
The central nervous system for monitoring running processes and hardware loads:
- **Operations Center**: Real-time agent status dashboard and engine health checks.
- **Org Chart**: Visual graph of the command chain from Department Lead to sub-agents.
- **Scheduled Jobs**: Manage recurring tasks (e.g., scraping weekly invoices, building weekly status updates).

---

## 3. Shared Folder & Asset Management

### 3.1 Workspace Manager (`Workspaces`)
Centralized coordination of project storage paths and shared files.

**Operational Features:**
- **Department Clusters**: Workspaces that group virtual employees by department (Accounting, Marketing, Admin) into shared local folders.
- **Lead Node Halo**: Every cluster highlights the primary leader agent (gold outline) responsible for overall project execution.
- **Environment Detection**: Automatically detects local office settings, network configurations, and databases.

### 3.2 Task Draft Branching Workflow
- To protect your core documents from mistakes, virtual employees do not edit primary business files directly.
- Agents write proposed edits to temporary draft files (e.g., `report_draft.md`).
- A draft queue appears on the Workspaces dashboard waiting for the operator's approval.
    - **[Button] Approve Draft**: Merges the draft edits into the primary project document.
    - **[Button] Reject Draft**: Discards the changes and prompts the agent to regenerate the work with feedback.

---

## 4. Core Intelligence Management Suite

These configuration pages manage provider API keys, local models, and virtual employee configurations.

### 4.1 LLM Provider Configuration (`Model_Manager`)
The credential manager for local and cloud AI providers.

> [!IMPORTANT]
> Accessing API keys requires typing your master passphrase. Credentials are encrypted using **PBKDF2 + AES-256-GCM** via the browser's native `SubtleCrypto` API and stored in browser `localStorage`. Encryption runs in a **background Web Worker** (`crypto.worker.ts`) to prevent UI lag during large workspace operations.

**Interactive Elements:**
- **[Input] Master Passphrase:** Password field required to unlock the credentials vault.
- **[Button] Commit Authorization:** Dispatches keys for decryption.
- **[Button] Emergency Vault Reset:** Purges all saved API keys in case of corrupted data or lost passwords.
- **[Button] Add Provider:** Add new connections (e.g., OpenAI, Anthropic, Google, Groq, Inception, or local Ollama).
- **[Card] Provider Card:** Click to edit endpoint URLs and security keys.
- **[Table] Model Inventory:** A list of all available models.
- **[Toggle] Show Limits:** Expands the row to reveal Rate Limits (Requests/Min, Tokens/Day).

#### 4.1.1 Secure Context Requirements
The vault uses the browser's standard **SubtleCrypto API**, requiring a **Secure Context**:
- **Local Access**: `localhost` and `127.0.0.1` are secure by default.
- **Remote Access**: Accessing the dashboard via a local network IP (e.g., `http://10.0.0.1:5173`) will disable credential features. You must set up **HTTPS** to access the vault remotely.

#### 4.1.2 Automated Capability Inference (IMR-01)
The engine automatically detects the feature set of your models:
- 👁️ **multimodal**: Supports processing images, diagrams, and PDFs.
- 🛠️ **tools**: Supports tool execution (e.g., writing files, calculations).
- 🧠 **reasoning**: Optimized for deep analytical logic (e.g., DeepSeek-R1, OpenAI o1/o3).

### 4.2 Custom Virtual Employee Profiles (`AgentConfigPanel`)
Set up virtual employees and define their scope of work:
- **Cognition Tab**: Toggle whether the agent retains long-term memory across sessions.
- **Governance Tab**: Set individual budget caps and toggle the "Requires Oversight" setting.
- **Inference Tuning**: Assign specific models and temperature parameters.
- **Reasoning Engine (Mythos)**: Tune the agent's internal monologue settings:
    - **Reasoning Depth (1-16 turns)**: Set to `1-4` for simple, quick tasks (e.g., email categorization). Set to `5-10` for complex analysis.
    - **ACT Halting Threshold**: Define the model's self-halting confidence level.

### 4.3 Local Data Synchronization (RAG) (`search_mission_knowledge`)
The data intelligence layer synchronizes your local folders, folders, and databases with the agent's memory store.
- **Multi-Factor Scoring (MFS)**: Combines vector semantics, project relevance, and document recency to find the most accurate records.
- **Data Crawling**: System sync workers automatically index changes in your mapped directories every few minutes (`SME_SYNC_INTERVAL_MINS`).
- **Markdown SOP Enforcement**: Place your business Standard Operating Procedures (SOPs) as markdown files in `data/workflows/` to force agents to follow strict execution rules.

---

## 5. Security, Approvals & Budget Safeguards

### 5.1 Oversight Gate & Approval Queue (`Oversight`)
The primary dashboard safety valve that keeps the business operator in control of all agent actions.

#### 5.1.1 The Approval Queue
- **Pending Actions**: Displays actions (like rewriting files, executing local scripts, or browsing the web) that require explicit confirmation before running.
- **[Button] Approve / Reject**: Confirms or blocks the execution.

#### 5.1.2 Emergency Controls
> [!CAUTION]
> These commands immediately stop active projects and suspend agent processes.

- **[Button] Halt Agents**: Instantly halts all active agent thinking loops without closing the dashboard. Triggers the `handle_kill_switch` handler.
- **[Button] Kill Engine**: Sends a shutdown signal to the backend Axum process. Triggers the `handle_kill_engine` handler.

### 5.2 Security Dashboard & Cost Quotas
The dashboard tracks hardware safety and cloud costs:
- **[Gauge] Budget Quotas**: Displays daily and weekly API spending against your limits.
- **[Metric] Swarm Health**: Tracks success ratios, failed tool calls, and throttling states for each virtual employee.
- **[Alert] RAM/VRAM Pressure**: Displays system memory usage and alerts you when approach limits.
- **[Toggle] Auto-Approve Safe Skills**: Allows low-risk tools (like reading files, search queries, or internal calculations) to execute without waiting for approvals, keeping the gate focused on high-risk operations (writing files, shell execution).

### 5.3 Secure Credentials Vault
All external API connections use the client-side **`use_vault_store`** framework. API keys are encrypted with **PBKDF2 + AES-256-GCM** and stored in browser `localStorage` under the key `tadpole-vault-secrets`. Plaintext keys are decrypted only in memory during active runs — never written to disk or transmitted to the server.

**Key Security Behaviors:**
- **Auto-Lock**: The vault automatically locks after **30 minutes** of inactivity, clearing the master key from memory.
- **Cross-Tab Sync**: The vault uses a `BroadcastChannel` (`tadpole-vault-sync`) to synchronize unlock state across all open browser tabs. Unlocking in one tab unlocks all others.
- **Crypto Offloading**: All encryption/decryption is performed inside a background **Web Worker** (`crypto.worker.ts`), preventing UI lag on large key operations.

---

## 6. The Workspace Heartbeat (Observability)

### 6.1 Visualizing the Team and Codebase (`Engine_Dashboard` + `Swarm_Visualizer`)
A real-time force-graph visualizer showing how your agents interact and how your project files depend on each other. Accessible via the **Engine** sidebar item.
- **Pulse Indicators**: Visualizes real-time communication messages via a 10Hz binary telemetry stream.
- **Navigation Controls**: Move backward and forward through your traversal path, similar to web browser navigation.
- **High-Res Export**: Download high-quality PNG charts of your workflow hierarchies directly from the canvas.
- **Detach & Recall**: Pop the visualizer into a dedicated browser window for persistent multi-monitor oversight. Recall it back from the placeholder panel on the main dashboard.

### 6.2 Neural Footprint & Token Cost Monitoring (`Command_Table`)
A cost-accounting dashboard for monitoring API expenses. The `Command_Table` component is embedded in the **Oversight** page:
- **Interaction Logs**: View every prompt and response text.
- **Latency Tracker**: Measures execution times in milliseconds to pinpoint slower cloud models.
- **Token Efficiency**: Details context lengths and token sizes to help optimize prompt structures.

---

## Appendix A: Daily Operational Workflows

### Example: Running a Private Customer Feedback Analysis
1. **Configure Your Team**: Create a "Support Analyst" using a local model (Ollama + Llama-3-8B) to keep customer data local and private.
2. **Setup the Project Workspace**: Put your customer review spreadsheet (`reviews.csv`) in your designated project folder.
3. **Issue Directive**: Type in the chat: *"Analyze reviews.csv, group them into categories, and write a summary report.txt in the same directory."*
4. **Approve Safe Actions**: Approve the `read_file` request for `reviews.csv` in the Oversight Queue.
5. **Review Draft**: Once completed, review the generated report draft on your Workspaces page and click **Approve** to merge it.

---

## Appendix B: Maintenance & Local Backups

### B.1 Directory Structure & Backups
To protect your configurations, include these folders in your regular office backups:
- **`data/tadpole.db`**: Mapped configurations, agent profiles, and project logs.
- **`data/memory.lance/`**: Local vector databases used for document search. *(Only present when the engine is built with the `vector-memory` feature flag.)*
- **`.env`**: Port definitions and local path configurations (keep private).

### B.2 Automatic Hardware Recovery
If your local computer runs low on GPU memory (VRAM) while loading a local Ollama model, the engine automatically attempts to reload a smaller, quantized fallback model (e.g. `q4_K_M`) to prevent out-of-memory crashes and preserve project progress.

[//]: # (Metadata: [OPERATIONS_MANUAL])
