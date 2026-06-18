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

> [!NOTE]
> **Branding Note**: This document and the system glossary reflect the transition of the legacy "Sovereign Reality" product name to the canonical "Tadpole OS" branding. Legacy terms are redirected where appropriate.

# 🧠 Tadpole OS: Business Operations Manual (SMB Edition)

> **Deployment Class**: Small & Medium Business (SMB) Local Node  
> **Status**: Verified Production-Ready  
> **Version**: 1.1.186  
> **Last Hardened**: 2026-06-11 (Relative citation, risk tag alignment, and cloud provider verification)  
> **Last Verified Against Tag/Commit**: v1.1.165 (commit adb41393) on 2026-06-11  
> **Data Privacy**: 100% Local-First / Zero Data Leaks  

---

## 📖 Terminology & Business Concepts

To navigate the local swarm engine of Tadpole OS, business operators must understand the core concepts that drive the virtual teams.

### Tier 1: Operator
- **Department Lead** (formerly Alpha Node) [DEFINE: Alpha Node]: The orchestrating agent in a workspace responsible for breaking down goals, recruiting specialist sub-agents, and compiling reports.
- **Project Workspace** (formerly Mission Cluster) [DEFINE: Mission Cluster]: A collaborative local directory grouping multiple virtual agents and shared assets toward a single business goal.
- **Private Swarm Environment** (formerly Sovereign Reality) [DEFINE: Sovereign Reality]: The secure, local-first runtime environment where virtual agents execute business processes under human control.
- **Department Scope** [DEFINE: Department Scope]: A communications constraint in the SovereignChat that restricts broadcasted instructions to all agents and sub-agents assigned to a specific workspace folder or cluster.

### Tier 2: Admin
- **Swarm Depth** [DEFINE: Swarm Depth]: The levels of delegation from the business operator down to terminal specialist nodes (Recommended Max: 3 for standard office hardware; Absolute Max: 5, configurable via `MAX_SWARM_DEPTH`).
- **Agent Scope / RAG Scope** [DEFINE: RAG Scope]: The data ingestion context constraint limiting an agent's retrieval to specific files, workspace directories, or long-term memory.

