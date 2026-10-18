---
name: context-compression
description: "Manage and compress conversation context in long sessions. Detect when context is growing large, summarize completed work phases, archive old findings while preserving key decisions. Prevents context degradation."
when_to_use: "When a session has 20+ turns, when context feels repetitive, when the agent is losing track of earlier work, or when the user says 'summarize what we've done'. NOT for short sessions."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / context-compression
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Context degradation, attention dilution, or loss of critical architectural decisions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Context Compression & Session Management

> **Purpose**: Prevent context degradation, attention dilution, and hallucination drift in long-running multi-turn sessions (20+ turns) by distilling working memory into high-signal checkpoints.
> **Workflow Bindings**: Integrated into [`/orchestrate`](file:///D:/TadpoleOS-Dev/.agent/workflows/orchestrate.md) and [`/onboard`](file:///D:/TadpoleOS-Dev/.agent/workflows/onboard.md).

---

## 📚 Progressive Disclosure & L3 Reference Index

Consult deep **L3 Reference Guides** and verification tools on demand:
- **Checkpoint & Handoff Templates**: [checkpoint_and_handoff_templates.md](file:///D:/TadpoleOS-Dev/.agent/skills/context-compression/references/checkpoint_and_handoff_templates.md)
- **Deterministic Context Compactor**: [context_compactor.py](file:///D:/TadpoleOS-Dev/.agent/skills/context-compression/scripts/context_compactor.py)

---

## 🚦 1. Compression Triggers & The 3-Tier Model

| Level | Scope | Trigger Condition | Execution Action & Output Target |
|---|---|---|---|
| **Level 1: Micro-Compact** | Large Tool Dumps | Grep, search, or file reads $> 100$ lines | Summarize key files, matched line numbers, and counts. Discard raw dump. |
| **Level 2: Phase Transition** | Lifecycle State Shift | Moving from Brainstorming $\rightarrow$ Coding | Record confirmed architectural choices; purge transient read logs. |
| **Level 3: Session Checkpoint** | Full Working Memory | 20+ turns or user requests summary | Emit full 5-section Session Checkpoint to `.tmp/handoffs/session_checkpoint.md`. |

---

## 🧭 2. Preservation vs Discard Directives

```
Preserve at All Costs:
├── Core Architectural Decisions & Rationale (the "Why")
├── Exact File Paths & Symbols Touched
├── Executable Verification Commands & Test Results
└── Active Blockers and Next Milestones

Discard Aggressively:
├── Full file contents already committed to disk
├── Repetitive tool invocation syntax and error retries
├── Raw directory trees and verbose package manifests
└── Ephemeral conversational filler and intermediate banter
```

---

## 🔄 3. Multi-Session Handoff Architecture

When approaching token context limits or pausing work across sessions:
1. **Emit Checkpoint**: Generate a standardized Level 3 markdown checkpoint.
2. **Persist Artifact**: Write to `.tmp/handoffs/session_checkpoint.md`.
3. **Cold-Start Resume**: Returning agents load only the checkpoint (~500–1,000 tokens) to achieve instant state parity without re-reading the entire transcript history.

---

## 🚫 4. Anti-Patterns & Failure Modes

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Context Bloat** | Accumulating 40+ turns without summarizing. | Trigger Level 3 compaction at turn 20. |
| **Amnesia Loops** | Forgetting user constraints stated early in the chat. | Pin constraints in the "Key Decisions" checkpoint section. |
| **Information Loss** | Over-compacting to the point where modified file names are lost. | Always maintain the exact "Modified Files Manifest" section. |
| **Uncompressed Tool Dumps** | Pasting 300 lines of ripgrep output into context. | Replace with 5-line Level 1 Micro-Compact summary. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate context footprints and checkpoint schemas:

```powershell
# Run built-in self tests
python .agent/skills/context-compression/scripts/context_compactor.py --test

# Verify schema compliance of a checkpoint file
python .agent/skills/context-compression/scripts/context_compactor.py --file .tmp/handoffs/session_checkpoint.md --verify-checkpoint
```

[//]: # (Metadata: [CONTEXT_COMPRESSION])