"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / sync_cluster_skills

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sqlite3
import json

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

DB_PATHS = [
    ROOT / "data" / "tadpole.db",
    ROOT / "server-rs" / "data" / "tadpole.db"
]

# Enhancements to guarantee complete cluster competence:
# 1. Alpha Commander (both 'Alpha' and 'alpha'):
#    Skills: add 'parallel-agents', 'mission-analyst', 'intelligent-routing'
#    Workflows: add 'orchestrate', 'report'
# 2. Elon (CTO):
#    Skills: add 'verify-changes', 'testing-patterns'
#    Workflows: add 'report', 'status', 'test'
# 3. system_architect:
#    Workflows: add 'report', 'status'
# 4. Agent 99 (QA-99):
#    Workflows: add 'status'
# 5. technical_writer:
#    Workflows: add 'status'

def enhance_skills_and_workflows():
    for db_path in DB_PATHS:
        if not db_path.exists():
            continue
        print(f"Enhancing {db_path}...")
        conn = sqlite3.connect(str(db_path))
        cur = conn.cursor()
        
        # Sync agents from data/tadpole.db to server-rs/data/tadpole.db if server-rs has stale agents
        if "server-rs" in str(db_path):
            source_conn = sqlite3.connect(str(ROOT / "data" / "tadpole.db"))
            source_cur = source_conn.cursor()
            source_agents = source_cur.execute("SELECT id, name, role, department, skills, workflows, metadata FROM agents").fetchall()
            for r in source_agents:
                cur.execute(
                    "UPDATE agents SET skills = ?, workflows = ? WHERE id = ?",
                    (r[4], r[5], r[0])
                )
            source_conn.close()
            conn.commit()

        # Now apply cluster optimizations
        # 1. Alpha / alpha
        for alpha_id in ["Alpha", "alpha"]:
            row = cur.execute("SELECT skills, workflows FROM agents WHERE id = ?", (alpha_id,)).fetchone()
            if row:
                skills = json.loads(row[0]) if row[0] else []
                workflows = json.loads(row[1]) if row[1] else []
                
                for s in ["coordinator-mode", "parallel-agents", "plan-writing", "intelligent-routing", "mission-analyst", "handoff", "context-compression"]:
                    if s not in skills:
                        skills.append(s)
                for w in ["orchestrate", "plan", "report", "status", "create", "enhance", "brainstorm"]:
                    if w not in workflows:
                        workflows.append(w)
                        
                cur.execute("UPDATE agents SET skills = ?, workflows = ? WHERE id = ?", (json.dumps(skills), json.dumps(workflows), alpha_id))

        # 2. Elon (CTO)
        row = cur.execute("SELECT skills, workflows FROM agents WHERE id = '3'").fetchone()
        if row:
            skills = json.loads(row[0]) if row[0] else []
            workflows = json.loads(row[1]) if row[1] else []
            for s in ["verify-changes", "testing-patterns", "systematic-debugging"]:
                if s not in skills:
                    skills.append(s)
            for w in ["report", "status", "test"]:
                if w not in workflows:
                    workflows.append(w)
            cur.execute("UPDATE agents SET skills = ?, workflows = ? WHERE id = '3'", (json.dumps(skills), json.dumps(workflows)))

        # 3. system_architect
        row = cur.execute("SELECT skills, workflows FROM agents WHERE id = 'system_architect'").fetchone()
        if row:
            workflows = json.loads(row[1]) if row[1] else []
            for w in ["report", "status"]:
                if w not in workflows:
                    workflows.append(w)
            cur.execute("UPDATE agents SET workflows = ? WHERE id = 'system_architect'", (json.dumps(workflows),))

        # 4. Agent 99
        row = cur.execute("SELECT skills, workflows FROM agents WHERE id = '99'").fetchone()
        if row:
            workflows = json.loads(row[1]) if row[1] else []
            for w in ["status"]:
                if w not in workflows:
                    workflows.append(w)
            cur.execute("UPDATE agents SET workflows = ? WHERE id = '99'", (json.dumps(workflows),))

        # 5. technical_writer
        row = cur.execute("SELECT skills, workflows FROM agents WHERE id = 'technical_writer'").fetchone()
        if row:
            workflows = json.loads(row[1]) if row[1] else []
            for w in ["status"]:
                if w not in workflows:
                    workflows.append(w)
            cur.execute("UPDATE agents SET workflows = ? WHERE id = 'technical_writer'", (json.dumps(workflows),))

        # 6. browser-specialist-01
        row = cur.execute("SELECT skills, workflows FROM agents WHERE id = 'browser-specialist-01'").fetchone()
        if row:
            cur.execute(
                "UPDATE agents SET skills = ?, workflows = ? WHERE id = 'browser-specialist-01'",
                (json.dumps(["explorer-scout", "research", "clean-code"]), json.dumps(["report", "status", "brainstorm"]))
            )

        # 7. Mock / Utility agents
        for uid, s_list, w_list in [
            ("utility_writer", ["documentation-templates", "clean-code"], ["codify", "report"]),
            ("utility_security_auditor", ["vulnerability-scanner", "testing-patterns"], ["audit", "report"]),
            ("utility_network_scout", ["explorer-scout", "research"], ["github_scout", "report"]),
        ]:
            cur.execute("UPDATE agents SET skills = ?, workflows = ? WHERE id = ?", (json.dumps(s_list), json.dumps(w_list), uid))

        conn.commit()
        conn.close()
        print(f"Successfully optimized {db_path}.")

if __name__ == "__main__":
    enhance_skills_and_workflows()

