> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: bash-linux
description: Bash/Linux terminal patterns. Critical commands, piping, error handling, scripting.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# Bash Linux Patterns

**The terminal is a sharp knife.**

## Critical Operators
- `&&` (Success), `||` (Fail), `|` (Pipe), `;` (Seq).

## Script Template
```bash
#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
log_info() { echo -e "\033[0;32m[INFO]\033[0m $1"; }
main() {
    log_info "Starting..."
    # Logic
}
main "$@"
```

---

## 🧠 Aletheia Reasoning Protocol (Shell Scripting)

### 1. Generator (Command Logic)
*   **Draft**: "Find files -> loop -> grep -> delete".
*   **One-liner vs Script**: "Too complex? Make a file".
*   **Tool Choice**: "Awk vs Python?".

### 2. Verifier (Destruction Check)
*   **The "Run on Root" Test**: "If `rm -rf $VAR/` runs and $VAR is empty?".
*   **Dry Run**: "Echo before rm".
*   **Exit Codes**: "What if grep fails?".

### 3. Reviser (Hardening)
*   **Strict Mode**: `set -euo pipefail`.
*   **Quoting**: Quote `"$VAR"`.

---

## 🛡️ Security & Safety Protocol (Bash)

1.  **User Input**: NEVER `eval` user input.
2.  **Sudoless**: Avoid `sudo` in scripts.
3.  **Pipe Hygiene**: Inspect before piping to bash.
4.  **Secrets**: No hardcoded secrets. Env vars only.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
