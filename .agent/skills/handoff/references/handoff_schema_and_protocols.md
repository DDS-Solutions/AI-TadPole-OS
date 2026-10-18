# Session Handoff Schema & State Transfer Protocols

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / handoff / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, lost session state, or uncoordinated agent transitions.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. Overview & Purpose

The **Session Handoff Protocol** defines a deterministic contract for serializing the complete working context of an active agent session into a compact, high-signal document. This artifact enables an incoming agent (or returning agent in a fresh session) to resume execution immediately with zero context bloat, zero architectural amnesia, and instant test parity.

---

## 2. The 6-Section Mandatory Handoff Schema

Every handoff artifact stored under `.tmp/handoffs/` must strictly adhere to the following 6-section schema:

```markdown
# Session Handoff: <Task Title>

> **Handoff Timestamp**: <YYYY-MM-DDTHH:MM:SSZ>
> **Source Agent**: <Agent Name / Role>
> **Target Agent**: <Incoming Agent / Fresh Session>
> **Working Directory**: `D:/TadpoleOS-Dev`

---

## 1. Goal & Context
- **Primary Objective**: <Concise 1-2 sentence statement of user's core intent>
- **Current Lifecycle State**: <BRAINSTORM | BLUEPRINT | IMPLEMENT | AUDIT | VERIFY>
- **Session Background**: <Key requirements or constraints explicitly given by user>

## 2. Completed Milestones & Architectural Decisions
- **Milestones Completed**:
  - <Milestone 1 description>
  - <Milestone 2 description>
- **Architectural Invariants & Decisions**:
  - <Decision 1 with rationale (the "Why")>
  - <Decision 2 with trade-off justification>

## 3. Modified Files Manifest
| File Path | Change Description | Symbols Touched |
|---|---|---|
| [`src/App.tsx`](file:///D:/TadpoleOS-Dev/src/App.tsx) | <Brief explanation of edits> | `SymbolName`, `function_name` |

## 4. Verification Commands & Test Results
- **Executable Verification Commands**:
  ```powershell
  <Exact command line 1>
  <Exact command line 2>
  ```
- **Latest Test Status**:
  - `[PASS]` <Suite name / test file> (<N> tests passed)

## 5. Active Blockers & Known Issues
- [ ] <Pending item 1 with file/line reference>
- [ ] <Unresolved blocker, error message, or decision required>
*(If no blockers remain, explicitly declare: `None. All active gates green.`)*

## 6. Cold-Start Resume Protocol
1. **Next Immediate Action**: <Exact first tool call or step the incoming agent must perform>
2. **Recommended Skills**:
   - [`handoff`](file:///D:/TadpoleOS-Dev/.agent/skills/handoff/SKILL.md)
3. **Execution Command**:
   ```powershell
   <Resume command or test runner>
   ```
```

---

## 3. Information Preservation vs. Discard Matrix

To minimize token consumption while maximizing signal fidelity:

| Component | Policy | Rationale |
|---|---|---|
| **Architectural Rationale** | **Preserve (100%)** | Explains *why* decisions were made, preventing incoming agents from undoing deliberate trade-offs. |
| **Exact File Paths & Symbols** | **Preserve (100%)** | Canonical paths and symbol names eliminate search and indexing overhead for incoming agents. |
| **Verification Commands** | **Preserve (100%)** | Allows incoming agents to run tests immediately without guessing test runner syntax. |
| **Active Blockers** | **Preserve (100%)** | Clear blockers prevent duplicate debugging effort or repeating dead-end attempts. |
| **Full File Dumps** | **Discard Aggressively** | Files are persisted on disk; duplicating code in handoffs wastes context window. |
| **Tool Execution Logs** | **Discard Aggressively** | Discard raw terminal traces, compiler warnings, or ripgrep output $> 10$ lines. |
| **Conversational Filler** | **Discard Aggressively** | Omit polite conversational exchanges, speculative brainstorming rejected earlier, etc. |

---

## 4. Multi-Agent PEC Handoff Lifecycle

When transitioning control between agents in multi-agent workflows (e.g. within [`/orchestrate`](file:///D:/TadpoleOS-Dev/.agent/workflows/orchestrate.md)):

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│     PLAN        │       │     EXECUTE     │       │     CONFIRM     │
│ Origin Agent    │ ───>  │ Target Agent    │ ───>  │ Supervisor / QA │
│ Emits Handoff   │       │ Consumes State  │       │ Validates Gate  │
└─────────────────┘       └─────────────────┘       └─────────────────┘
```

1. **Plan (Origin Agent)**:
   - Consolidates state into `.tmp/handoffs/handoff-<timestamp>.md`.
   - Runs `python .agent/skills/handoff/scripts/handoff_manager.py --validate <path>`.
   - Passes only the file path to the next agent, never the raw text dump.

2. **Execute (Target Agent)**:
   - Reads the handoff artifact using `view_file`.
   - Inspects the **Modified Files Manifest** and **Cold-Start Resume Protocol**.
   - Executes the declared verification commands to verify the environment.
   - Begins execution of the next active blocker or milestone.

3. **Confirm (Supervisor / QA)**:
   - Validates that the handover produced expected artifacts.
   - Verifies that no context loss or regression occurred.

---

## 5. Storage Conventions & Retention Policy

- **Directory**: `.tmp/handoffs/`
- **Filename Convention**: `handoff-<timestamp>.md` (e.g. `handoff-20261017-183000.md`) or `session_checkpoint.md` for active running session sync.
- **Git Status**: `.tmp/` is ignored by Git, ensuring local session state does not pollute repository commits.
- **Verification Tool**: [`scripts/handoff_manager.py`](file:///D:/TadpoleOS-Dev/.agent/skills/handoff/scripts/handoff_manager.py)
