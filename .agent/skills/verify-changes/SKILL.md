---
name: verify-changes
description: "Prove code works by running it, not just checking it exists. Verification through execution rather than inspection. Use after writing or modifying code to ensure it actually functions correctly."
when_to_use: "After writing code, completing a feature, or fixing a bug. When the user says 'does this work?', 'test this', 'verify', or when declaring work complete. NOT for writing new code."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / verify-changes
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Declaring tasks complete without execution evidence, uninspected compiler errors, or broken invariant tests.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Verify Changes — Prove Code Works

> **Core Principle**: "Code that exists" $\ne$ "Code that works." Verification by inspection ("it looks fine") or assumption ("types check out") is prohibited. Verification requires **empirical execution evidence**.  
> **Workflow Bindings**: Integrated into [`/test`](file:///D:/TadpoleOS-Dev/.agent/workflows/test.md), [`/refactor`](file:///D:/TadpoleOS-Dev/.agent/workflows/refactor.md), and [`/deploy`](file:///D:/TadpoleOS-Dev/.agent/workflows/deploy.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/verification_tiers_and_evidence_matrix.md`](file:///D:/TadpoleOS-Dev/.agent/skills/verify-changes/references/verification_tiers_and_evidence_matrix.md) | The 4-Tier Verification Hierarchy, exit criteria, and evidence presentation formats | Planning verification passes after non-trivial edits |
| [`scripts/change_verifier.py`](file:///D:/TadpoleOS-Dev/.agent/skills/verify-changes/scripts/change_verifier.py) | Automated git status inspector mapping modified files to required compilation/test gates | Determining mandatory verification commands |

---

## 🚦 1. The 4-Tier Verification Hierarchy

Never declare a task "done" without executing the applicable tier:

```
[TIER 1: COMPILATION] ───> [TIER 2: STATIC LINT] ───> [TIER 3: AUTOMATED TEST] ───> [TIER 4: RUNTIME PROOF]
 tsc / cargo check          eslint / clippy             vitest / cargo test          CLI run / HTTP check
```

1. **Tier 1 (Compilation & Types)**:
   - TypeScript: `npx tsc --noEmit`
   - Rust: `cargo check --manifest-path server-rs/Cargo.toml`
2. **Tier 2 (Static Analysis)**:
   - TypeScript: `npm run lint`
   - Rust: `cargo clippy --manifest-path server-rs/Cargo.toml -- -D warnings`
3. **Tier 3 (Automated Test Witness)**:
   - Run Vitest or Cargo tests asserting the modified code paths.
4. **Tier 4 (Runtime Execution Evidence)**:
   - Execute the actual script or call the endpoint to verify runtime behavior.

---

## 🧠 2. Socratic Gate for Verification Proof

Before reporting task completion, verify:
- **Gate 1 (Empirical Execution)**: Did you run the actual compiler, linter, or test commands and verify exit code 0?
- **Gate 2 (Zero Warnings Policy)**: Did Clippy and TypeScript run with zero warnings and zero `@ts-ignore` escapes?
- **Gate 3 (Evidence in Response)**: Have you provided the user with clear evidence (exit code, test count, or output snippet)?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Verification by Inspection** | "I checked the code and it looks correct." | Run the compiler/test command; present the output. |
| **Silent Type Errors** | Assuming TypeScript passes without running `tsc`. | Run `npx tsc --noEmit` after all frontend edits. |
| **Masking Lint Warnings** | Leaving clippy warnings unaddressed. | Run `cargo clippy -- -D warnings` to enforce zero-warning policy. |

---

## 🛠️ 4. Deterministic Verification Gate

Determine required verification gates for current modifications:

```powershell
# 1. Run built-in self-tests
python .agent/skills/verify-changes/scripts/change_verifier.py --test

# 2. Inspect active git modifications and print required verification gates
python .agent/skills/verify-changes/scripts/change_verifier.py --git-status
```