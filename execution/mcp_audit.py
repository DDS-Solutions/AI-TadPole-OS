"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Core technical resource for the Tadpole OS Sovereign infrastructure.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[mcp_audit]` in system logs.
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

    # 3. Verify Env Mappings
    for name, server in servers.items():
        env_vars = server.get("env", {})
        for key, val in env_vars.items():
            if val.startswith("${") and val.endswith("}"):
                env_key = val[2:-1]
                # Check if set in environment (may not be in current CLI process)
                if not os.getenv(env_key):
                    # Only info, as we expect some keys to be missing on dev machines
                    print(f"[INFO] [MCP] {name}: Env var {env_key} is not set in local shell.")

    if issues == 0:
        print("[OK] [MCP] Sovereign Intelligence Audit Passed. Expansion complete.")
        return True
    else:
        print(f"[FAIL] [MCP] Audit Failed with {issues} issues.")
        return False

if __name__ == "__main__":
    success = audit_mcp()
    sys.exit(0 if success else 1)

# Metadata: [mcp_audit]
