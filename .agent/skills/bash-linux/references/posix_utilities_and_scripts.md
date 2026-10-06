> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / bash-linux
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unquoted variable expansions or unhandled subshell pipe failures.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[BASH_LINUX]`)

# POSIX Utilities & Shell Automation Reference (L3)

---

## 1. Advanced Text Processing & Filtering

```bash
# 1. Column extraction via awk
ps aux | awk '{print $2, $11}'

# 2. In-place regex replacement via sed
sed -i 's/OLD_HOST/NEW_HOST/g' config.env

# 3. JSON parsing via jq
curl -s http://localhost:8000/v1/health | jq '.status'
```

---

## 2. Hardened Bash Script Boilerplate

```bash
#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly LOG_FILE="/tmp/tadpole_deploy.log"

log() { echo "[$(date -u +'%Y-%m-%dT%H:%M:%SZ')] $*" | tee -a "$LOG_FILE"; }

main() {
    log "Initializing POSIX automation routine..."
}

main "$@"
```