#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / map_agent_workflows
- **Primary Entrypoints**: `get_workflows_for_agent`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import json
import sqlite3
from pathlib import Path

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
DB_PATH = ROOT / "data" / "tadpole.db"
WORKFLOWS_DIR = ROOT / ".agent" / "workflows"

VALID_WORKFLOWS = {p.stem for p in WORKFLOWS_DIR.glob("*.md")}

def get_workflows_for_agent(agent_id: str, name: str, role: str, department: str) -> list[str]:
    aid = agent_id.lower()
    aname = name.lower()
    arole = role.lower()
    adept = department.lower()

    # 1. Utilities & Direct ID Handlers
    if aid == "utility_writer":
        return ["codify", "report", "onboard", "brainstorm"]
    if aid == "utility_security_auditor":
        return ["audit", "adversary", "debug", "status", "report"]
    if aid == "utility_network_scout":
        return ["github_scout", "brainstorm", "report", "status"]

    # 2. Executive / Strategic Leads
    if aid in ["1", "agent of nine", "ceo"] or "strategic intelligence lead" in arole:
        return ["orchestrate", "plan", "report", "status", "codify", "onboard"]
    if aid in ["2", "tadpole alpha"] or arole == "coo" or aid == "githubscraper":
        return ["orchestrate", "plan", "report", "status", "codify", "onboard"]
    if aid in ["3", "elon"] or arole == "cto":
        return ["plan", "refactor", "migrate", "preview", "deploy", "codify"]

    # 3. Documentation, Writing, Marketing, Community
    if "writer" in aid or "doc" in aid or "technical_writer" in aid or "copywriter" in arole:
        return ["codify", "report", "onboard", "brainstorm"]
    if "marketing" in aid or "community" in aid:
        return ["brainstorm", "report", "onboard", "status"]

    # 4. Security, Risk & Red Team
    if "security" in aid or "sec-1" in aid or "risk" in aid:
        return ["audit", "adversary", "debug", "status", "report"]

    # 5. QA, Auditors, Quality Assurance
    if aid == "99" or "qa-99" in aname or "quality auditor" in arole or "quality-auditor" in arole or "checkmate" in aname or "audit" in aid or aid == "filer":
        return ["audit", "test", "debug", "anneal", "adversary", "report"]

    # 6. GitHub Specialists, Crawlers & Scrapers
    if "github" in aid or "scraper" in aid or "crawler" in aid:
        return ["github_scout", "report", "status", "enhance"]

    # 7. Research, Web Search, External Intelligence
    if "research" in aid or "search" in aid or "news" in aid or "external_data" in aid or "scout" in aid:
        return ["brainstorm", "github_scout", "report", "status"]

    # 8. Architecture & Tadpole Core Specialists
    if "architect" in aid or "architect" in aname:
        return ["plan", "refactor", "migrate", "preview", "deploy", "codify"]
    if "tadpole_os_specialist" in aid:
        return ["plan", "refactor", "preview", "deploy", "enhance", "test"]

    # 9. Engineering & Coders
    if aid == "coder" or "designer" in arole or "developer" in arole:
        return ["create", "enhance", "refactor", "ui-ux-pro-max", "preview", "test"]
    if "repo_setup" in aid:
        return ["create", "enhance", "preview", "test", "deploy"]

    # 10. Planners, Coordinators, Product
    if "plan" in aid or "coordinator" in aid or "manager" in aid or aid in ["alpha", "23"]:
        return ["plan", "brainstorm", "create", "enhance", "status"]

    # 11. Assistants & Automation
    if "calendar" in aid or "weather" in aid:
        return ["status", "report", "onboard"]

    # Default fallback
    return ["orchestrate", "plan", "status", "report"]

def main():
    if not DB_PATH.exists():
        print(f"Database not found at {DB_PATH}")
        sys.exit(1)

    conn = sqlite3.connect(str(DB_PATH))
    cursor = conn.cursor()

    cursor.execute("SELECT id, name, role, department, workflows FROM agents;")
    rows = cursor.fetchall()

    print(f"Mapping workflows for {len(rows)} agents using .agent/workflows repository ({len(VALID_WORKFLOWS)} available workflows)...")
    updated_count = 0

    for agent_id, name, role, department, current_workflows in rows:
        assigned_workflows = get_workflows_for_agent(agent_id, name, role or "", department or "")
        valid_assigned = [w for w in assigned_workflows if w in VALID_WORKFLOWS]
        serialized = json.dumps(valid_assigned)

        if current_workflows != serialized:
            print(f"  [+] Agent '{agent_id}' ({name} - {role}):")
            print(f"      Workflows -> {serialized}")
            cursor.execute(
                "UPDATE agents SET workflows = ? WHERE id = ?;",
                (serialized, agent_id)
            )
            updated_count += 1

    conn.commit()
    conn.close()

    print(f"\n[Workflows Alignment Complete] Successfully mapped workflows for {updated_count} agents.")

if __name__ == "__main__":
    main()
