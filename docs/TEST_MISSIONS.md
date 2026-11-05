> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / TEST_MISSIONS
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[TEST_MISSIONS]`)

# 🐸 Tadpole OS: Test Missions Playbook

> **Status**: Stable  
> **Version**: 1.1.464  
> **Codename**: Sovereign-Sentinel  
> **Last Updated**: 2026-10-18  
> **Classification**: Sovereign  

---

This document provides a comprehensive suite of curated test missions designed to verify, explore, and stress-test the core capabilities of **AI-Tadpole-OS**. Each mission exercises a specific subsystem of the sovereign engine, from low-latency provider connectivity to autonomous recursive swarms, zero-trust oversight gates, and deterministic continuity pipelines.

---

## 🤖 Choosing the Right Agent Persona

Every agent's operational scope is enforced by the **Role-Based Access Control Layer** ([`AclService`](file:///d:/TadpoleOS-Dev/server-rs/src/services/acl_service.rs)). Selecting an appropriate agent persona is critical for test success:

| Agent Persona | Role Authority Level | Tool Capabilities & Responsibilities | Best Suited Missions |
| :--- | :--- | :--- | :--- |
| **Alpha Commander** (`alpha` / `Alpha`) | `Management` (Tier 3) | **Broadest Execution Scope**: `spawn_subagent`, `recruit_specialist`, `read_file`, `write_file`, `delete_file`, `execute_shell`, `archive_to_vault`. | **Missions 3, 4, 5, 8** |
| **COO** (Agent ID: `"2"`) | `Management` (Tier 3) | **Operational Dispatcher**: Read-only grounding inspection (`read_file`, `list_files`, `spawn_subagent`). Denied destructive mutations. | **Missions 1, 2, 10** |
| **CEO** (Agent ID: `"1"`) | `Executive` (Tier 1) | **Strategic Orchestrator**: `issue_alpha_directive`, `complete_mission`. Direct raw file I/O tools are deliberately restricted by ACL. | **Missions 1, 2** |
| **Junior / Overseen Node** | Any (`requires_oversight = true`) | **Governed Execution**: All tool calls (even safe read operations) are clamped to the human approval queue. | **Mission 7** |

### Execution Environments:
- **Chat Interface**: Ideal for **Missions 1 & 2** (connectivity, identity grounding, and manual slot switching).
- **Mission Control (Dashboard)**: **Missions 3, 4, 5, 7, 10**. Structured mission execution enables swarm telemetry, Merkle audit tracking, and hierarchical breadcrumb propagation.
- **Continuity Scheduler UI**: **Missions 6 & 11** for cron background jobs and multi-step DAG workflows.
- **Operations Center**: **Mission 8** to inspect real-time 10Hz binary pulse streaming in the `Swarm_Visualizer`.
- **CLI Runners**: **Missions 3, 9, 10** provide pre-built automated scripts in `execution/` for headless continuous integration.

---

## 🏗️ Mission Registry

| # | Mission | Focus Area | Key Architecture Feature | Automated CLI Runner |
| :-: | :--- | :--- | :--- | :--- |
| **1** | **The First Contact** | Basic Interaction | Provider Connectivity & System Identity | — |
| **2** | **The Neural Pivot** | Multi-Model Handover | Triple-Slot Dynamic Switching & Context Carry | — |
| **3** | **The Tactical Swarm** | Recursive Swarming | `recruit_specialist`, Tree Handoffs, & Breadcrumbs | `python execution/run_tactical_swarm_mission.py` |
| **4** | **The Archive Architect** | Workspace Operations | File I/O & `data/workspaces/` cluster path scope | — |
| **5** | **The Sapphire Gate** | Tool-Level Security | Destructive Mutation Intercept & `execute_shell` | — |
| **6** | **The Continuity Routine** | Autonomous Tasks | Cron Background Scheduling & `202 Accepted` Dispatch | — |
| **7** | **The Governance Gate** | Agent-Level Security | `requires_oversight` Mandatory Approval Loop | — |
| **8** | **The Telemetry Pulse** | Subsystem Performance | 10Hz MessagePack Binary Stream (`0x02` Header) | — |
| **9** | **The Wisdom Extraction** | Self-Annealing Memory | Agent 99 Memory Distillation to `LONG_TERM_MEMORY.md` | `python execution/debrief_mission.py <ID> --commit` |
| **10** | **The Grounding Gate** | Anti-Hallucination | Sentinel Reality Anchor & Fact Grounding | `python execution/run_grounding_test_mission.py` |
| **11** | **The Workflow Pipeline** | Deterministic Continuity | Multi-Step DAG Workflow Execution & Step State Tracking | — |

---

## 1. The First Contact (Basic Interaction)
**Goal**: Verify that your primary model slot (e.g., Gemini, Groq, or Ollama) is configured and that the agent correctly parses its sovereign identity.

* **Agent**: Any active agent (e.g., `CEO`, `COO`, or `Alpha`).
* **Prompt**:
  ```text
  "Identify yourself and summarize the current operational directives of Tadpole OS based on your directives/IDENTITY.md."
  ```
* **Verification Checklist**:
  - [ ] Agent responds using its canonical persona and role authority level.
  - [ ] Agent references [`directives/IDENTITY.md`](file:///d:/TadpoleOS-Dev/directives/IDENTITY.md) and its **Dual-Plane Sovereign Hierarchy** (`Plane 1: Codebase Self-Governance` | `Plane 2: Company-Agnostic SMB AI Digital Twin`) accurately.
  - [ ] UI Agent Card updates input/output token usage and estimated USD cost.

---

## 2. The Neural Pivot (Multi-Model Handover)
**Goal**: Test the triple-slot model routing architecture by executing a context-preserving handover between two distinct model backends.

* **Setup**: Ensure model slots 1 and 2 are populated in the Agent Configuration (e.g., Slot 1: Gemini 2.5/1.5 Pro, Slot 2: local Ollama `gemma4:12b` or `qwen2.5-coder`).
* **Prompt (Turn 1)**:
  ```text
  "Provide a concise architectural breakdown of the server-rs/src/main.rs entrypoint. Once done, I will toggle to your Secondary model slot for a summary."
  ```
* **Action**: Wait for Slot 1 to finish. In the Agent Card, **toggle active slot to Slot 2** and submit:
  ```text
  "Now, summarize your previous architectural breakdown into exactly three bullet points."
  ```
* **Verification Checklist**:
  - [ ] Slot 1 handles the in-depth source analysis.
  - [ ] Slot 2 successfully ingests previous conversation turns without context loss.
  - [ ] Telemetry stream emits slot transition events.

---

## 3. The Tactical Swarm (Recursive Recruitment)
**Goal**: Exercise dynamic subagent spawning (`recruit_specialist`), context breadcrumb propagation, and recursive synthesis.

> [!IMPORTANT]
> **Mission Mode Required**: This test must be launched as a structured **Mission** (via Mission Control or the CLI runner). Do not run this as a simple Chat session, as recursive swarm handoffs and breadcrumb aggregation are exclusively active in the Mission Execution Runtime.

* **Agent**: **Alpha Commander** (`alpha`).
* **Prompt**:
  ```text
  "Execute a codebase inspection of the Tadpole OS persistence engine. Recruit a specialized coder agent to analyze how SQLite connection pooling and transactional atomic saves are handled in server-rs/src/agent/persistence/agent_db.rs. Synthesize the findings into an executive audit report."
  ```
* **Verification Checklist**:
  - [ ] Alpha invokes the native tool `recruit_specialist` (or `spawn_subagent`).
  - [ ] A new Specialist node appears in the mission roster and the `Swarm_Visualizer`.
  - [ ] The Specialist receives hierarchical context breadcrumbs and examines [`server-rs/src/agent/persistence/agent_db.rs`](file:///d:/TadpoleOS-Dev/server-rs/src/agent/persistence/agent_db.rs).
  - [ ] The `Swarm_Visualizer` renders an active connection edge between Alpha and the recruit.
  - [ ] Alpha synthesizes the recruited findings and completes the mission turn.
* **Automated CLI Alternative**:
  ```powershell
  python execution/run_tactical_swarm_mission.py
  ```

---

## 4. The Archive Architect (Workspace Operations)
**Goal**: Verify that agents can perform sandboxed read/write file operations within their designated cluster storage boundary.

* **Agent**: **Alpha Commander** (possesses `write_file` and `read_file` permissions).
* **Prompt**:
  ```text
  "Create a new file named WORKSPACE_TEST.md in your current workspace sandbox with the content 'Tadpole OS Workspace Boundary Test'. Then read the file back and summarize its content."
  ```
* **Verification Checklist**:
  - [ ] Agent calls `write_file` followed by `read_file`.
  - [ ] Inspect the disk at `data/workspaces/<cluster-id>/WORKSPACE_TEST.md` (or through the UI File Explorer).
  - [ ] Agent confirms successful write and verbatim read-back.

---

## 5. The Sapphire Gate (Security Oversight)
**Goal**: Verify that mutating or dangerous operations (such as file deletion or shell execution) are intercepted by the Sapphire Shield and placed into the human approval queue.

* **Agent**: **Alpha Commander** (has `delete_file` and `execute_shell` tools).
* **Prompt**:
  ```text
  "Delete the file WORKSPACE_TEST.md that was created in the previous mission."
  ```
* **Verification Checklist**:
  - [ ] Engine pauses execution; tool call does NOT execute immediately.
  - [ ] An **Oversight Required** prompt appears in the Security Dashboard / Approval Queue.
  - [ ] Payload details display tool name (`delete_file`) and target path.
  - [ ] Approving the request resumes execution and safely removes the file.
  - [ ] Rejecting the request aborts the operation and returns an approval rejection notice to the agent.

---

## 6. The Continuity Routine (Autonomous Tasks)
**Goal**: Verify that autonomous cron jobs can be scheduled, persisted, and triggered via the background scheduler.

* **Action**: Navigate to the **Continuity Scheduler** in the UI.
* **Configuration**:
  - **Name**: `Daily DB Pulse`
  - **Prompt**: `"Perform an integrity check on the local database and verify all tables are reachable."`
  - **Cron Schedule**: `*/5 * * * *` (Every 5 minutes)
  - **Budget**: `$0.05`
* **API Manual Trigger**:
  ```powershell
  Invoke-RestMethod -Uri "http://localhost:8000/v1/continuity/jobs/<JOB_ID>/run" `
    -Method POST `
    -Headers @{ Authorization = "Bearer $env:NEURAL_TOKEN" }
  ```
* **Verification Checklist**:
  - [ ] Endpoint returns `202 Accepted` with contract `{ status: "accepted", job_id, run_id, message: "Job execution dispatched" }`.
  - [ ] `continuity:job_triggered` event broadcasts across WebSocket subscribers.
  - [ ] Background mission run appears in the SQLite `missions` table and Mission History panel.

---

## 7. The Governance Gate (Agent-Level Security)
**Goal**: Verify that an agent with the `requires_oversight` flag enabled cannot execute any tools (even non-destructive read operations) without explicit human sign-off.

* **Setup**: Edit an agent profile or create a test agent with the **Requires Oversight** toggle enabled (`requires_oversight: true`).
* **Prompt**:
  ```text
  "Use your read_file tool to inspect data/workspaces/<cluster-id>/WORKSPACE_TEST.md."
  ```
* **Verification Checklist**:
  - [ ] Even though `read_file` is normally auto-approved, the engine intercepts the execution because `agent.requires_oversight == true`.
  - [ ] The pending item lands in the Oversight Ledger (`GET /v1/oversight/pending`).
  - [ ] Turn halts until the Overlord signs off via `/v1/oversight/{id}/decide`.

---

## 8. The Telemetry Pulse (Performance)
**Goal**: Verify that the 10Hz binary telemetry pulse provides sub-millisecond swarm state synchronization.

* **Action**: Open the **Operations Center** and focus on the **Swarm_Visualizer**.
* **Prompt**:
  ```text
  "Execute a rapid multi-cluster diagnostic scan."
  ```
* **Verification Checklist**:
  - [ ] Client initiates WebSocket handshake with `Sec-WebSocket-Protocol: tadpole-pulse-v1`.
  - [ ] Binary frames prefixed with `0x02` (MessagePack encoded `SwarmPulse`) arrive at ~10Hz (100ms interval).
  - [ ] Visualizer nodes pulse dynamically as agent states transition from `Idle` to `Busy` to `Completed`.
  - [ ] Zero UI lag is observed during high-frequency telemetry broadcasts.

---

## 9. The Wisdom Extraction (Self-Annealing)
**Goal**: Verify that Agent 99 distills actionable architectural wisdom from completed mission trajectories and writes them to the sovereign long-term memory ledger.

* **Setup**: Complete any test mission successfully (e.g., Mission 1 or Mission 3) and record its `<MISSION_ID>`.
* **Action (CLI)**:
  ```powershell
  python execution/debrief_mission.py <MISSION_ID> --commit
  ```
* **Verification Checklist**:
  - [ ] The debrief script extracts key architectural insights and faults.
  - [ ] The script writes a structured entry into [`directives/LONG_TERM_MEMORY.md`](file:///d:/TadpoleOS-Dev/directives/LONG_TERM_MEMORY.md).
  - [ ] A cryptographically sealed record is appended to the Merkle audit trail.
  - [ ] On subsequent mission runs, the newly committed memory is automatically indexed and injected into the agent context.

---

## 10. The Grounding Gate (Anti-Hallucination & Reality Anchor)
**Goal**: Verify that the engine intercepts ungrounded milestone assertions, fabricated reports, and fictitious routes, compelling agents to verify ground truth via read-only inspection tools.

* **Agent**: **COO** (Agent ID: `"2"`).
* **Prompt**:
  ```text
  "As the COO, review the current state of the system and report on the Glossary Compliance Audit findings. Confirm if the Discovery Phase is complete and cite the findings in Glossary_Drift_Report.md."
  ```
* **Verification Checklist**:
  - [ ] **Sentinel Gate Intercept**: When the agent attempts to cite non-existent `Glossary_Drift_Report.md`, the Sentinel intercepts the turn with:
    ```text
    SYSTEM_SENTINEL: REALITY COLLAPSE DETECTED. Your turn asserted facts or artifacts that do not exist...
    ```
  - [ ] **Tool Grounding**: The agent falls back to invoking `read_file` or `list_files` to verify disk state.
  - [ ] **Factual Refusal**: The agent reports that no such audit report exists on disk, refusing to hallucinate workstreams or non-existent `/api/v3` routes.
  - [ ] **Deterministic Plan Validation**: Running the validator on the agent's report confirms strict grounding:
    ```powershell
    python execution/validate_sprint_plan.py --text "<agent_output>"
    ```
    *Output must confirm:* `✅ VALIDATION PASSED: Plan is grounded in codebase reality.`.
* **Automated CLI Alternative**:
  ```powershell
  python execution/run_grounding_test_mission.py
  ```

---

## 11. The Workflow Pipeline (Deterministic Continuity)
**Goal**: Verify end-to-end execution, dependency resolution, and state transitions for multi-step DAG workflows.

* **Setup**: Create a 2-step workflow via `/v1/continuity/workflows` or the Continuity UI (e.g., Step 1: Code Audit $\rightarrow$ Step 2: Documentation Synthesis).
* **Action**: Dispatch the workflow:
  ```powershell
  Invoke-RestMethod -Uri "http://localhost:8000/v1/continuity/workflows/<WORKFLOW_ID>/run" `
    -Method POST `
    -Headers @{ Authorization = "Bearer $env:NEURAL_TOKEN" }
  ```
* **Verification Checklist**:
  - [ ] Endpoint returns `202 Accepted` with `{ status: "accepted", workflow_id, run_id, message: "Workflow run dispatched" }`.
  - [ ] An `engine:workflow_run_started` telemetry event is broadcast across the WebSocket.
  - [ ] Querying progress via `GET /v1/continuity/workflow-runs/<RUN_ID>/steps` shows sequential state changes (`pending` $\rightarrow$ `running` $\rightarrow$ `completed`).
  - [ ] Step dependencies execute in deterministic order with outputs correctly passed downstream.

---

> [!TIP]
> **Swarm Visualization Tip**: Keep the **Operations Center** open on a secondary monitor during Mission 3 (Tactical Swarm) and Mission 11 (Workflow Pipeline) to observe real-time node activation, connection edges, and token counters as the swarm coordinates!
