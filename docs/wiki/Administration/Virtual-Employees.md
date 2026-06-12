---
title: "Virtual Employees"
tier: "2"
status: "verified"
version: "1.1.165"
last-verified: "2026-06-11"
commit: "b1c347b1"
network-badge: "none"
risk-tags:
  - "RISK: MEDIUM"
---

# 👥 Virtual Employees

This page covers the creation and configuration of custom virtual employee profiles within Tadpole OS. Virtual employees are AI agents assigned to specific roles, departments, and projects.

---

## 1. Agent Configuration Panel (`AgentConfigPanel`)

The primary interface for designing custom virtual employee profiles.

### 1.1 Cognition Tab

- **[Tab] Cognition Tab**
  - *Default state*: Active/focused slot.
  - *Visible location*: Top-left header of the `AgentConfigPanel`.
  - *Observable side effect*: Focuses the active model slot configurations (Primary, Secondary, Tertiary).
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Toggle] Long-Term Memory Toggle**
  - *Default state*: Enabled (`true`).
  - *Visible location*: Memory section under the Cognition tab.
  - *Observable side effect*: Enforces saving agent conversation context into LanceDB vector memory.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Input] Working Memory**
  - *Default state*: Ephemeral JSON scratchpad (`{}`).
  - *Visible location*: Text area block in the memory panel.
  - *Observable side effect*: Manually edits the agent's short-term context state.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 1.2 Governance Tab

- **[Input] Budget Cap**
  - *Default state*: `1.0` USD (or defined by `DEFAULT_AGENT_BUDGET_USD` [RISK: MEDIUM]).
  - *Visible location*: Security governance section.
  - *Observable side effect*: Restricts the agent's total spending per active mission.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Toggle] Requires Oversight**
  - *Default state*: Enabled (`true`).
  - *Visible location*: Security governance section.
  - *Observable side effect*: Redirects all write-tool executions to the Oversight Queue.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 1.3 Inference Tuning

- **[Selector] Model Assignment**
  - *Default state*: First available Ollama model.
  - *Visible location*: Model slot dropdown selector.
  - *Observable side effect*: Switches the active LLM provider and model ID for the active slot.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Slider] Temperature**
  - *Default state*: `0.7` (or default of the model).
  - *Visible location*: Mid panel of the Cognition configurations.
  - *Observable side effect*: Sets model creativity slider. Range `0.00` to `2.00` with `0.05` steps.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Input] Max Tokens**
  - *Default state*: `4096` (or model default).
  - *Visible location*: Next to the temperature slider.
  - *Observable side effect*: Restricts response length per API call.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Input] System Prompt**
  - *Default state*: Standard agent directive template.
  - *Visible location*: System prompt textarea block.
  - *Observable side effect*: Defines the role identity instructions for the agent model.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 1.4 Reasoning Engine (Mythos)

Tune the agent's internal monologue and reasoning settings (verified in [`ModelSlotConfig.tsx:L134-164`](../src/components/agent-config/ModelSlotConfig.tsx#L134-L164)):

- **[Slider] Reasoning Depth**
  - *Default state*: `1` turn.
  - *Visible location*: Mythos parameters box.
  - *Observable side effect*: Sets the maximum reasoning loop depth. Range `1` to `16` turns with step `1`.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Slider] ACT Halting Threshold**
  - *Default state*: `90%` (`0.90`).
  - *Visible location*: Mythos parameters box.
  - *Observable side effect*: Defines the confidence level below which the agent continues reasoning. Range `0%` to `100%` (`0.00` to `1.00`) with step `0.01`.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

## 2. Agent Authority Hierarchy

Agents operate within a strict authority hierarchy (defined in [[Glossary#roleauthoritylevel|RoleAuthorityLevel]]):

| Level | Role | Capabilities |
|-------|------|-------------|
| **Executive** | CEO, Overlord | Strategic oversight and delegation across all departments |
| **Management** | [[Glossary#department-lead|Department Lead]] (formerly Alpha Node) | Tactical coordination, sub-agent recruitment, report compilation |
| **Specialist** | Standard task executor | Individual tool execution within assigned scope |
| **Observer** | Read-only auditor | Monitoring and reporting only, no write operations |

---

## 3. Agent Scope & Sandboxing

Each agent operates within strict boundaries:

- **Workspace Root**: The `workspace_root` field in [[Glossary#runcontext|RunContext]] defines the sandbox directory for file operations.
- **Filesystem Adapter**: A zero-trust adapter (`FilesystemAdapter`) restricts path traversals to prevent escaping the sandbox.
- **Allowed Files**: Optional whitelist constraints can limit which specific files an agent can access.
- **Safe Mode**: When `safe_mode` is enabled, hazardous commands/actions require explicit user confirmation.

---

## 4. Skills & Workflows

Agents can be equipped with custom capabilities:

- **Skills**: Custom script files registered to the agent. Skill manifests are stored in `data/skills/*.json` and validated by [[Parity-Guard|Parity Guard]].
- **Workflows**: SOP (Standard Operating Procedure) files placed in `data/workflows/` that enforce step-by-step execution rules.
- **MCP Tools**: External tools exposed by active Model Context Protocol servers.

→ *See [[RAG-&-Memory|§3]] for how SOPs are registered and enforced.*
→ *See [[Approvals-&-Quotas|§1]] for oversight queue configuration per agent.*

---

## 5. Swarm Depth & Delegation

The [[Glossary#department-lead|Department Lead]] can recruit sub-agents to handle specialized subtasks. Delegation is controlled by:

| Setting | Default | Description |
|---------|---------|-------------|
| `MAX_SWARM_DEPTH` [RISK: MEDIUM] | `5` | Hard limit on recursive agent-spawning depth. |
| `MAX_AGENTS` [RISK: MEDIUM] | `50` | Registry limit for maximum concurrent agent identities. Code default is `50` (defined in [state/mod.rs:L547-551](../server-rs/src/state/mod.rs#L547-L551)). `.env.example` ships `100`. |

> [!TIP]
> **Recommended Max Depth**: 3 for standard office hardware. Deeper delegation increases memory usage and latency. The absolute maximum is 5.

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](../GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Virtual-Employees)
