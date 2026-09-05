"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / build_socratic_envelope
- **Primary Entrypoints**: `load_env_configs`, `load_error_registry`, `get_agent_data`, `compile_socratic_envelope`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import os
import sys
import json
import sqlite3
import argparse
from pathlib import Path
from typing import Dict, Any, List, Optional

# UTF-8 stdout setup for Windows PowerShell
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding='utf-8')
        sys.stderr.reconfigure(encoding='utf-8')
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
DB_PATH = ROOT / "data" / "tadpole.db"
ENV_PATH = ROOT / ".env"
ERROR_REGISTRY_PATH = ROOT / "docs" / "ERROR_REGISTRY.json"

def load_env_configs() -> Dict[str, Any]:
    """Loads environment configuration thresholds from .env with fallback defaults."""
    env_vars = {
        "PRIVACY_MODE": False,
        "DEFAULT_AGENT_BUDGET_USD": 1.0,
        "MAX_SWARM_DEPTH": 5,
        "PROVIDER_TIMEOUT_SECS": 300,
        "CONTINUITY_JOB_TIMEOUT_SECS": 300,
        "OLLAMA_HOST": "http://127.0.0.1:11434"
    }

    if ENV_PATH.exists():
        try:
            with open(ENV_PATH, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line or line.startswith("#") or "=" not in line:
                        continue
                    k, v = line.split("=", 1)
                    k = k.strip()
                    v = v.strip().strip('"').strip("'")
                    if k == "PRIVACY_MODE":
                        env_vars["PRIVACY_MODE"] = v.lower() in ["true", "1", "yes"]
                    elif k in ["DEFAULT_AGENT_BUDGET_USD", "MAX_SWARM_DEPTH", "PROVIDER_TIMEOUT_SECS", "CONTINUITY_JOB_TIMEOUT_SECS"]:
                        try:
                            env_vars[k] = float(v) if "." in v else int(v)
                        except ValueError:
                            pass
                    elif k == "OLLAMA_HOST":
                        env_vars["OLLAMA_HOST"] = v
        except Exception as e:
            print(f"⚠️ Warning reading .env: {e}", file=sys.stderr)

    return env_vars

def load_error_registry() -> Dict[str, Any]:
    """Loads known error mappings from docs/ERROR_REGISTRY.json."""
    if ERROR_REGISTRY_PATH.exists():
        try:
            with open(ERROR_REGISTRY_PATH, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception as e:
            print(f"⚠️ Warning reading ERROR_REGISTRY: {e}", file=sys.stderr)
    return {}

def get_agent_data(agent_id: str) -> Optional[Dict[str, Any]]:
    """Fetches agent details from SQLite."""
    if not DB_PATH.exists():
        return None

    try:
        conn = sqlite3.connect(DB_PATH)
        conn.row_factory = sqlite3.Row
        cur = conn.cursor()
        cur.execute("SELECT * FROM agents WHERE id = ?", (agent_id,))
        row = cur.fetchone()
        conn.close()
        if row:
            return dict(row)
    except Exception as e:
        print(f"⚠️ Database error: {e}", file=sys.stderr)
    return None

def compile_socratic_envelope(
    agent_id: str,
    mission_vector: str,
    target_paths: Optional[List[str]] = None,
    blast_radius_level: int = 2,
    custom_mode: Optional[str] = None
) -> Dict[str, Any]:
    """
    Compiles the 4-Pillar Socratic Context Envelope deterministically.
    """
    env_cfg = load_env_configs()
    agent_data = get_agent_data(agent_id) or {}
    
    agent_role = agent_data.get("role", "General Intelligence Node")
    agent_name = agent_data.get("name", agent_id)
    budget = agent_data.get("budget_usd", env_cfg["DEFAULT_AGENT_BUDGET_USD"])
    active_slot = agent_data.get("active_model_slot", 2)
    is_privacy_mode = env_cfg["PRIVACY_MODE"]
    
    # 1. Pillar 1: [SCOPE_CONTRACT]
    scoped_targets = target_paths if target_paths else ["src/", "server-rs/src/"]
    scope_contract = {
        "target_paths": scoped_targets,
        "blast_radius_level": blast_radius_level,
        "mutation_allowed": blast_radius_level > 1,
        "database_mutation_allowed": False,
        "mission_vector": mission_vector
    }

    # 2. Pillar 2: [PERFORMANCE_THRESHOLD]
    latency_threshold_ms = 1500 if active_slot == 2 else 10000
    performance_threshold = {
        "budget_usd_cap": budget,
        "max_swarm_depth": env_cfg["MAX_SWARM_DEPTH"],
        "target_turn_latency_ms": latency_threshold_ms,
        "active_model_slot": active_slot,
        "timeout_seconds": env_cfg["PROVIDER_TIMEOUT_SECS"]
    }

    # 3. Pillar 3: [ARCHITECTURE_MODE]
    mode = custom_mode or (
        "Nexus Engineer Mode (Zero Trust Auditor + Principal QA)"
        if "audit" in mission_vector.lower() or "security" in agent_role.lower()
        else "Sovereign 3-Layer Architecture (Directives -> Orchestration -> Execution)"
    )
    architecture_mode = {
        "mode_name": mode,
        "privacy_shield_enforced": is_privacy_mode,
        "standards_compliance": ["Zero Trust", "L1/L2/L3 agentskills.io", "DESIGN.md"],
        "assigned_persona": agent_role
    }

    # 4. Pillar 4: [FAILURE_MODES_PRE-CLEARED]
    pre_cleared_failures = [
        "Circuit Breaker: Halt after 3 non-convergent iterations as Logic-Blocker (Directive #3).",
        "Air-Gap Shield: Local Ollama fallback strictly enforced if cloud providers blocked (PRIVACY_MODE).",
        "Tool Execution Rule: Check execution/ for existing tools before writing ad-hoc scripts (Layer 1/2/3).",
        "Verification Gate: Require automated validation pass (parity_guard / vitest) before state commit."
    ]

    envelope = {
        "envelope_version": "1.0",
        "status": "PRE_CLEARED_GATE_PASS",
        "target_agent": {
            "id": agent_id,
            "name": agent_name,
            "role": agent_role
        },
        "scope_contract": scope_contract,
        "performance_threshold": performance_threshold,
        "architecture_mode": architecture_mode,
        "pre_cleared_failure_modes": pre_cleared_failures
    }

    return envelope

def format_envelope_markdown(envelope: Dict[str, Any]) -> str:
    """Formats the envelope as a markdown comment block for prompt injection."""
    sc = envelope["scope_contract"]
    pt = envelope["performance_threshold"]
    am = envelope["architecture_mode"]
    pf = envelope["pre_cleared_failure_modes"]
    ag = envelope["target_agent"]

    paths_str = ", ".join([f"`{p}`" for p in sc["target_paths"]])

    md = f"""<!-- SOCRATIC_GATE_ENVELOPE: PRE-CLEARED -->
### 🛡️ Pre-Injected Socratic Context Contract (Zero-Stall Gate Pass)
*Target Node: `{ag['name']}` (`{ag['role']}`) | Mission: {sc['mission_vector']}*

1. 🎯 **[SCOPE_CONTRACT]**
   - **Target Paths**: {paths_str}
   - **Blast Radius Level**: Level {sc['blast_radius_level']} (Mutations: {'Allowed' if sc['mutation_allowed'] else 'Read-Only'})
   - **Database Mutation**: {'Allowed' if sc['database_mutation_allowed'] else 'Blocked (Read-Only Registry)'}

2. ⚡ **[PERFORMANCE_THRESHOLD]**
   - **Fiscal Budget Cap**: `${pt['budget_usd_cap']:.2f} USD` (Local Mode: Zero Egress)
   - **Max Swarm Depth**: `Depth <= {pt['max_swarm_depth']}` | **Turn Latency Target**: `< {pt['target_turn_latency_ms']}ms` (Slot {pt['active_model_slot']})
   - **Execution Timeout**: `{pt['timeout_seconds']}s`

3. 🏛️ **[ARCHITECTURE_MODE]**
   - **Active Mode**: `{am['mode_name']}`
   - **Privacy Shield**: `{'ENFORCED (100% Local Air-Gap)' if am['privacy_shield_enforced'] else 'Standard'}`
   - **Governance Compliance**: `Zero Trust`, `L1/L2/L3 agentskills.io`, `DESIGN.md`

4. ⚖️ **[PRE-CLEARED FAILURE POLICIES & TRADE-OFFS]**
"""
    for item in pf:
        md += f"   - {item}\n"

    md += "<!-- /SOCRATIC_GATE_ENVELOPE -->\n"
    return md

def main():
    parser = argparse.ArgumentParser(description="Tadpole OS Socratic Gate Context Auto-Injection Compiler")
    parser.add_argument("--agent", default="99", help="Target agent ID (e.g. '99', 'system_architect')")
    parser.add_argument("--vector", default="System Audit & Quality Verification", help="Mission vector or goal")
    parser.add_argument("--paths", nargs="*", default=["src/", "server-rs/src/"], help="Target scoped file paths")
    parser.add_argument("--format", choices=["json", "markdown"], default="markdown", help="Output format")
    args = parser.parse_args()

    envelope = compile_socratic_envelope(
        agent_id=args.agent,
        mission_vector=args.vector,
        target_paths=args.paths
    )

    if args.format == "json":
        print(json.dumps(envelope, indent=2))
    else:
        print(format_envelope_markdown(envelope))

if __name__ == "__main__":
    main()
