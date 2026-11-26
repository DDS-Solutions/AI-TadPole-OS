---
title: "Chat & Voice"
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

# 💬 Chat & Voice

☁️ REQUIRES NETWORK

Tadpole OS provides two primary communication interfaces for directing your virtual employees: a text-based **Business Chat Portal** (`SovereignChat`) and a voice-driven **Voice Interface** (`VoiceClient` / `Standups`).

---

## 1. Business Chat Portal (`SovereignChat`)

The primary natural language interface for issuing directives to your virtual departments. It supports granular audience targeting:

- **Agent Scope** ([[RAG Scope|Glossary#rag-scope]]): 1:1 communication with a selected virtual employee. Context is limited to that agent's specific role memory.
- **[[Department Scope|Glossary#department-scope]]**: Department-wide instructions (e.g., `#marketing`).
- **Global Scope** ([[Swarm Pulse|Glossary#swarm-pulse]]): Broadcasts messages to all active agents.

### 1.1 Command Syntax

| Pattern | Scope | Example |
|---------|-------|---------|
| Standard input (`Enter`) | Active scope | `Summarize the Q2 revenue report.` |
| `@AgentName <Message>` | Targeted agent | `@BookkeeperBot Reconcile this month's invoices.` |
| `#DepartmentName <Message>` | Department broadcast | `#marketing Draft three social media posts for the product launch.` |

### 1.2 Interactive Elements

- **[Selector] 3-Mode Execution Selector (`Chat` · `Plan` · `Swarm`) & `Launch Swarm` Handoff**
  - *Default state*: `'swarm'` (`execution_mode`, synchronized with `is_safe_mode`).
  - *Visible location*: Above the input field in `SovereignChat` and below the target selector in `Standups`.
  - *Observable side effect*:
    - **`Chat`**: Lightweight conversational brainstorming — zero tools, no `mission_history` row, stays on Slot 1 (unless Privacy Shield enforces a local model).
    - **`Plan`**: Read-only research — read-only tools only, no `mission_history` row, retains RAG memory.
    - **`Swarm`**: Full tracked Mission lifecycle (`mission_history`, Flight Recorder dossier, Conductor DAG, Oversight Gate, Agent 99 debrief).
    - **`Launch Swarm`**: Appears in `Chat`/`Plan` after at least one user/agent turn; switches to `Swarm` and starts a tracked Mission carrying the formatted brainstorm thread via `recent_findings`.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Toggle] Safe Execution**
  - *Default state*: `false` (`is_safe_mode`, synced with `execution_mode !== 'swarm'`).
  - *Visible location*: Bottom command bar of the `SovereignChat` panel.
  - *Observable side effect*: Quick-toggles between `Chat` (tools forbidden) and `Swarm` (active Mission execution). Legacy payloads with `safe_mode: true` and no `execution_mode` default to `plan`.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Detach Interface**
  - *Default state*: Embedded (Not Detached).
  - *Visible location*: Top-right header of the `SovereignChat` panel.
  - *Observable side effect*: Spawns a new native browser portal window containing the chat widget, retaining full state synchronization.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Minimize Dock**
  - *Default state*: Maximized (Open).
  - *Visible location*: Top-right header of the `SovereignChat` panel (minimize icon).
  - *Observable side effect*: Collapses the chat panel into a floating circular button in the bottom-right corner of the viewport.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Selector] Scope Tab**
  - *Default state*: `'agent'` (Active scope).
  - *Visible location*: Sub-header row of the `SovereignChat` panel.
  - *Observable side effect*: Swaps active context filtering and targets between Agent, Department (Cluster), and Global (Swarm) scopes.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Breadcrumb] Lineage Path**
  - *Default state*: `'Overlord'` root.
  - *Visible location*: Directly below the scope selector in `SovereignChat` when the Agent tab is active.
  - *Observable side effect*: Visualizes parent-to-child delegation lines for the active virtual employee.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Input] Message Field**
  - *Default state*: Empty string (`""`).
  - *Visible location*: Bottom portion of the `SovereignChat` panel.
  - *Observable side effect*: Holds draft prompt content.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Send**
  - *Default state*: Enabled (if message field is not empty).
  - *Visible location*: Bottom-right of the `SovereignChat` input card.
  - *Observable side effect*: Submits the message to the active agent execution loop.
  - *Keyboard shortcut*: `Enter` key (without Shift) triggers submit.
  - *Missing in build*: Always present.

- **[Button] Attach File**
  - *Default state*: Enabled.
  - *Visible location*: Left of the text input area in the bottom toolbar.
  - *Observable side effect*: Opens file browser to upload documents to the active workspace.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 1.3 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `[Shortcut: Ctrl+K]` or `[Shortcut: Ctrl+/]` | Toggle Command Palette |
| `[Shortcut: 1-6]` | Quick-navigate to tabs (when input fields are out of focus) |

> [!NOTE]
> Keyboard shortcuts cited in [`useLayoutNavigation.ts:L51-83`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/src/hooks/layout/useLayoutNavigation.ts#L51-L83).

### 1.4 Swarm Visualizer Integration

The `SovereignChat` is integrated with the visual **Team Graph** (rendered by `Swarm_Visualizer`). Clicking any agent node on the graph automatically focuses that agent's project context:

- **[Card] Agent Node**
  - *Default state*: Active and selectable.
  - *Visible location*: Central canvas of the Swarm Visualizer.
  - *Observable side effect*: Selects that agent and opens its dedicated text context in `SovereignChat` for direct 1:1 interaction.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

→ *See [[Vault-&-Credentials]] for vault security details.*

---

## 2. Voice Interface (`VoiceClient` / `Standups`)

☁️ REQUIRES NETWORK *(for cloud-based speech-to-text providers; fully local with `neural-audio` feature flag)*

A voice-driven extension providing hands-free inputs and a dedicated "Standup Meeting" portal for team coordination.

> [!IMPORTANT]
> Local offline Whisper STT/VAD requires compiling with `--features neural-audio` (`WHISPER_MODEL_PATH`, `VAD_MODEL_PATH`). Cloud Live Voice (`GET /v1/engine/live-voice`, Gemini Live proxy) is available without `neural-audio`, strictly requires `Sec-WebSocket-Protocol: bearer.<token>` pre-auth, is budget-metered (`system:live_voice`, 100 MB session cap), and is **blocked with `403 Forbidden` whenever Privacy Shield (`PRIVACY_MODE`) is active** (`server-rs/src/routes/ws/live.rs`). See [[Configuration]] for build feature flags.

### 2.1 Core Interface Elements

- **Status Header**: Displays connection state (e.g., `SECURE LIVE CHANNEL` vs `Ready for Voice`).
- **Target Selection Matrix**: A control box for defining voice handshake destinations.
- **3-Mode Execution Selector (`Chat` · `Plan` · `Swarm`)**: Shared with `SovereignChat` so voice standups can brainstorm in `Chat`/`Plan` (ephemeral, no Mission History row) and promote the spoken thread to a tracked Mission via **`Launch Swarm`**.
- **Volume Visualizer**: Renders active microphone volume levels.

### 2.2 Connection Controls

- **[Button] Start/End Channel**
  - *Default state*: Inactive (`'idle'`).
  - *Visible location*: Center console of the `VoiceClient` pane.
  - *Observable side effect*: Opens or terminates local microphone streaming buffers, updating state metrics to live recording.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Local Mic Mute**
  - *Default state*: Unmuted (`false`).
  - *Visible location*: Next to the connection button on the command bar.
  - *Observable side effect*: Mutes microphone capturing lines without severing the audio interface connection.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Gauge] Volume Visualizer**
  - *Default state*: `0` (no input).
  - *Visible location*: Center console of the `VoiceClient` pane.
  - *Observable side effect*: Renders audio input frequency lines.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Toggle] Agent Node**
  - *Default state*: Selected if directing a single agent.
  - *Visible location*: Target Selection Matrix.
  - *Observable side effect*: Limits voice transmission to a specific agent's input stream.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Toggle] Workspace Cluster**
  - *Default state*: Unselected.
  - *Visible location*: Target Selection Matrix.
  - *Observable side effect*: Broadcasts the speech inputs to all agents inside the selected directory.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Selector] Target Dropdown**
  - *Default state*: First active agent.
  - *Visible location*: Target Selection Matrix.
  - *Observable side effect*: Selects the specific agent or cluster to bound the conversation.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 2.3 Live Transcript Log

- **Identity Attribution**: Color-coded identifiers showing speakers (`U` for User, `A` for Agent).
- **Active Telemetry**: Displays `REC` (Recording) vs `IDLE` status and real-time "Agent X is speaking..." feedback.

### 2.4 Network Data Path (Voice)

| Data Sent | Endpoint | Trigger | Notes |
|-----------|----------|---------|-------|
| Live Voice bidirectional audio | `GET /v1/engine/live-voice` (WS proxy to `generativelanguage.googleapis.com`) | User starts Live Voice channel | Requires `Sec-WebSocket-Protocol: bearer.<token>` pre-auth; **blocked (`403 Forbidden`) when Privacy Shield is active** |
| Audio transcription (STT) | `POST /v1/engine/transcribe` | Speech-to-text request | Uses local Whisper when built with `neural-audio`, or configured STT provider |
| Text-to-Speech (TTS) | `POST /v1/engine/speak` | Voice playback request | Uses local Piper (`PIPER_MODEL_PATH`) or configured TTS provider |

> [!WARNING]
> Live Voice (`GET /v1/engine/live-voice`) streams audio to Google Gemini Live unless blocked by Privacy Shield (`PRIVACY_MODE=true`). For local-only speech processing, build with `--features neural-audio` and configure `WHISPER_MODEL_PATH`, `VAD_MODEL_PATH`, and `PIPER_MODEL_PATH`.

→ *See [[LLM-Providers]] for provider configuration.*
→ *See [[Network-Boundaries]] for network egress rules.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Chat-&-Voice)


[//]: # (Metadata: [wiki-ops])
