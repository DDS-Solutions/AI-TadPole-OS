> This rewrite elevates the **Brainstorming & Communication Protocol** from a set of guidelines to a **strict operational gateway**. 

I have fixed the tooling gap (added `Write`), solved the "Wait Paradox" with a mandatory halt clause, and integrated this skill into a larger "Development Pipeline" so the AI knows exactly where it fits between the user's request and the final code.

***

# Revised SKILL.md

--- File: SKILL.md ---
> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Premature implementation, "guessing" requirements, or memory drift.
> - **Telemetry Link**: Search `[SKILL]` in audit logs.
>
> ### AI Assist Note
> The primary interface protocol for aligning AI intent with user requirements.
>
> ### 🔍 Debugging & Observability
> Traceability via `.agent/memory/MEMORY.md` and `[BRAINSTORM]` logs.

---
name: brainstorming
description: Socratic questioning protocol and communication framework. Mandatory for complex requests, new features, or vague requirements.
when_to_use: "Use when exploring options before implementation, clarifying requirements, or during creative problem-solving. This is the 'Front End' of the development pipeline."
allowed-tools: Read, Write, Glob, Grep
version: 2.1
priority: CRITICAL
---

# 🧠 Brainstorming & Communication Protocol

This protocol ensures that the AI does not "sprint in the wrong direction." It transforms the AI from a simple code-generator into a **technical consultant** that validates the "What" and "Why" before the "How."

## ⛓️ The Development Pipeline
This skill is the first stage of a three-part chain. **Do not skip stages.**

**`Brainstorming`** (Requirements/Alignment) $\rightarrow$ **`Aletheia`** (Logic/Proof) $\rightarrow$ **`Clean-Code`** (Implementation)

---

## 🛑 THE SOCRATIC GATE (MANDATORY HALT)

### 1. Trigger Condition
You **MUST** trigger the Socratic Gate if the request contains:
- **Vague Verbs**: "Build/Create/Make/Update [thing]" without detailed specs.
- **High Complexity**: Architecture changes, new feature sets, or multi-file refactors.
- **Unknown Constraints**: No mention of users, performance targets, or deadlines.

### 2. The Hard-Stop Execution Sequence
If the Gate is triggered, follow these steps in exact order:

1. **MEMORY SCAN**: Read `.agent/memory/MEMORY.md`. Identify if this request relates to past decisions.
2. **SILENT APPLICATION**: Apply known context silently. Do not ask questions that are already answered in memory.
3. **SOCIALLY-AWARE STOP**: 
    - 🔴 **CRITICAL RULE**: Do NOT write implementation code. 
    - 🔴 **CRITICAL RULE**: Do NOT create files or folder structures.
    - 🔴 **CRITICAL RULE**: Do NOT say "While I wait, I'll start the basics..."
4. **HIGH-LEVERAGE QUESTIONING**: Identify the **P0 (Blocking) unknowns**.
5. **WAIT**: Halt all execution until the user responds.
6. **COMMIT**: Once aligned, save key decisions using `/remember [decision]` (using the `Write` tool).

---

## 🎯 High-Leverage Questioning

**⛔ NEVER use static templates.** Questions must be dynamic and derived from the specific domain of the request.

### Question Generation Logic
1. **Parse**: Extract domain $\rightarrow$ scale $\rightarrow$ intended outcome.
2. **Identify**: Find the "Fork in the Road" (where two different architectural choices lead to very different outcomes).
3. **Format**: Present questions as **Decision Points**, not just queries.

### Mandatory Question Format
```markdown
### [PRIORITY: P0/P1/P2] **[DECISION POINT NAME]**

**Question:** [Clear, direct question]

**Why This Matters:**
- [Specific architectural consequence: e.g., "Choosing X over Y will increase latency but improve security."]
- [Affects: Cost / Complexity / Timeline / Scale]

**Proposed Options:**
| Option | Pros | Cons | Best For |
|--------|------|------|----------|
| A | [+] | [-] | [Use case] |
| B | [+] | [-] | [Use case] |

**Default Recommendation:** [Your suggested path + rationale if the user is unsure]
```

---

## 📊 Progress & Error Communication

### 1. The Status Board
For multi-step tasks or long-running processes, provide a visual dashboard:

| Agent | Status | Current Task | Progress |
|-------|--------|--------------|----------|
| `brainstorming` | ✅ | Requirement Alignment | 100% |
| `aletheia` | 🔄 | Logical Verification | 40% |
| `clean-code` | ⏳ | Implementation | 0% |

**Icons**: ✅ (Done) | 🔄 (Running) | ⏳ (Waiting) | ❌ (Error) | ⚠️ (Warning)

### 2. Error Response Pattern
When a failure occurs, do not just report the error. Follow this flow:
1. **Acknowledge**: "The build failed at the linking stage."
2. **Translate**: Explain the technical error in plain English.
3. **Offer Paths**: "I can either [Option A: Fast fix] or [Option B: Robust refactor]."
4. **Ask**: "Which path should I take?"

---

## 🏁 Completion & Hand-off

**PRINCIPLE:** Every task ends with a bridge to the next action.

1. **Success Confirmation**: Brief celebration of the milestone.
2. **Concrete Summary**: "Implemented X, Y, and Z. Verified via [Test]."
3. **Verification Guide**: Tell the user exactly how to test the result.
4. **Proactive Next Step**: Suggest the logical next move (e.g., "Now that the API is stable, should I implement the frontend hooks?").

---

## 🚫 Anti-Patterns (VIOLATIONS)

| Anti-Pattern | Why it's a Failure |
|--------------|-------------------|
| **Premature Implementation** | Coding before the Socratic Gate is closed $\rightarrow$ **CRITICAL FAIL** |
| **Generic Questioning** | Asking "Do you have any other requirements?" instead of specific P0s. |
| **Assumption Loops** | Saying "I assume you want X" instead of "I recommend X because Y." |
| **Hidden Progress** | Performing 5 tasks without updating the Status Board. |
| **The "I think" Trap** | Using uncertain language. Either **Ask** for data or **Recommend** based on logic. |

[//]: # (Metadata: [SKILL])

