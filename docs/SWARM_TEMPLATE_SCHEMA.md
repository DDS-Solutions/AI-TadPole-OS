# Swarm Template Schema (`swarm.json`)

The `swarm.json` file is the entry point for completely redefining Tadpole OS's intelligence. By packaging agents, skills, and workflows into a single folder structure driven by this schema, you can export and import complex swarms seamlessly.

## General Structure

A complete swarm template directory typically looks like this:

```text
/my-custom-swarm/
├── swarm.json               # The master configuration file
├── agents/                  # Individual agent definitions (one JSON per agent)
│   ├── archivist.json
│   └── researcher.json
├── skills/                  # Local tool scripts or definitions
│   └── search_db.json
├── workflows/               # Markdown SOPs injected into agent context
│   └── data_pipeline.md
└── mcps.json                # A list of external MCP servers required
```

## `swarm.json` Specification

The `swarm.json` file dictates the hierarchy, global defaults, and pointers to the required agents.

```json
{
  "$schema": "https://tadpoleos.dev/schemas/swarm-v1.json",
  "name": "Legal Intelligence Swarm",
  "version": "1.0.0",
  "author": "SMB Legal Inc.",
  "description": "A high-performance swarm tailored for contract analysis and patent law.",
  "industry": "legal",
  "tags": ["law", "contracts", "auditing"],
  
  "defaults": {
    "model": "gemini-pro-latest",
    "temperature": 0.2
  },

  "roster": [
    {
      "id": "legal_ceo",
      "path": "agents/managing_partner.json",
      "supervisor": null,
      "priority": "critical"
    },
    {
      "id": "contract_auditor",
      "path": "agents/auditor.json",
      "supervisor": "legal_ceo",
      "priority": "high"
    }
  ],

  "required_mcps": "mcps.json",
  "global_workflows": [
    "workflows/contract_intake.md"
  ]
}
```

### Schema Definitions

#### `roster`
The `roster` array defines the graph/tree of the swarm. 
* `id`: A unique string identifying the agent within this swarm.
* `path`: Relative path to the specific agent definition JSON file.
* `supervisor`: The `id` of the parent agent that governs this agent. If `null`, this is an "Alpha" node (top-level).

#### Agent Definition File (`agents/*.json`)

Each agent referenced in the `roster` must have a corresponding file. This allows clean, modular agent design.

Example `agents/managing_partner.json`:
```json
{
  "name": "Harvey",
  "role": "Managing Partner",
  "department": "Executive",
  "description": "Oversees all legal operations and delegates audits.",
  "system_prompt": "You are Harvey, the Managing Partner. You coordinate the internal legal audit swarm...",
  "primary_model": "gemini-pro-latest",
  "budget_usd": 50.00,
  "skills": ["issue_alpha_directive", "delegate"],
  "workflows": ["workflows/contract_intake.md"]
}
```

## Security Requirements (Sapphire Shield)

If the swarm requires potentially dangerous external MCP servers or dangerous local skills, the user will be warned during installation.

*   Templates **must not** contain `.exe`, `.bat`, `.sh` files directly.
*   Required MCP servers must be clearly defined in `mcps.json`, and if they require API keys, Tadpole OS will prompt the user to configure them locally rather than embedding them in the template.
