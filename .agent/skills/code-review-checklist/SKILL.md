---
name: code-review-checklist
description: "Expert guidelines, principles, and procedures for AI Assist Note."
---

---
> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Superficial reviews, "rubber-stamping" bad code, or over-refactoring legacy systems.
> - **Telemetry Link**: Search `[CODE_REVIEW]` in audit logs.
>
> ### AI Assist Note
> The formal rubric for the `REVIEW` behavioral mode.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py` and `[REVIEW]` tags.

---

>This rewrite transforms the **Code Review Checklist** from a passive list of tips into a **Rigorous Audit Engine**. 

I have integrated it directly into the `REVIEW` mode of the `behavioral-modes` skill and the standards of the `clean-code` skill. I also added a "Sovereign Context" section to handle the difference between new and legacy code, ensuring the AI doesn't waste time over-engineering old files while remaining ruthless with new ones.

***

# Revised SKILL.md

--- File: SKILL.md

name: code-review-checklist
description: Professional audit rubric for code quality, security, and performance. Mandatory for all `REVIEW` mode operations.
when_to_use: "When executing `REVIEW` mode, auditing PRs, performing security scans, or validating a feature before it enters `SHIP` mode."
allowed-tools: Read, Write, Glob, Grep, Execute
version: 2.0
priority: CRITICAL
---

# 🛡️ Code Review Audit Engine

This skill is the **Audit Engine** for the `REVIEW` behavioral mode. It ensures that all code entering the production pipeline meets the "Sovereign Reality" standard of excellence.

## ⛓️ Pipeline Integration
This skill is the "Judge" in the PEC (Plan-Execute-Critic) loop:
**`IMPLEMENT`** $\rightarrow$ **`REVIEW` (This Skill)** $\rightarrow$ **`DEBUG` (If flaws found)** $\rightarrow$ **`SHIP`**

---

## 📏 The Sovereign Standard (Contextual Auditing)

The AI must adjust its severity based on the **Code Context**:

| Context | Review Rigor | Primary Goal | Rule |
|----------|-------------|--------------|------|
| **Greenfield** (New Code) | 🔴 **Ruthless** | Perfect Architecture | Zero tolerance for `clean-code` violations. |
| **Feature Update** | 🟡 **Strict** | Stability & Parity | New code must be perfect; existing code must not degrade. |
| **Legacy Fix** | 🟢 **Pragmatic** | Risk Mitigation | **Boy Scout Rule:** Leave it better than you found it, but don't rewrite the world. |

---

## 📋 The Master Audit Rubric

### 1. Correctness & Logic
- [ ] **Functional Parity**: Does the code actually solve the problem stated in `BRAINSTORM`?
- [ ] **Edge Case Resilience**: Handled `null`, `undefined`, timeouts, and empty states?
- [ ] **Aletheia Validation**: Does the logic follow a verifiable path, or is there "wishful thinking"?
- [ ] **Error Handling**: Are errors caught, logged, and handled gracefully (not just `console.log`)?

### 2. Security (Sovereign Hardening)
- [ ] **Input Sinks**: All user inputs validated and sanitized?
- [ ] **Injection**: No SQL/NoSQL/Command injection risks?
- [ ] **Credential Leakage**: No hardcoded secrets or `.env` variables in code?
- [ ] **AI-Native Security**: 
    - Protection against **Prompt Injection** in AI-driven inputs.
    - AI outputs are sanitized before being passed to critical system sinks (e.g., `eval`, `innerHTML`).
    - Non-deterministic AI outputs are validated against a schema.

### 3. Performance & Scale
- [ ] **Complexity**: No $O(n^2)$ loops where $O(n)$ is possible.
- [ ] **Database Efficiency**: No N+1 queries; appropriate indexing suggested.
- [ ] **Memory/Bundle**: No massive dependencies added for a single function.
- [ ] **Caching**: Appropriate use of `memo`, `cache`, or TTL strategies.

### 4. Code Quality (`clean-code` Adherence)
- [ ] **SRP**: Does every function do exactly one thing?
- [ ] **Naming**: Intent-revealing names (e.g., `userCount` not `n`).
- [ ] **Flatness**: Guard clauses used to avoid deep nesting (Max 2 levels).
- [ ] **Sizing**: Functions $\le 20$ lines.
- [ ] **Typing**: No `any` types; strict TypeScript interfaces used.

### 5. Testing & Documentation
- [ ] **Coverage**: New logic is accompanied by at least one Unit Test.
- [ ] **Verification**: Does a `test_runner.py` or Playwright script exist for this change?
- [ ] **Documentation**: Complex logic explained via code (not comments). Public APIs documented.

---

## 🚩 Anti-Pattern Flagging (Visual Guide)

When flagging issues, provide a direct comparison:

| Pattern | ❌ Bad | ✅ Sovereign Standard |
|---------|------|-----------------------|
| **Magic Numbers** | `if (status === 3)` | `if (status === OrderStatus.SHIPPED)` |
| **Deep Nesting** | `if (a) { if (b) { ... } }` | `if (!a) return; if (!b) return;` |
| **Type Safety** | `const data: any = ...` | `const data: UserProfile = ...` |
| **AI Prompts** | `ai.generate(input)` | `ai.generate({ system: "...", input: sanitize(input), schema: X })` |

---

## 📝 Review Output Format

The AI must deliver reviews in this structured format to ensure clarity and actionability:

### 1. Executive Summary
- **Verdict**: [🟢 APPROVED | 🟡 MINOR FIXES | 🔴 BLOCKING]
- **Context**: [Greenfield / Update / Legacy]
- **Key Takeaway**: (One sentence summary of the code's health).

### 2. Detailed Audit
Use the following markers:
- 🔴 **BLOCKING**: Must be fixed before merge (Security, Crashes, Logic Failures).
- 🟡 **SUGGESTION**: Improvement for performance, readability, or `clean-code`.
- 🟢 **NIT**: Minor style preference.
- ❓ **QUESTION**: Clarification needed on intent.

**Example:**
> 🔴 **BLOCKING**: SQL Injection risk in `UserService.ts:42`. Use parameterized queries.
> 🟡 **SUGGESTION**: This loop is $O(n^2)$. Consider a Map for $O(1)$ lookup.

### 3. Final Action Plan
1. [ ] Fix $\🔴$ issue A.
2. [ ] Address $\🟡$ issue B.
3. [ ] Run `lint_runner.py` to verify.

---

## 🔄 The Review Loop (Operational Flow)

1. **Scan**: Read the target file + dependent files.
2. **Rubric Check**: Run the Master Audit Rubric.
3. **Categorize**: Sort findings by severity ($\🔴 \rightarrow \🟡 \rightarrow \🟢$).
4. **Report**: Output the "Review Report."
5. **Execute**: If the user approves the report, transition to `IMPLEMENT` mode to apply the fixes.

[//]: # (Metadata: [CODE_REVIEW])
