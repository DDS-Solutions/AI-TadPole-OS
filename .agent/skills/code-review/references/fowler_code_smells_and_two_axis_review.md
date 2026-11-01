# Fowler Code Smells & Two-Axis Review Architecture (L3 Reference)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / code-review / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Merged-axis bias, unverified requirement coverage, ungrounded review comments.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document provides the reference catalog of Martin Fowler's code smells with concrete remedies, alongside the specification verification methodology for Two-Axis reviews.

---

## 1. The Fowler Code Smell Catalog & Remedies

| Code Smell | Architectural Symptom | Concrete Refactoring Remedy |
|---|---|---|
| **Mysterious Name** | Identifier fails to communicate intent (`n`, `data`, `handle`). | Rename to reveal specific intent (`active_user_count`, `parse_telemetry_payload`). |
| **Duplicated Code** | Similar logic shapes across multiple modules or branches. | Extract shared helper or domain function into common domain module. |
| **Feature Envy** | A function consumes more fields from another struct than its own. | Move function or logic into the module owning the data. |
| **Data Clumps** | Same group of 3+ primitive parameters passed together. | Introduce dedicated domain struct or parameter object. |
| **Primitive Obsession** | Using raw strings/numbers for domain concepts (e.g., raw string for email). | Wrap in type-safe newtype or validated domain value object. |
| **Repeated Switches** | Identical `match` or `switch` cascades repeated across files. | Replace with trait implementation, polymorphism, or lookup table. |
| **Shotgun Surgery** | A single logical change requires touching 5+ scattered files. | Consolidate responsibility into a unified domain module. |
| **Divergent Change** | A single module is modified for multiple unrelated business reasons. | Split module into single-responsibility components (SRP). |
| **Speculative Generality** | Interfaces and abstractions created for hypothetical future needs. | Inline or delete abstraction; implement only what is needed now (YAGNI). |
| **Message Chains** | Long method call chains (`obj.get_a().get_b().get_c()`). | Hide delegate; introduce direct convenience method on the root object. |
| **Middle Man** | A class or module that merely forwards all calls to another object. | Remove middle man and have caller communicate directly with provider. |
| **Refused Bequest** | Subclass ignores or overrides methods of parent with no-ops. | Replace inheritance with composition. |

---

## 2. Spec Axis Verification Protocol

The **Spec Axis** operates completely independently from code style. Its sole responsibility is verifying that every requirement in the originating spec/issue is implemented faithfully without omissions or unapproved scope creep.

### Step-by-Step Spec Audit Process
1. **Requirement Extraction**:
   - Parse the originating issue, PRD, or blueprint into an atomic checklist of acceptance criteria:
     `[REQ-1]`, `[REQ-2]`, `...`, `[REQ-N]`.
2. **Hunk Mapping**:
   - For each requirement, locate the corresponding git diff hunk that fulfills it.
3. **Scope Creep Audit**:
   - Check if the diff introduces features, routes, or behaviors not mentioned in the specification.
   - Classify scope creep as **Blocking** if it introduces attack surface or schema changes, or **Advisory** if it is minor tooling.
4. **Failure State Handling**:
   - Verify that all failure paths specified in the spec are tested and handled.

---

## 3. Two-Axis Report Output Schema

```markdown
# 🔍 Two-Axis Code Review Report

## 📌 Pinned Base: `<fixed-point>`...HEAD (`<N>` commits)
**Spec Source**: [PR #123 / docs/blueprint.md / Issue #45]

---

## 📐 Axis 1: Standards Findings
*Evaluated against repo conventions, ADG v2, and Fowler Code Smells.*

| Severity | Smell / Convention | Location | Actionable Recommendation |
|---|---|---|---|
| Critical | Feature Envy | `src/services/billing.rs:45` | Move `calculate_tax` to `TaxEngine` struct |
| Minor | Mysterious Name | `src/utils.ts:12` | Rename `d` to `elapsed_duration_ms` |

---

## 🎯 Axis 2: Spec Findings
*Evaluated against originating specification requirements.*

| Requirement ID | Status | Diff Evidence | Notes / Divergence |
|---|---|---|---|
| `[REQ-01]` OAuth Callback | ✅ Implemented | `src/routes/auth.rs:88-120` | Full token exchange verified |
| `[REQ-02]` Auto-Refresh | ❌ Missing | Not found in diff | Missing refresh timer scheduled job |
| `[SCOPE-CREEP]` Export CSV | ⚠️ Unapproved | `src/routes/export.rs` | Not declared in original spec |

---

## 🏁 Final Review Summary
- **Standards Status**: [PASS / FAIL] — Total: X findings (Worst: Critical)
- **Spec Status**: [PASS / FAIL] — 1/2 requirements satisfied, 1 missing
```
