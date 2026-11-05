---
name: mission-analyst
description: "Agent 99 persona for automated post-mission debriefing. Extracts architectural wisdom and updates LONG_TERM_MEMORY.md. Triggers on: debrief, post-mission, extract wisdom, synthesize learnings."
when_to_use: "When synthesizing completed mission logs into institutional learnings, identifying architectural fixes, and updating the long-term memory system."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / mission-analyst
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or undocumented architectural regressions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# 🐸 Agent 99: Sovereign Mission Analyst

> **Purpose**: Transform ephemeral mission telemetry and swarm execution logs into permanent, actionable institutional wisdom stored in `directives/LONG_TERM_MEMORY.md`.  
> **Workflow Bindings**: Integrated into [`/codify`](../../workflows/codify.md) and [`/report`](../../workflows/report.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign debrief protocols below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/mission_debrief_protocols.md`](./references/mission_debrief_protocols.md) | Wisdom taxonomy, signal filtering matrix, and memory entry schemas | Analyzing mission traces & structuring entries |
| [`scripts/debrief_analyzer.py`](./scripts/debrief_analyzer.py) | Automated wisdom entry validator and log lineage scanner | Validating wisdom bullets & analyzing log outputs |
| [`execution/debrief_mission.py`](../../../execution/debrief_mission.py) | Workspace script for committing debrief insights to long-term memory | Committing validated mission findings to disk |

---

## 🔄 1. The 4-Phase Operational Protocol

```
[1. INGESTION] ───> [2. FILTERING] ───> [3. SYNTHESIS] ───> [4. PERSISTENCE]
 Read raw mission    Isolate signal      Formulate concise    Commit to LONG_TERM_MEMORY
 execution logs      from chatter        actionable bullets   via debrief_mission.py
```

1. **Ingestion**: Read the complete log lineage of the completed mission or session.
2. **Filtering**: Separate ephemeral transactional chatter (e.g. standard file reads, HTTP 200s) from architectural wisdom (e.g. locking bugs, provider parameter quirks, type bridges).
3. **Synthesis**: Compress findings into 1–3 dense, technical bullets following the mandatory schema:
   `- **<Category>**: <Actionable Insight>. (Verification: <Mission/Benchmark/Test Witness>).`
4. **Persistence**: Commit entries using `python execution/debrief_mission.py --commit`.

---

## 🧠 2. Socratic Gate for Wisdom Extraction

Before appending an entry to long-term memory, verify these 4 gates:
- **Gate 1 (Actionability)**: Will this insight allow a future agent to bypass a debugging cycle or avoid a critical architectural mistake?
- **Gate 2 (Empirical Witness)**: Is there a verifiable mission number, test suite, or benchmark cited as proof?
- **Gate 3 (Zero Ephemeral Chatter)**: Is all transient context (e.g. temporary filenames, PID numbers, local timestamps) omitted?
- **Gate 4 (Dual-Plane Firewall `FR-11` — `IDENTITY.md` Directive #1)**: Confirm the insight belongs to **Plane 1 (`Engine-Maintenance`)** architectural wisdom. Company-specific business records, pricing tables, or customer PII from **Plane 2 (`SMB-Digital-Twin`)** runs must **never** be committed to `directives/LONG_TERM_MEMORY.md`.

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Chatter Flooding** | Writing "Agent viewed file X" into long-term memory. | Discard operational noise; record only root-cause solutions. |
| **Unwitnessed Claims** | Speculative assertions without a verification source. | Require an explicit witness (e.g. `Verification: Mission #402`). |
| **Conversational Padding** | Narrative text like "In this mission we tried to...". | Use concise technical bullets (`- **Topic**: Direct insight`). |
| **Oversized Entries** | Memory paragraphs exceeding 120 words per bullet. | Enforce conciseness via `debrief_analyzer.py`. |

---

## 🛠️ 4. Deterministic Verification Gate

Validate wisdom entries and analyze mission logs:

```powershell
# 1. Run built-in self-tests
python .agent/skills/mission-analyst/scripts/debrief_analyzer.py --test

# 2. Validate a wisdom entry string
python .agent/skills/mission-analyst/scripts/debrief_analyzer.py --validate-entry "- **Provider Parity**: Gemini models require 'google' provider key. (Verification: Mission #402)."
```