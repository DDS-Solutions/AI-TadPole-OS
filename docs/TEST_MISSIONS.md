# 🐸 Tadpole OS: Test Missions Playbook

This document provides a set of curated "Small Missions" to help you verify and explore the core skills of **Tadpole OS**. Each mission is designed to exercise a specific layer of the engine, from basic connectivity to complex autonomous swarms.

---

## 🤖 Choosing the Right Agent

While you can technically use any agent for these missions, their success depends on the **Skills** assigned to them. 

> [!IMPORTANT]
> **Recommendation**: For the best results, use the **CEO (Agent of Nine)** or a high-level **Alpha** agent. These agents typically have the broadest set of permissions (Skills) required for all missions.

### Environment Selection:
- **Missions 1 & 2**: **Chat-Ready**. These can be run directly in the Chat interface to test basic connectivity and model slot switching.
- **Mission 3 (Tactical Swarm)**: **Mission Mode Required**. This MUST be run as a structured Mission to exercise the recursive recruitment, context breadcrumbs, and swarm hierarchy logic.
- **Missions 4 & 5 (Workspace/Security)**: Recommended as **Missions** to ensure Audit Trail visibility, but simple CRUD can be tested in Chat if skills are granted.

---

## 🏗️ Mission Registry

| Mission | Focus | Key Feature Tested |
| :--- | :--- | :--- |
| **1. The First Contact** | Basic Interaction | Provider Connectivity & Identity |
| **2. The Neural Pivot** | Multi-Model Handover | Triple-Slot Switching |
| **3. The Tactical Swarm** | Recursive Swarming | `recruit_specialist` & Hierarchy |
| **4. The Archive Architect** | Workspace Operations | File I/O & Cluster Sandboxing |
| **5. The Sapphire Gate** | Security Oversight | Human-in-the-loop Approval |
| **6. The Continuity Routine** | Autonomous Tasks | Cron-Driven Background Execution |

---

## 1. The First Contact (Basic Interaction)
**Goal**: Verify that your primary model (e.g., Gemini) is correctly configured and the agent understands its identity.

*   **Prompt**: `"Identify yourself and summarize the current state of Tadpole OS based on your IDENTITY.md."`
*   **Verification**:
    - [ ] Agent responds using its core identity.
    - [ ] Agent references `IDENTITY.md` accurately.
    - [ ] UI shows correct Token/TPM usage.

## 2. The Neural Pivot (Multi-Model Handover)
**Goal**: Test the "Triple-Slot" architecture by forcing a context switch between models.

*   **Setup**: Ensure you have model slots 1 and 2 configured (e.g., Slot 1: Gemini 1.5 Pro, Slot 2: Llama 3/Ollama).
*   **Prompt**: `"Provide a complex architectural analysis of the server-rs/src/main.rs file. Once done, I will ask you to switch to your Secondary slot for a summary."`
*   **Action**: Wait for analysis, then **manually toggle to Slot 2** in the Agent Card and say: `"Now, summarize your previous analysis in three bullet points."`
*   **Verification**:
    - [ ] Slot 1 handles the heavy analysis.
    - [ ] Slot 2 successfully accesses the previous turn's context for the summary.

## 3. The Tactical Swarm (Recursive Recruitment)
**Goal**: Exercise the `recruit_specialist` tool and hierarchical context injection (Breadcrumbs).

> [!IMPORTANT]
> **Mission Mode Required**: This test **must** be run as a structured **Mission** (via Mission Control or the Dashboard). Do not run this as a simple Chat, as the swarm handoff and context breadcrumb propagation logic are specifically part of the Mission Execution runtime.

*   **Prompt**: `"Research the Tadpole OS persistence layer and recruit a specialist to explain how SQLite is used in persistence.rs."`
*   **Verification**:
    - [ ] Alpha agent calls `recruit_specialist`.
    - [ ] A new Specialist agent appears in the "Recruits" list.
    - [ ] The Specialist provides a technical explanation of `persistence.rs`.
    - [ ] The Alpha synthesizes the Specialist's findings into a final report.

## 4. The Archive Architect (Workspace Operations)
**Goal**: Verify that agents can read and write files within their cluster sandbox.

*   **Prompt**: `"Create a new file called WORKSPACE_TEST.md in your current workspace with the text 'Tadpole OS Workspace Test'. Then, read it back and summarize its contents."`
*   **Verification**:
    - [ ] check `workspaces/<cluster-id>/WORKSPACE_TEST.md` on your disk.
    - [ ] Agent confirms the file was written and read correctly.

## 5. The Sapphire Gate (Security Oversight)
**Goal**: Test the "Sapphire Shield" manual approval mechanism.

*   **Setup**: Use an agent with the `delete_file` or `shell:execute` skill granted.
*   **Prompt**: `"Attempt to delete the file WORKSPACE_TEST.md that we created in the previous mission."`
*   **Verification**:
    - [ ] The engine pauses execution.
    - [ ] An "Oversight Required" alert appears in the Security Dashboard.
    - [ ] You (The Overlord) must Approve or Reject the action.

## 6. The Continuity Routine (Autonomous Tasks)
**Goal**: Set up a background mission that runs without manual prompting.

*   **Action**: Go to the **Continuity Scheduler** in the UI.
*   **Configuration**:
    - **Mission**: `"Perform a health check on the local database and log the result to health.log"`
    - **Schedule**: `*/5 * * * *` (Every 5 minutes)
    - **Budget**: `$0.05`
*   **Verification**:
    - [ ] check the SQLite `missions` table or `Mission History` UI for background entries.
    - [ ] Verify `health.log` is created/updated in the workspace.

---

> [!TIP]
> **Pro-Tip**: Use the **Neural Map** visualization during Mission 3 (Tactical Swarm) to watch the connection traces light up as the Alpha recruits its specialists!
