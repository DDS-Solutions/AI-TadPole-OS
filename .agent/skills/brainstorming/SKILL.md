---
name: brainstorming
description: "Socratic questioning protocol and communication framework. Mandatory operational gateway for complex requests, ambiguous requirements, and architectural trade-off alignment."
when_to_use: "Use when exploring options before implementation, clarifying requirements, or during creative problem-solving. This is the 'Front End' of the development pipeline."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / brainstorming
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Premature implementation, unverified architectural assumptions, missing Socratic Gate.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# 🧠 Brainstorming & Communication Protocol

This protocol prevents premature execution—the tendency for an AI agent to immediately begin coding before validating scope, requirements, and constraints. It positions the agent as a technical consultant who validates the "What" and "Why" before executing the "How."

## 📚 Progressive Disclosure & L3 Reference Index

For deep question generation heuristics, domain decision banks, and deterministic validation tooling:
- **Dynamic Question Generation**: [dynamic-questioning.md](file:///D:/TadpoleOS-Dev/.agent/skills/brainstorming/references/dynamic-questioning.md)
- **Socratic Decision Framework**: [socratic_decision_framework.md](file:///D:/TadpoleOS-Dev/.agent/skills/brainstorming/references/socratic_decision_framework.md)
- **Deterministic Socratic Gate Validator**: [socratic_gate_validator.py](file:///D:/TadpoleOS-Dev/.agent/skills/brainstorming/scripts/socratic_gate_validator.py)

---

## ⛓️ The Development Pipeline

This skill serves as the entry gate to the Sovereign development lifecycle. **Do not skip stages.**

**`Brainstorming`** (Requirements & Socratic Gate) $\rightarrow$ **`Orchestrate`** (Decomposition) $\rightarrow$ **`Aletheia`** (Invariants & Logic) $\rightarrow$ **`Clean-Code`** (Implementation)

---

## 🛑 The Socratic Gate (Mandatory Halt)

### 1. Trigger Conditions
The Socratic Gate **MUST** be triggered whenever a request contains:
- **Vague Verbs**: "Build/Create/Make/Update/Optimize [feature]" without unambiguous API contracts or target files.
- **High Complexity**: Architectural transitions, new subsystems, or multi-file refactoring.
- **Unknown Constraints**: Unspecified latency targets, security boundaries, or deployment environments.
- **Ambiguity Score $\ge 3$**: Per the [Socratic Decision Framework](file:///D:/TadpoleOS-Dev/.agent/skills/brainstorming/references/socratic_decision_framework.md).

### 2. Hard-Stop Execution Sequence
When the Socratic Gate triggers, execute these steps in strict sequence:

1. **MEMORY SCAN**: Inspect `.agent/memory/` and repository configuration. Identify historical decisions and conventions.
2. **SILENT CONTEXT APPLICATION**: Integrate known patterns silently. Never ask questions already answered in existing docs or memory.
3. **MANDATORY HARD STOP**:
   - 🔴 **CRITICAL**: Do NOT write implementation code.
   - 🔴 **CRITICAL**: Do NOT generate speculative file or folder structures.
   - 🔴 **CRITICAL**: Do NOT state "While you decide, I have started scaffolding..."
4. **HIGH-LEVERAGE QUESTIONING**: Formulate targeted P0 (blocking) decision points highlighting architectural trade-offs.
5. **HALT EXECUTION**: Wait for user confirmation or explicit option selection before proceeding.
6. **DECISION COMMIT**: Record aligned decisions into project memory or specifications.

---

## 🎯 High-Leverage Questioning

**⛔ NEVER use generic or open-ended questions.** Inquiries must directly illuminate architectural trade-offs affecting cost, complexity, latency, or security.

### Decision Point Anatomy
Format all Socratic inquiries using the structured Decision Point schema:

```markdown
### [PRIORITY: P0/P1/P2] **[DECISION POINT NAME]**

**Question:** [Clear, direct question]

**Why This Matters:**
- [Concrete architectural consequence: e.g., "Choosing X over Y will increase latency by ~20ms but enforce zero-trust isolation."]
- [Affects: Performance / Maintenance / Cost / Complexity]

**Proposed Options:**
| Option | Pros | Cons | Best For |
|---|---|---|---|
| **A: [Name]** | [Advantage] | [Trade-off] | [Primary use case] |
| **B: [Name]** | [Advantage] | [Trade-off] | [Primary use case] |

**Default Recommendation:** [Recommended path with architectural justification]
```

---

## 📊 Progress & Error Communication

### 1. Multi-Step Task Status Dashboard
For complex tasks spanning multiple phases, provide clear visual tracking:

| Agent / Phase | Status | Current Milestone | Progress |
|---|---|---|---|
| `brainstorming` | ✅ | Requirement Alignment & Socratic Gate | 100% |
| `orchestrate` | 🔄 | Task Decomposition & Roadmap | 50% |
| `aletheia` | ⏳ | Invariant & Correctness Proofs | 0% |
| `clean-code` | ⏳ | Implementation & Test Execution | 0% |

*Status Indicators*: ✅ Complete | 🔄 Active | ⏳ Pending | ❌ Blocked | ⚠️ Warning

### 2. Error Response Pattern
When operational or build failures occur, adhere to the 4-step communication protocol:
1. **Acknowledge**: State the exact failure point cleanly ("The compilation failed at Tauri IPC command registration").
2. **Translate**: Summarize the technical root cause in plain English.
3. **Offer Concrete Paths**: Present structured options ("Option A: Relax boundary constraint; Option B: Introduce async serialization").
4. **Direct Ask**: Recommend the optimal path and seek confirmation.

---

## 🏁 Completion & Hand-Off

Every completed mission must bridge cleanly to the subsequent action:
1. **Milestone Summary**: Concise summary of what was verified or accomplished.
2. **Verification Evidence**: Specific commands to reproduce and verify results.
3. **Proactive Next Step**: Recommend the immediate logical follow-up without unapproved execution.

---

## 🚫 Anti-Patterns & Violations

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Premature Implementation** | Emitting implementation code before the Socratic Gate is closed. | **CRITICAL FAIL.** Halt coding, present options, await user confirmation. |
| **Generic Questioning** | Asking "Do you have any other requirements?" or "What do you think?" | Pose structured questions with explicit options and architectural trade-offs. |
| **Assumption Loops** | Stating "I assume you want X so I built it." | State "I recommend X because Y. Should I proceed?" |
| **Hidden Progress** | Modifying multiple components without updating status tracking. | Emit structured status board on multi-step executions. |
| **The "I Think" Trap** | Using uncertain or speculative language. | Ground statements in codebase telemetry or ask clarifying questions. |

---

## 🧪 Deterministic Verification

Validate Socratic proposals and chat outputs against anti-pattern rules using the deterministic validator:

```powershell
# Run built-in self tests
python .agent/skills/brainstorming/scripts/socratic_gate_validator.py --test

# Audit a markdown proposal or chat transcript
python .agent/skills/brainstorming/scripts/socratic_gate_validator.py --file path/to/proposal.md
```

[//]: # (Metadata: [SKILL])
