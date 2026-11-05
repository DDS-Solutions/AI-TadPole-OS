---
name: adg-governance
description: "Active Documentation Guard (ADG v2) protocol, machine-maintained witness manifests, AST symbol blast radius analysis, and parity guard compliance."
when_to_use: "Before modifying Rust routes, React state stores, or parser logic. When running pre-refactor blast guards or verifying documentation alignment."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: critical
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / adg-governance
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy v1 claims, unverified documentation divergence.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Active Documentation Guard (ADG v2) Protocol

> **Purpose**: Guarantee zero architectural drift between codebase reality and sovereign documentation through automated AST symbol mapping and machine-maintained manifests.
> **Workflow Binding**: Used during [`/audit`](../../workflows/audit.md), [`/refactor`](../../workflows/refactor.md), and pre-commit verification.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/adg_v2_manifest_and_blast_rules.md`](./references/adg_v2_manifest_and_blast_rules.md) | Header specifications, forbidden v1 anti-patterns, AST blast radius tooling | Writing or updating code file headers |
| [`scripts/adg_linter.py`](./scripts/adg_linter.py) | Deterministic scanner for forbidden v1 claims (`Witness Tests:`) and `@docs` tags | Pre-commit file header validation |

---

## 🏛️ Core Architectural Invariants

1. **No Hand-Written Fact Claims**: Never write `Witness Tests:` or freeform `Invariants:` in source file headers. Those are forbidden v1 anti-patterns flagged by `npm run adg:lint`.
2. **Machine-Maintained Manifest**: Test witness relationships are automatically derived by `npm run adg:generate` into `adg.manifest.json`.
3. **Mandatory Header Block**: All production source files must contain the standard `@docs ARCHITECTURE:...` block.
4. **Mandatory Pre-Refactor Guard**: Before modifying shared contracts, run `npm run graph:blast:guard -- --path <file>` to map affected callers and callees.

---

## 🛡️ Socratic Gate: Blast Radius & Documentation Invariants

Before submitting code changes:
1. **Blast Radius Check**: Did you run `npm run graph:blast:guard` on modified Rust/TS files?
2. **Header Compliance**: Does the file contain the proper `@docs ARCHITECTURE:...` tag?
3. **Parity Check**: Did you run `python execution/parity_guard.py` to confirm 0 errors?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Writing `Witness Tests:` manually | Flagged as anti-pattern by `adg:lint` | Let `npm run adg:generate` derive witnesses |
| Editing core routes without blast guard | Unintended cascading breakages | Run `npm run graph:blast:guard` first |
| Leaving phantom `@docs` links | Parity guard failure | Reference valid subsystems defined in architecture docs |

---

## 🛠️ Verification & Execution Commands

```powershell
# Inspect blast radius before editing
npm run graph:blast:guard -- --path server-rs/src/routes/system.rs

# Validate file header compliance
python .agent/skills/adg-governance/scripts/adg_linter.py <file_path>

# Run ADG verification suite
npm run adg:verify
npm run adg:lint

# Run self-test suite
python .agent/skills/adg-governance/scripts/adg_linter.py --test
```
