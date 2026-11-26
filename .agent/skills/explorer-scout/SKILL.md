---
name: explorer-scout
description: Scouting and architectural analysis for GitHub repositories. Extracts manifest dependencies, licensing, and structural patterns.
when_to_use: "When researching external GitHub repositories, analyzing their directory structures, or extracting architectural patterns. Use with /github_scout workflow."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash, RunCommand
version: "2.1.0"
priority: medium
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / explorer-scout
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# GitHub & External Repository Scouting Protocol

> **Purpose**: Systematic discovery, architectural extraction, and capability indexing of external open-source repositories.
> **Workflow Binding**: Used directly during the [`/github_scout`](../../workflows/github_scout.md) workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/repo_scouting_recipes.md`](./references/repo_scouting_recipes.md) | Branch isolation procedures, dependency extraction checklists, licensing rules | Inspecting or cloning third-party codebases |
| [`scripts/repo_scout.py`](./scripts/repo_scout.py) | Deterministic repository manifest scanner, license finder, and entrypoint extractor | Automated repository analysis or self-test verification |

---

## 🔄 The 5-Phase Scouting Workflow

```
1. SEARCH          ➔ Discover candidate repositories matching architectural requirements.
2. BRANCH ISOLATE  ➔ Check out a disposable branch (`git checkout -b research/<repo>`).
3. ARCHAEOLOGY     ➔ Scan manifests (`Cargo.toml`, `package.json`), entrypoints, and license.
4. SYNTHESIS       ➔ Document architectural wins into a structured research dossier.
5. CLEANUP         ➔ Discard throwaway branches to keep sovereign working tree clean.
```

---

## 🛡️ Socratic Gate: Licensing & Sandboxing

Before integrating patterns or code from external repositories:
1. **License Compatibility**: Is the external code licensed permissively (MIT, Apache 2.0, BSD)? (Avoid AGPL/GPL contamination).
2. **Dependency Overhead**: Does the external pattern introduce fragile third-party dependencies?
3. **Sandbox Isolation**: Is external exploration isolated on throwaway branches?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Direct Clone onto Main | Pollutes git history with foreign files | Always use isolated `research/*` branches |
| Copying Unverified Code | Injects security vulnerabilities or IP risks | Audit dependencies and license before codification |
| Blindly Following Readmes | Outdated setup commands fail | Run `repo_scout.py` to inspect actual manifests |

---

## 🛠️ Verification & Execution Commands

```powershell
# Scout local or cloned repository
python .agent/skills/explorer-scout/scripts/repo_scout.py <repo_dir>

# Run self-test suite
python .agent/skills/explorer-scout/scripts/repo_scout.py --test
```