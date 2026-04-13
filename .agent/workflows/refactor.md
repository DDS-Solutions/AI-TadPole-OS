---
description: Architectural Hardening & Refactoring. Focuses on structural integrity and decoupling without adding feature bloat.
---

# /refactor - Architectural Hardening

$ARGUMENTS

---

## 🎯 Primary Objective
Perform non-destructive structural improvements to the codebase to align with the **4-Pillars** architecture and reduce technical debt.

## 🏗️ Refactor Gates

### 1. Hub Decoupling
- **Goal**: Ensure clean boundaries between Gateway, Runner, Registry, and Security.
- **Action**: Audit `state.rs` and `mod.rs` for tight coupling or circular dependencies.

### 2. Optimization
- **Goal**: Minimize bundle size (Frontend) and improve runtime efficiency (Backend).
- **Action**: Profile logic using the `performance-profiling` skill.

### 3. Debt Liquidation
- **Goal**: Remove legacy placeholders and "ghost" logic.
- **Action**: Replace `TODO` or `FIXME` comments with Sovereign code.

---

## 🛠️ Execution Protocol

1. **Audit Baseline**: Run `python execution/verify_all.py .` to identify fragile nodes.
2. **Plan Modification**: Create an implementation plan focusing strictly on **Hardening** (No new features).
3. **Apply & Verify**: Implement changes and re-run the full verification suite.

---

## Output Format

```markdown
## 🏗️ Architectural Refactor: [Module]

### 🧹 Debt Removed
- [Legacy Pattern 1] → [Sovereign Pattern 1]

### 🚀 Optimization Gains
- **Build Size**: -[X]kB
- **Complexity**: Reduced by [Y]%

### ✅ Verification
- **Verify All**: PASSED
- **Parity Guard**: SYNCED
```
