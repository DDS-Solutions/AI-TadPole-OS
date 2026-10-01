---
name: skillify
description: Auto-create new skills from repetitive workflows. When you notice yourself doing the same multi-step process repeatedly, extract it into a reusable SKILL.md that any agent can use.
when_to_use: "When the user says 'make this a skill', 'create a skill for this', 'I keep doing this same thing', or when a repetitive multi-step pattern is observed. NOT for one-off tasks."
allowed-tools: Read, Write, Glob, Grep
effort: low
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / skillify
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Skillify — Auto-Create Skills from Workflows

> Turn repetitive patterns into reusable skills. If you've done it three times, it should be a skill.

## When to Skillify

✅ **Good candidates:**
- You've seen the user ask for the same type of work 3+ times
- A workflow involves 5+ consistent steps
- The pattern works across different projects
- Other agents could benefit from this knowledge

❌ **Bad candidates:**
- One-off tasks (just do them)
- Project-specific hacks (use memory instead)
- Already covered by existing skills (check first)

---

## Skill Creation Protocol

### Step 1: Identify the Pattern
```
What triggers this workflow? (user says X, file type Y, domain Z)
What steps are always the same?
What parts vary between uses?
What's the expected output?
```

### Step 2: Generate SKILL.md

> **Skill Authoring Rules (L1/L2/L3 Progressive Disclosure Specification)**:
> 1. **L1 (Metadata)**: `name` MUST be kebab-case ($\le 64$ chars). `description` MUST be trigger-optimized with specific action keywords ($\le 1024$ chars).
> 2. **L2 (Instructions)**: Keep `SKILL.md` body under 5,000 words focusing purely on procedural instructions. Use **bold emphasis** for non-negotiable assertions.
> 3. **L3 (Resources)**: Offload heavy API schemas or docs to `references/`, boilerplate to `assets/`, and fragile execution logic to deterministic scripts in `scripts/`.
> 4. **Checkable Completion Criteria**: Every step MUST end on an explicit, checkable criterion.

Use this template:

```markdown
---
name: [kebab-case-name-max-64-chars]
description: [Actionable, trigger-optimized summary describing when to activate this skill, max 1024 chars]
when_to_use: "[When the user asks X, works with Y files, or Z domain. NOT for A.]"
allowed-tools: [Read, Write, Edit, Grep, Glob, Bash — only what's needed]
disable-model-invocation: true # Set to true if manually invoked only
effort: [low | medium | high]
---

# [Skill Name] — [Short Subtitle]

> [One-line philosophy or principle]

## Overview
[2-3 sentences explaining what this skill enables]

## When to Use
✅ Good for: [list]
❌ Not for: [list]

## Protocol
### Step 1: [First Action]
[Action-first instructions with **bold emphasis** on strict constraints]

### Step 2: [Second Action]
[Instructions]

### Step N: [Verification]
[Explicit, checkable completion criterion]

## Best Practices
[3-5 key rules]
```

### Step 3: Place the Skill & Resources
```text
.agent/skills/[skill-name]/
├── SKILL.md          # Core instructions (L1 + L2)
├── scripts/          # (Optional) Deterministic Python/JS scripts for fragile steps
├── references/       # (Optional) Detailed documentation & API schemas
└── assets/           # (Optional) Templates & static non-executable resources
```

### Step 4: Verify
- [ ] Frontmatter `name` is valid kebab-case ($\le 64$ characters)
- [ ] `description` includes target action keywords ($\le 1024$ characters)
- [ ] `when_to_use` clearly defines triggers AND exclusions
- [ ] Steps use **bold text** for critical constraints and end with checkable criteria
- [ ] Complex/heavy data is offloaded to `references/` or `scripts/`

---

## Naming Conventions

| Pattern | Skill Name | Example |
|---|---|---|
| Action-based | `[verb]-[noun]` | `verify-changes`, `batch-operations` |
| Domain-based | `[domain]-[aspect]` | `database-design`, `api-patterns` |
| Tool-based | `[tool]-patterns` | `tailwind-patterns`, `prisma-expert` |

**Rules:**
- kebab-case only ($\le 64$ chars)
- 2-3 words maximum
- Descriptive, not clever
- No abbreviations unless universally known

---

## Quality Checklist

Before finalizing a new skill:

| Check | Criteria |
|---|---|
| **Uniqueness** | No existing skill covers this (grep `.agent/skills/`) |
| **L1 Discovery** | `name` ($\le 64$ chars) and `description` ($\le 1024$ chars) optimized for model triggering |
| **L2 Conciseness** | `SKILL.md` body is concise ($\le 5\text{k}$ words), action-first, and uses bolding for strict constraints |
| **L3 Separation** | Schemas/docs in `references/`, code in `scripts/`, templates in `assets/` |
| **Checkable Criteria**| Every protocol step ends on an explicit verification gate |