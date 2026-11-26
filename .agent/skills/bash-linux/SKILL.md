---
name: bash-linux
description: Bash/Linux terminal patterns. Critical commands, piping, error handling, scripting. Use when working on macOS or Linux systems.
when_to_use: "When working on macOS or Linux systems, writing bash scripts, or using terminal commands. NOT for Windows/PowerShell environments."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
version: "2.1.0"
priority: medium
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

| L3 Reference Guide / Tool | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/posix_utilities_and_scripts.md`](./references/posix_utilities_and_scripts.md) | Advanced text processing (`awk`/`sed`/`jq`), script boilerplates (`set -euo pipefail`) | Linux scripting & text pipelines |
| [`scripts/bash_validator.py`](./scripts/bash_validator.py) | Deterministic shell script validator with safety checks & `--test` suite | Automated bash script validation or self-test verification |

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

## 🛡️ Socratic Gate: POSIX Target & Shell Boundary

Before writing or running bash scripts, confirm:
1. **Target Environment**: Is the runtime guaranteed to be Linux/macOS? (On Windows, use `powershell-windows`).
2. **Strict Mode Invariant**: Does the script specify `set -euo pipefail`?
3. **Safe Variable Expansion**: Are all path variables enclosed in quotes (e.g. `"$DIR"`)?

---

## ⚠️ Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Operational Risk | Sovereign Remedy |
|---|---|---|
| Unquoted `rm -rf $DIR/*` | Empty variable wipes filesystem root | Always quote: `rm -rf "${DIR:?}/"*` |
| Missing `set -e` in CI scripts | Silent failures cascade into corrupted states | Enforce `set -euo pipefail` on Line 2 |
| Interactive `sudo` commands | Agent hangs waiting for terminal password | Run non-interactive or leverage local sandbox directories |

---

## 🛠️ Verification & Execution Commands

```bash
# Validate shell script
python .agent/skills/bash-linux/scripts/bash_validator.py scripts/deploy.sh

# Run self-test suite
python .agent/skills/bash-linux/scripts/bash_validator.py --test
```