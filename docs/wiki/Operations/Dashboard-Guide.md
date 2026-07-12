---
title: "Dashboard Guide"
tier: "1"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "optional"
risk-tags:
  - "RISK: LOW"
---

# 🖥️ Dashboard Guide

The Tadpole OS dashboard is a high-performance, multi-tab web interface for orchestrating your virtual employee swarm. It is designed for **multi-monitor office environments** and supports detachable portals for distributed oversight.

---

## 1. Multi-Tab Business Interface

The primary navigation system allows for multi-context orchestration within a single browser window.

> [!NOTE]
> **Runtime Governance Anchor**: All persistence paths (workspaces, local databases, and temporary caches) resolve from the backend `AppState.base_dir`, ensuring local-dev and server-mounted office networks use the same folder root.

### 1.1 The Multi-Tab Bar

Located permanently at the top of the viewport, the Multi-Tab Bar manages your active workspaces:

- **Dynamic Contexts**: Tabs represent individual operational sectors (Dashboard, Projects, Team Graph, Settings).
- **Context Preservation**: Switching tabs preserves the state of each page, enabling seamless multitasking.
- **Detachable Portals**: Click the **External Link** icon revealed on hover to pop out any tab into a native browser window for dual-monitor layouts.
- **Header Sync**: The global `PageHeader` automatically updates its action controls to match the active tab.

**Interactive Elements:**

- **[Button] Detach/Recall Tab**
  - *Default state*: Embedded (Not Detached).
  - *Visible location*: Right-hand side of individual tab headers in the Multi-Tab Bar when hovered.
  - *Observable side effect*: Spawns a new browser window hosting the page, replacing the embedded view, and changes the icon to a minimize/recall icon.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Close Tab**
  - *Default state*: Open.
  - *Visible location*: Far right of hovered tab headers in the Multi-Tab Bar.
  - *Observable side effect*: Closes the selected tab and shifts focus to the next available active tab.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

### 1.2 Unified Dashboard Header (`PageHeader`)

A persistent command surface at the top of the page:

- **Engine Status**: Real-time local connection status (**🟢 ONLINE**), synchronized via the `useEngineStatus` hook (defined in [`use_engine_status.ts:L42`](../../../src/hooks/use_engine_status.ts#L42)).
- **Page-Specific Actions**: Direct action shortcuts (e.g., "Start New Project" when on the Projects tab).
- **Core Metrics**: High-level telemetry tailored to the active operation.

### 1.3 Tab Quick-Reference

| Tab Index | Tab Name | Purpose |
|-----------|----------|---------|
| 1 | Dashboard | Central operations hub with engine health metrics |
| 2 | Org Chart | Visual force-graph of your virtual employee hierarchy |
| 3 | Voice Interface (Standups) | Voice-driven team coordination portal |
| 4 | Workspaces | Manage project folders, documents, and file assets |
| 5 | Docs | Embedded documentation and knowledge bases |
| 6 | Settings | Provider config, agent profiles, system parameters |

**Keyboard Shortcuts:**
- `[Shortcut: Ctrl+K]` or `[Shortcut: Ctrl+/]` — Toggle Command Palette (cited in [`useLayoutNavigation.ts:L51-83`](../../../src/hooks/layout/useLayoutNavigation.ts#L51-L83)).
- `[Shortcut: 1-6]` — Quick-navigate to tabs when input fields are out of focus.

---

## 2. Activity Lineage Stream (`Lineage_Stream`)

A real-time telemetry sidebar that visualizes the propagation of data and instructions through your virtual organization.

- **Real-time Feed**: Scrollable timeline of every agent event, tool execution, and system message.
- **Chain of Command Tracker**: Visualizes the path an instruction took from the [[Glossary#department-lead|Department Lead]] down to the final terminal agent.
- **Payload Panel**: Displays the raw text content of individual agent thoughts and actions.

**Interactive Controls:**

- **[Sidebar] Resizable Boundary**
  - *Default state*: Fixed standard width.
  - *Visible location*: Outer border of the Lineage Stream drawer.
  - *Observable side effect*: Expands or contracts the sidebar panel width.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Card] Event Node**
  - *Default state*: Unselected.
  - *Visible location*: Scrolling logs within the stream.
  - *Observable side effect*: Triggers and opens the high-density Cinematic Depth View overlay.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Overlay] Cinematic Depth View**
  - *Default state*: Closed (`false`).
  - *Visible location*: Center-screen modal overlay.
  - *Observable side effect*: Displays the selected agent's full parameters, outputs, and JSON payloads.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

## 3. Swarm Visualizer & Neural Map

Tadpole OS provides two high-performance force-directed graph visualizers:

### 3.1 Swarm Visualizer (`Swarm_Visualizer`)
Exposes live virtual employee interactions, parent-child hierarchies, and task flow tracking. It pulses in real-time at 10Hz using system telemetry streams.

### 3.2 Neural Map / Knowledge Graph (`KnowledgeGraph`)
Exposes structural codebase references and semantic knowledge networks. Accessible via the **Neural Map** sidebar tab.
- **Visual Mode HUD Toggle**: Switch between **Codebase Symbols** (Tree-sitter AST nodes, files, methods, dependencies) and the **Semantic OKF Graph** (IKS knowledge assets aligned with the Open Knowledge Format).
- **Monochromatic Zinc Aesthetic**: Adheres to the strict Zinc theme scale; color variations indicate entity kinds, and thin Lucide icons denote concept types (e.g. database icon for table entries).
- **Semantic Health Highlights**: Reserves vibrant colors for live states:
  - **🟢 Verified Node**: Highlighting live status.
  - **⚪ Active Node**: Focus ring outline.
  - **🟡 Expiring Node**: Warning dot indicator.
  - **🔴 Broken Link**: Red border indicating target reference config mismatch.
- **BFS Pathfinder**: Computes and traces shortest call or dependency paths between symbols.
- **Navigation Stack**: Browser-style Back/Forward stack controls to walk through nodes.

---

→ *See [[Cost-Monitoring|§1]] for token cost tracking integrated into the dashboard.*
→ *See [[Kill-Switches|§1.2]] for emergency controls accessible from the dashboard header.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](../GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Dashboard-Guide)


[//]: # (Metadata: [wiki-ops])
