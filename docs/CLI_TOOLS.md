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

# 🛠️ CLI Operations: Tadpole OS Tooling

![Status: Verified](https://img.shields.io/badge/Status-Verified-green)

Tadpole OS provides a suite of deterministic Python scripts for lifecycle management, memory synthesis, and post-mission debriefing. These tools are located in the `execution/` directory.

---

## 🧠 Mission Debriefing (`debrief_mission.py`)

The primary tool for extracting institutional memory and architectural wisdom from mission logs.

### Usage
```bash
python execution/debrief_mission.py [OPTIONS]
```

### Options
| Flag | Description | Default |
|:--- |:--- |:--- |
| `--mission_id` | Specific ID to debrief. If omitted, uses the latest mission. | Latest |
| `--commit` | **Commit Learning**: Appends synthesized insights to `directives/LONG_TERM_MEMORY.md`. | `false` |
| `--verbose` | Displays detailed synthesis logic and LLM reasoning. | `false` |

### Workflow
1.  **Extraction**: Reads the SQLite mission logs from the `workspaces/` directory.
2.  **Synthesis**: Uses Groq/Ollama to identify "Strategic Failures" and "Technical Wins".
3.  **Persistence**: Formats insights into standard markdown for the OS's long-term memory.

---

## 🛡️ Parity Guard (`parity_guard.py`)

The integrity gatekeeper used to ensure that code and documentation remain in a state of "Sovereign Parity."

### Usage
```bash
python execution/parity_guard.py [FIX=1]
```

### Checks Performed
- **API Parity**: Compares Axum routes (`router.rs`) against `openapi.yaml`.
- **Tag Verification**: Scans all source files for `@docs` tags and verifies the destination file exists.
- **Environment Safety**: Checks if all `std::env::var` calls in the engine are documented in `.env.example`.
- **Skill Validation**: Verifies that every JSON skill manifest has a corresponding execution script on disk.

---

## 🧠 Graph Intelligence CLI (`graph_query`)

The scriptable binary for symbol graph queries, audit context generation, and Active Documentation Guard enforcement. Located at `server-rs/src/bin/graph_query/` (decomposed into 5 modules in ADG-05).

### Usage
```powershell
cargo run --bin graph_query -- <SUBCOMMAND> [OPTIONS]
# or via npm scripts:
npm run graph:lookup  -- --name SymbolName
npm run graph:file    -- --path src/pages/Neural_Map.tsx
npm run graph:blast   -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius
npm run graph:export
```

### Subcommands
| Subcommand | Description |
|:--- |:--- |
| `export` | Export the full symbol graph as JSON (`--pretty`, `--out`) |
| `audit-context` | Generate high-connectivity audit context for AI agents (`--limit`) |
| `lookup` | Find all definitions of a named symbol (`--name`, `--json`, `--out`) |
| `file` | List all symbols defined in a file (`--path`, `--json`, `--out`) |
| `blast` | Calculate blast radius of a symbol (`--name`, `--path`, `--depth`, `--format`) |
| `validate` | Run Active Documentation Guard on docstring symbols (`--strict`, `--diff`, `--fix`) |

### `blast` Format Options (`--format`)
| Value | Output |
|:--- |:--- |
| `text` (default) | Human-readable stdout |
| `mermaid` | Mermaid `graph TD` flowchart |
| `html` | Interactive Cytoscape.js dark-mode HTML page |

### `validate` Flags
| Flag | Description |
|:--- |:--- |
| `--strict` | Fail with exit code 1 if any symbol is unresolved |
| `--diff` | Restrict scan to git-modified files only (fast pre-commit mode) |
| `--fix` | Auto-correct drifted symbol names using Jaro-Winkler similarity |

---

## 🏗️ Deployment Engine (`deploy.py`)

Handles safe environment transitions and sidecar process management.

### Features
- **Environment Discovery**: Automatically detects if running in a "Bunker" node vs. a local workstation.
- **Port Orchestration**: Manages the coordination between the Rust engine (8000) and the Vite dashboard (5173).

[//]: # (Metadata: [CLI_TOOLS])
