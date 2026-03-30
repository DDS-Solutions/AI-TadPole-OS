# 🐝 Swarm Orchestration Guide

> [!NOTE]
> **Status**: Verified / Production-Ready  
> **Version**: 1.2.0  
> **Last Updated**: 2026-03-27 (SME Data Intelligence Update)  
> **Documentation Level**: Professional (Antigravity Std)

## Table of Contents

- [📐 The Hierarchical Command Pattern](#1-the-hierarchical-command-pattern)
- [🤝 The Sovereignty Handoff Pattern](#2-the-sovereignty-handoff-pattern-top-tier)
- [🌌 Shared Wisdom: The Context Bus](#3-shared-wisdom-the-context-bus)
- [⚡ Parallel Swarm Execution](#4-parallel-swarm-execution-perf-06)
- [📅 Autonomous Background Missions](#4b-autonomous-background-missions-continuity-scheduler)
- [🤖 Deterministic Workflows (Engine-Level)](#deterministic-workflows-engine-level)
- [🧩 Pattern: Fan-Out / Fan-In](#pattern-fan-out--fan-in)
- [🎯 Designing a "Top Tier" Mission](#5-designing-a-top-tier-mission)
- [🛡️ Orchestration Safety & Financial Guardrails](#orchestration-safety--financial-guardrails)
- [📡 Advanced Swarm Observability](#7-advanced-swarm-observability)
- [🔄 Mission Analysis Feedback Loop](#7c-the-mission-analysis-feedback-loop-top-tier)
- [🧠 Neural Swarm Optimization](#8-neural-swarm-optimization-intelligent-guidance)
- [🏰 Multi-Node Bunker Infrastructure](#9-multi-node-bunker-infrastructure-discovery)
- [📁 Workspace File Operations](#9-workspace-file-operations-in-swarms)
- [✅ Swarm Efficiency Checklist](#10-swarm-efficiency-checklist)

---

Moving from a single-agent chat to a **Hierarchical Swarm** is how you achieve "Top Tier" success with Tadpole OS. This guide outlines the strategies for designing robust, autonomous intelligence clusters.

## 1. The Hierarchical Command Pattern
Tadpole OS enforces a strict command hierarchy to prevent "Swarm Drift" (where agents lose track of the original objective).

1.  **CEO (Sovereign)**: The **Agent of Nine (ID 1)**. Receives strategic intent (via Voice/Neural Handoff) and issues refined directives to Alphas.
2.  **Alpha Node (Tactical)**: Coordinates the cluster. Spawns specialized sub-agents and handles synthesis.
3.  **Specialists (Execution)**: Focused on specific skills. In Tadpole OS, these are standardized as **MCP Tools** (Model Context Protocol), allowing for a unified execution layer across native scripts and system delegates.

## 2. The Sovereignty Handoff Pattern (Top Tier)
The most advanced orchestration strategy in Tadpole OS.

- **Objective**: Execute complex user intent with zero "micro-delegation" effort.
- **Protocol**:
    1. User speaks a high-level goal to the **Voice Interface**.
    2. **Local Whisper** (or Groq) transcribes it with high fidelity.
    3. **Agent of Nine (CEO)** receives the transcript, applies strategic "Best Practices," and uses the `issue_alpha_directive` tool.
    4. **Tadpole Alpha (COO)** receives a perfectly formatted tactical mission and begins swarm execution immediately.
- **Benefit**: Decouples the user from the "nitty-gritty" of cluster configuration.

## 3. Shared Wisdom: The Context Bus
Agents do not communicate secrets; they broadcast findings. To optimize your swarm:
- **Be Descriptive**: When an agent reports a finding, ensure it includes source citations.
- **Synthesis Turns**: Use the Alpha Node's "Thinking" phases to merge conflicting findings before the final report.

## 🤖 Deterministic Workflows (Engine-Level)
Tadpole OS supports structured, multi-step pipelines managed by the `WorkflowEngine` (`server-rs/src/agent/continuity/workflow.rs`). Unlike natural language swarming, deterministic workflows guarantee order and state propagation.

### Workflow Structure
A workflow consists of multiple `WorkflowStep` entries:
```json
{
  "name": "Market Analysis Pipeline",
  "steps": [
    {
      "order": 1,
      "agent_id": 5,
      "prompt": "Research current AI trends in crypto.",
      "context_injection": "full"
    },
    {
      "order": 2,
      "agent_id": 1,
      "prompt": "Synthesize the findings from Step 1 into a strategic report.",
      "context_injection": "previous_findings"
    }
  ]
}
```

### Context Injection Principles
- **Findings Propagation**: The `WorkflowEngine` automatically captures the `findings` abstraction from Step N and injects it as a persistent system message for Step N+1.
- **Stateful Missions**: A `WorkflowRun` maintains a shared Mission ID across all steps, ensuring a unified Merkle Audit Trail for the entire pipeline.

## 🧩 Pattern: Fan-Out / Fan-In
The `WorkflowEngine` excels at managing parallel execution and subsequent synthesis.
- **Fan-Out**: A single step can recruit multiple specialists concurrently. The `WorkflowEngine` waits for all parallel branches to complete before proceeding.
- **Fan-In**: A subsequent step (often an Alpha or CEO) receives the combined findings from all parallel specialists, enabling efficient synthesis.

## 4. Parallel Swarm Execution (PERF-06)
**Tadpole OS** enables **Concurrent Tooling**.
- **The Optimization**: When an agent decides to use multiple tools (e.g., `spawn_subagent` for three different specialists), the engine executes them in parallel.
- **Impact**: Swarm startup time remains near-constant even as the number of initial specialists increases. Use this to recruit "Department Clusters" in a single turn.

## 4c. SME Data Intelligence in Swarms
The **4-phase LlamaIndex-inspired** data intelligence layer enriches swarm operations:

### Hybrid RAG (Phase 1)
All agents using `search_mission_knowledge` benefit from **dual-signal retrieval**: vector similarity combined with keyword proximity scoring. This reduces hallucination when agents reason over domain-specific SME data ingested via connectors.

### Background Data Connectors (Phase 2)
Agents can be configured with **Connector Configs** that point to local directories. The **Ingestion Worker** daemon (a Tokio background task) periodically crawls these sources and embeds new content into each agent's `VectorMemory`. This means swarm agents passively accumulate institutional knowledge between missions.
- **SyncManifest**: Each source tracks `last_sync_at` and `checksum` to ensure only changed files are re-embedded.
- **Configuration**: Set `SME_SYNC_INTERVAL_MINS` env var (default: 30 minutes) to control crawl frequency.

### SOP Workflow Integration (Phase 3)
Agents assigned a **Deterministic Workflow** (`data/workflows/*.md`) will execute their mission as a fixed-order SOP instead of a free-form intelligence loop. This is ideal for:
- Compliance checklists that must execute in exact order
- Customer onboarding sequences where each step depends on the previous
- Audit workflows requiring guaranteed step-by-step execution

### Layout-Aware Ingestion (Phase 4)
The Ingestion Worker uses a **Layout-Aware Parser** (`parser.rs`) that preserves document structure (markdown headers, CSV row context, PDF page boundaries) during chunking. This produces higher-quality embeddings compared to naive full-text splitting.

## 4b. Autonomous Background Missions (Continuity Scheduler)
Tadpole OS supports "Set and Forget" proactive swarming via the **Continuity Scheduler**.
- **Cron-Driven Action**: You can define standard Unix cron expressions (e.g., `0 * * * *` for hourly) for any mission payload to execute without user prompting.
- **Background Execution**: The engine spins up the swarm securely in the background, persisting output to the SQLite database and writing files to the cluster workspace seamlessly.
- **Defensive Ceilings**: Background tasks adhere strictly to assigned `budgetUSD` ceilings. If an autonomous task encounters `maxFailures` consecutive LLM or tool crashing states, the Continuity Scheduler will automatically suspend the job to prevent unbounded token expenditure.

## 5. Designing a "Top Tier" Mission
Follow this template for maximum reliability:

> [!TIP]
> **Don't want to build from scratch?** Head to the **System Configuration > Swarm Template Store** to download pre-configured, industry-specific execution pipelines via the GitHub Native Hub. The Rust engine will hot-load these templates directly into your active roster by cloning them locally.

### Phase A: Discovery (Depth 0-1)
The Overlord (Entity 0) (AKA Human-in-the-loop) identifies the scope.
> "Alpha, research the impact of Quantum Computing on the current finance market."

### Phase B: Parallelization (Depth 2)
The Alpha recruits specialists using the standardized **`recruit_specialist`** MCP tool.
> `recruit_specialist(agentId: "researcher_a", message: "Analyze GPU stock trends")`
> `recruit_specialist(agentId: "researcher_b", message: "Analyze theoretical cryptography breakthroughs")`

### Phase D: Working Memory Persistence
The Alpha agent uses the `update_working_memory` tool to maintain a persistent mission scratchpad. This ensures that strategy goals and intermediate findings survive across multi-turn synthesis phases.

### Best Practice: Strategic Intent & Repo Mapping
When recruitment is necessary, the engine automatically injects:
1. **Strategic Intent**: The parent's current "Strategic Thought" payload.
2. **Repo Map**: A high-level summary of the **Hydra-RS Code Graph**.

This ensures that a researcher spawned by a CEO knows exactly *why* they are researching and *where* relevant modules are located, improving the depth and speed of the initial response.

## 🛡️ Orchestration Safety & Financial Guardrails
Autonomous orchestration requires strict fiscal and behavioral containment.

### 1. The Budget Gate
Every `WorkflowRun` is assigned a cumulative `budget_usd`. The **Budget Guard** uses **Debounced Persistence** to track the aggregate real-time spend of all agents. If the budget is exhausted, even in-memory, the mission is intercepted immediately.

### 2. Failure Handling (The "Circuit Breaker")
The `AgentHealth` module monitors step success.
- **Fail-Fast**: If an agent hits a critical provider error, the workflow pauses and emits a `system:message` via WebSocket.
- **Manual Resume**: An operator can fix the context or provider settings and "Resume" the workflow from the failed step via the Continuity UI.

### 3. Loop Prevention
The engine maintains a `LineageChain`. Any attempt by Agent A to recruit Agent B (who is already an ancestor in the active mission) is blocked, preventing infinite recruitment recursion.

- **Cost Awareness**: Always set a `budgetUsd` for hierarchical swarms. Recursive spawning can consume tokens quickly. **Real-time USD burn and token usage** are tracked per-node on the agent card.
- **Recursion Limits**: Tadpole OS enforces a strict **Depth Limit of 5**. Missions reaching this depth will stop recruitment to prevent infinite token consumption.
- **Lineage Awareness**: The engine tracks the recruitment chain (A → B → C). Agents are strictly prohibited from recruiting anyone already in their lineage.
- **Model Inheritance**: Sub-agents automatically inherit the parent node's model identity and provider credentials.
- **Rate Limiting**: RPM and TPM limits set on a model (in the Model Registry) are **automatically enforced** by the engine. Agents will wait for quota windows to reset rather than fail. Configure limits to stay within your provider's free tier.
- **Skill Manifests (Sapphire Phase 1)**: All skills are constrained by strongly-typed JSON schema requirements (`skill.json`).
- **Agent-Written Skills (Sapphire Phase 2)**: Agents can autonomously propose and physicalize new skills using the `propose_capability` tool. Upon human approval via the Oversight Dashboard, the skill manifest is instantly saved to disk and loaded into the `CapabilitiesRegistry`, expanding the swarm's abilities permanently.
- **Oversight Auto-Gate**: Tools marked with permissions for `budget:spend` or `shell:execute`, as well as all agent-written skills, automatically force an oversight lock, preventing agents from destroying host files or blowing API budgets without human approval.
- **Agent-Level Gate (Junior Mode)**: Enabling the **Requires Oversight** flag on a specific agent node forces *all* its tool calls into the Approval Queue, regardless of the tool's individual safety rating.
- **Mission-Level Quotas**: Clusters can be assigned a dedicated USD cap. This provides a secondary layer of fiscal containment, ensuring that a single "runaway" research cluster cannot exhaust the global system budget.
- **MCP Standardization**: All skills (skills, workflows, recruitment) are now routed through the **McpHost**. This ensures that every tool call follows a consistent protocol, providing better error handling and future-proof extensibility for external MCP servers.
- **Bulk Capability Syncing**: Operators can perform a **Sovereignty Sync** from the Skills Hub, pushing a single MCP tool or workflow to an entire department cluster in one turn. This ensures architectural parity across the swarm without manual node-by-node configuration.

### 4. Security & Integrity Monitoring (Resource Guard)
The engine provides a non-bypassable security layer for every swarm turn:
- **Merkle Integrity Score**: Every tool call is cryptographically signed and chained. If a mission's Merkle Score drops below 100%, the engine alerts the operator of potential integrity corruption.
- **RAM Pressure Awareness**: Swarms involving deep recursion (Depth 5+) or massive data ingestion are monitored for memory pressure. If the engine detects a risk of OOM (Out of Memory), it automatically pauses execution to protect the host system.
- **Sandbox Lockdown**: The engine verifies its container environment (Docker/K8s) to ensure filesystem sandboxes are enforced with the highest available isolation primitives.

### 5. Persistent Vector Memory
All agents automatically retain terminal findings in a permanent LanceDB directory (`memory.lance`). On mission startup, relevant embeddings are extracted into a localized `scope.lance`, allowing agents to query cross-session knowledge utilizing the `search_mission_knowledge` tool autonomously without prompt bloat.
- **Neural Engine Access Token Sync**: The engine requires a valid **Neural Engine Access Token** (formerly Neural Token) to be configured in the UI and `.env` for secure cross-bunker orchestration.
- **Reactive Parity**: The swarm is powered by a **Lazy Proxy Socket** and **Zustand** stores. Any update to agent configuration or infrastructure settings is reflected instantly across the entire hierarchy.

## 7. Advanced Swarm Observability ("God View")

Tadpole OS provides a real-time, hierarchical overview of the entire swarm execution, allowing for absolute transparency and trust.

### 7.1 The God View (Live Telemetry Graph)
Located in the **Mission Hub**, the Live Telemetry Graph (powered by `reactflow`) visualizes the "Chain of Thought" as it happens.
- **Hierarchical Spans**: Every mission is broken down into a tree of spans (Setup -> Context -> Intel -> Tool).
- **Recruitment Visualization**: When an agent recruits a specialist, a new branch is dynamically added to the graph.
- **Status Badges**: Nodes change color and display status pulses (Running, Success, Error) in real-time.
- **Mission Filtering**: Use the dropdown to isolate specific Mission IDs or view the Global Swarm history.
- **Interactive Nodes**: Each node provides duration metrics and status indicators for deep-dive performance analysis.

### 7.2 Neural Waterfall (Atomic UI Updates)
Message rendering is no longer "all or nothing."
- **Incremental Patching**: Messages appear in the chat as they are generated by the LLM, eliminating the wait for a full response.
- **Status Elevation**: Critical internal agent states (e.g., "Analyzing Codebase...") are displayed as distinct pulses in the UI, providing constant feedback.

### 7.3. RESTful Observability (HATEOAS)
All cluster and agent resources expose a `_links` navigation map, allowing for programmatic discovery of related actions (e.g., `pause`, `resume`, `test`). Errors are signaled via **RFC 9457 Problem Details**, providing standardized `type`, `title`, and `detail` fields for automated recovery logic.

### 7b. Mission Visualization: The Neural Map
The Mission Cluster detail view is now enhanced with a **Neural Map** that provides real-time visual feedback on cluster connectivity.
- **Visual Connection Traces**: SVG-based animated paths visualize the operational link between the Alpha Node and its specialists.
- **Node Status Glow**: Agents actively engaged in tasks exhibit a "Neural Glow," allowing for at-a-glance monitoring of cluster utilization.
- **Interactive Toggling**: Switch between the Team List and the Neural Map via the cluster dashboard.

Use these visualizations to verify your hierarchical layout and identify disconnected specialists.

### 7c. The Mission Analysis Feedback Loop (Top Tier)
For mission-critical swarms, enable **Mission Analysis** to establish a continuous improvement cycle utilizing LanceDB vector memory.
- **Automated Debrief**: Once a mission graduates, a specialized **Success Auditor** (Agent 99) reviews the context. For large tasks, it employs **Semantic Pruning** to extract only the key blockers and decisions, saving thousands of tokens.
- **Pattern Recognition**: The auditor queries its own `memory.lance` vector space to detect if current errors have happened historically, offering permanent architecture fixes rather than isolated tweaks.
- **Behavioral Auditing**: The engine automatically screens the final output's vector embedding against the agent's core identity, generating a **Behavioral Drift** warning if the agent violated its strategic constraints.
- **Optimization Prescriptions**: The auditor identifies redundant tool calls, prompt inefficiencies, and strategic gaps.
- **Learn and Refine**: Use these reports to update your agent roles and department workflows.

### 7d. Swarm Capability Discovery (Auto-Registration)
A "Top Tier" orchestration strategy where the swarm grows more intelligent with every mission.
- **Discovery Loop**: As sub-agents execute tasks, the engine monitors for newly defined skills or workflows generated during the reasoning phase.
- **Registry Injection**: Discovered capabilities are automatically registered in the **AI Services** category of the Skills Hub.
- **Viral Intelligence**: Once a capability is registered, any other agent in the swarm can immediately utilize it, eliminating the need for redundant "First Principle" reasoning in future missions.

## 8. Neural Swarm Optimization (Intelligent Guidance)
The Tadpole Engine includes a proactive optimization layer that analyzes mission objectives to suggest ideal cluster configurations.

- **Heuristic Analysis**: The engine monitors mission objectives for strategic keywords:
    - `security`, `audit`, `vault`: Triggers suggestions for security-specialized roles and models.
    - `scale`, `performance`, `bench`: Suggests high-throughput models and parallel swarming configurations.
- **The Optimization Cycle**:
    1.  **Generation**: The engine computes a `SwarmProposal` when a mission starts or the objective changes.
    2.  **Notification**: A pulsing **"Brain" icon** appears on the Alpha Node in the Hierarchy View.
    3.  **Governance**: The operator reviews the **Neural Reasoning Trace** (identifying *why* the suggestion was made) and either Authorizes or Dismisses the strategy.
- **Authorize Sync**: (Coming Soon) Automatically applies the AI's recommended configuration to the cluster, re-provisioning agents and models instantly.
- **Dismiss**: Ignores the suggestion, permanently removing it from the cluster's active state.

> [!NOTE]
> **Dynamic Scaling**: The number of clusters in your swarm is not static. You can create new Mission Clusters or retire old ones directly from the UI. These groupings are preserved in your browser's LocalStorage, ensuring your custom organizational structure persists across sessions.

## 9. Multi-Node Bunker Infrastructure (Discovery)
Tadpole OS supports a decentralized swarm across multiple logical or physical "Bunkers." 

- **Infrastructure Discovery**: Use the **'Discover Nodes'** trigger on the **Operations Center** to scan the local network for secondary bunkers.
- **Credentialed Access**: Node discovery uses secure, credentialed handshakes via the **Neural Engine Access Token** (formerly Neural Token) protocol.
- **Unified Oversight**: Discovered nodes are integrated into the main dashboard, allowing for cross-node swarm monitoring from a single sovereign interface.

## 10. Swarm Efficiency Checklist
- [ ] Does the Overlord have a high-temperature model for creative planning?
- [ ] Do specialists have precise models (low temperature) for data extraction?
- [ ] Is the `fetch_url` skill granted ONLY to the nodes that need it?
- [ ] Has the Alpha been given a system prompt emphasizing "Synthesis and Conflict Resolution"?
- [ ] Have RPM/TPM limits been set on all models to prevent provider quota overruns?
- [ ] Are file-writing agents assigned to a cluster with a dedicated workspace?
- [ ] Is `delete_file` skill omitted from agents that only need to read?

## 11. Workspace File Operations in Swarms

Agents can now read and write files within their cluster's physical workspace — enabling multi-turn collaborative document generation.

### Example: Research → Write → Summarize Pipeline
```
Specialist A: web_search → write_file("raw_research.md")
Specialist B: read_file("raw_research.md") → write_file("analysis.md")
Alpha Node:   read_file("analysis.md") → archive_to_vault("final_report.md")
```

- Files are isolated per cluster under `workspaces/<cluster-id>/`.
- All file paths are sandboxed — agents cannot escape their designated directory.
- `delete_file` always requires **Oversight Gate** approval.
