---
name: code-review-checklist
description: "Systematic code review audit engine and master rubric across logic, security hardening, performance, clean-code adherence, and test coverage."
when_to_use: "When executing REVIEW mode, auditing PRs, performing security scans, or validating a feature before it enters SHIP mode."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / code-review-checklist
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Inconsistent review standards, missed security vulnerabilities, ungrounded nitpicking.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# 🛡️ Code Review Audit Engine

This skill serves as the **Audit Engine** for the `REVIEW` operational mode. It ensures that all code entering the deployment pipeline meets the Sovereign standard of architectural excellence and security hardening.

---

## 📚 Progressive Disclosure & L3 Reference Index

Consult deep **L3 Reference Guides** and verification tools on demand:
- **Audit Rubric & Security Guide**: [audit_rubric_and_security_guide.md](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-checklist/references/audit_rubric_and_security_guide.md)
- **Deterministic Rubric Evaluator**: [audit_rubric_evaluator.py](file:///D:/TadpoleOS-Dev/.agent/skills/code-review-checklist/scripts/audit_rubric_evaluator.py)

---

## ⛓️ Pipeline Integration

This skill acts as the automated "Judge" in the Plan-Execute-Critic (PEC) loop:
**`IMPLEMENT`** $\rightarrow$ **`REVIEW` (This Skill)** $\rightarrow$ **`DEBUG` (If flaws found)** $\rightarrow$ **`SHIP`**

---

## 📏 Contextual Auditing Standards

Calibrate audit severity according to the codebase context:

| Context | Review Rigor | Primary Goal | Rule |
|---|---|---|---|
| **Greenfield** (New Code) | 🔴 **Ruthless** | Flawless Architecture | Zero tolerance for architectural drift, smells, or missing tests. |
| **Feature Update** | 🟡 **Strict** | Stability & Parity | New code must be clean; existing behavior and API contracts must not regress. |
| **Legacy Fix** | 🟢 **Pragmatic** | Risk Mitigation | **Boy Scout Rule:** Leave it cleaner than found; avoid uncontrolled scope creep. |

---

## 📋 The Master Audit Rubric (5 Pillars)

### 1. Correctness & Logic
- [ ] **Functional Parity**: Does the code faithfully solve the problem declared in the specification?
- [ ] **Edge-Case Resilience**: Are empty lists, null/None states, divide-by-zero, and connection timeouts handled?
- [ ] **Aletheia Validation**: Is logical execution verifiable and deterministic without wishful assumptions?
- [ ] **Error Handling**: Are errors wrapped in contextual domain types rather than logged and dropped?

### 2. Security (Sovereign Hardening)
- [ ] **Input Sanitization**: Are user queries, IPC commands, and webhook payloads strictly validated against schemas?
- [ ] **Injection Prevention**: Zero raw string concatenation in SQL queries (`sqlx::query!` parameterized) or shell commands.
- [ ] **Zero Secret Leakage**: No hardcoded API keys, secrets, or `.env` strings committed in code.
- [ ] **AI-Native Defense**: Prompt injection defenses active; model outputs sanitized before reaching critical sinks (`eval`, `innerHTML`, shell).

### 3. Performance & Scale
- [ ] **Algorithmic Complexity**: No $O(n^2)$ nested loops where $O(n)$ or $O(1)$ lookups are feasible.
- [ ] **Database Efficiency**: No N+1 query patterns; use batching or SQL `JOIN`s.
- [ ] **Resource Hygiene**: File handles, network connections, and Tokio tasks cleanly dropped on cancellation.

### 4. Code Quality (`clean-code` Compliance)
- [ ] **Single Responsibility**: Each function performs exactly one logical operation ($\le 50$ lines).
- [ ] **Guard Clauses**: Conditionals inverted to return early; nesting depth $\le 3$.
- [ ] **Type Safety**: Zero `any` in TypeScript; zero unhandled `.unwrap()` in production Rust.

### 5. Testing & Observability
- [ ] **Test Coverage**: New logic accompanied by corresponding unit or invariant tests.
- [ ] **Verification**: Verified using deterministic runners (`cargo test`, `vitest`).
- [ ] **Telemetry Compliance**: Emitter tags match declared targets (`[Subsystem]`, `[ERROR]`, `[OK]`).

---

## 🚩 Anti-Pattern Visual Guide

| Pattern | ❌ Unacceptable | ✅ Sovereign Standard |
|---|---|---|
| **Magic Numbers** | `if (status === 3)` | `if (status === OrderStatus.SHIPPED)` |
| **Deep Nesting** | `if (a) { if (b) { ... } }` | `if (!a) return; if (!b) return;` |
| **Type Escape** | `const data: any = JSON.parse(str)` | `const data: UserProfile = JSON.parse(str)` |
| **Security Sink** | `el.innerHTML = input` | `el.textContent = input` |

---

## 📝 Review Output Format

Structure all review outputs to maximize actionability:

### 1. Executive Summary
- **Verdict**: [🟢 APPROVED | 🟡 MINOR FIXES | 🔴 BLOCKING]
- **Context**: [Greenfield / Feature Update / Legacy Fix]
- **Key Takeaway**: One-sentence assessment of code health.

### 2. Detailed Findings
Categorize by severity:
- 🔴 **BLOCKING**: Must be resolved before merge/ship (Security flaws, crashes, logic divergence).
- 🟡 **SUGGESTION**: Performance improvements, Fowler smells, clean-code refactoring.
- 🟢 **NIT**: Minor naming or formatting preferences.
- ❓ **QUESTION**: Intent clarification required.

### 3. Final Action Plan
Ordered list of remediation steps with priority tags (`[P0]`, `[P1]`, `[P2]`).

---

## 🛠️ Deterministic Verification Gate

Audit source code or PR files against the Master Rubric:

```powershell
# Run built-in self tests
python .agent/skills/code-review-checklist/scripts/audit_rubric_evaluator.py --test

# Audit a target source file
python .agent/skills/code-review-checklist/scripts/audit_rubric_evaluator.py --file src/routes/system.rs
```

[//]: # (Metadata: [CODE_REVIEW_CHECKLIST])
