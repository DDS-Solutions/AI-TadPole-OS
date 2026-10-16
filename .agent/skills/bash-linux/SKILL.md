---
name: bash-linux
description: Bash/Linux terminal patterns. Critical commands, piping, error handling, scripting. Use when working on macOS or Linux systems.
when_to_use: "When working on macOS or Linux systems, writing bash scripts, or using terminal commands. NOT for Windows/PowerShell environments."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / bash-linux
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Bash / Linux Terminal Patterns

> **Scope**: Essential CLI execution, process management, and scripting patterns for macOS / Linux environments.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** terminal rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/posix_utilities_and_scripts.md`](./references/posix_utilities_and_scripts.md) | Advanced text processing (`awk`/`sed`/`jq`), script boilerplates (`set -euo pipefail`) | Linux scripting & text pipelines |

---

## 1. Operator & Execution Syntax

| Operator | Action / Behavior | Example |
|---|---|---|
| `&&` | Execute second command only if first succeeds | `cargo build --release && ./target/release/server` |
| `\|\|` | Execute second command only if first fails | `npm test \|\| echo "Tests failed"` |
| `\|` | Stream stdout to stdin of next command | `ps aux \| grep "tadpole"` |

---

## 2. Process & Port Control

```bash
# 1. Inspect processes listening on a port
lsof -i :8000

# 2. Terminate rogue process by port
kill -9 $(lsof -t -i :8000)

# 3. Stream live logs
tail -n 100 -f data/logs/engine.jsonl
```

---

## 🚫 3. Critical Safety Rules

1. **Quote Variable Expansions**: Always quote variables (`rm -rf "$TARGET_DIR"`) to prevent accidental root deletions.
2. **Fail Fast in Scripts**: Always include `set -euo pipefail` at the top of executable scripts.