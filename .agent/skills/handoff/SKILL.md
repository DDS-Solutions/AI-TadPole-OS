---
name: handoff
description: "Compacts current session state into a clean, deterministic handoff document for incoming agents or multi-session state transfers. Prevents context drift and cold-start latency."
when_to_use: "When completing a major milestone, reaching 15-20+ conversation turns, preparing to switch agents or sessions, or pausing work on a multi-phase task. NOT for trivial single-turn questions."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / handoff
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, lost session state, or uncoordinated agent transitions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Session Handoff & State Transfer Protocol

> **Purpose**: Compact active session state into a high-signal, zero-bloat handoff artifact so a fresh agent session or specialist subagent can resume execution immediately with zero context token degradation.  
> **Workflow Bindings**: Integrated into [`/orchestrate`](../../workflows/orchestrate.md), [`/onboard`](../../workflows/onboard.md), and [`/status`](../../workflows/status.md).

---

## 📚 Progressive Disclosure & L3 Reference Index

Consult deep **L3 Reference Guides** and verification tools on demand:
- **Handoff Schema & Protocols**: [`references/handoff_schema_and_protocols.md`](./references/handoff_schema_and_protocols.md)
- **Deterministic Handoff Manager**: [`scripts/handoff_manager.py`](./scripts/handoff_manager.py)
- **Context Compactor & Checkpoints**: [`context_compactor.py`](../context-compression/scripts/context_compactor.py)

---

## 🚦 1. When to Trigger a Session Handoff

Trigger a session handoff under any of the following operational conditions:
1. **Turn Count Ceiling**: Active session reaches 15–20+ conversational turns.
2. **Phase Boundary Transition**: Shifting from Planning/Architecture to Deep Implementation, or from Implementation to Security Audit / Testing.
3. **Agent Delegation (PEC)**: Transferring task ownership from a Coordinator Agent (e.g. Agent 2 / Tadpole Alpha) to a Worker Subagent.
4. **Session Suspension**: Halting work across sessions or prompting the user before a long-running execution.

---

## 📋 2. The 6-Section Mandatory Handoff Schema

All handoffs must be saved to `.tmp/handoffs/handoff-<timestamp>.md` and contain all 6 standardized sections:

```markdown
# Session Handoff: <Task Title>

> **Handoff Timestamp**: <YYYY-MM-DDTHH:MM:SSZ>
> **Execution Plane**: <Engine-Maintenance | SMB-Digital-Twin>
> **Source Agent**: <Originating Agent Name / Role>
> **Target Agent**: <Incoming Agent / Fresh Session>
> **Working Directory**: `D:/TadpoleOS-Dev`

---

## 1. Goal & Context
- **Primary Objective**: <Concise statement of user intent>
- **Current Lifecycle State**: <BRAINSTORM | BLUEPRINT | IMPLEMENT | AUDIT | VERIFY>
- **Session Background**: <Explicit user constraints or requirements>

## 2. Completed Milestones & Architectural Decisions
- **Milestones Completed**:
  - <Itemized achievements>
- **Architectural Invariants & Decisions**:
  - <Key decisions with rationale (the "Why")>

## 3. Modified Files Manifest
| File Path | Change Description | Symbols Touched |
|---|---|---|
| [`src/App.tsx`](../../../src/App.tsx) | <Summary of changes> | `SymbolName` |

## 4. Verification Commands & Test Results
- **Executable Verification Commands**:
  ```powershell
  <Command 1>
  <Command 2>
  ```
- **Latest Test Status**:
  - `[PASS]` <Test Suite> (<N> tests passed)

## 5. Active Blockers & Known Issues
- [ ] <Unresolved blocker, open question, or pending edge case>
*(If clean, declare: `None. All active gates green.`)*

## 6. Cold-Start Resume Protocol
1. **Next Immediate Action**: <Exact first tool call or step the incoming agent must perform>
2. **Recommended Skills**:
   - [`handoff`](./SKILL.md)
3. **Execution Command**:
   ```powershell
   <Resume command or test runner>
   ```
```

---

## 🔄 3. Operational Workflow: The Handoff Lifecycle

```
[1. SCAFFOLD] ───> [2. POPULATE] ───> [3. VERIFY] ───> [4. TRANSFER]
  Generate           Fill sections       Run manager       Pass file path
  schema             pruning bloat       validator         only (no dumps)
```

1. **Step 1: Scaffold**
   Run the deterministic tool to instantiate the template:
   ```powershell
   python .agent/skills/handoff/scripts/handoff_manager.py --scaffold --title "Feature Migration"
   ```
2. **Step 2: Populate**
   Fill each of the 6 mandatory sections. Prune conversation filler, tool retries, and raw directory dumps. Preserve exact file paths, symbols, commands, and blockers.
3. **Step 3: Verify**
   Validate that the handoff conforms to schema and has no lingering `<...>` placeholders:
   ```powershell
   python .agent/skills/handoff/scripts/handoff_manager.py --validate .tmp/handoffs/handoff-<timestamp>.md
   ```
4. **Step 4: Transfer**
   Inform the incoming agent or user by citing only the relative path (e.g. `.tmp/handoffs/handoff-20261017-183000.md`). **Never paste the full text of the handoff into the conversation turn.**

---

## 🧠 4. Socratic Gate for Handoff Alignment

Before completing a handoff, answer these 3 validation gates:
- **Gate 1 (Completeness)**: Does the handoff contain all 6 mandatory sections with zero unresolved `<placeholder>` tokens?
- **Gate 2 (Actionability)**: Can a fresh agent with zero prior session memory immediately execute the cold-start resume commands without searching?
- **Gate 3 (Test Parity)**: Are the exact verification commands and latest pass/fail states documented?

---

## 🚫 5. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Raw Context Dump** | Pasting hundreds of lines of chat history or ripgrep logs into the handoff. | Discard tool output; record only the discovered file paths and line ranges. |
| **Placeholder Leakage** | Leaving `<Task Title>` or `<YYYY-MM-DD>` unpopulated in the handoff document. | Run `handoff_manager.py --validate` which strictly flags template placeholders. |
| **Phantom Paths** | Referencing files that do not exist or broken relative links. | Verify file paths exist and use canonical repo-relative links. |
| **Vague Blockers** | Writing "Needs more testing" or "Some bugs remaining". | Document exact test commands, failing test names, and stack traces in Section 5. |
| **Content Inlining** | Pasting the entire handoff markdown into the chat response. | Pass file path only: *"Handoff artifact saved to `.tmp/handoffs/...`."* |

---

## 🛠️ 6. Deterministic Verification Gate

Validate the handoff manager and any generated handoffs:

```powershell
# Run built-in self-tests
python .agent/skills/handoff/scripts/handoff_manager.py --test

# Validate a specific handoff artifact
python .agent/skills/handoff/scripts/handoff_manager.py --validate .tmp/handoffs/handoff-example.md
```