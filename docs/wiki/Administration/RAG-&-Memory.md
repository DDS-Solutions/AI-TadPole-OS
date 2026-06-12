---
title: "RAG & Memory"
tier: "2"
status: "verified"
version: "1.1.165"
last-verified: "2026-06-11"
commit: "b1c347b1"
network-badge: "none"
risk-tags:
  - "RISK: MEDIUM"
---

# 🧠 RAG & Memory

This page covers the local data intelligence layer that synchronizes your office files and databases with the agent's memory store. All indexing and retrieval happens on your local hardware.

---

## 1. Local Data Synchronization (`search_mission_knowledge`)

The data intelligence layer synchronizes your local folders and databases with the agent's memory store.

### 1.1 Multi-Factor Scoring (MFS)

Combines three scoring dimensions to find the most accurate records. Defined in [`rag_scoring.rs`](../server-rs/src/types/rag_scoring.rs):

| Factor | Weight | Description |
|--------|--------|-------------|
| **Vector Semantics** | `0.7` | Cosine similarity weight (`semantic_weight`) |
| **Project Relevance** | `0.2` | Boost added if the memory belongs to the active mission (`affinity_boost`) |
| **Document Recency** | `0.1` | Refresh rate weight decaying linearly over 48 hours (max age: 172800s) (`recency_weight`) |

### 1.2 Data Crawling

System sync workers automatically index changes in your mapped directories:

- **Sync Interval**: `SME_SYNC_INTERVAL_MINS` [RISK: MEDIUM] — default `30` minutes. Controls how often the data sync worker re-indexes mapped directories.
- **Deduplication Threshold**: `LANCEDB_DEDUPE_THRESHOLD` [RISK: MEDIUM] — default `0.9`. Similarity score above which documents are considered duplicates.
- **Drift Threshold**: `LANCEDB_DRIFT_THRESHOLD` [RISK: MEDIUM] — default `0.8`. Similarity score below which re-indexing is triggered.

---

## 2. Vector Memory (LanceDB)

> [!IMPORTANT]
> Vector memory requires the `vector-memory` [RISK: MEDIUM] build feature flag to be enabled at compile time. Without it, the engine falls back to keyword-based retrieval only.

When enabled, vectors are stored locally in `data/memory.lance/`.

### 2.1 Storage Architecture

| Component | Path | Description |
|-----------|------|-------------|
| **Mission Knowledge** | `data/memory.lance/` | Per-mission document embeddings |
| **Swarm Vault** | `ResourceHub::swarm_vault` | Global swarm-wide knowledge vault for cross-mission intelligence |
| **Knowledge Store** | `ResourceHub::knowledge_store` | Persistent cross-cluster Institutional Knowledge Store |

### 2.2 Compilation

**[Command] Build with Vector Memory**
```powershell
cd server-rs
cargo build --release --features vector-memory
```

→ *See [[Installation|§3.2]] for complete build instructions.*

---

## 3. Markdown SOP Enforcement

Place your business Standard Operating Procedures (SOPs) as markdown files in `data/workflows/` to force agents to follow strict execution rules.

### 3.1 Registration Flow

1. Create a markdown file (e.g., `contract_review_sop.md`) with numbered steps.
2. Place the file in the `data/workflows/` directory.
3. Open the dashboard in the browser and navigate to the **Settings** page.
4. Select the target virtual employee from the **[Table] Agent Registry**.
5. Locate the **[Card] Agent Capabilities** panel.
6. In the **[Selector] Workflows** list, toggle on your newly placed SOP workflow name (e.g. `contract_review_sop`).
7. Click the **[Button] Save Configuration** to apply.
8. The agent will follow the exact steps, checking each item in order.

### 3.2 Example SOP

```markdown
# Contract Review Checklist
1. Verify all party names are correct
2. Check payment terms match the agreed schedule
3. Confirm termination clause exists
4. Flag any non-standard liability provisions
5. Write findings to review_report.md
```

---

## 4. Sync User Interface

Configure and monitor the RAG indexing processes using the following visual indicators:

- **[Gauge] Sync Progress**
  - *Default state*: `0%` or `100%` (Idle).
  - *Visible location*: Mapped directories sidebar under the Workspaces page.
  - *Observable side effect*: Animates dynamically during background indexing crawls.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Force Re-index**
  - *Default state*: Enabled.
  - *Visible location*: Top-right corner of the workspace settings pane.
  - *Observable side effect*: Triggers a manual sync scan immediately, bypassing the `SME_SYNC_INTERVAL_MINS` interval.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Table] Indexed Sources**
  - *Default state*: Displays current directories, checksum state, and total files indexed.
  - *Visible location*: Center panel of the Workspace storage manager.
  - *Observable side effect*: Lists total bytes and files found.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

## 5. Code Graph & Symbol Intelligence

The engine maintains structural knowledge graphs for codebase awareness:

| Graph | Resource | Description |
|-------|----------|-------------|
| **Code Graph** | `ResourceHub::code_graph` | Graph of code relationships for RAG-enhanced tool search |
| **Symbol Graph** | `ResourceHub::symbol_graph` | Symbol-level knowledge graph for structural analysis |

These graphs enhance the agent's ability to understand file dependencies and locate relevant code symbols during development tasks.

---

## 6. Agent Scope & RAG Scope

The [[Glossary#rag-scope|RAG Scope]] defines the data ingestion context that limits an agent's retrieval:

- **File-level**: Agent can only access specific files in its `allowed_files` list.
- **Workspace-level**: Agent retrieves from all indexed documents in its assigned workspace.
- **Global-level**: Agent can access the swarm-wide knowledge vault (when enabled).

→ *See [[Virtual-Employees|§3]] for agent sandboxing and scope controls.*
→ *See [[Configuration|§7]] for LanceDB and sync interval settings.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](../GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: RAG-&-Memory)
