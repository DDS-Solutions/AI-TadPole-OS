# Skill Authoring Specification & Templates (agentskills.io Open Standard)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / skillify / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. The L1 / L2 / L3 Progressive Disclosure Specification

```
.agent/skills/<skill-name>/
├── SKILL.md                          ← L1 (Frontmatter) + L2 (Procedural Body <= 5k words)
├── references/                       ← L3 Deep schemas, matrices, cheatsheets
│   └── <topic_guides>.md
└── scripts/                          ← L3 Deterministic Python tools with --test
    └── <tool_name>.py
```

### Layer 1: Metadata (Frontmatter)
- `name`: Strictly lowercase kebab-case $\le 64$ characters matching directory name.
- `description`: Action-first summary with trigger keywords $\le 1024$ characters.
- `when_to_use`: Clear scenario triggers and negative scopes ("NOT for X").
- `allowed-tools`: Specific comma-separated tools needed (`Read, Write, Edit, Glob, Grep, RunCommand`).
- `version`: Semantic version string (e.g. `"2.1.0"`).
- `priority`: `CRITICAL`, `HIGH`, or `MEDIUM`.

### Layer 2: Action-First Procedural Body
- AI Context & Knowledge Heritage header with failure path and parity guard target.
- Progressive Disclosure & L3 Reference Index table linking to `references/` and `scripts/`.
- Numbered, concise sections with bold emphasis on non-negotiables.
- Socratic Gate asking 3 clarifying/validation questions before execution.
- Anti-Patterns & Sovereign Remedies table.
- Deterministic Verification Gate with exact command line snippets.

### Layer 3: Deterministic Tools & Heavy Resources
- Move all documentation tables, deep schemas, and cheatsheets into `references/`.
- All fragile operations and repetitive checks must be implemented in a testable Python script in `scripts/` with a `--test` self-test suite and Windows UTF-8 stdout/stderr reconfiguration.
