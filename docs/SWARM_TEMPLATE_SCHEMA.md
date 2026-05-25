> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Automated governance and architectural tracking.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.

# 📄 Swarm Template Schema
**Intelligence Level**: High
**Preferred Format**: Sovereign Recipe (YAML)
**Legacy Format**: Swarm Template (JSON)
**Last Updated**: 2026-04-16 (OBLITERATUS Integration)

---

## 📚 Table of Contents

- [Sovereign Recipe (YAML)](#sovereign-recipe-yaml)
- [Legacy Swarm Template (JSON)](#legacy-swarm-template-json)
- [Security Requirements (Sapphire Shield)](#security-requirements-sapphire-shield)

---

## 🧬 Sovereign Recipe (YAML) [Recommended]

The **Sovereign Recipe** is the modern, declarative standard for auto-provisioning agent departments. These blueprints reside in `.agent/recipes/` and are automatically ingested by the engine on startup.

### File Structure
```yaml
name: "Neural Research Departement"
description: "A high-performance cluster for deep technical research and synthesis."
agents:
  - id: "lead_researcher"
    name: "Turing-01"
    role: "Lead Researcher"
    department: "Intelligence"
    description: "Orchestrates technical deep dives and validates findings."
    model_id: "llama3:70b-instruct"
    skills: ["read_file", "search_web", "google_scout"]
    workflows: ["research_protocol.md"]

  - id: "documentation_specialist"
    name: "Lovelace-02"
    role: "Technical Writer"
    department: "Documentation"
    description: "Synthesizes research notes into industry-standard manuals."
    model_id: "gpt-4o"
    skills: ["write_to_file", "git_push"]
```

### Schema Attributes

#### `SwarmRecipe` (Root)
* `name`: Human-readable name for the swarm.
* `description`: Detailed mission objective for the cluster.
* `agents`: A list of `AgentRecipe` objects.

#### `AgentRecipe`
* `id`: Unique technical identifier for the agent (used for recruitment).
* `name`: Call-sign for the agent.
* `role`: Strategic role definition.
* `department`: Organizational department.
* `description`: Core mission for this specific agent.
* `model_id`: The LLM model to use (supports quantization fallback, e.g., `:q4_K_M`).
* `skills`: List of tools/skills assigned by default.
* `workflows`: List of markdown-based SOPs.

---

## 📜 Legacy Swarm Template (JSON) [Deprecated]

Legacy templates use a nested directory structure with a `swarm.json` master file and individual agent JSONs. It is recommended to migrate to Sovereign Recipes for all new swarms.

```text
/my-custom-swarm/
├── swarm.json               # The master configuration file
├── agents/                  # Individual agent definitions (one JSON per agent)
│   ├── archivist.json
│   └── researcher.json
...
```

### `swarm.json` Specification
```json
{
  "name": "Legal Intelligence Swarm",
  "version": "1.0.0",
  "roster": [
    {
      "id": "legal_ceo",
      "path": "agents/managing_partner.json",
      "supervisor": null
    }
  ]
}
```

---

## 🛡️ Security Requirements (Sapphire Shield)

Every recipe or template is subject to the **Sapphire Shield** security protocol:
- **No Executables**: Templates must not contain binary executables (`.exe`, `.sh`).
- **Oversight Gate**: Any tool with `shell:execute` or `fs:write` requires manual human approval by default.
- **Privacy Mode**: If `Privacy Shield` is active, cloud-based models in recipes will fail over to the local `null_provider`.

[//]: # (Metadata: [SWARM_TEMPLATE_SCHEMA])

[//]: # (Metadata: [SWARM_TEMPLATE_SCHEMA])
