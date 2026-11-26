"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / mcp_audit
- **Primary Entrypoints**: `audit_mcp`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[ERROR]`, `[MCP]`, `[FAIL]`, `[INFO]`, `[OK]`
- **Witness Tests**: none declared
"""

import json
import os
import sys
from pathlib import Path

def audit_mcp():
    config_path = Path(".agent/mcp_config.json")
    if not config_path.exists():
        print(f"[ERROR] [MCP] Config missing at {config_path}")
        return False
    
    try:
        with open(config_path, "r") as f:
            config = json.load(f)
    except json.JSONDecodeError:
        print("[ERROR] [MCP] Malformed JSON in mcp_config.json")
        return False

    servers = config.get("mcpServers", {})
    issues = 0

    # 1. Check for Placeholder Hardcoding
    config_str = json.dumps(config)
    placeholders = ["YOUR_API_KEY", "YOUR_TOKEN", "PLACEHOLDER"]
    for p in placeholders:
        if p in config_str:
            print(f"[FAIL] [MCP] P0 RISK: Placeholder '{p}' found in config!")
            issues += 1

    # 2. Check for required Departmental Servers
    required_servers = ["github", "brave-search", "google-sheets"]
    for server in required_servers:
        if server not in servers:
            print(f"[FAIL] [MCP] MISSING: Required departmental server '{server}' not registered.")
            issues += 1

    # 3. Verify Transport Configuration & Env/Header Mappings
    for name, server in servers.items():
        has_command = bool(server.get("command"))
        has_url = bool(server.get("url"))
        has_http_url = isinstance(server.get("http"), dict) and bool(server.get("http", {}).get("url"))
        has_stdio_fallback = isinstance(server.get("stdio_fallback"), dict) and bool(server.get("stdio_fallback", {}).get("command"))

        if not (has_command or has_url or has_http_url or has_stdio_fallback):
            print(f"[FAIL] [MCP] Server '{name}' must specify either 'command', 'url', 'http.url', or 'stdio_fallback.command'.")
            issues += 1

        env_vars = dict(server.get("env", {}))
        if isinstance(server.get("stdio_fallback"), dict) and isinstance(server["stdio_fallback"].get("env"), dict):
            env_vars.update(server["stdio_fallback"]["env"])

        for key, val in env_vars.items():
            if isinstance(val, str) and val.startswith("${") and val.endswith("}"):
                env_key = val[2:-1]
                # Check if set in environment (may not be in current CLI process)
                if not os.getenv(env_key):
                    # Only info, as we expect some keys to be missing on dev machines
                    print(f"[INFO] [MCP] {name}: Env var {env_key} is not set in local shell.")

        headers = dict(server.get("headers", {}))
        if isinstance(server.get("http"), dict) and isinstance(server["http"].get("headers"), dict):
            headers.update(server["http"]["headers"])

        for key, val in headers.items():
            if isinstance(val, str) and val.startswith("${") and val.endswith("}"):
                env_key = val[2:-1]
                if not os.getenv(env_key):
                    print(f"[INFO] [MCP] {name}: Header placeholder {env_key} is not set in local shell.")

    if issues == 0:
        print("[OK] [MCP] Sovereign Intelligence Audit Passed. Expansion complete.")
        return True
    else:
        print(f"[FAIL] [MCP] Audit Failed with {issues} issues.")
        return False

if __name__ == "__main__":
    success = audit_mcp()
    sys.exit(0 if success else 1)