### Tier 3: Security
- **Secure Vault** (formerly Neural Vault) [DEFINE: Neural Vault]: The encrypted, client-side repository for all LLM API keys and provider credentials. Auto-locks after **30 minutes** of inactivity and syncs unlock state across browser tabs via `BroadcastChannel`.
- **Swarm Pulse** [DEFINE: Swarm Pulse]: A real-time binary telemetry event stream showing system communications, agent thinking state, and tool calls.

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
    - [1.3.1 Business Chat Portal](#131-business-chat-portal-sovereignchat)
    - [1.3.2 Voice Interface](#132-voice-interface-voiceclient--standups)
    - [1.3.3 Swarm Visualizer Interaction](#133-swarm-visualizer-interaction)
    - [1.3.4 Activity Lineage Stream](#134-activity-lineage-stream-lineage_stream)
- [2. Project & Folder Management](#2-project--folder-management)
  - [2.1 Defining Project Goals](#21-defining-project-goals-missions)
  - [2.2 Shared Directories & File Assets](#22-shared-directories--file-assets-workspaces)
  - [2.3 Operations Center & Command Chain](#23-operations-center--command-chain-ops_dashboard)

### Tier 2: The Systems Administrator (Config & Cost Control)
*For managing engine parameters, API keys, and local hardware.*
- [3. Shared Folder & Asset Management](#3-shared-folder--asset-management)
  - [3.1 Workspace Manager](#31-workspace-manager-workspaces)
  - [3.2 Task Draft Branching Workflow](#32-task-draft-branching-workflow)
- [4. Core Intelligence Management Suite](#4-core-intelligence-management-suite)
  - [4.1 LLM Provider Configuration](#41-llm-provider-configuration-model_manager)
  - [4.2 Custom Virtual Employee Profiles](#42-custom-virtual-employee-profiles-agentconfigpanel)
  - [4.3 Local Data Synchronization (RAG)](#43-local-data-synchronization-rag-search_mission_knowledge)

### Tier 3: Security & Performance Optimization
*For budget-guarding, hardware tuning, and diagnostics.*
- [5. Security, Approvals & Budget Safeguards](#5-security-approvals--budget-safeguards)
  - [5.1 Oversight Gate & Approval Queue](#51-oversight-gate--approval-queue-oversight)
    - [5.1.1 The Approval Queue](#511-the-approval-queue)
    - [5.1.2 Emergency Controls](#512-emergency-controls)
  - [5.2 Security Dashboard & Cost Quotas](#52-security-dashboard--cost-quotas)
  - [5.3 Secure Credentials Vault](#53-secure-credentials-vault)
  - [5.4 Client-Side API Resilience (Circuit Breaker)](#54-client-side-api-resilience-circuit-breaker)
- [6. The Workspace Heartbeat (Observability)](#6-the-workspace-heartbeat-observability)
  - [6.1 Visualizing the Team and Codebase](#61-visualizing-the-team-and-codebase-engine_dashboard--swarm_visualizer)
  - [6.2 Neural Footprint & Token Cost Monitoring](#62-neural-footprint--token-cost-monitoring-command_table)
  - [6.3 Parity Guard CLI](#63-parity-guard)

- [Appendix A: Daily Operational Workflows](#appendix-a-daily-operational-workflows)
- [Appendix B: Maintenance & Local Backups](#appendix-b-maintenance--local-backups)
  - [B.1 Directory Structure & Backups](#b1-directory-structure--backups)
  - [B.2 Automatic Hardware Recovery](#b2-automatic-hardware-recovery)
  - [B.3 Global Environment Variables](#b3-global-environment-variables)
  - [B.4 Build Feature Flags](#b4-build-feature-flags)

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

**Interactive Elements:**
- **[Button] Detach/Recall tab**
  - *Default state*: Embedded (Not Detached).
  - *Visible location*: Right-hand side of individual tab headers in the Multi-Tab Bar when hovered.
  - *Observable side effect*: Spawns a new browser window hosting the page, replacing the embedded view, and changes the icon to a minimize/recall icon.
- **[Button] Close tab**
  - *Default state*: Open.
  - *Visible location*: Far right of hovered tab headers in the Multi-Tab Bar.
  - *Observable side effect*: Closes the selected tab and shifts focus to the next available active tab.

### 1.2 Unified Dashboard Header (`PageHeader`)
A persistent command surface at the top of the page:
- **Engine Status**: Real-time local connection status (**🟢 ONLINE**).
- **Page-Specific Actions**: Direct action shortcuts (e.g., "Start New Project" when on the Projects tab).
- **Core Metrics**: High-level telemetry tailored to the active operation, synchronized via the `useEngineStatus` hook.

### 1.3 Directing Agents via Chat & Voice

#### 1.3.1 Business Chat Portal (`SovereignChat`)
The primary natural language interface for issuing directives to your virtual departments. It supports granular audience targeting:
- **Agent Scope** [DEFINE: RAG Scope]: 1:1 communication with a selected virtual employee. Context is limited to that agent's specific role memory.
- **Department Scope** [DEFINE: Department Scope]: Department-wide instructions (e.g., `#marketing`).
- **Global Scope** [DEFINE: Swarm Pulse]: Broadcasts messages to all active agents.

**Command Syntax:**
- **Standard Input**: Type a message and hit `Enter` to send in the active scope.
- **Targeted Agent**: Use `@AgentName <Message>` to send a task to a specific agent regardless of active scope.
- **Targeted Department**: Use `#DepartmentName <Message>` to broadcast to all agents in a specific work folder.

**Interactive Elements:**
- **[Toggle] Safe Execution**
  - *Default state*: `false` (Unchecked, allowing safe-mode execution configurations via `is_safe_mode`).
  - *Visible location*: Bottom command bar of the `SovereignChat` panel.
  - *Observable side effect*: When enabled, enforces strict approval requirements, blocking agents from performing modifications without explicit confirmation in the Oversight queue.
- **[Button] Detach Interface**
  - *Default state*: Embedded (Not Detached).
  - *Visible location*: Top-right header of the `SovereignChat` panel.
  - *Observable side effect*: Spawns a new native browser portal window containing the chat widget, retaining full state synchronization.
- **[Button] Minimize Dock**
  - *Default state*: Maximized (Open).
  - *Visible location*: Top-right header of the `SovereignChat` panel (minimize icon).
  - *Observable side effect*: Collapses the chat panel into a floating circular button in the bottom-right corner of the viewport.
- **[Selector] Scope Tab**
  - *Default state*: `'agent'` (Active scope).
  - *Visible location*: Sub-header row of the `SovereignChat` panel.
  - *Observable side effect*: Swaps active context filtering and targets between Agent, Department (Cluster), and Global (Swarm) scopes.
- **[Breadcrumb] Lineage path**
  - *Default state*: `'Overlord'` root.
  - *Visible location*: Directly below the scope selector in `SovereignChat` when the Agent tab is active.
  - *Observable side effect*: Visualizes parent-to-child delegation lines for the active virtual employee.

**Keyboard Shortcuts:**
- **Toggle Command Palette**: `[Shortcut: Ctrl+K]` or `[Shortcut: Ctrl+/]` (toggles the global utility search bar, cited in [useLayoutNavigation.ts:L51-83](../src/hooks/layout/useLayoutNavigation.ts#L51-L83)).
- **Tab Quick-Navigation**: `[Shortcut: 1-6]` (Quick-routes to: 1. Dashboard, 2. Org Chart, 3. Standups, 4. Workspaces, 5. Docs, 6. Settings when input fields are out of focus, cited in [useLayoutNavigation.ts:L51-83](../src/hooks/layout/useLayoutNavigation.ts#L51-L83)).

→ *See §5.3 for vault security details.*

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
    - **[Button] Start/End Channel**
      - *Default state*: Inactive (`'idle'`).
      - *Visible location*: Center console of the `VoiceClient` pane.
      - *Observable side effect*: Opens or terminates local microphone streaming buffers, updating state metrics to live recording.
    - **[Button] Local Mic Mute**
      - *Default state*: Unmuted (`false`).
      - *Visible location*: Next to the connection button on the command bar.
      - *Observable side effect*: Mutes microphone capturing lines without severing the audio interface connection.
- **Live Transcript Log**:
    - **Identity Attribution**: Color-coded identifiers showing speakers (`U` for User, `A` for Agent).
    - **Active Telemetry**: Displays `REC` (Recording) vs `IDLE` status and real-time "Agent X is speaking..." feedback.

→ *See §4.1 for provider configuration.*

#### 1.3.3 Swarm Visualizer Interaction
The **`SovereignChat`** is integrated with the visual **Team Graph**. Clicking any virtual employee node on the graph automatically focuses that agent's project context in the chat, allowing for instant coordination.

#### 1.3.4 Activity Lineage Stream (`Lineage_Stream`)
A real-time telemetry sidebar that visualizes the propagation of data and instructions through your virtual organization.
- **Real-time Feed**: Scrollable timeline of every agent event, tool execution, and system message.
- **Chain of Command Tracker**: Visualizes the path an instruction took from the Department Lead down to the final terminal agent.
- **Payload Panel**: Displays the raw text content of individual agent thoughts and actions.

**Interactive Controls:**
- **[Sidebar] Resizable Boundary**
  - *Default state*: Fixed standard width.
  - *Visible location*: Outer border of the Lineage Stream drawer.
  - *Observable side effect*: Expands or contracts the sidebar panel width.
- **[Card] Event Node**
  - *Default state*: Unselected.
  - *Visible location*: Scrolling logs within the stream.
  - *Observable side effect*: Triggers and opens the high-density Cinematic Depth View overlay.
- **[Overlay] Cinematic Depth View**
  - *Default state*: Closed (`false`).
  - *Visible location*: Center-screen modal overlay.
  - *Observable side effect*: Displays the selected agent's full parameters, outputs, and JSON payloads.

---

## 2. Project & Folder Management

### 2.1 Defining Project Goals (`Missions`)
Clear project scoping ensures virtual teams execute tasks quickly and stay within your budget:
- **New Project**: Initialize standard business objectives.
- **Scope Parameters**: Define the target goal, constraints, and outputs.
- **Assign Team**: Select which virtual employee profiles to assign to the project folder.
- **Collaboration Graph**: SVG-rendered diagram showing how agents will interact and delegate.

#### Project Lifecycle Status States
Swarm missions transition through a strict state machine representing different phases of execution:
- **Pending**: The project has been created by the operator but has not yet been initiated.
- **SpecReview**: The Department Lead agent reviews the project goals, scopes the recipes, and validates the required inputs and skills before beginning work.
- **Active**: Specialists are actively executing tools, processing documents, and reasoning in the workspace sandbox.
- **Completed**: The primary goal is achieved, and a final summary report is compiled by the Department Lead.
- **Failed**: The swarm encountered a terminal error, exceeded the budget, or reached maximum delegation depth.
- **Paused**: Processing is temporarily suspended by the operator or awaiting manual oversight resolution.

#### Workspace Cluster Limits
To prevent system overload on local nodes, the maximum number of concurrent projects is capped by the `MAX_CLUSTERS` engine setting. If the active workspace count matches this limit:
- Any attempts by the operator to initialize a new project workspace are blocked.
- A warning log is emitted by the system: `Cluster limit reached` (logged as level `'warning'` by [workspace_service.ts:L65-68](../src/services/workspace_service.ts#L65-L68)).

→ *See §4.1 for provider configuration.*

### 2.2 Shared Directories & File Assets (`Workspaces`)
Centralized coordination of local storage paths, department folders, and local files:
- **Department Folders**: Shared folders grouping virtual employees by function (e.g., Marketing, Bookkeeping).
- **Lead Assignee**: The primary manager agent responsible for final reports compiling.
- **Local Scan**: Auto-scans local workspaces for active developer files, spreadsheets, or text logs.

→ *See §4.3 for local data synchronization details.*

### 2.3 Operations Center & Command Chain (`Ops_Dashboard`)
The central nervous system for monitoring running processes and hardware loads:
- **Operations Center**: Real-time agent status dashboard and engine health checks.
- **Org Chart**: Visual graph of the command chain from Department Lead to sub-agents.
- **Scheduled Jobs**: Manage recurring tasks (e.g., scraping weekly invoices, building weekly status updates).

→ *See §5.2 for security dashboard and cost quota details.*  
→ *See §6.1 for the swarm visualizer that powers the Org Chart.*  

---

## 3. Shared Folder & Asset Management

### 3.1 Workspace Manager (`Workspaces`)
Centralized coordination of project storage paths and shared files.

**Operational Features:**
- **Department Clusters**: Workspaces that group virtual employees by department (Accounting, Marketing, Admin) into shared local folders.
- **Lead Node Halo**: Every cluster highlights the primary leader agent (gold outline) responsible for overall project execution.
- **Environment Detection**: Automatically detects local office settings, network configurations, and databases.

→ *See §4.3 for local data synchronization details.*

### 3.2 Task Draft Branching Workflow
- To protect your core documents from mistakes, virtual employees do not edit primary business files directly.
- Agents write proposed edits to temporary draft files (e.g., `report_draft.md`).
- A draft queue appears on the Workspaces dashboard waiting for the operator's approval.

**Interactive Elements:**
- **[Button] Approve Draft**
  - *Default state*: Pending approval.
  - *Visible location*: Proposal card inside the Workspace Manager dashboard.
  - *Observable side effect*: Overwrites the canonical file with the draft contents and clears the proposal from the active list.
- **[Button] Reject Draft**
  - *Default state*: Pending approval.
  - *Visible location*: Proposal card inside the Workspace Manager dashboard.
  - *Observable side effect*: Deletes the draft file, cancels the operation, and feeds rejection parameters to the agent.

→ *See §5.3 for vault security details.*

---

## 4. Core Intelligence Management Suite

These configuration pages manage provider API keys, local models, and virtual employee configurations.

### 4.1 LLM Provider Configuration (`Model_Manager`)
The credential manager for local and cloud AI providers.

> [!IMPORTANT]
> Accessing API keys requires typing your master passphrase. Credentials are encrypted using **PBKDF2 + AES-256-GCM** via the browser's native `SubtleCrypto` API and stored in browser `localStorage`. Encryption runs in a **background Web Worker** (`crypto.worker.ts`) to prevent UI lag during large workspace operations.

☁️ REQUIRES NETWORK
> **External Cloud Connections**: Triggering generation queries on external cloud providers sends prompt payloads and decrypted API tokens directly to the provider endpoints. The currently supported cloud endpoints (with their env-var keys) are:
> - `api.openai.com` (`OPENAI_API_KEY`) — cited in [provider.rs:L384](../server-rs/src/agent/runner/provider.rs#L384) & [model_manager.rs:L172](../server-rs/src/routes/model_manager.rs#L172)
> - `api.anthropic.com` (`ANTHROPIC_API_KEY`) — cited in [anthropic.rs:L116](../server-rs/src/agent/anthropic.rs#L116) & [model_manager.rs:L126](../server-rs/src/routes/model_manager.rs#L126)
> - `generativelanguage.googleapis.com` (`GOOGLE_API_KEY`) — cited in [gemini.rs:L142](../server-rs/src/agent/gemini.rs#L142) & [model_manager.rs:L135](../server-rs/src/routes/model_manager.rs#L135)
> - `api.groq.com` (`GROQ_API_KEY`) — cited in [groq.rs:L136](../server-rs/src/agent/groq.rs#L136) & [model_manager.rs:L171](../server-rs/src/routes/model_manager.rs#L171)
> - `api.inceptionlabs.ai` (`INCEPTION_API_KEY`) — key resolution cited in [provider.rs:L393](../server-rs/src/agent/runner/provider.rs#L393)
> - `api.deepseek.com` (`DEEPSEEK_API_KEY`) — key resolution cited in [provider.rs:L396-414](../server-rs/src/agent/runner/provider.rs#L396-L414)
> - `api.replicate.com` (`REPLICATE_API_KEY` - used for semantic audio replicate caching) — cited in [res.rs:L51-53](../server-rs/src/state/hubs/res.rs#L51-L53)
>
> **Zero Leaks Pledge**: Payload structures are processed completely in memory; plaintext keys are never written to disk or logged by Tadpole OS.

**Interactive Elements:**
- **[Input] Master Passphrase**
  - *Default state*: Empty string (`""`).
  - *Visible location*: Center modal panel when Vault is locked.
  - *Observable side effect*: Dispatches key derivation routines inside the Web Worker.
- **[Button] Commit Authorization**
  - *Default state*: Enabled.
  - *Visible location*: Next to the passphrase entry field.
  - *Observable side effect*: Unlocks the provider configuration view, loading active API profiles.
- **[Button] Emergency Vault Reset**
  - *Default state*: Enabled.
  - *Visible location*: Bottom actions corner of the unlocked Vault panel.
  - *Observable side effect*: Destructive action that purges all local storage API keys and resets vault settings.
- **[Button] Add Provider**
  - *Default state*: Enabled.
  - *Visible location*: Top header of the provider listing page.
  - *Observable side effect*: Appends a configuration template card to the UI.
- **[Card] Provider Card**
  - *Default state*: Populated with configured URL and status.
  - *Visible location*: Main provider grid.
  - *Observable side effect*: Expands inputs for URL endpoints, API keys, and custom providers.
- **[Table] Model Inventory**
  - *Default state*: Auto-scanned list of system-integrated models.
  - *Visible location*: Center region of `Model_Manager`.
  - *Observable side effect*: Displays feature matrix capabilities.
- **[Toggle] Show Limits**
  - *Default state*: Collapsed (`false`).
  - *Visible location*: Row elements inside the Model Inventory table.
  - *Observable side effect*: Exposes RPM and TPM threshold configurations.

#### 4.1.1 Secure Context Requirements
The vault uses the browser's standard **SubtleCrypto API**, requiring a **Secure Context**:
- **Local Access**: `localhost` and `127.0.0.1` are secure by default.
- **Remote Access**: Accessing the dashboard via a local network IP (e.g., `http://10.0.0.1:5173`) will disable credential features. You must set up **HTTPS** to access the vault remotely.

#### 4.1.2 Automated Capability Inference (IMR-01)
The engine automatically detects the feature set of your models:
- 👁️ **multimodal**: Supports processing images, diagrams, and PDFs.
- 🛠️ **tools**: Supports tool execution (e.g., writing files, calculations).
- 🧠 **reasoning**: Optimized for deep analytical logic (e.g., DeepSeek-R1, OpenAI o1/o3).

→ *See §5.3 for vault security details.*

### 4.2 Custom Virtual Employee Profiles (`AgentConfigPanel`)
Set up virtual employees and define their scope of work:
- **Cognition Tab**: Toggle whether the agent retains long-term memory across sessions.
- **Governance Tab**: Set individual budget caps and toggle the "Requires Oversight" setting.
- **Inference Tuning**: Assign specific models and temperature parameters.
- **Reasoning Engine (Mythos)**: Tune the agent's internal monologue settings:
    - **Reasoning Depth (1-16 turns)**: Set to `1-4` for simple, quick tasks (e.g., email categorization). Set to `5-10` for complex analysis.
    - **ACT Halting Threshold**: Define the model's self-halting confidence level.

→ *See §5.1 for oversight queue details.*

### 4.3 Local Data Synchronization (RAG) (`search_mission_knowledge`)
The data intelligence layer synchronizes your local folders and databases with the agent's memory store.
- **Multi-Factor Scoring (MFS)**: Combines vector semantics, project relevance, and document recency to find the most accurate records.
- **Data Crawling**: System sync workers automatically index changes in your mapped directories every few minutes (`SME_SYNC_INTERVAL_MINS`).
- **Markdown SOP Enforcement**: Place your business Standard Operating Procedures (SOPs) as markdown files in `data/workflows/` to force agents to follow strict execution rules.

---

## 5. Security, Approvals & Budget Safeguards

### 5.1 Oversight Gate & Approval Queue (`Oversight`)
The primary dashboard safety valve that keeps the business operator in control of all agent actions.

#### 5.1.1 The Approval Queue
- **Pending Actions**: Displays actions (like rewriting files, executing local scripts, or browsing the web) that require explicit confirmation before running.
- **[Button] Approve / Reject**
  - *Default state*: Waiting.
  - *Visible location*: Individual tool request cards in the Oversight timeline.
  - *Observable side effect*: Resolves the backend blocking promise, either allowing the tool payload execution (`approved`) or returning a cancellation trace to the agent loop (`rejected`).

#### 5.1.2 Emergency Controls
> [!CAUTION]
> **Definitive System Termination**: These buttons trigger immediate thread suspension or service termination. Use with care.

- **[Button] Halt Agents**
  - *Default state*: Enabled/Online.
  - *Visible location*: Oversight dashboard header panel.
  - *Observable side effect*: Prompts the user with `confirm_halt_agents` dialog, then halts all active agent thinking loops by sending thread abort signals via the `handle_kill_switch` handler.
- **[Button] Kill Engine**
  - *Default state*: Enabled/Online.
  - *Visible location*: Oversight dashboard header panel.
  - *Observable side effect*: Prompts the user with `confirm_kill_engine` verification, demands typing the uppercase word `"SHUTDOWN"` in the text input, and then terminates the Axum service process immediately via the `handle_kill_engine` handler.

### 5.2 Security Dashboard & Cost Quotas
The dashboard tracks hardware safety and cloud costs:
- **[Gauge] Budget Quotas**
  - *Default state*: `0.0` USD used.
  - *Visible location*: Upper metrics grid of the Oversight dashboard.
  - *Observable side effect*: Dynamically reflects real-time cumulative LLM API expenses against configured caps.
- **[Metric] Swarm Health**
  - *Default state*: Perfect health indicator.
  - *Visible location*: Center-right of the metrics grid.
  - *Observable side effect*: Evaluates execution failures and warns of service namespace breakers tripping.
- **[Alert] RAM/VRAM Pressure**
  - *Default state*: Inactive.
  - *Visible location*: Diagnostics monitoring segment.
  - *Observable side effect*: Emits amber/red warning banners when system RAM or GPU memory utilization approaches VRAM limits.
- **[Toggle] Auto-Approve Safe Skills**
  - *Default state*: `true` (Enabled).
  - *Visible location*: Security settings panel on the `Oversight` page.
  - *Observable side effect*: When active, automatically permits standard read/search actions, queueing only write or shell tool calls.

### 5.3 Secure Credentials Vault
All external API connections use the client-side **`use_vault_store`** framework. API keys are encrypted with **PBKDF2 + AES-256-GCM** and stored in browser `localStorage` under the key `tadpole-vault-secrets`. Plaintext keys are decrypted only in memory during active runs — never written to disk or transmitted to the server.

**Key Security Behaviors:**
- **Auto-Lock**: The vault automatically locks after **30 minutes** of inactivity, clearing the master key from memory.
- **Cross-Tab Sync**: The vault uses a `BroadcastChannel` (`tadpole-vault-sync`) to synchronize unlock state across all open browser tabs. Unlocking in one tab unlocks all others.
- **Crypto Offloading**: All encryption/decryption is performed inside a background **Web Worker** (`crypto.worker.ts`), preventing UI lag on large key operations.

### 5.4 Client-Side API Resilience (Circuit Breaker)

<!-- Last verified against tag/commit v1.1.165 (commit adb41393) on 2026-06-11 -->

To shield the front-end dashboard from backend service failures, network timeouts, or offline logical bunkers, the application implements a client-side **Circuit Breaker** system for all core service namespaces (e.g., `infra`, `engine`, `continuity`).

#### 5.4.1 Breaker States & Behaviors
- **CLOSED**: Normal operation. All API requests pass through to the backend service.
- **OPEN**: Triggered when a service namespace encounters 5 consecutive failures (`VITE_SYSTEM_BREAKER_FAILURE_THRESHOLD` [RISK: HIGH]). In this state, further calls to the namespace are instantly rejected on the client side with a `CircuitBreakerOpenError` to prevent UI freezing.
- **HALF_OPEN**: After a 10-second cooldown period (`VITE_SYSTEM_BREAKER_COOLDOWN_MS` [RISK: HIGH]), the breaker enters a probe state, permitting a single request to test the backend's health. If the request succeeds twice, the breaker resets to **CLOSED**. If the request fails, it trips back to **OPEN** and starts a new cooldown.

#### 5.4.2 Log Observability
- 📡 `[Circuit Breaker] NAMESPACE entered HALF_OPEN probe state` - The cooldown expired, and the system is probing the service namespace.
- ❌ `[Circuit Breaker] NAMESPACE tripped back to OPEN from HALF_OPEN due to trial failure` - The health probe failed, indicating the service is still down (e.g., local Rust engine is stopped).

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

**Interactive Elements:**
- **[Table Row] Interaction Logs**
  - *Default state*: Populated with recent prompt-response pairs.
  - *Visible location*: Center region of the `Command_Table` panel.
  - *Observable side effect*: Clicking a row expands the panel to show raw request and response body text.
- **[Column Header] Latency Tracker**
  - *Default state*: `0ms` (or latency of the most recent query).
  - *Visible location*: Upper-right column header of `Command_Table`.
  - *Observable side effect*: Displays execution durations in milliseconds to pinpoint slower local or cloud models.
- **[Cell] Token Efficiency**
  - *Default state*: `0 tokens` (accumulated).
  - *Visible location*: Inside each query row of `Command_Table`.
  - *Observable side effect*: Details prompt and response token lengths to help optimize prompt templates.

### 6.3 Parity Guard

<!-- Last verified against tag/commit v1.1.165 (commit adb41393) on 2026-06-11 -->

The `parity_guard.py` script serves as the codebase's canonical drift detector, ensuring documentation, Rust routes, environmental variables, and skill manifests remain in absolute alignment.

#### 6.3.1 Invocation Details
Run the verification check via the Python entry point:
```powershell
python execution/parity_guard.py .
```
To run the automated fix (such as synchronizing `API_REFERENCE.md` with modified schemas):
```powershell
python execution/parity_guard.py . FIX=1
```

#### 6.3.2 Failure Modes Detected
The script evaluates and alerts on five specific drift types:
1. **Axum Route Drift**: Flags endpoints defined in `server-rs/src/router.rs` that are missing from `docs/openapi.yaml` (legacy root endpoints trigger warnings, `/v1` endpoints trigger errors).
2. **Environmental Drift**: Flags Rust logic calling `std::env::var("VAR")` that is not declared in `server-rs/.env.example`.
3. **Manifest Drift**: Validates `data/skills/*.json` manifests to ensure their designated execution scripts exist on disk.
4. **Documentation Drift**: Evaluates `@docs` tags in Rust code. If code changes occur, it requires corresponding markdown file timestamp updates.
5. **API Doc Drift**: Checks if `API_REFERENCE.md` timestamps lag behind `openapi.yaml` modifications.

#### 6.3.3 Exit Code Translation
- **Exit Code 0**: Full alignment. The system has no detected drift.
- **Exit Code 1**: Mismatches detected. Detailed failures are printed to standard output.

> [!NOTE]
> **Parity Verification**: The `parity_guard.py` script does not consume `OPERATIONS_MANUAL.md` directly. It runs independent AST scans against the Rust source directories, OpenAPI files, and environment templates.

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

### B.3 Global Environment Variables
Tadpole OS leverages environment variables to manage hardware performance, network bindings, database files, and security policies.

<details>
<summary>🔐 Secrets & Authentication</summary>

- `NEURAL_TOKEN` [RISK: HIGH] — Type: string; Default: `""`. Secret API token required to authenticate all client-dashboard requests.
- `NEURAL_ENGINE_ACCESS_TOKEN` [RISK: HIGH] — Type: string; Default: `""`. Core sidecar access token.
- `AUDIT_PRIVATE_KEY` [RISK: HIGH] — Type: string; Default: `""`. The private key seed used to cryptographically sign transaction logs.
- `WORKFLOW_ENCRYPTION_KEY` [RISK: HIGH] — Type: string; Default: `""`. Encryption key for securing workflow states.
- `CAPABILITY_KEY_CURR` [RISK: HIGH] — Type: string; Default: `""`. Current signing key for client tokens.
- `CAPABILITY_KEY_PREV` [RISK: HIGH] — Type: string; Default: `""`. Previous signing key for client tokens.
- `TEST_PROVIDER_KEY` [RISK: HIGH] — Type: string; Default: `""`. Used in integration tests to bypass API keys.
- `OPENAI_API_KEY` [RISK: HIGH] — Type: string; Default: `""`. ☁️ REQUIRES NETWORK API key for OpenAI model querying.
- `GROQ_API_KEY` [RISK: HIGH] — Type: string; Default: `""`. ☁️ REQUIRES NETWORK API key for Groq model querying.
- `ANTHROPIC_API_KEY` [RISK: HIGH] — Type: string; Default: `""`. ☁️ REQUIRES NETWORK API key for Anthropic model querying.
- `GOOGLE_API_KEY` [RISK: HIGH] — Type: string; Default: `""`. ☁️ REQUIRES NETWORK API key for Google Gemini model querying.
- `INCEPTION_API_KEY` [RISK: HIGH] — Type: string; Default: `""`. ☁️ REQUIRES NETWORK API key for Inception model querying.
- `DEEPSEEK_API_KEY` [RISK: HIGH] — Type: string; Default: `""`. ☁️ REQUIRES NETWORK API key for DeepSeek model querying.
- `REPLICATE_API_KEY` [RISK: HIGH] — Type: string; Default: `""`. ☁️ REQUIRES NETWORK API key for Replicate model querying.
- `DISCORD_WEBHOOK` [RISK: HIGH] — Type: string; Default: `""`. ☁️ REQUIRES NETWORK Discord notification webhook url.

</details>

<details>
<summary>🛡️ Security Posture</summary>

- `PRIVACY_MODE` [RISK: HIGH] — Type: boolean; Default: `false`. When true, strictly blocks outbound cloud AI connections.
- `TADPOLE_ALLOW_LOCAL_HTTP` [RISK: HIGH] — Type: boolean; Default: `false`. Bypasses secure HTTPS checks for local development loops.
- `AUTO_APPROVE_SAFE_SKILLS` [RISK: HIGH] — Type: boolean; Default: `true`. Allows safe read-only operations to bypass the Oversight Queue.

</details>

<details>
<summary>🌐 Network & Binding</summary>

- `PORT` [RISK: LOW] — Type: number; Default: `8000`. The HTTP server listen port.
- `BIND_ADDRESS` [RISK: HIGH] — Type: string; Default: `127.0.0.1`. The local loopback IP address of the engine.
- `ALLOWED_ORIGINS` [RISK: HIGH] — Type: string; Default: `http://localhost:5173`. List of permitted browser CORS locations.
- `TRUST_PRIVATE_NETWORKS` [RISK: HIGH] — Type: boolean; Default: `false`. Bypasses proxy verification on local subnet cards.
- `ALLOW_UNSAFE_CORS` [RISK: HIGH] — Type: boolean; Default: `false`. Enables unsafe open CORS profiles.
- `TRUSTED_PROXIES` [RISK: HIGH] — Type: string; Default: `""`. Permitted proxy IP lists.
- `VITE_SYSTEM_BREAKER_FAILURE_THRESHOLD` [RISK: HIGH] — Type: number; Default: `5` (enforced client-side). Maximum consecutive failures allowed before tripping breaker.
- `VITE_SYSTEM_BREAKER_COOLDOWN_MS` [RISK: HIGH] — Type: number; Default: `10000` (enforced client-side). Cooldown time before attempting Half-Open checks.

</details>

<details>
<summary>💾 Persistence & Storage</summary>

- `DATABASE_URL` [RISK: HIGH] — Type: string; Default: `sqlite://data/tadpole.db`. Database connection file path.
- `DATA_DIR` [RISK: MEDIUM] — Type: string; Default: `./data`. Root data storage root path.
- `WORKSPACE_ROOT` [RISK: MEDIUM] — Type: string; Default: `..`. Base sandboxing root path for relative file operations.
- `RESOURCE_ROOT` [RISK: LOW] — Type: string; Default: `./resources`. Storage path for system instructions and graphics.
- `STATIC_DIR` [RISK: MEDIUM] — Type: string; Default: `./static`. Directory path for static frontend assets.

</details>

<details>
<summary>⚙️ Engine Limits</summary>

- `MAX_AGENTS` [RISK: MEDIUM] — Type: number; Default: `100`. The registry limit boundary for maximum concurrent agent identities.
- `MAX_CLUSTERS` [RISK: MEDIUM] — Type: number; Default: `10`. Caps active project directories.
- `MAX_SWARM_DEPTH` [RISK: MEDIUM] — Type: number; Default: `5`. Hard limits recursive agent-spawning depth on local computers.
- `MAX_TASK_LENGTH` [RISK: MEDIUM] — Type: number; Default: `32768`. Caps token consumption boundaries per prompt.
- `ENGINE_RATE_LIMIT` [RISK: MEDIUM] — Type: number; Default: `2000`. Maximum HTTP requests allowed per minute per IP to prevent network overloading.
- `DEFAULT_AGENT_BUDGET_USD` [RISK: MEDIUM] — Type: number; Default: `1.0`. The spending budget limit allocated to new sub-agents.
- `WORKFLOW_CONCURRENCY_LIMIT` [RISK: MEDIUM] — Type: number; Default: `5`. Limits simultaneous execution loops for SOP workflows.
- `WORKFLOW_AGENT_TIMEOUT_SECS` [RISK: MEDIUM] — Type: number; Default: `600`. Time limit for sub-agent executions in workflows.
- `MAX_CONCURRENT_RUNNERS` [RISK: MEDIUM] — Type: number; Default: `10`. Upper limit on concurrently running agent worker tasks.

</details>

<details>
<summary>🤖 Provider Endpoints</summary>

- `OLLAMA_HOST` [RISK: MEDIUM] — Type: string; Default: `http://localhost:11434`. Endpoint address for local model runs.
- `LANCEDB_DEDUPE_THRESHOLD` [RISK: MEDIUM] — Type: number; Default: `0.9`. Similarity score threshold for RAG deduplication.
- `LANCEDB_DRIFT_THRESHOLD` [RISK: MEDIUM] — Type: number; Default: `0.8`. Similarity score threshold for re-indexing triggers.
- `SME_SYNC_INTERVAL_MINS` [RISK: MEDIUM] — Type: number; Default: `30`. Re-indexing interval for background data sync workers.
- `FAILOVER_AMBER_THRESHOLD` [RISK: MEDIUM] — Type: number; Default: `3`. Failure count leading to provider health warning.
- `FAILOVER_RED_THRESHOLD` [RISK: MEDIUM] — Type: number; Default: `5`. Failure count leading to provider offline state.
- `FAILOVER_MAX_ATTEMPTS` [RISK: MEDIUM] — Type: number; Default: `3`. Maximum retry attempts to alternative providers.
- `PROVIDER_TIMEOUT_SECS` [RISK: MEDIUM] — Type: number; Default: `60`. Timeout limit on active model generations.
- `PIPER_MODEL_PATH` [RISK: MEDIUM] — Type: string; Default: `""`. Local path mapping to Piper text-to-speech weights.
- `VAD_MODEL_PATH` [RISK: MEDIUM] — Type: string; Default: `""`. Local path mapping to Silero VAD voice model weights.
- `WHISPER_MODEL_PATH` [RISK: MEDIUM] — Type: string; Default: `""`. Local path mapping to Whisper speech recognition weights.

</details>

<details>
<summary>🔍 Observability & Telemetry</summary>

- `RUST_LOG` [RISK: SAFE] — Type: string; Default: `server_rs=info,tower_http=debug`. Diagnostic logging output levels.
- `DISABLE_TELEMETRY` [RISK: LOW] — Type: boolean; Default: `false`. Stops logging OTel telemetry.
- `HEARTBEAT_INTERVAL_SECS` [RISK: LOW] — Type: number; Default: `30`. Hearts update ticks.

</details>

<details>
<summary>🛠️ Dev / Test / CI</summary>

- `BUNKER_NODES` [RISK: HIGH] — Type: string; Default: `bunker-1:Swarm Bunker 1:localhost`. Node identification lists formatted as `<id>:<display_name>:<host>` tuples, comma-separated for multiple entries.
- `TADPOLE_NULL_PROVIDERS` [RISK: MEDIUM] — Type: boolean; Default: `false`. Mocks LLM responses for CI/testing.
- `LEGACY_JSON_BACKUP` [RISK: MEDIUM] — Type: boolean; Default: `false`. Keeps old JSON backup files active.
- `SKIP_DB_SEED` [RISK: MEDIUM] — Type: boolean; Default: `false`. Skip database default seeding.
- `CLUSTER_ID` [RISK: LOW] — Type: string; Default: `local-1`. Unique identifier tag for the local node.
- `KUBERNETES_SERVICE_HOST` [RISK: LOW] — Type: string; Default: `""`. Automatically loaded host address within cluster environments.

</details>

### B.4 Build Feature Flags
Tadpole OS provides compilation flags that allow optimizing VRAM footprint on office computers.

- `vector-memory` [RISK: MEDIUM] — Compiles LanceDB and Apache Arrow libraries into the server to enable persistent semantic memory indexing. Stores vectors locally inside `data/memory.lance/`. Disabled by default for simplified installation on Windows office nodes.
- `neural-audio` [RISK: MEDIUM] — Integrates the ONNX runtime (`ort`), Whisper speech-to-text, and audio utilities to support real-time voice streaming. Disabled by default on legacy CPUs.

---

[//]: # (Metadata: [OPERATIONS_MANUAL])
<!-- Last verified against tag/commit v1.1.165 (commit adb41393) on 2026-06-11 -->
