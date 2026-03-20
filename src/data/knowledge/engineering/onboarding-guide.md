# Tadpole OS: Unified Onboarding Protocol

Welcome to the command center. This guide will walk you through setting up your first Mission Cluster with a dedicated Alpha Node and a specialized agent swarm.

## Step 1: Initialize Persistence & NeuralVault
The engine now uses a persistent SQLite foundation.
1. **Database Check**: On first boot, the engine migrates mission history and agent registries into `tadpole.db`.
2. **Settings**: Verify the **Engine API URL** is set to `http://localhost:8000`.
3. **NeuralVault**: Input your **Google Gemini API Key**. The OS now uses this key to authorize sub-agent swarming and external adapters.
4. Click **Save Changes**. Registry v2 will persist these settings to the database immediately.

## Step 2: Establish a Mission Cluster
Missions are the high-level objectives that guide your agent swarm.
1. Navigate to **Mission Management** (Target icon 🎯 in the sidebar).
2. Click the **NEW MISSION** button at the top of the sidebar.
3. Enter a descriptive **Mission Name** (e.g., "Operation Sky-Net").
4. Select a **Department** (e.g., Engineering) and click **CREATE**.
5. **Set the Objective**: In the **Operational Objective** field, describe the specific end-goal for the cluster.
   *   *Example: "Optimize the neural feedback loop to reduce latency by 15%."*
6. **Activate the Mission**: Click the **Zap (⚡)** icon on your new cluster card. The UI will shift to an emerald theme, indicating this mission is now the global command focus. *Note: Only one mission can be active at a time.*

## Step 3: Swarm Allocation
Now, assemble your team of three agents.
1. Select your active mission from the sidebar.
2. Under **"Assign New Agents"**, locate 3 available agents.
3. Click the **Plus (+)** icon on each agent card to move them into the cluster.

## Step 4: Designate the Alpha Node
Every cluster requires a leader to coordinate the swarm.
1. In the **"Team Collaborators"** section, look at your 3 assigned agents.
2. Hover over the **Agent Initial (e.g., "A")** on the left of an agent card.
3. You will see a **Ghost Crown** appear. Click it to promote that agent.
4. The agent is now the **Alpha Node**. A permanent **Amber Crown** icon will now appear next to their name across the entire OS (Ops Dashboard, Mission Management, and Hierarchy).

## Step 5: Verify Intelligence Sync
1. Navigate to the **Hierarchy** page (Layout icon 📊).
2. You will see your active mission cluster visualized as a dedicated command chain.
3. **Active Highlighting**: Because the mission is active, the entire chain will pulse with an emerald glow and show an **"ACTIVE" status badge**.
4. **Alpha Recognition**: The Alpha Node will be positioned at the top of the cluster, featuring the unique Crown icon.

## Step 6: Swarm Ignition & Nested Intelligence
1. Return to the **Mission Management** dashboard.
2. Click **RUN MISSION**. This initializes a persistent mission entry in the SQLite history.
3. **Nested Spawning**: High-level agents (Alpha Nodes) can now recruit specialists via the `spawn_subagent` tool. These sub-agents are ephemeral and operate within the same mission context.
4. **Context Bus**: Agents share findings using the `share_finding` tool. All shared context is automatically injected into the prompts of other swarm members, enabling collective intelligence without prompt bloat.
5. **Oversight**: Switch to the **Oversight Dashboard** (🛡️) to monitor recursive tool calls. Every call is tagged with its parent `mission_id` for perfect traceability.

**Mission Ready.** Your autonomous swarm is now operational.

## Step 7: Surface Adapters & Connectivity
The "Surface" layer connects the swarm to the physical world and external apps.
1. **Local Vault (Obsidian)**: Agents with the `archive_to_vault` tool can append mission reports and findings to local `.md` files in the `server-rs/vault/` directory.
2. **Secure Notifier (Discord)**: Set your `DISCORD_WEBHOOK` in the `.env` to allow agents to broadcast mission-critical alerts.
3. **Audit Trail**: Every adapter action is logged to the persistent `oversight_log` for security reviews.

### Common Terminal Commands
*   `swarm sync --active`: Synchronizes all nodes in the active mission cluster.
*   `agent capability --scan`: Prompts the Alpha Node to reassess swarm strengths.
*   `mission status`: Returns a neural-link telemetry report.

### Sample Objectives
*   **DevOps**: "Automate scaling of edge-compute nodes to maintain 99.9% uptime."
*   **Engineering**: "Refactor legacy telemetry hooks into a unified event-bus architecture."
*   **Security**: "Execute a red-team simulation against the primary identity-provider."
