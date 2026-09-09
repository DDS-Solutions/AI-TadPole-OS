#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / map_agent_skills
- **Primary Entrypoints**: `get_skills_for_agent`, `main`

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
SKILLS_DIR = ROOT / ".agent" / "skills"

VALID_SKILLS = {p.name for p in SKILLS_DIR.iterdir() if (p / "SKILL.md").exists()}

def get_skills_for_agent(agent_id: str, name: str, role: str, department: str) -> list[str]:
    aid = agent_id.lower()
    aname = name.lower()
    arole = role.lower()
    adept = department.lower()
    
    # 1. Specific Utilities & Direct ID Handlers
    if aid == "utility_writer":
        return ["documentation-templates", "digital-twin-voice", "clean-code", "i18n-localization", "web-design-guidelines"]
    if aid == "utility_security_auditor":
        return ["vulnerability-scanner", "red-team-tactics", "pii-redaction", "code-review"]
    if aid == "utility_network_scout":
        return ["explorer-scout", "research", "seo-fundamentals", "world-model-synthesis", "geo-fundamentals"]

    # 2. Executive / Strategic Leads
    if aid in ["1", "agent of nine", "ceo"] or "strategic intelligence lead" in arole:
        return [
            "intelligent-routing",
            "coordinator-mode",
            "parallel-agents",
            "aletheia-reasoning",
            "mission-analyst",
            "context-compression"
        ]
    if aid in ["2", "tadpole alpha"] or arole == "coo":
        return [
            "coordinator-mode",
            "parallel-agents",
            "plan-writing",
            "mission-analyst",
            "context-compression",
            "intelligent-routing",
            "handoff"
        ]
    if aid in ["3", "elon"] or arole == "cto":
        return [
            "architecture",
            "improve-codebase-architecture",
            "rust-pro",
            "clean-code",
            "server-management",
            "performance-profiling",
            "deployment-procedures",
            "bash-linux",
            "powershell-windows",
            "gemma4-local"
        ]

    # 3. Documentation, Writing, Marketing, Community
    if "writer" in aid or "doc" in aid or "technical_writer" in aid or "copywriter" in arole:
        return [
            "documentation-templates",
            "digital-twin-voice",
            "clean-code",
            "i18n-localization",
            "web-design-guidelines"
        ]
    if "marketing" in aid or "community" in aid:
        return [
            "digital-twin-voice",
            "seo-fundamentals",
            "documentation-templates",
            "brainstorming"
        ]

    # 4. Security, Risk & Red Team
    if "security" in aid or "sec-1" in aid or "risk" in aid:
        return [
            "vulnerability-scanner",
            "red-team-tactics",
            "pii-redaction",
            "behavioral-modes",
            "aletheia-reasoning"
        ]

    # 5. QA, Auditors, Quality Assurance
    if aid == "99" or "qa-99" in aname or "quality auditor" in arole or "quality-auditor" in arole:
        return [
            "code-review-graph",
            "code-review",
            "code-review-checklist",
            "testing-patterns",
            "systematic-debugging",
            "mission-analyst"
        ]
    if "audit" in aid or "auditor" in arole or "checkmate" in aname:
        return [
            "code-review-graph",
            "code-review",
            "testing-patterns",
            "tdd-workflow",
            "webapp-testing",
            "lint-and-validate",
            "to-spec"
        ]
    if aid == "filer":
        return [
            "code-review",
            "lint-and-validate",
            "verify-changes",
            "batch-operations"
        ]

    # 6. GitHub Specialists, Crawlers & Scrapers
    if "github" in aid or "scraper" in aid or "crawler" in aid:
        return [
            "explorer-scout",
            "batch-operations",
            "lint-and-validate",
            "clean-code",
            "research"
        ]

    # 7. Research, Web Search, External Intelligence
    if "research" in aid or "search" in aid or "news" in aid or "external_data" in aid or "scout" in aid:
        return [
            "explorer-scout",
            "research",
            "smb-data-ingestion",
            "seo-fundamentals",
            "world-model-synthesis",
            "geo-fundamentals"
        ]

    # 8. Architecture & Tadpole Core Specialists
    if "architect" in aid or "architect" in aname:
        return [
            "architecture",
            "domain-modeling",
            "database-design",
            "api-patterns",
            "improve-codebase-architecture",
            "mcp-builder",
            "deployment-procedures"
        ]
    if "tadpole_os_specialist" in aid:
        return [
            "architecture",
            "clean-code",
            "rust-pro",
            "nextjs-react-expert",
            "systematic-debugging",
            "performance-profiling",
            "mcp-builder"
        ]

    # 9. Engineering & Coders
    if aid == "coder" or "designer" in arole or "developer" in arole:
        return [
            "clean-code",
            "rust-pro",
            "nextjs-react-expert",
            "frontend-design",
            "tailwind-patterns",
            "simplify-code",
            "verify-changes",
            "python-patterns",
            "nodejs-best-practices",
            "web-design-guidelines",
            "mobile-design-android",
            "mobile-design-ios",
            "game-development"
        ]
    if "repo_setup" in aid:
        return [
            "app-builder",
            "prototype",
            "batch-operations",
            "verify-changes",
            "clean-code",
            "nodejs-best-practices",
            "bash-linux"
        ]

    # 10. Planners, Coordinators, Product
    if "plan" in aid or "coordinator" in aid or "manager" in aid or aid in ["alpha", "23"]:
        return [
            "plan-writing",
            "brainstorming",
            "behavioral-modes",
            "app-builder",
            "coordinator-mode",
            "to-spec",
            "wayfinder",
            "handoff"
        ]

    # 11. Assistants & Automation
    if "calendar" in aid or "weather" in aid:
        return [
            "memory-system",
            "context-compression",
            "skillify",
            "powershell-windows"
        ]

    # Default fallback
    return [
        "intelligent-routing",
        "context-compression",
        "memory-system",
        "clean-code"
    ]

def main():
    if not DB_PATH.exists():
        print(f"Database not found at {DB_PATH}")
        sys.exit(1)

    conn = sqlite3.connect(str(DB_PATH))
    cursor = conn.cursor()

    cursor.execute("SELECT id, name, role, department, skills FROM agents;")
    rows = cursor.fetchall()

    print(f"Mapping skills for {len(rows)} agents across .agent/skills repository ({len(VALID_SKILLS)} available skills)...")
    updated_count = 0

    for agent_id, name, role, department, current_skills in rows:
        assigned_skills = get_skills_for_agent(agent_id, name, role or "", department or "")
        valid_assigned = [s for s in assigned_skills if s in VALID_SKILLS]
        serialized = json.dumps(valid_assigned)

        if current_skills != serialized:
            print(f"  [+] Agent '{agent_id}' ({name} - {role}):")
            print(f"      Skills -> {serialized}")
            cursor.execute(
                "UPDATE agents SET skills = ? WHERE id = ?;",
                (serialized, agent_id)
            )
            updated_count += 1

    conn.commit()
    conn.close()

    print(f"\n[Skills Alignment Complete] Successfully updated {updated_count} agents.")

if __name__ == "__main__":
    main()
