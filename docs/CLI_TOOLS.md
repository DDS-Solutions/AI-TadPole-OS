> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / CLI_TOOLS
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# 🛠️ CLI Operations: Tadpole OS Tooling

![Status: Verified](https://img.shields.io/badge/Status-Verified-green)

Tadpole OS provides a suite of deterministic Python scripts for lifecycle management, memory synthesis, and post-mission debriefing. These tools are located in the `execution/` directory.

---

## 🧠 Mission Debriefing (`debrief_mission.py`)

The primary tool for extracting institutional memory and architectural wisdom from mission logs.

### Usage
```bash
python execution/debrief_mission.py <MISSION_ID> [--commit]
```

### Options
| Flag | Description | Default |
|:--- |:--- |:--- |
| `<MISSION_ID>` | Required positional mission ID to debrief. | None |
| `--commit` | **Commit Learning**: Appends synthesized insights to `directives/LONG_TERM_MEMORY.md`. | `false` |

### Workflow
1.  **Extraction**: Reads mission history and logs from the configured SQLite database.
2.  **Synthesis**: Uses the configured Groq-compatible endpoint to identify failures and technical lessons.
3.  **Persistence**: Formats insights into standard markdown for the OS's long-term memory.

---

## 🐍 Python Script Audit (`audit_python_scripts.py`)

Performs a side-effect-free inspection of every workspace `.py` file. It parses and
compiles each source file without importing it, checks import availability, records
network/database/process capabilities, and flags non-portable paths or unsafe APIs.

### Usage
```bash
npm run audit:python
# Fail on warnings that require manual review:
python execution/audit_python_scripts.py . --strict-warnings
```

The machine-readable report is written to
`reports/intelligence/python_script_audit.json`. Generated dependency, build, virtual
environment, and `.tmp` directories are excluded; repository `tmp/` scripts remain in scope.

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
npm run graph:blast:guard -- --path src/pages/Neural_Map.tsx
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

`graph:blast:guard` is the mandatory pre-change alias for the `file` query. The graph CLI does not expose `prune` or `watch` commands; each invocation rebuilds its source snapshot before querying.

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

## 🏗️ Deployment Scripts

The maintained PowerShell tools build Linux desktop artifacts and deploy a `.deb` to
a configured Linux host.

### Features
- `scripts/build-linux-light.ps1`: builds `.deb` and `.AppImage` artifacts using Docker.
- `scripts/deploy-linuxlite.ps1`: transfers and installs a built `.deb` over SSH.