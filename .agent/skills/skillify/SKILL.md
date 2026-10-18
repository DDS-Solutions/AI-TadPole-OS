---
name: skillify
description: "Auto-create new skills from repetitive workflows. When you notice yourself doing the same multi-step process repeatedly, extract it into a reusable SKILL.md that any agent can use."
when_to_use: "When the user says 'make this a skill', 'create a skill for this', 'I keep doing this same thing', or when a repetitive multi-step pattern is observed. NOT for one-off tasks."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / skillify
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Creating non-compliant skill layouts, oversized instruction bodies (>5k words), or omitting deterministic scripts in `scripts/`.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Skillify — Auto-Create Sovereign Skills from Workflows

> **Philosophy**: Turn repetitive patterns into reusable skills. If you have executed the same multi-step pattern 3+ times, codify it as an agentskills.io compliant skill package with L1 metadata, L2 instructions, and L3 references/scripts.  
> **Workflow Bindings**: Integrated into [`/codify`](file:///D:/TadpoleOS-Dev/.agent/workflows/codify.md) and [`/anneal`](file:///D:/TadpoleOS-Dev/.agent/workflows/anneal.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/skill_authoring_spec_and_templates.md`](file:///D:/TadpoleOS-Dev/.agent/skills/skillify/references/skill_authoring_spec_and_templates.md) | Full agentskills.io Open Specification, L1/L2/L3 disclosure hierarchy, frontmatter schemas | Authoring new skills or auditing existing skills |
| [`scripts/skill_scaffolder.py`](file:///D:/TadpoleOS-Dev/.agent/skills/skillify/scripts/skill_scaffolder.py) | Automated skill package generator creating `SKILL.md`, `references/`, and `scripts/` with tests | Scaffolding new skills from the command line |

---

## 🧭 1. When to Skillify

### ✅ Good Candidates:
- The user or swarm has requested the same multi-step workflow 3+ times.
- The process requires 4+ consistent procedural steps with deterministic tools.
- The workflow applies across multiple features, components, or repos.
- Other agents benefit from shared domain wisdom and automated scripts.

### ❌ Bad Candidates:
- One-off bug fixes or ad-hoc queries (just do them directly).
- Ephemeral user preferences (store in `.agent/memory/user-preferences.md`).
- Overlapping logic already handled by an existing skill.

---

## 📐 2. The L1/L2/L3 Progressive Disclosure Protocol

1. **L1 (Metadata)**: YAML frontmatter with `name` in kebab-case ($\le 64$ chars), trigger-dense `description` ($\le 1024$ chars), and strict `when_to_use`.
2. **L2 (Instructions)**: Procedural instructions in `SKILL.md` under 5,000 words. Non-negotiables marked in **bold**.
3. **L3 (Resources & Scripts)**:
   - Deep cheatsheets, schemas, and matrices live in `references/`.
   - Fragile or repetitive operations MUST live in deterministic Python scripts in `scripts/` with a `--test` self-test suite.

---

## 🧠 3. Socratic Gate for Skill Creation

Before scaffolding or modifying a skill, verify:
- **Gate 1 (Novelty & Deduplication)**: Does this workflow duplicate an existing skill in `.agent/skills/`?
- **Gate 2 (L1/L2/L3 Compliance)**: Does the design include L1 frontmatter, lean L2 procedural body, L3 reference guide, and an executable script with `--test`?
- **Gate 3 (Deterministic Verification)**: Does `python execution/verify_skill_specs.py` confirm 100% compliance?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Instruction Body Bloat** | Putting 10,000 words of reference docs inside `SKILL.md`. | Move deep reference material into `references/<topic>.md`. |
| **No Executable Scripts** | Writing prose instructions for mechanical tasks without tooling. | Implement deterministic Python script in `scripts/` with `--test`. |
| **Loose Files in Skill Root** | Scattering 10 markdown files in the skill root folder. | Keep only `SKILL.md` in root; place supporting files in `references/` or `scripts/`. |
| **Non-kebab-case Name** | Names with underscores or uppercase letters fail discovery. | Enforce lowercase kebab-case $\le 64$ characters. |

---

## 🛠️ 5. Deterministic Verification Gate

Scaffold and validate new skill packages:

```powershell
# 1. Run built-in self-tests
python .agent/skills/skillify/scripts/skill_scaffolder.py --test

# 2. Scaffold a new Sovereign skill
python .agent/skills/skillify/scripts/skill_scaffolder.py --name "sqlite-optimizer" --desc "SQLite database optimization and WAL mode tuning" --when "When tuning SQLite database performance"

# 3. Verify against workspace specification audit
python execution/verify_skill_specs.py
```